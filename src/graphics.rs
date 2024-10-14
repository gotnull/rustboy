extern crate sdl2;

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use sdl2::Sdl;

const SCREEN_WIDTH: u32 = 160;
const SCREEN_HEIGHT: u32 = 144;

pub struct Graphics {
    sdl_context: Sdl,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    texture_creator: TextureCreator<WindowContext>,
    framebuffer: [u8; (SCREEN_WIDTH * SCREEN_HEIGHT * 3) as usize], // 3 bytes per pixel (RGB)
    event_pump: sdl2::EventPump,                                    // Add event handling
}

impl Graphics {
    pub fn new() -> Self {
        // Initialize SDL2
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        // Create the window
        let window = video_subsystem
            .window("GBC Emulator", SCREEN_WIDTH * 4, SCREEN_HEIGHT * 4)
            .position_centered()
            .build()
            .unwrap();

        // Create the canvas and texture creator
        let mut canvas = window.into_canvas().build().unwrap();
        let texture_creator = canvas.texture_creator();

        // Set the canvas color (black)
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        // Event pump for handling SDL events
        let event_pump = sdl_context.event_pump().unwrap();

        Self {
            sdl_context,
            canvas,
            texture_creator,
            framebuffer: [0; (SCREEN_WIDTH * SCREEN_HEIGHT * 3) as usize], // Initialize framebuffer to black
            event_pump,                                                    // Initialize event pump
        }
    }

    // Render the current frame from the framebuffer
    pub fn render(&mut self) {
        let mut texture = self
            .texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();

        texture
            .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                buffer.copy_from_slice(&self.framebuffer);
            })
            .unwrap();

        // Clear the canvas and copy the texture to it
        self.canvas.clear();
        self.canvas
            .copy(
                &texture,
                None,
                Some(Rect::new(0, 0, SCREEN_WIDTH * 4, SCREEN_HEIGHT * 4)),
            )
            .unwrap();
        self.canvas.present();
    }

    // Example: Set a pixel in the framebuffer (used for testing)
    pub fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) {
        let idx = ((y * SCREEN_WIDTH + x) * 3) as usize;
        self.framebuffer[idx] = r;
        self.framebuffer[idx + 1] = g;
        self.framebuffer[idx + 2] = b;
    }

    // Poll for SDL2 events and return whether to quit
    pub fn handle_events(&mut self) -> bool {
        for event in self.event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => {
                    return true; // Signal to quit the emulator
                }
                _ => {}
            }
        }
        false
    }

    // Convert color ID to RGB (using placeholder palette)
    fn get_color_from_palette(&self, color_id: u8) -> (u8, u8, u8) {
        match color_id {
            0 => (255, 255, 255), // White
            1 => (192, 192, 192), // Light gray
            2 => (96, 96, 96),    // Dark gray
            3 => (0, 0, 0),       // Black
            _ => (255, 255, 255), // Default to white
        }
    }

    // Render a single tile at the given position in the framebuffer
    pub fn render_tile(&mut self, tile_data: [u8; 16], x: u32, y: u32) {
        for ty in 0..8 {
            let byte1 = tile_data[ty * 2];
            let byte2 = tile_data[ty * 2 + 1];
            for tx in 0..8 {
                let color_bit = 7 - tx;
                let color_id = ((byte1 >> color_bit) & 1) | (((byte2 >> color_bit) & 1) << 1);
                let (r, g, b) = self.get_color_from_palette(color_id);
                self.set_pixel(x + tx as u32, y + ty as u32, r, g, b);
            }
        }
    }

    // Render the tile map to the screen (for now, render the first 20x18 tiles from the tile map)
    pub fn render_tile_map(&mut self, mmu: &crate::mmu::MMU) {
        for tile_y in 0..18 {
            for tile_x in 0..20 {
                let tile_index = mmu.get_tile_index_from_map(tile_x, tile_y); // Fetch the tile index from the map
                let tile_data = mmu.get_tile_data(tile_index); // Fetch the tile data using the tile index

                // Print which tile index is being rendered
                // println!(
                //     "Rendering tile at position ({}, {}): Tile Index = {}",
                //     tile_x, tile_y, tile_index
                // );

                self.render_tile(tile_data, (tile_x * 8) as u32, (tile_y * 8) as u32);
            }
        }
    }

    // Fetch tile index from the background map
    fn fetch_tile_index(&self, tile_x: u32, tile_y: u32, mmu: &crate::mmu::MMU) -> u8 {
        let tile_map_base = 0x9800; // Example address for background map
        let map_x = tile_x as usize;
        let map_y = tile_y as usize;

        // Get the tile index from the background map in VRAM
        mmu.read_byte(tile_map_base + (map_y * 32 + map_x) as u16)
    }

    // Fetch tile data from VRAM based on the tile index
    fn fetch_tile_data(&self, tile_index: u8, mmu: &crate::mmu::MMU) -> [u8; 16] {
        let tile_data_base = 0x8000; // Example address for tile data
        let tile_address = tile_data_base + (tile_index as u16) * 16;

        // Fetch 16 bytes of tile data (8x8 pixels, 2 bytes per row)
        let mut tile_data = [0; 16];
        for i in 0..16 {
            tile_data[i] = mmu.read_byte(tile_address + i as u16);
        }
        tile_data
    }
}
