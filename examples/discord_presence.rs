use discord_presence::Client;

mod helpers;

fn main() {
    helpers::logging::init_logging();

    let mut drpc = Client::new(1003450375732482138);

    drpc.on_ready(|_ctx| {
        println!("READY!");
    })
    .persist();

    drpc.on_error(|ctx| {
        eprintln!("An error occured, {:?}", ctx.event);
    })
    .persist();

    drpc.start();

    loop {
        let mut buf = String::new();

        std::io::stdin().read_line(&mut buf).unwrap();
        buf.pop();

        if buf.is_empty() {
            if let Err(why) = drpc.clear_activity() {
                println!("Failed to clear presence: {}", why);
            }
        } else if let Err(why) = drpc.set_activity(|a| {
            a.state("Running examples")
                .assets(|ass| {
                    ass.large_image("ferris_wat")
                        .large_text("wat.")
                        .small_image("rusting")
                        .small_text("rusting...")
                })
                .append_buttons(|button| button.label("Click Me!").url("https://google.com/"))
        }) {
            println!("Failed to set presence: {}", why);
        }
    }
    // drpc.block_on().unwrap();
}
