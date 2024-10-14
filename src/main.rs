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

        // Execute CPU cycles (placeholder for now)
        cpu.step(&mut mmu);

        // For testing: Set a few pixels to random colors
        graphics.set_pixel(10, 10, 255, 0, 0); // Red
        graphics.set_pixel(20, 20, 0, 255, 0); // Green
        graphics.set_pixel(30, 30, 0, 0, 255); // Blue

        // Render the graphics to the screen
        graphics.render(&mmu);

        // Update audio and input (placeholders for now)
        audio.update();
        input.poll();

        // Limit frame rate to ~60 FPS
        let frame_time = frame_start.elapsed();
        if frame_time < frame_duration {
            std::thread::sleep(frame_duration - frame_time);
        }
    }
}
