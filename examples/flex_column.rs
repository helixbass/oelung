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
      %FlexColumn
        children => [
          %FlexColumn
            children => [
              %Text
                text => "Top area"
                cursor => %Cursor.Relative
                  x => 0
                  y => 0
            ]
            flex_grow => 1
          %Text "some_file.rs [1%] 999 lines |1"
          %Text "This looks great. Hit q to quit"
        ]
    })?;

    Ok(())
}
