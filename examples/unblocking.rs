use std::{mem::forget, thread::sleep, time::Duration};

use discord_presence::Client;

mod helpers;

fn main() {
    helpers::logging::init_logging();

    let mut client = Client::new(1003450375732482138);

    client.start();

    tracing::error!("Due to the way unblocking activity setting works, this example does not seem to work currently (at least on Windows).");
    {
        let ready = client.on_ready({
            let client = client.clone();
            move |_ctx| {
                let mut client = client.clone();
                println!("READY!");

                client
                    .set_activity(|a| {
                        a.state("Rust")
                            .details("Programming")
                            .assets(|a| a.large_image("rust"))
                    })
                    .unwrap();
            }
        });

        // we can `std::mem::forget` the event listener's handle to keep it
        // registered until `drpc` is dropped
        forget(ready);
    }

    // an alternative is to store the handle until you're ready to unregister the
    // listener
    let _error = client.on_error(|ctx| {
        eprintln!("An error occured, {:?}", ctx.event);
    });

    tracing::trace!("Made it to the final line");

    // keep the main thread alive
    loop {
        sleep(Duration::from_secs(100));
    }
}
