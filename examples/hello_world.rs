use crossterm::event::{self, Event, KeyCode};

use oelung::{soft, Renderer, RendererBuilder};

fn main() -> Result<(), anyhow::Error> {
    let mut renderer = RendererBuilder::default().build()?;

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
    renderer.render(soft! {
        %Text
          cursor => %Cursor.Relative
            x => 0
            y => 0
          text => "Hello world (hit q to quit)"
    })?;

    Ok(())
}
