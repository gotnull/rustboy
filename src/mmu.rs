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
}
