use std::path::PathBuf;

#[derive(Debug, Clone, clap::Parser)]
#[command(version, about = "Runs brainfuck using interpreter.")]
pub struct Cli {
    #[arg(help = "Path to file with source code")]
    pub source: Option<PathBuf>,
}
