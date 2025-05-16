use capybar::{clock::Clock, label::Label, root::Root};
use fontconfig::Fontconfig;
use fontdue::FontSettings;
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

    let font = fontdue::Font::from_bytes(bytes, settings).unwrap();

    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;

    let mut root = Root::new(&globals, &mut event_queue)?;

    root.add_widget(Box::new(Label::new("test1", &font, 30.0)));
    root.add_widget(Box::new(Label::new("test2", &font, 15.0)));
    root.add_widget(Box::new(Clock::new(&font, 30.0)));

    root.init(&mut event_queue)?.run(&mut event_queue)?;

    Ok(())
}
