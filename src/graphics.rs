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
    pub fn render(&mut self, mmu: &crate::mmu::MMU) {
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

    // Convert color ID to RGB (placeholder palette)
    fn get_color_from_palette(&self, color_id: u8) -> (u8, u8, u8) {
        match color_id {
            0 => (255, 255, 255), // White
            1 => (192, 192, 192), // Light gray
            2 => (96, 96, 96),    // Dark gray
            3 => (0, 0, 0),       // Black
            _ => (255, 255, 255), // Fallback to white
        }
    }

    // Render a single tile at the given position in the framebuffer
    pub fn render_tile(&mut self, tile_data: [[u8; 8]; 8], x: u32, y: u32) {
        for ty in 0..8 {
            for tx in 0..8 {
                let color_id = tile_data[ty][tx];
                let (r, g, b) = self.get_color_from_palette(color_id);
                self.set_pixel(x + tx as u32, y + ty as u32, r, g, b);
            }
        }
    }

    // Render the tile map to the screen (for now, render the first 20x18 tiles)
    pub fn render_tile_map(&mut self, mmu: &crate::mmu::MMU) {
        for tile_y in 0..18 {
            for tile_x in 0..20 {
                let tile_index = (tile_y * 20 + tile_x) as u16; // Just a simple sequential tile index for now
                let tile_data = mmu.get_tile_data(tile_index);
                self.render_tile(tile_data, (tile_x * 8) as u32, (tile_y * 8) as u32);
            }
        }
    }
}
