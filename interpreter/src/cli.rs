use std::path::PathBuf;

#[derive(Debug, Clone, clap::Parser)]
#[command(version, about = "Compiles brainfuck into object files/LLVM IR.")]
pub struct Cli {
    #[arg(help = "Path to file with source code")]
    pub source: Option<PathBuf>,
    #[arg(short, long, help = "Path to output file")]
    pub output: Option<PathBuf>,
}
