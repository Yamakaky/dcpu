# A DCPU-16 emulator and DASM (dis)assembler

[![Build Status](https://travis-ci.org/Yamakaky/dcpu.svg?branch=master)](https://travis-ci.org/Yamakaky/dcpu)

## Features

- Full instruction set
- Tick accurate
- Devices support
  - Clock
  - LEM1802
  - Keyboard
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

`cargo run --release --bin <bin> -- <bin-args>`

Available binaries are assembler, disassembler, emulator and sprite.

Some features are only available on Rust nightly. To enable them, install Rust
nightly then run;

`cargo run --release --features nightly --bin ...`

## Convert images to LEM format

The `sprite` utility can:

- Convert a font image (`--font-file`) and a palette image (`--palette-file`) to
  a LEM1802-compatible format, either binary or hexadecimal
- Convert an image (`--image`) to VRam + font + palette

## Documentation

The library interface is documented [here](https://docs.rs/dcpu).
