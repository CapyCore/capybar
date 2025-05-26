use std::env;

use capybar::{
    util::Color,
    widgets::{
        clock::{Clock, ClockSettings},
        containers::row::{Row, RowSettings},
        text::{Text, TextSettings},
        WidgetData, WidgetNew,
    },
    Root,
};
use wayland_client::{globals::registry_queue_init, Connection};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // this method needs to be inside main() method
    env::set_var("RUST_BACKTRACE", "1");
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;

    let mut bar = Root::new(&globals, &mut event_queue)?;

    bar.add_font_by_name("Mono".to_string())?;

    let mut row = Row::new(
        None,
        RowSettings {
            background: Some(Color::CYAN),
            border: Some((10, Color::PINK)),
            alignment: capybar::widgets::containers::row::Alignment::CenteringHorizontal,
            data: WidgetData {
                margin: (10, 0, 10, 0), //does not do anything because not inside of container
                //idk if that should stay that way
                width: 1000,
                position: (10, 10),
                ..WidgetData::default()
            },
            ..RowSettings::default()
        },
    )?;

    row.create_child(
        Text::new,
        TextSettings {
            text: "Test1".to_string(),
            ..TextSettings::default()
        },
    )?;

    row.create_child(Clock::new, ClockSettings::default())?;

    bar.add_widget(row)?;

    bar.init(&mut event_queue)?.run(&mut event_queue)?;

    Ok(())
}
