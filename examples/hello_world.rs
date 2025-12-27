use crossterm::event::{self, Event, KeyCode};

use oelung::{Renderer, TextBuilder};

fn main() -> Result<(), anyhow::Error> {
    let renderer = Renderer::new();

    render_screen(&mut renderer)?;

    loop {
        match event::read()? {
            Event::Key(key_event) if key_event.code == KeyCode::Char('q') => break,
            _ => {}
        }
    }

    Ok(())
}

fn render_screen(renderer: &mut Renderer) -> Result<(), anyhow::Error> {
    renderer.render(
        TextBuilder::default()
            .text_child("Hello world")
            .text_child(" ")
            .text_child("(hit q to quit)")
            .build()?
            .into(),
    );

    Ok(())
}
