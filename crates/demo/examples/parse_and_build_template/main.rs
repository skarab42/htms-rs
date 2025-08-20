use color_eyre::Result;
use htms::template;

fn main() -> Result<()> {
    color_eyre::install()?;

    template::parse_and_build(
        "crates/demo/examples/parse_and_build_template/index.input.html",
        "crates/demo/examples/parse_and_build_template/index.output.html",
    )?;

    Ok(())
}
