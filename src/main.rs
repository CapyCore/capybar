use capybar::{
    util::Color,
    widgets::{
        containers::row::{Row, RowSettings},
        Text, WidgetData,
    },
    Root,
};
use fontconfig::Fontconfig;
use fontdue::{Font, FontSettings};
use wayland_client::{globals::registry_queue_init, Connection};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fc = Fontconfig::new().unwrap();

    let font = fc.find("Mono", None).unwrap();
    let size: f32 = 200.0;

    let settings = FontSettings {
        scale: size,
        ..fontdue::FontSettings::default()
    };

    let bytes = std::fs::read(&font.path.as_path())?;

    let font = Font::from_bytes(bytes, settings).unwrap();

    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;

    let mut bar = Root::new(&globals, &mut event_queue)?;

    bar.add_font(font);

    let mut row = Box::new(Row::new(
        WidgetData {
            height: 30,
            margin: (10, 0, 10, 0), //does not do anything because not inside of container
                                    //idk if that should stay that way
            position: (10, 10),
            ..WidgetData::new()
        },
        Some(RowSettings {
            background: Some(Color::GREEN),
            ..RowSettings::default()
        }),
    ));
    row.add_child(Box::new(Text::new(
        "test1".to_string(),
        &mut bar.fonts(),
        25.0,
        WidgetData {
            width: 40,
            margin: (50, 10, 0, 0),
            ..WidgetData::new()
        },
    )));

    row.add_child(Box::new(Text::new(
        "test2".to_string(),
        &mut bar.fonts(),
        20.0,
        WidgetData {
            width: 100,
            margin: (10, 30, 50, 0),
            ..WidgetData::new()
        },
    )));

    bar.add_widget(row);

    bar.init(&mut event_queue)?.run(&mut event_queue)?;

    Ok(())
}
