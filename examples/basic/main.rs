use capybar::{
    root::Root,
    util::Color,
    widgets::{
        battery::{Battery, BatterySettings},
        clock::{Clock, ClockSettings},
        containers::bar::{Bar, BarSettings},
        cpu::{CPUSettings, CPU},
        text::TextSettings,
        Margin, Style, WidgetData, WidgetNew,
    },
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
        font: Color::from_hex(0xf5e0dcff),
    };

    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;

    let mut bar = Bar::new(
        None,
        BarSettings {
            default_data: WidgetData {
                width: 1920,
                ..WidgetData::default()
            },
            padding: (10, 10, 10),

            style: Style {
                background: Some(catpuccin_mocha.background),
                border: Some((1, catpuccin_mocha.border)),

                ..Style::default()
            },

            ..BarSettings::default()
        },
    )?;

    // Left widgets
    bar.create_widget_left(
        CPU::new,
        CPUSettings {
            update_rate: 1000,
            text_settings: TextSettings {
                font_color: catpuccin_mocha.font,
                size: 25.0,

                ..TextSettings::default()
            },
            default_data: WidgetData {
                ..WidgetData::default()
            },
            style: Style {
                margin: Margin {
                    left: 10,
                    right: 0,
                    up: 0,
                    down: 0,
                },
                ..Default::default()
            },
            ..CPUSettings::default()
        },
    )?;

    //Center widgets
    bar.create_widget_center(
        Clock::new,
        ClockSettings {
            font_color: catpuccin_mocha.font,
            size: 25.0,

            ..ClockSettings::default()
        },
    )?;

    // Right widgets
    bar.create_widget_right(
        Battery::new,
        BatterySettings {
            text_settings: TextSettings {
                font_color: catpuccin_mocha.font,
                size: 25.0,

                ..TextSettings::default()
            },
            default_data: WidgetData {
                ..WidgetData::default()
            },
            style: Style {
                margin: Margin {
                    left: 0,
                    right: 10,
                    up: 0,
                    down: 0,
                },
                ..Default::default()
            },
            ..BatterySettings::default()
        },
    )?;

    let mut capybar = Root::new(&globals, &mut event_queue, Some(bar))?;

    // Fonts can be replaces by your liking. The first font added will be used for normal text, the
    // second for emoji
    //capybar.add_font_by_name("mono")?;
    capybar.add_font_by_name("jetbrainsmononerdfont")?;
    capybar.add_font_by_name("jetbrainsmononerdfont")?;

    capybar.run(&mut event_queue)?;

    Ok(())
}
