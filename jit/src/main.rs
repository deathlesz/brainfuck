use color_eyre::{
    eyre::{Context as _, ContextCompat as _},
    Result,
};

mod compiler;

fn main() -> Result<()> {
    color_eyre::install()?;

    let src = std::env::args()
        .nth(1)
        .wrap_err("source file is not specified")?;
    let contents = std::fs::read(src).wrap_err("source file does not exist")?;

    let parser = parser::Parser::new(contents);
    let instructions = parser.parse()?;

    let compiler = compiler::Compiler::<30_000>::new(instructions);
    compiler.run()?;

    Ok(())
}
