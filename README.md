# Discord RPC

[![crates.io](https://img.shields.io/crates/v/discord-presence.svg)](https://crates.io/crates/discord-presence)
[![crates.io](https://img.shields.io/crates/d/discord-presence.svg)](https://crates.io/crates/discord-presence)
[![docs.rs](https://docs.rs/discord-presence/badge.svg)](https://docs.rs/discord-presence)

Discord RPC client for Rust forked from [Discord RPC Client](https://gitlab.com/valeth/discord-rpc-client.rs)

> Note: If you are looking to add this into a game, check out the [Bevy implementation](https://github.com/jewlexx/bevy-discord-rpc)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
discord-presence = "1.2"
```

or run:

```shell
cargo add discord-presence
```

## Example

```rust
use std::{env, thread, time};
use discord_presence::{Client, Event};

fn main() {
    // Get our main status message
    let state_message = env::args().nth(1).expect("Requires at least one argument");

    // Create the client
    let mut drpc = Client::new(1003450375732482138);

    // Register event handlers with the corresponding methods
    drpc.on_ready(|_ctx| {
        println!("ready?");
    });

    // or

    drpc.on_event(Event::Ready, |ctx| {
        println!("READY!");
    });

    // Start up the client connection, so that we can actually send and receive stuff
    drpc.start();

    // Set the activity
    drpc.set_activity(|act| act.state(state_message))
        .expect("Failed to set activity");

    // Wait 10 seconds before exiting
    thread::sleep(time::Duration::from_secs(10));
}
```

> More examples can be found in the examples directory.

## Changelog

See [CHANGELOG.md](CHANGELOG.md)

## Contributions

See [CONTRIBUTING.md](/CONTRIBUTING.md)
