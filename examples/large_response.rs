use discord_presence::Event;

mod helpers;

fn main() -> anyhow::Result<()> {
    helpers::logging::init_logging();

    let url_base = "https://example.com/".to_string();
    let mut client = discord_presence::Client::new(1286481105410588672);
    client.start();
    client.block_until_event(Event::Ready)?;
    client.set_activity(|activity| {
        const ACTIVITY_TEXT_LIMIT: usize = 128;
        const ASSET_URL_LIMIT: usize = 256;

        activity
            .state("A".repeat(128))
            .details("A".repeat(128))
            .assets(|assets| {
                assets
                    .large_text("A".repeat(ACTIVITY_TEXT_LIMIT))
                    .large_image("A".repeat(ASSET_URL_LIMIT))
                    .small_text("A".repeat(ACTIVITY_TEXT_LIMIT))
                    .small_image("A".repeat(ASSET_URL_LIMIT))
            })
            .append_buttons(|buttons| {
                buttons
                    .url(url_base.clone() + &"A".repeat(ASSET_URL_LIMIT - url_base.len()))
                    .label("A".repeat(32))
            })
    })?;

    client.block_on()?;

    Ok(())
}
