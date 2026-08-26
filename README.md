# NES Emulator

## Sources
https://www.nesdev.org/obelisk-6502-guide/reference.html
https://6502.org/tutorials/6502opcodes.html
https://bugzmanov.github.io/nes_ebook/

sdl binaries: https://github.com/Rust-SDL2/rust-sdl2?tab=readme-ov-file#sdl20-development-libraries
```sh
sudo apt-get install libsdl2-dev
```

## notes

The cpu gets access to memory using three buses

address bus carries the address of the desired location
control bus notifies if it is a read or write access
data bus carries the byte of data being read or written.

The bus is wiring between components

RAM address space [0x000 .. 0x0800] (2 KiB) is mirrored thr
The bus needs to zero out the 2 highest bits of addresses.

