# Flyer Animals Together

This project implements a Rust-based overlay for "Climber Animals: Together".
It consists of two parts:
1. `overlay`: A DLL that hooks into the game's rendering loop (DirectX 11) to draw a notification box.
2. `injector`: A command-line tool to inject the DLL into the running game process.

## Prerequisites
- Rust (installed via rustup)
- Windows OS

## Building
Run the following command in the project root:
```bash
cargo build --release
```

## Usage

### Run from Release
1. Go to the [Latest Releases](https://github.com/MemoryXL/Flyer-Animals-Together/releases/latest) page.
2. Download `YAT.zip`.
3. Extract the zip file.
4. Start the game "Climber Animals: Together".
5. Run `injector.exe`.
6. Click `RShift` to toggle the overlay.

### Run from Source
1. Start the game "Climber Animals: Together".
2. Run the following command in the project root:
   ```bash
   cargo build --release
   ```
2. Run the injector:
   ```bash
   ./target/release/injector.exe
   ```
