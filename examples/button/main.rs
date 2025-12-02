use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Result;
use capybar::{
    root::Root,
    util::Color,
    widgets::{
        button::{Button, ButtonCallback, ButtonSettings},
        containers::bar::{Bar, BarSettings},
        text::TextSettings,
        Style, WidgetData, WidgetNew,
    },
};
use wayland_client::{globals::registry_queue_init, Connection};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Establish connection to Wayland compositor
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;

    // Create a bar with basic styling
    let mut bar = Bar::new(
        None,
        BarSettings {
            default_data: WidgetData {
                width: 1920,
                ..WidgetData::default()
            },
            padding: (10, 10, 10),
            style: Style {
                background: Some(Color::BLACK),
                border: Some((1, Color::WHITE)),
                ..Style::default()
            },
            ..BarSettings::default()
        },
    )?;

    // Create button settings with text, styling, and click callback
    let button_settings = ButtonSettings {
        text_settings: TextSettings {
            text: "Pressed 0 times".into(),
            font_color: Color::WHITE,
            size: 25.0,
            ..TextSettings::default()
        },
        style: Style {
            border: Some((1, Color::WHITE)),
            background: Some(Color::from_hex(0x333333ff)),
            ..Style::default()
        },
        hover_background: Color::from_hex(0x555555ff),
        press_background: Color::from_hex(0x111111ff),
        callback: ButtonCallback::new(|button| {
            // Simple counter to demonstrate button interactivity
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            COUNTER.fetch_add(1, Ordering::Relaxed);
            button
                .change_text(format!("Pressed {} times", COUNTER.load(Ordering::Relaxed)).as_str());
            Ok(())
        }),
        ..ButtonSettings::default()
    };

    // Add the button to the left side of the bar
    bar.create_widget_left(Button::new, button_settings)?;

    // Initialize the capybar root with the bar
    let mut capybar = Root::new(&globals, &mut event_queue, Some(bar))?;

    // Load a font for rendering text (first for normal text, second for emoji if needed)
    capybar.add_font_by_name("jetbrainsmononerdfont")?;

    // Start the event loop to run the bar
    capybar.run_sync(&mut event_queue)?;

    Ok(())
}
