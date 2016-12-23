# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## Added

- Add eeprom support
- Add m35fd support
- Limited C API
- clock v2 support
- Feature to use the old device id
- Remove/warn for unused labels

## Changed

- Limit the keyboard buffer to 8 item
- Move the screen convertion from the device to the render thread
- Huge improvement in the debugger interface
- Big library interface changes
- Most dependencies can be disabled
- Better backend failure handling

## [0.4.0]

### Added

- serde derives with feature `nightly`
- the assembler can generate a symbol file
- in the debugger, `b` can take an expression with labels

### Changed

- Move image related features to own crate
- Replace `--log-map` with `--log-litterals`.
- Better logging of executed instructions.
- Use generics for types::*
- New symbols structure in assembler

### Fixed

- `s` instead of `s 1` in the debugger

## [0.3.0]

### Added

- Utility to generate LEM font and palette from image
- Utility to convert an image to frame + font + palette

### Fixed

- If cascade handling
- Better screen visibility accuracy

## [0.2.0]

### Added

- Ticks per second counter for the emulator
- Add tickrate limiter
- Ability to map LOG n to human-readable strings

### Changed

- Don't drop hardware interrupts when queuing is enabled
- Better error reporting when hwi with invalid command
- Much better debugging interface with command completer

### Fixed

- Fix memory leak
- Part of the keyboard keys

### Misc

- Use `error-chain` crate
- Don't strip release binaries

## [0.1.2]

### Added

- `hook` debugger command
- Empty command in the debugger repeats the last command

### Fixed

- IF* conditions where inverted

### Changed

- Enable LTO on release

### Misc

- Add helper script to compiler on linux and windows

## [0.1.1]

### Changed

- Update metadata

## [0.1.0]

First serious release.


[Unreleased]: https://github.com/Yamakaky/dcpu/compare/0.4.0...HEAD
[0.4.0]: https://github.com/Yamakaky/dcpu/compare/0.3.0...0.4.0
[0.3.0]: https://github.com/Yamakaky/dcpu/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/Yamakaky/dcpu/compare/0.1.2...0.2.0
[0.1.2]: https://github.com/Yamakaky/dcpu/compare/0.1.1...0.1.2
[0.1.1]: https://github.com/Yamakaky/dcpu/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/Yamakaky/dcpu/tree/0.1.0
