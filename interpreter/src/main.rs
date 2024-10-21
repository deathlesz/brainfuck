use std::sync::LazyLock;

use clap::Parser as _;
use cli::Cli;
use color_eyre::{eyre::Context as _, Result};

mod cli;
mod interpreter;

pub static ARGS: LazyLock<Cli> = LazyLock::new(Cli::parse);

fn main() -> Result<()> {
    color_eyre::install()?;

    let source = match ARGS.source {
        Some(ref source) => std::fs::read(source).wrap_err("source file does not exist")?,
        None => {
            use std::io::{stdin, Read};

            println!("! Live mode. Press ^D to finish.");

            let mut source = Vec::new();
            stdin()
                .read_to_end(&mut source)
                .wrap_err("failed to read from stdin")?;

            source
        }
    };

    let parser = parser::Parser::new(source);
    let instructions = parser.parse().wrap_err("failed to parse")?;

    let interpreter = interpreter::Interpreter::<30_000>::new(instructions);
    interpreter.run().wrap_err("failed to interpret")?;

    Ok(())
}
