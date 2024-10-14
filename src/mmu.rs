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

        // Debugging write operations to the tile map region (0x9800-0x9BFF)
        if addr >= 0x9800 && addr < 0x9C00 {
            println!(
                "Tile Map Write: Addr = 0x{:04X}, Value = 0x{:02X}",
                addr, value
            );
        }
    }

    pub fn get_tile_index_from_map(&self, x: u16, y: u16) -> u16 {
        let map_address = 0x9800 + y * 32 + x; // Each row has 32 tiles
        let tile_index = self.read_byte(map_address) as u16;

        // Debug: Print the content of the tile map from 0x9800 to 0x9BFF
        // for addr in 0x9800..0x9C00 {
        //     println!(
        //         "Tile map address 0x{:04X}: 0x{:02X}",
        //         addr, self.memory[addr as usize]
        //     );
        // }

        // println!(
        //     "Tile Index at ({}, {}): {} from address: 0x{:04X}",
        //     x, y, tile_index, map_address
        // ); // Debug output to check tile indices

        tile_index
    }

    pub fn get_tile_data(&self, tile_index: u16) -> [u8; 16] {
        let start_addr = 0x8000 + (tile_index * 16) as usize;
        let mut tile_data = [0u8; 16];
        tile_data.copy_from_slice(&self.memory[start_addr..start_addr + 16]);

        // Print the tile data for debugging
        // println!(
        //     "Tile Data for tile index {} at VRAM address 0x{:04X}: {:?}",
        //     tile_index, start_addr, tile_data
        // );

        tile_data
    }
}
