use capybar::{
    util::Color,
    widgets::{
        battery::{Battery, BatterySettings},
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

    bar.add_font_by_name("mono")?;
    bar.add_font_by_name("jetbrainsmononerdfont")?;

    let mut row = Row::new(
        None,
        RowSettings {
            background: Some(catpuccin_mocha.background),
            border: Some((2, catpuccin_mocha.border)),
            alignment: capybar::widgets::containers::row::Alignment::CenteringHorizontal,
            default_data: WidgetData {
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

            default_data: WidgetData {
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
        Battery::new,
        BatterySettings {
            text: TextSettings {
                color: catpuccin_mocha.font,
                size: 25.0,

                ..TextSettings::default()
            },
            icon: TextSettings {
                color: catpuccin_mocha.font,
                size: 25.0,

                ..TextSettings::default()
            },
            default_data: WidgetData {
                margin: (0, 10, 0, 0),
                ..WidgetData::default()
            },
            ..BatterySettings::default()
        },
    )?;

    bar.add_widget(row)?;

    bar.init(&mut event_queue)?.run(&mut event_queue)?;

    Ok(())
}
