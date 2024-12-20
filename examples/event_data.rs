use discord_presence::{models::EventData, Client, Event};

mod helpers;

fn main() -> anyhow::Result<()> {
    helpers::logging::init_logging();

    let mut drpc = Client::new(1003450375732482138);

    drpc.on_ready(|ctx| {
        let EventData::Ready(data) = ctx.event else {
            unreachable!()
        };

        let _user = data.user;
    })
    .persist();

    drpc.start();

    drpc.block_until_event(Event::Ready)?;

    assert!(Client::is_ready());

    // Set the activity
    drpc.set_activity(|act| {
        act.state("rusting frfr")
            .append_buttons(|button| button.label("Click Me!").url("https://google.com/"))
    })
    .unwrap();

    // TODO: Implement "remote" shutdown
    // ctrlc::set_handler(move || {
    //     println!("Exiting...");
    //     drpc.clear_activity().unwrap();
    //     std::process::exit(0);
    // })?;

    drpc.block_on()?;

    Ok(())
}
