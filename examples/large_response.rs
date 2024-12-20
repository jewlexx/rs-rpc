use discord_presence::Event;

mod helpers;

fn main() -> anyhow::Result<()> {
    helpers::logging::init_logging();

    let url_base = "https://example.com/".to_string();
    let mut client = discord_presence::Client::new(1286481105410588672);
    client.start();
    client.block_until_event(Event::Ready)?;
    client.set_activity(|activity| {
        activity
            .state("A".repeat(128))
            .details("A".repeat(128))
            .assets(|assets| {
                assets
                    .large_text("A".repeat(128))
                    .large_image("A".repeat(256))
                    .small_image("A".repeat(256))
                    .small_text("A".repeat(128))
            })
            .append_buttons(|buttons| {
                buttons
                    .url(url_base.clone() + &"A".repeat(256 - url_base.len()))
                    .label("A".repeat(32))
            })
    })?;

    client.block_on()?;

    Ok(())
}
