pub struct CPU {
    pub registers: Registers,
    pub interrupts_enabled: bool, // New field to track interrupts
}

impl CPU {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            interrupts_enabled: true, // Start with interrupts enabled by default
        }
    }

    pub fn step(&mut self, mmu: &mut crate::mmu::MMU) {
        let pc = self.registers.pc;
        let opcode = mmu.read_byte(pc);

        // Log the current opcode and PC
        // println!("PC: 0x{:04X}, Opcode: 0x{:02X}", pc, opcode);

        // Execute the opcode
        self.decode_and_execute(opcode, mmu);

        // // Log the state of the registers after execution
        // println!(
        //     "A: 0x{:02X}, F: 0x{:02X}, BC: 0x{:04X}, DE: 0x{:04X}, HL: 0x{:04X}, SP: 0x{:04X}",
        //     self.registers.a,
        //     self.registers.f,
        //     self.registers.bc(),
        //     self.registers.de(),
        //     self.registers.hl(),
        //     self.registers.sp,
        // );
    }

    fn decode_and_execute(&mut self, opcode: u8, mmu: &mut crate::mmu::MMU) {
        match opcode {
            0x00 => {
                // NOP - No operation
                // // println!("NOP instruction");
                self.registers.pc += 1;
            }
            0x01 => {
                let value = self.read_word(mmu);
                self.registers.set_bc(value);
                self.registers.pc += 3; // 1 byte for opcode + 2 bytes for immediate value
            }
            0x04 => {
                self.registers.b = self.increment_byte(self.registers.b);
                self.registers.pc += 1;
            }
            0x11 => {
                let value = self.read_word(mmu);
                self.registers.set_de(value);
                self.registers.pc += 3;
            }
            0xD6 => {
                let value = mmu.read_byte(self.registers.pc + 1);
                let result = self.registers.a.wrapping_sub(value);
                self.set_flags(
                    result == 0,
                    true,
                    (self.registers.a & 0xF) < (value & 0xF),
                    Some(self.registers.a < value),
                );
                self.registers.a = result;
                self.registers.pc += 2;
            }
            0x06 => {
                self.registers.b = mmu.read_byte(self.registers.pc + 1);
                self.registers.pc += 2;
            }
            0xC3 => {
                // JP nn - Jump to address nn
                let addr = self.read_word(mmu);
                // println!("JP instruction - Jumping to address 0x{:04X}", addr);
                self.registers.pc = addr;
            }
            0x3E => {
                // LD A, n - Load immediate 8-bit value into A
                let value = mmu.read_byte(self.registers.pc + 1);
                // println!("LD A, 0x{:02X}", value);
                self.registers.a = value;
                self.registers.pc += 2;
            }
            0x7C => {
                // LD A, H - Load the value of register H into A
                // println!("LD A, H");
                self.registers.a = self.registers.h;
                self.registers.pc += 1;
            }
            0x7D => {
                // LD A, L - Load the value of register L into A
                // println!("LD A, L");
                self.registers.a = self.registers.l;
                self.registers.pc += 1;
            }
            0xE0 => {
                // LDH (n), A - Store A into memory at (0xFF00 + n)
                let offset = mmu.read_byte(self.registers.pc + 1);
                let addr = 0xFF00 + offset as u16;
                mmu.write_byte(addr, self.registers.a);
                // println!("LDH (0xFF00 + 0x{:02X}), A", offset);
                self.registers.pc += 2;
            }
            0x26 => {
                // LD H, n - Load immediate 8-bit value into H
                let value = mmu.read_byte(self.registers.pc + 1);
                // println!("LD H, 0x{:02X}", value);
                self.registers.h = value;
                self.registers.pc += 2;
            }
            0x31 => {
                // LD SP, d16 - Load 16-bit immediate into SP
                let value = self.read_word(mmu);
                self.registers.sp = value;
                // println!("LD SP, 0x{:04X}", value);
                self.registers.pc += 3; // Opcode + 2-byte immediate
            }
            0x18 => {
                // JR n - Jump relative to current PC by signed 8-bit value n
                let offset = mmu.read_byte(self.registers.pc + 1) as i8;
                // println!("JR {}", offset);
                self.registers.pc = (self.registers.pc as i16 + offset as i16 + 2) as u16;
            }
            0xFE => {
                // CP n - Compare A with immediate 8-bit value
                let value = mmu.read_byte(self.registers.pc + 1);
                let result = self.registers.a.wrapping_sub(value);
                // println!("CP A, 0x{:02X}", value);
                // Set flags: Z if A == value, N=1, H if borrow from bit 4, C if A < value
                self.set_flags(
                    result == 0,
                    true,
                    (self.registers.a & 0xF) < (value & 0xF),
                    Some(self.registers.a < value),
                );
                self.registers.pc += 2;
            }
            0xFF => {
                // RST 38H - Call to address 0x38
                self.registers.sp = self.registers.sp.wrapping_sub(2);
                mmu.write_byte(self.registers.sp, (self.registers.pc & 0xFF) as u8);
                mmu.write_byte(self.registers.sp + 1, (self.registers.pc >> 8) as u8);
                self.registers.pc = 0x38;
                // println!("RST 38H - Jumping to 0x38");
            }
            0x8F => {
                // ADC A, A - Add A + A + carry flag
                let carry = if self.registers.f & 0x10 != 0 { 1 } else { 0 };
                let result = self
                    .registers
                    .a
                    .wrapping_add(self.registers.a)
                    .wrapping_add(carry);
                self.set_flags(
                    result == 0,
                    false,
                    (self.registers.a & 0xF) + (self.registers.a & 0xF) + carry > 0xF,
                    Some(result < self.registers.a),
                );
                self.registers.a = result;
                // println!("ADC A, A");
                self.registers.pc += 1;
            }
            0x0B => {
                // DEC BC - Decrement BC register
                let bc = self.registers.bc().wrapping_sub(1);
                self.registers.set_bc(bc);
                // println!("DEC BC");
                self.registers.pc += 1;
            }
            0x21 => {
                // LD HL, d16 - Load 16-bit immediate into HL
                let value = self.read_word(mmu);
                self.registers.set_hl(value);
                // println!("LD HL, 0x{:04X}", value);
                self.registers.pc += 3;
            }
            0xDF => {
                // RST 18H - Call to address 0x18
                self.registers.sp = self.registers.sp.wrapping_sub(2);
                mmu.write_byte(self.registers.sp, (self.registers.pc & 0xFF) as u8);
                mmu.write_byte(self.registers.sp + 1, (self.registers.pc >> 8) as u8);
                self.registers.pc = 0x18;
                // println!("RST 18H - Jumping to 0x18");
            }
            0xF3 => {
                // DI - Disable interrupts
                self.interrupts_enabled = false;
                // println!("DI - Disable interrupts");
                self.registers.pc += 1;
            }
            0xFB => {
                // EI - Enable interrupts
                self.interrupts_enabled = true;
                // println!("EI - Enable interrupts");
                self.registers.pc += 1;
            }
            0x1B => {
                let de = self.registers.de().wrapping_sub(1);
                self.registers.set_de(de);
                self.registers.pc += 1;
            }
            0x7A => {
                self.registers.a = self.registers.d;
                self.registers.pc += 1;
            }
            0xB3 => {
                self.registers.a |= self.registers.e;
                self.set_flags(self.registers.a == 0, false, false, Some(false));
                self.registers.pc += 1;
            }
            0xCD => {
                // CALL nn - Call to address nn, push PC to stack
                let addr = self.read_word(mmu);
                let sp = self.registers.sp.wrapping_sub(2);
                mmu.write_byte(sp + 1, (self.registers.pc >> 8) as u8); // Push high byte of PC
                mmu.write_byte(sp, self.registers.pc as u8); // Push low byte of PC
                                                             // println!(
                                                             //     "CALL 0x{:04X} - Pushing PC 0x{:04X} onto stack at SP 0x{:04X}",
                                                             //     addr, self.registers.pc, sp
                                                             // );
                self.registers.sp = sp;
                self.registers.pc = addr;
            }
            0xC9 => {
                // RET - Return from subroutine, pop PC from stack
                let low = mmu.read_byte(self.registers.sp) as u16;
                let high = mmu.read_byte(self.registers.sp + 1) as u16;
                let addr = (high << 8) | low;
                println!(
                    "RET - Returning to 0x{:04X} from stack address 0x{:04X}",
                    addr, self.registers.sp
                );
                self.registers.sp += 2;
                self.registers.pc = addr;
            }
            0x32 => {
                // LD (HL-), A - Store A into memory at HL, then decrement HL
                let addr = self.registers.hl();
                mmu.write_byte(addr, self.registers.a);
                println!(
                    "LD (HL-), A - Writing A (0x{:02X}) to address 0x{:04X}",
                    self.registers.a, addr
                );
                self.registers.set_hl(addr.wrapping_sub(1));
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
                let addr = self.read_word(mmu); // Read the 16-bit address
                mmu.write_byte(addr, self.registers.a);
                // println!("LD (0x{:04X}), A: 0x{:02X}", addr, self.registers.a);
                self.registers.pc += 3; // Increment PC past the instruction
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
            0xF1 => {
                // POP AF - Pop two bytes from the stack into AF register pair
                let low = mmu.read_byte(self.registers.sp);
                let high = mmu.read_byte(self.registers.sp + 1);
                self.registers.set_af(((high as u16) << 8) | low as u16);
                // println!("POP AF");
                self.registers.sp += 2;
                self.registers.pc += 1;
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
