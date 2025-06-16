use anyhow::Result;
use capybar::{config::Config, Root};
use wayland_client::{globals::registry_queue_init, Connection};

fn main() -> Result<()> {
    let config = Config::parse_toml("./examples/toml_config/config.toml".into())?;

    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;

    let mut capybar = Root::new_from_config(&globals, &mut event_queue, config)?;

    capybar.init(&mut event_queue)?.run(&mut event_queue)?;

    Ok(())
}
