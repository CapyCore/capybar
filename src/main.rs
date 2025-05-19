use capybar::{
    widget::{Clock, Label},
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

    bar.add_widget(Box::new(Label::new("test1", bar.fonts(), 30.0)));
    bar.add_widget(Box::new(Label::new("test2", bar.fonts(), 15.0)));
    bar.add_widget(Box::new(Clock::new(bar.fonts(), 30.0)));

    bar.init(&mut event_queue)?.run(&mut event_queue)?;

    Ok(())
}
