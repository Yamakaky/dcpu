# A DCPU-16 emulator and DASM (dis)assembler

[![Build Status](https://travis-ci.org/Yamakaky/dcpu.svg?branch=master)](https://travis-ci.org/Yamakaky/dcpu)
[![Clippy Linting Result](https://clippy.bashy.io/github/Yamakaky/dcpu/master/badge.svg)](https://clippy.bashy.io/github/Yamakaky/dcpu/master/log)

## Features

- Full instruction set
- Tick accurate
- Devices support
  - Clock
  - LEM1802
  - Keyboard
  - m35fd
- Disassembler
- Assembler
- gdb-like debugger
- Image-to-LEM-compatible-format utility

## Quick usage

Compiled versions for Windows and Linux are available at
https://github.com/Yamakaky/dcpu/releases/. No dependencies are required.

Note: the Windows version of the emulation currently fails with an OpenGL error.
If anyone knows why...

All binaries support a `--help` flag for more infos.

## Building

You need to install the [rust compiler](https://www.rust-lang.org/) to build this software.

    # cargo run --release --bin <bin> -- <bin-args>

Available binaries are assembler, disassembler, emulator and sprite.

Some features are only available on Rust nightly. To enable them, install Rust
nightly then run;

    # cargo run --release --features nightly --bin ...

### Build features

The following build features are available ([x] means "enabled by default"):

- [x] `bins`: only useful to build the binaries, should be disabled for the
      library.
- [x] `debugger-cli`: command line parsing for the debugger, should also be
      disabled for the library.
- [x] `glium`: OpenGL backend for the lem1802 + keyboard, can be useful in the
      library.
- [ ] `nightly`: implementation of `serde::{Serialize, Deserialize}` for some of
      the types. Requires Rust nightly.

### Build the C library

To build a dynamic library (`.so`):

    # cargo rustc --lib --no-default-features -- --crate-type=dylib

To build a static library (`.a`):

    # cargo rustc --lib --no-default-features -- --crate-type=staticlib

See `src/c_api.h` for the available functions.

## Convert images to LEM format

The `sprite` utility can:

- Convert a font image (`--font-file`) and a palette image (`--palette-file`) to
  a LEM1802-compatible format, either binary or hexadecimal
- Convert an image (`--image`) to VRam + font + palette

## Documentation

The library interface is documented [here](https://docs.rs/dcpu).
