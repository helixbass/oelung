use crossterm::event::{self, Event, KeyCode};

use oelung::{Cursor, Renderer, TextBuilder, soft};

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
    renderer.render(
        FlexColumnBuilder::default()
            .child(
                FlexColumnBuilder::default()
                    .child(
                        TextBuilder::default()
                            .text_child("Top area")
                            .cursor_child(Cursor::relative().x(0).y(0))
                            .build()?
                            .into(),
                    )
                    .flex_grow(1)
                    .build()?
                    .into(),
            )
            .child(
                TextBuilder::default()
                    .text_child("some_file.rs [1%] 999 lines |1")
                    .build()?
                    .into(),
            )
        TextBuilder::default()
            .text_child("Hello world")
            .text_child(" ")
            .text_child("(hit q to quit)")
            .cursor_child(Cursor::relative().x(0).y(0))
            .build()?
            .into(),
    )?;

    Ok(())
}
