use capybar::{
    util::Color,
    widgets::{
        clock::Clock,
        containers::row::{Row, RowSettings},
        text::{Text, TextSettings},
        WidgetData,
    },
    Root,
};
use wayland_client::{globals::registry_queue_init, Connection};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;

    let mut bar = Root::new(&globals, &mut event_queue)?;

    bar.add_font_by_name("Mono".to_string())?;

    let mut row = Box::new(Row::new(
        WidgetData {
            margin: (10, 0, 10, 0), //does not do anything because not inside of container
            //idk if that should stay that way
            width: 1000,
            position: (10, 10),
            ..WidgetData::new()
        },
        Some(RowSettings {
            background: Some(Color::CYAN),
            border: Some((10, Color::PINK)),
            alignment: capybar::widgets::containers::row::Alignment::CenteringHorizontal,
            ..RowSettings::default()
        }),
    ));
    row.add_child(Box::new(Text::new(
        "test1".to_string(),
        &mut bar.fonts(),
        WidgetData {
            width: 60,
            margin: (10, 0, 0, 0),
            ..WidgetData::new()
        },
        TextSettings {
            size: 25.0,
            background: Some(Color::WHITE),
            color: Color::PURPLE,
            ..TextSettings::default()
        },
    )))?;

    row.add_child(Box::new(Text::new(
        "test2".to_string(),
        &mut bar.fonts(),
        WidgetData {
            width: 65,
            ..WidgetData::new()
        },
        TextSettings {
            size: 25.0,
            background: Some(Color::BLACK),
            color: Color::YELLOW,
            ..TextSettings::default()
        },
    )))?;

    row.add_child(Box::new(Clock::new(&mut bar.fonts(), 25.0)))?;

    bar.add_widget(row);

    bar.init(&mut event_queue)?.run(&mut event_queue)?;

    Ok(())
}
