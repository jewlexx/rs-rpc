use discord_presence::Client;

mod helpers;

fn main() {
    helpers::logging::init_logging();

    let mut drpc = Client::new(1003450375732482138);

    let _ready = drpc.on_ready(|_ctx| {
        println!("ready?");
    });

    let _activity_join_request = drpc.on_activity_join_request(|ctx| {
        println!("Join request: {:?}", ctx.event);
    });

    let _activity_join = drpc.on_activity_join(|ctx| {
        println!("Joined: {:?}", ctx.event);
    });

    let _activity_spectate = drpc.on_activity_spectate(|ctx| {
        println!("Spectate: {:?}", ctx.event);
    });

    drpc.start();

    if let Err(why) = drpc.set_activity(|a| {
        a.state("Running examples").assets(|ass| {
            ass.large_image("ferris_wat")
                .large_text("wat.")
                .small_image("rusting")
                .small_text("rusting...")
        })
    }) {
        println!("Failed to set presence: {}", why);
    }

    drpc.block_on().unwrap();
}
