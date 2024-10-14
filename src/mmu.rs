pub struct MMU {
    pub memory: [u8; 0x10000], // 64KB memory
}

impl MMU {
    pub fn new() -> Self {
        Self {
            memory: [0; 0x10000], // Initialize memory to 0
        }
    }

    pub fn load_rom(&mut self, path: String) {
        let rom_data = std::fs::read(path).expect("Failed to read ROM file");
        let memory_size = self.memory.len();
        let rom_size = rom_data.len();

        for (i, byte) in rom_data.iter().enumerate() {
            if i < memory_size {
                self.memory[i] = *byte;
            } else {
                println!("Warning: ROM size exceeds memory limit, truncating...");
                break;
            }
        }

        println!("ROM loaded, size: {} bytes", rom_size);
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value;
    }

    // Method to read tile data from VRAM
    pub fn get_tile_data(&self, tile_index: u16) -> [[u8; 8]; 8] {
        let base_address = 0x8000 + (tile_index * 16);
        let mut tile_data = [[0u8; 8]; 8]; // 8x8 tile

        for y in 0..8 {
            let byte1 = self.read_byte(base_address + y * 2); // First byte of the row
            let byte2 = self.read_byte(base_address + y * 2 + 1); // Second byte of the row

            for x in 0..8 {
                let bit1 = (byte1 >> (7 - x)) & 1; // Get the (7-x)th bit from byte1
                let bit2 = (byte2 >> (7 - x)) & 1; // Get the (7-x)th bit from byte2
                let color_id = (bit2 << 1) | bit1; // Combine the two bits to get the color index (0-3)
                tile_data[y as usize][x as usize] = color_id; // Store the color ID in the tile data
            }
        }
        tile_data
    }
}
