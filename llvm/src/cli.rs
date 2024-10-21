use std::path::PathBuf;

#[derive(Debug, Clone, clap::Parser)]
#[command(version, about = "Compiles brainfuck into object files/LLVM IR.")]
pub struct Cli {
    #[arg(help = "Path to file with source code")]
    pub source: Option<PathBuf>,
    #[arg(short, long, help = "Path to output file")]
    pub output: Option<PathBuf>,
    #[arg(short, long, help = "Target to compile for (e.g. x86_64-pc-linux-gnu)")]
    pub target: Option<String>,
    #[arg(
        short,
        long,
        help = "Features to enable (e.g. +sse2,+cx16,+sahf,-tbm). You can use 'native' to enable all features that current machine supports"
    )]
    pub features: Option<String>,
    #[arg(
        short,
        long,
        help = "Use JIT compiler and run program",
        conflicts_with = "emit"
    )]
    pub run: bool,
    #[arg(value_enum, short = 'e', long, default_value = "object")]
    pub emit: Emit,
    #[arg(short = 'O', help = "Enable optimization passes")]
    pub optimize: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, clap::ValueEnum)]
pub enum Emit {
    #[default]
    #[value(help = "Emit object file")]
    Object,
    #[value(help = "Emit generated LLVM IR")]
    LLVMIr,
    #[value(help = "Emit generated assembly")]
    Assembly,
}

impl Emit {
    pub fn extension(&self) -> &str {
        use Emit::*;

        match self {
            Object => "o",
            LLVMIr => "ll",
            Assembly => "as",
        }
    }
}
