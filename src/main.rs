extern crate chip8;

use chip8::Emulator;

fn main() {
    let mut emulator = Emulator::new();
    emulator.run();
}
