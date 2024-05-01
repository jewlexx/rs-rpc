use super::{Connection, Socket};
use crate::{
    error::{DiscordError, Result},
    event_handler::HandlerRegistry,
    models::{payload::Payload, ErrorEvent, Event, Message},
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use parking_lot::Mutex;
use serde_json::Value as JsonValue;
use std::{
    io::ErrorKind,
    sync::{atomic::Ordering, Arc},
    thread,
    time::{self, Duration},
};

type Tx = Sender<Message>;
type Rx = Receiver<Message>;

// TODO: Refactor connection manager
#[derive(Clone)]
pub struct Manager {
    connection: Arc<Option<Mutex<Socket>>>,
    client_id: u64,
    outbound: (Rx, Tx),
    inbound: (Rx, Tx),
    handshake_completed: bool,
    event_handler_registry: Arc<HandlerRegistry>,
    error_sleep: Duration,
    connection_attempts: Arc<Mutex<Option<usize>>>,
}

impl Manager {
    pub fn new(
        client_id: u64,
        event_handler_registry: Arc<HandlerRegistry>,
        error_sleep: Duration,
        connection_attempts: Option<usize>,
    ) -> Self {
        let connection = Arc::new(None);
        let (sender_o, receiver_o) = unbounded();
        let (sender_i, receiver_i) = unbounded();

        Self {
            connection,
            client_id,
            handshake_completed: false,
            inbound: (receiver_i, sender_i),
            outbound: (receiver_o, sender_o),
            event_handler_registry,
            error_sleep,
            connection_attempts: Arc::new(Mutex::new(connection_attempts)),
        }
    }

    pub fn start(&mut self, rx: Receiver<()>) -> std::thread::JoinHandle<()> {
        let mut manager_inner = self.clone();
        let error_sleep = self.error_sleep;
        let connection_attempts = self.connection_attempts.clone();
        thread::spawn(move || {
            // TODO: Refactor so that JSON values are consistent across errors
            send_and_receive_loop(&mut manager_inner, &rx, error_sleep, &connection_attempts);
        })
    }

    pub fn send(&self, message: Message) -> Result<()> {
        self.outbound.1.send(message)?;

        Ok(())
    }

    pub fn recv(&self) -> Result<Message> {
        self.inbound.0.recv().map_err(DiscordError::from)
    }

    fn connect(&mut self) -> Result<()> {
        if self.connection.is_some() {
            return Ok(());
        }

        trace!("Connecting");

        let mut new_connection = Socket::connect()?;

        trace!("Performing handshake");
        let msg = new_connection.handshake(self.client_id)?;
        let payload: Payload<JsonValue> = serde_json::from_str(&msg.payload)?;

        // TODO: Ensure it works without clone
        // Only handle the ready event if the client was not already ready
        if !crate::READY.load(std::sync::atomic::Ordering::Relaxed) {
            trace!("Discord client is ready!");
            crate::READY.store(true, Ordering::Relaxed);

            self.event_handler_registry.handle(
                Event::Ready,
                Event::Ready.parse_data(into_error!(payload.data)?),
            );
        }

        trace!("Handshake completed");

        self.connection = Arc::new(Some(Mutex::new(new_connection)));

        trace!("Connected");

        Ok(())
    }

    fn disconnect(&mut self) {
        self.handshake_completed = false;
        self.connection = Arc::new(None);
    }
}

fn send_and_receive_loop(
    manager: &mut Manager,
    rx: &Receiver<()>,
    err_sleep: Duration,
    connection_attempts: &Arc<Mutex<Option<usize>>>,
) {
    trace!("Starting sender loop");

    let mut inbound = manager.inbound.1.clone();
    let outbound = manager.outbound.0.clone();

    loop {
        if rx.try_recv().is_ok() {
            break;
        }

        let connection = manager.connection.clone();

        match *connection {
            Some(ref conn) => {
                let mut connection = conn.lock();
                match send_and_receive(
                    &mut connection,
                    &manager.event_handler_registry,
                    &mut inbound,
                    &outbound,
                ) {
                    Err(DiscordError::IoError(ref err)) if err.kind() == ErrorKind::WouldBlock => {}
                    Err(DiscordError::IoError(_) | DiscordError::ConnectionClosed) => {
                        manager.disconnect();
                    }
                    Err(DiscordError::TimeoutError(_)) => continue,
                    Err(why) => trace!("discord error: {}", why),
                    _ => {}
                }

                thread::sleep(time::Duration::from_millis(500));
            }
            None => match manager.connect() {
                Err(err) => {
                    manager.event_handler_registry.handle(
                        Event::Error,
                        crate::models::EventData::Error(ErrorEvent {
                            code: None,
                            message: Some(err.to_string()),
                        }),
                    );

                    if err.should_break() {
                        break;
                    }
                    error!("Failed to connect: {:?}", err);

                    let mut attempts = connection_attempts.lock();
                    if let Some(ref mut attempts) = *attempts {
                        if *attempts == 0 {
                            break;
                        }

                        *attempts -= 1;
                    }

                    thread::sleep(err_sleep);
                }
                _ => manager.handshake_completed = true,
            },
        }
    }
}

fn send_and_receive(
    connection: &mut Socket,
    event_handler_registry: &Arc<HandlerRegistry>,
    inbound: &mut Tx,
    outbound: &Rx,
) -> Result<()> {
    while let Ok(msg) = outbound.try_recv() {
        trace!("Sending message");
        connection.send(&msg)?;
        trace!("Sent message");
    }

    trace!("Receiving from connection");
    let msg = connection.recv()?;
    trace!("Received from connection");

    let payload: Payload<JsonValue> = serde_json::from_str(&msg.payload)?;

    trace!("Received payload");

    if let Payload {
        evt: Some(event), ..
    } = &payload
    {
        trace!("Got event");
        let event_data = event.parse_data(into_error!(payload.data.clone())?);
        event_handler_registry.handle(*event, event_data);
    } else {
        trace!("Got message");
        inbound.send(msg)?;
    }

    Ok(())
}
