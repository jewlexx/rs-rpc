use discord_presence::{Client, Event};

mod helpers;

fn main() -> anyhow::Result<()> {
    helpers::logging::init_logging();

    let mut drpc = Client::new(1003450375732482138);

    drpc.on_ready(|_ctx| {
        println!("ready?");
    })
    .persist();

    drpc.start();

    drpc.block_until_event(Event::Ready)?;

    assert!(Client::is_ready());

    // Set the activity
    // drpc.set_activity(|act| {
    //     act.state("rusting frfr")
    //         .append_buttons(|button| button.label("Click Me!").url("https://google.com/"))
    // })
    // .unwrap();

    drpc.shutdown()?;

    Ok(())
}
