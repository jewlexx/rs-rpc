use crate::{
    error::{DiscordError, Result},
    models::message::{FrameHeader, Message, OpCode},
    utils,
};
use bytes::BytesMut;
use serde_json::json;
use std::{
    io::{Read, Write},
    marker::Sized,
    path::PathBuf,
    thread, time,
};

/// Wait for a non-blocking connection until it's complete.
macro_rules! try_until_done {
    [ $e:expr ] => {
        loop {
            match $e {
                Ok(v) => break v,
                Err(why) => if !why.io_would_block() { return Err(why); },
            }

            thread::sleep(time::Duration::from_millis(500));
        }
    }
}

pub trait Connection: Sized {
    type Socket: Write + Read;

    /// The internally stored socket connection.
    fn socket(&mut self) -> &mut Self::Socket;

    /// The base path were the socket is located.
    fn ipc_path() -> PathBuf;

    /// Establish a new connection to the server.
    fn connect() -> Result<Self>;

    /// The full socket path.
    fn socket_path(n: u8) -> PathBuf {
        let socket_path = format!("discord-ipc-{n}");
        let base_path = Self::ipc_path().join(socket_path.clone());

        if base_path.exists() {
            base_path
        } else {
            // This fixes issues with Unix implementations
            Self::ipc_path()
                .join("app")
                .join("com.discordapp.Discord")
                .join(socket_path)
        }
    }

    /// Perform a handshake on this socket connection.
    /// Will block until complete.
    fn handshake(&mut self, client_id: u64) -> Result<Message> {
        let hs = json![{
            "client_id": client_id.to_string(),
            "v": 1,
            "nonce": utils::nonce()
        }];

        let msg = Message::new(OpCode::Handshake, hs)?;
        try_until_done!(self.send(&msg));
        let msg = try_until_done!(self.recv());

        Ok(msg)
    }

    /// Ping the server and get a pong response.
    /// Will block until complete.
    fn ping(&mut self) -> Result<OpCode> {
        let message = Message::new(OpCode::Ping, json![{}])?;
        try_until_done!(self.send(&message));
        let response = try_until_done!(self.recv());
        Ok(response.opcode)
    }

    /// Send a message to the server.
    fn send(&mut self, message: &Message) -> Result<()> {
        match message.encode() {
            Err(why) => error!("{:?}", why),
            Ok(bytes) => {
                self.socket().write_all(&bytes)?;
            }
        };
        trace!("-> {:?}", message);
        Ok(())
    }

    /// Receive a message from the server.
    fn recv(&mut self) -> Result<Message> {
        // Read header
        let mut buf = BytesMut::new();
        buf.resize(std::mem::size_of::<FrameHeader>(), 0);

        let n = self.socket().read(&mut buf)?;
        trace!("Received {} bytes for header", n);

        if n == 0 {
            return Err(DiscordError::ConnectionClosed);
        }

        let Some(header) = (unsafe { FrameHeader::from_bytes(buf.as_ref()) }) else {
            return Err(DiscordError::HeaderLength);
        };

        let mut message_buf = BytesMut::new();
        message_buf.resize(header.message_length() as usize, 0);

        let n = self.socket().read(&mut message_buf)?;
        trace!("Received {} bytes for payload", n);

        if n == 0 {
            return Err(DiscordError::NoMessage);
        }

        let mut payload = String::with_capacity(header.message_length() as usize);
        message_buf.as_ref().read_to_string(&mut payload)?;
        trace!("<- {:?} = {:?}", header.opcode(), payload);

        Ok(Message {
            opcode: header.opcode(),
            payload,
        })
    }
}
