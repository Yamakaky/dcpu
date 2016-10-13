#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

cargo build --target=x86_64-unknown-linux-musl --release --features=nightly
cargo build --target=x86_64-pc-windows-gnu --release --features=nightly

(
    cd image
    cargo build --target=x86_64-unknown-linux-musl --release
    cargo build --target=x86_64-pc-windows-gnu --release
)

mkdir -p target/bins
cp target/*/release/{emulator,emulator.exe,assembler,assembler.exe,disassembler,disassembler.exe} target/bins/
cp image/target/*/release/{sprite,sprite.exe} target/bins
