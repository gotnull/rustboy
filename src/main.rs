mod audio;
mod cpu;
mod graphics;
mod input;
mod mmu;

fn main() {
    // Initialize components
    let mut cpu = cpu::CPU::new();
    let mut mmu = mmu::MMU::new();
    let mut graphics = graphics::Graphics::new();
    let mut input = input::Input::new();
    let mut audio = audio::Audio::new();

    // Load the cartridge into the emulator
    let rom_path = std::env::args().nth(1).expect("Please provide a ROM file.");
    mmu.load_rom(rom_path);

    // Main emulation loop
    loop {
        // Execute CPU cycles
        cpu.step(&mut mmu);

        // Update graphics, audio and input
        graphics.render(&mmu);
        audio.update();
        input.poll();

        // Handle timing for the Gameboy Color's clock (4194304 Hz)
        std::thread::sleep(std::time::Duration::from_micros(1)); // Adjust based on cycles per frame
    }
}
