use oelung::TextBuilder;

fn main() -> Result<(), anyhow::Error> {
    let _guard = oelung::take_over_screen();

    oelung::render(
        TextBuilder::default()
            .text_child("Hello world")
            .build()?
            .into(),
    );

    Ok(())
}
