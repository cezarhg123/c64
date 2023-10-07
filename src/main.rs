pub mod emulator;

fn main() {
    let mut args = std::env::args();
    let bin_filename = args.nth(1).unwrap();
    let bin = std::fs::read(bin_filename).unwrap();

    let mut emulator = emulator::Emulator::new(&bin);

    emulator.run();
}
