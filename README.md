# A DCPU-16 emulator and DASM (dis)assembler

[![Build Status](https://travis-ci.org/Yamakaky/dcpu.svg?branch=master)](https://travis-ci.org/Yamakaky/dcpu)

## Features

- [x] Full instruction set
- [x] Tick accurate
- [x] Devices support
  - [x] Clock
  - [x] LEM1802
  - [x] Keyboard
- [x] Disassembler
- [x] Assembler
- [x] gdb-like debugger

## Usage

Compiled versions for Windows and Linux are available at
https://pydio.chocolytech.info:4443/data/public/051666. No dependencies are
required.

All binaries support a `--help` flag for more infos.

## Building

You need to install the [rust compiler](https://www.rust-lang.org/) to build this software.

`cargo run --release --bin <bin> -- <bin-args>`

Available binaries are assembler, disassembler, emulator and sprite.

## Documentation

The library interface is documented [here](https://docs.rs/dcpu).
