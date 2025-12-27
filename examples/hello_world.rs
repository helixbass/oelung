use crossterm::event::{self, Event, KeyCode};

use oelung::TextBuilder;

fn main() -> Result<(), anyhow::Error> {
    let _guard = oelung::take_over_screen();

    render_screen()?;

    loop {
        match event::read()? {
            Event::Key(key_event) if key_event.code == KeyCode::Char('q') => break,
            _ => {}
        }
    }

    Ok(())
}

fn render_screen() -> Result<(), anyhow::Error> {
    oelung::render(
        TextBuilder::default()
            .text_child("Hello world")
            .text_child(" ")
            .text_child("(hit q to quit)")
            .build()?
            .into(),
    );

    Ok(())
}
