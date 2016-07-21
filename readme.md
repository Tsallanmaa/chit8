### Chit8
`Chit8` is a toy emulator / interpreter for the CHIP8 virtual machine written in
the Rust programming language.

Currently it only sports a very basic disassembler and a semi-complete CPU emulation without display and real keyboard.

This project has served as my introduction to the Rust language and tooling. As such, the code quality may vary.

##### Requirements
* [Rust Programming Language](https://www.rust-lang.org/en-US/downloads.html) Nightly version preferred, but any newish should work.

##### Building
Clone the repository and run `cargo build [--release]` to retrieve dependencies and build the emulator (optionally in release mode). Run `chit8.exe <path-to-rom>` to execute the emulator.

Alternatively use `cargo run <path-to-rom>` to run the emulator.

##### Tests
Use `cargo test` to run the test suite. Currently only the CPU opcodes are covered by tests.

##### Docs
Use `cargo doc` to generate documentation.

##### Acknowledgements

* [sprocketnes](https://github.com/pcwalton/sprocketnes) by Patrick Walton for inspiration and coding tips
* [Cowgod's Chip-8 Technical Reference v1.0](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
* [Mastering CHIP-8](http://mattmik.com/files/chip8/mastering/chip8.html) by Matthew Mikolay for more technical reference on implementing the emulator