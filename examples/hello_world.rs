use crossterm::event::{self, Event, KeyCode};

use oelung::{soft, Renderer};

fn main() -> Result<(), anyhow::Error> {
    let mut renderer = Renderer::try_new()?;

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
          text => "Hello world (hit q to quit)"
          cursor => %Cursor.Relative
            x => 0
            y => 0
    })?;

    Ok(())
}
