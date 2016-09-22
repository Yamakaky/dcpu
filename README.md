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

You need to install the [rust compiler](https://www.rust-lang.org/) to build this software.

`cargo run --release --bin <bin> -- <bin-args>`

Available binaries are assemble, disassemble and emulator.
All binaries support a `--help` flag.

## Documentation

The library interface is documented [here](https://yamakaky.github.io/dcpu/dcpu/index.html).
