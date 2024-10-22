use std::path::PathBuf;

#[derive(Debug, Clone, clap::Parser)]
#[command(version, about = "Runs brainfuck using JIT compiler.")]
pub struct Cli {
    #[arg(help = "Path to file with source code")]
    pub source: Option<PathBuf>,
}
