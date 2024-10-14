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

    let frame_duration = std::time::Duration::from_millis(16); // Roughly 60 FPS

    // Main emulation loop
    loop {
        let frame_start = std::time::Instant::now();

        // Handle events (quit if needed)
        if graphics.handle_events() {
            break; // Exit the loop if the user closes the window
        }

        // Execute CPU cycles
        cpu.step(&mut mmu);

        // Render the tile map from VRAM
        graphics.render_tile_map(&mmu);

        // Render the graphics to the screen
        graphics.render();

        // Update audio and input
        audio.update();
        input.poll();

        // Limit frame rate to ~60 FPS
        let frame_time = frame_start.elapsed();
        if frame_time < frame_duration {
            std::thread::sleep(frame_duration - frame_time);
        }
    }
}
