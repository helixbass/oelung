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
    // renderer.render(soft! {
    //   %FlexRow
    //     children => [
    //       %FlexColumn
    //         children => [
    //           %Text
    //             text => "Left half"
    //             cursor => %Cursor.Relative
    //               x => 0
    //               y => 0
    //         ]
    //         flex_grow => 1
    //       %FlexColumn
    //         children => [
    //           %Text "Right half"
    //         ]
    //         flex_grow => 1
    //     ]
    //     flex_grow => 1
    // })?;
    renderer.render(
        oelung::FlexRowBuilder::default()
            .children(vec![
                soft! {
                    %FlexColumn
                      children => [
                        %Text
                          text => "Left half"
                          cursor => %Cursor.Relative
                            x => 0
                            y => 0
                      ]
                      flex_grow => 1
                },
                soft! {
                    %FlexColumn
                      children => [
                        %Text "Right half"
                      ]
                      flex_grow => 1
                },
            ])
            .flex_grow(1)
            .build()
            .unwrap()
            .into(),
    )?;

    Ok(())
}
