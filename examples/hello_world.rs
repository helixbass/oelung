use oelung::TextBuilder;

fn main() -> Result<(), anyhow::Error> {
    oelung::render(TextBuilder::default().text_child("Hello world").build()?);

    Ok(())
}
