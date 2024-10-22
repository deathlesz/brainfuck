
# Brainfuck

An optimizing interpreter, JIT compiler and LLVM frontend all-in-one for [Brainfuck](https://brainfuck.org/brainfuck.html).
## Installation

You can either download precompiled binary from [here](https://github.com/deathlesz/brainfuck/releases) or build it yourself.

### Building
- Clone this repository using `git`:
```sh
$ git clone https://github.com/deathlesz/brainfuck.git
```
- Build it using `cargo` in either debug or release mode
```sh
# For release mode
$ cargo build --release # to build all subprojects (interpreter, jit, llvm)
$ cargo build -p [project_name] --release # build specific subproject
# For debug mode, remove --release
```
- The compiled binary should be in `./target/release` or `./target/debug`.
- Alternatively, you can run the binary using `cargo run` with the same arguments as `cargo build`
## Usage

All subprojects except for parser are CLIs. `Interpreter` and `JIT` binaries have the same interface.

### `Interpreter/JIT`
```
Runs brainfuck using interpreter/JIT compiler.

Usage: interpreter [SOURCE]

Arguments:
  [SOURCE]  Path to file with source code

Options:
  -h, --help     Print help
  -V, --version  Print version
```
For example, `./interpreter src.b`.
### `LLVM`
```
Compiles brainfuck into object files/LLVM IR.

Usage: llvm [OPTIONS] [SOURCE]

Arguments:
  [SOURCE]
          Path to file with source code

Options:
  -o, --output <OUTPUT>
          Path to output file

  -t, --target <TARGET>
          Target to compile for (e.g. x86_64-pc-linux-gnu)

  -f, --features <FEATURES>
          Features to enable (e.g. +sse2,+cx16,+sahf,-tbm). You can use 'native' to enable all features that current machine supports

  -r, --run
          Use JIT compiler and run program

  -e, --emit <EMIT>
          [default: object]

          Possible values:
          - object:   Emit object file
          - llvm-ir:  Emit generated LLVM IR
          - assembly: Emit generated assembly

  -s, --safe
          Enable bounds check on >/<. Can be really slow

  -O
          Enable optimization passes

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
For example, `./llvm src.b`, `./llvm src.b -o out.o`, `./llvm src.b -e llvm-ir -o out.ll`, etc.
### Live mode
`Interpreter`, `JIT` and `LLVM` subprojects all have live mode. You can use it by running the binary without `SOURCE`. You'll be prompted with the following:
```
! Live mode. Press ^D to finish
```
You can enter any brainfuck code and then press Ctrl+D to interpret/compile it. You can also pipe files into live mode, e.g. 
```sh
$ echo "+++++[->++++++++++<]>." | ./llvm -o out.ll -e llvm-ir -O
```
## Acknowledgements
- [Brainfuck archive](https://sange.fi/esoteric/brainfuck)  by Jeff Johnston and Panu Kalliokoski
- [Some brainfuck fluff](https://brainfuck.org) by Daniel Cristofani
- And many other resources <3
## License

This project is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.