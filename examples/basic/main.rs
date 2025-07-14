use capybar::{
    root::Root,
    util::Color,
    widgets::{
        battery::{Battery, BatterySettings},
        clock::{Clock, ClockSettings},
        containers::bar::{Bar, BarSettings},
        cpu::{CPUSettings, CPU},
        text::TextSettings,
        Style, WidgetData, WidgetNew,
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

    let mut capybar = Root::new(&globals, &mut event_queue)?;

    // Fonts can be replaces by your liking. The first font added will be used for normal text, the
    // second for emoji
    //capybar.add_font_by_name("mono")?;
    capybar.add_font_by_name("jetbrainsmononerdfont")?;
    capybar.add_font_by_name("jetbrainsmononerdfont")?;

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
    bar.create_child_left(
        CPU::new,
        CPUSettings {
            update_rate: 1000,
            text_settings: TextSettings {
                font_color: catpuccin_mocha.font,
                size: 25.0,

                ..TextSettings::default()
            },
            default_data: WidgetData {
                margin: (10, 0, 0, 0),
                ..WidgetData::default()
            },
            ..CPUSettings::default()
        },
    )?;

    //Center widgets
    bar.create_child_center(
        Clock::new,
        ClockSettings {
            font_color: catpuccin_mocha.font,
            size: 25.0,

            ..ClockSettings::default()
        },
    )?;

    // Right widgets
    bar.create_child_right(
        Battery::new,
        BatterySettings {
            text_settings: TextSettings {
                font_color: catpuccin_mocha.font,
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

    capybar.add_widget(bar)?;

    capybar.init(&mut event_queue)?.run(&mut event_queue)?;

    Ok(())
}
