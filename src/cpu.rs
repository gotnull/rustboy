pub struct CPU {
    pub registers: Registers,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
        }
    }

    pub fn step(&mut self, mmu: &mut crate::mmu::MMU) {
        // Fetch the current opcode from memory
        let opcode = mmu.read_byte(self.registers.pc);
        println!("PC: 0x{:04X}, Opcode: 0x{:02X}", self.registers.pc, opcode);

        // Execute the opcode
        self.decode_and_execute(opcode, mmu);

        // Print the state of the registers after execution
        println!(
            "A: 0x{:02X}, F: 0x{:02X}, BC: 0x{:04X}, DE: 0x{:04X}, HL: 0x{:04X}, SP: 0x{:04X}",
            self.registers.a,
            self.registers.f,
            self.registers.bc(),
            self.registers.de(),
            self.registers.hl(),
            self.registers.sp,
        );
    }

    fn decode_and_execute(&mut self, opcode: u8, mmu: &mut crate::mmu::MMU) {
        match opcode {
            0x00 => {
                // NOP - No operation
                self.registers.pc += 1;
            }
            0x3F => {
                // CCF - Complement Carry Flag
                let carry = self.registers.f & 0x10 != 0;
                self.registers.f &= !0x40; // Reset subtract flag
                self.registers.f &= !0x20; // Reset half-carry flag
                if carry {
                    self.registers.f &= !0x10; // Clear carry flag
                } else {
                    self.registers.f |= 0x10; // Set carry flag
                }
                self.registers.pc += 1;
            }
            0x3C => {
                // INC A - Increment register A
                self.registers.a = self.increment_byte(self.registers.a);
                self.registers.pc += 1;
            }
            0xEA => {
                // LD (a16), A - Store A at 16-bit address
                let addr = self.read_word(mmu);
                mmu.write_byte(addr, self.registers.a);
                self.registers.pc += 3; // Opcode + 2-byte address
            }
            0x20 => {
                // JR NZ, r8 - Jump relative if Zero flag is not set
                let offset = mmu.read_byte(self.registers.pc + 1) as i8;
                if self.registers.f & 0x80 == 0 {
                    self.registers.pc =
                        (self.registers.pc as i16).wrapping_add(offset as i16 + 2) as u16;
                } else {
                    self.registers.pc += 2; // Skip the instruction if condition is not met
                }
            }
            0x78 => {
                // LD A, B - Load B into A
                self.registers.a = self.registers.b;
                self.registers.pc += 1;
            }
            0x37 => {
                // SCF - Set Carry Flag
                self.registers.f &= !0x40; // Reset subtract flag
                self.registers.f &= !0x20; // Reset half-carry flag
                self.registers.f |= 0x10; // Set carry flag
                self.registers.pc += 1;
            }
            0xE6 => {
                // AND d8 - AND A with immediate byte
                let value = mmu.read_byte(self.registers.pc + 1);
                self.registers.a &= value;
                self.set_flags(self.registers.a == 0, false, true, Some(false));
                self.registers.pc += 2;
            }
            0xF6 => {
                // OR d8 - OR A with immediate byte
                let value = mmu.read_byte(self.registers.pc + 1);
                self.registers.a |= value;
                self.set_flags(self.registers.a == 0, false, false, Some(false));
                self.registers.pc += 2;
            }
            0xCB => {
                // Prefix CB instruction - Handle two-byte opcodes
                let next_opcode = mmu.read_byte(self.registers.pc + 1);
                self.execute_cb_opcode(next_opcode, mmu);
                self.registers.pc += 2;
            }
            _ => {
                // Handle unknown opcodes
                println!(
                    "Unknown opcode: 0x{:02X}, at PC: 0x{:04X}",
                    opcode, self.registers.pc
                );
                self.registers.pc += 1;
            }
        }
    }

    fn execute_cb_opcode(&mut self, opcode: u8, mmu: &mut crate::mmu::MMU) {
        match opcode {
            0x11 => {
                // Example CB opcode - RL C (Rotate left through carry)
                let carry = self.registers.c & 0x80 != 0;
                self.registers.c =
                    self.registers.c << 1 | if self.registers.f & 0x10 != 0 { 1 } else { 0 };
                self.set_flags(self.registers.c == 0, false, false, Some(carry));
            }
            _ => {
                // Handle unknown CB opcodes
                println!(
                    "Unknown CB-prefixed opcode: 0x{:02X}, at PC: 0x{:04X}",
                    opcode, self.registers.pc
                );
            }
        }
    }

    fn increment_byte(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.set_flags(result == 0, false, (value & 0xF) == 0, None);
        result
    }

    fn decrement_byte(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.set_flags(result == 0, true, (value & 0xF) == 0xF, None);
        result
    }

    fn set_flags(&mut self, z: bool, n: bool, h: bool, c: Option<bool>) {
        self.registers.f = 0;
        if z {
            self.registers.f |= 0x80;
        } // Zero flag
        if n {
            self.registers.f |= 0x40;
        } // Subtract flag
        if h {
            self.registers.f |= 0x20;
        } // Half-carry flag
        if let Some(carry) = c {
            if carry {
                self.registers.f |= 0x10;
            } // Carry flag
        }
    }

    fn read_word(&self, mmu: &mut crate::mmu::MMU) -> u16 {
        let low = mmu.read_byte(self.registers.pc + 1) as u16;
        let high = mmu.read_byte(self.registers.pc + 2) as u16;
        (high << 8) | low
    }
}

pub struct Registers {
    pub a: u8,
    pub f: u8, // Flags register
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16, // Stack Pointer
    pub pc: u16, // Program Counter
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0xFFFE, // Initial stack pointer value
            pc: 0x100,  // Start after BIOS
        }
    }

    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = value as u8 & 0xF0; // Lower nibble of F is always 0
    }

    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }

    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }
}
