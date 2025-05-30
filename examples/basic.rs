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

struct Palete {
    background: Color,
    border: Color,
    font: Color,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let catpuccin_mocha = Palete {
        background: Color::from_hex(0x1e1e2eff),
        border: Color::from_hex(0x74c7ecff),
        font: Color::from_hex(0xf5e0dc00),
    };

    // this method needs to be inside main() method
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;

    let mut bar = Root::new(&globals, &mut event_queue)?;

    bar.add_font_by_name("Mono".to_string())?;

    let mut row = Row::new(
        None,
        RowSettings {
            background: Some(catpuccin_mocha.background),
            border: Some((2, catpuccin_mocha.border)),
            alignment: capybar::widgets::containers::row::Alignment::CenteringHorizontal,
            data: WidgetData {
                width: 1920,
                ..WidgetData::default()
            },
            ..RowSettings::default()
        },
    )?;

    row.create_child(
        Text::new,
        TextSettings {
            text: "Workspaces placeholder".to_string(),
            color: catpuccin_mocha.font,
            size: 25.0,

            data: WidgetData {
                margin: (10, 0, 0, 0),
                ..WidgetData::default()
            },

            ..TextSettings::default()
        },
    )?;

    row.create_child(
        Clock::new,
        ClockSettings {
            font_color: catpuccin_mocha.font,
            size: 25.0,

            ..ClockSettings::default()
        },
    )?;

    row.create_child(
        Text::new,
        TextSettings {
            text: "Battery placeholder".to_string(),
            color: catpuccin_mocha.font,
            size: 25.0,

            data: WidgetData {
                margin: (0, 10, 0, 0),
                ..WidgetData::default()
            },

            ..TextSettings::default()
        },
    )?;

    bar.add_widget(row)?;

    bar.init(&mut event_queue)?.run(&mut event_queue)?;

    Ok(())
}
