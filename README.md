# CHIP-8 Emulator With Rust and SDL

## Prerequisites
Please ensure you have [SDL](https://www.libsdl.org/) installed on your machine.

```shell
# MacOS
brew install sdl2 sdl2_image sdl2_ttf
# Add to ~/.<zshrc/bashrc/etc> if not already present
export LIBRARY_PATH="$LIBRARY_PATH:/opt/homebrew/lib"

# Ubuntu/Debian (note the -dev suffix)
sudo apt-get install libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev
```

## Running
You can build this project with `cargo build` and then run with 
`./target/debug/afoley-chip8 --rom ./roms/<insert_rom>.ch8`.

## Example
Please see the example below. The command line invocation is:

```shell
./target/debug/afoley-chip8 --rom ./roms/tetris.ch8
```


https://github.com/afoley587/rust-chip8/assets/54959686/90f4bf77-011a-42d6-b162-27b3ebc56eae

