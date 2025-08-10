use crate::system::System;

#[derive(Default, Debug)]
pub struct CPU {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,

    zf: bool,
    nf: bool,
    hf: bool,
    cf: bool,
    
    sp: u16,
    pc: u16,

    ime: bool,
    pub halted: bool,
}

impl CPU {
    pub fn new() -> Self {
        let mut cpu = Self::default();
        cpu.pc = 0x100;
        cpu.sp = 0xfffe;
        cpu
    }

    fn af(&self) -> u16 {
        ((self.a as u16) << 8)
        | ((self.zf as u16) << 7)
        | ((self.nf as u16) << 6)
        | ((self.hf as u16) << 5)
        | ((self.cf as u16) << 4)
    }

    fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.zf = ((value >> 7) & 0x1) != 0;
        self.nf = ((value >> 6) & 0x1) != 0;
        self.hf = ((value >> 5) & 0x1) != 0;
        self.cf = ((value >> 4) & 0x1) != 0;
    }

    fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | self.c as u16
    }

    fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xff) as u8;
    }

    fn de(&self) -> u16 {
        ((self.d as u16) << 8) | self.e as u16
    }

    fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0xff) as u8;
    }

    fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | self.l as u16
    }

    fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0xff) as u8;
    }

    fn hl_inc(&mut self) -> u16 {
        let hl = self.hl();
        self.set_hl(hl.wrapping_add(1));
        hl
    }

    fn hl_dec(&mut self) -> u16 {
        let hl = self.hl();
        self.set_hl(hl.wrapping_sub(1));
        hl
    }

    fn fetch8(&mut self, system: &mut System) -> u8 {
        let byte = system.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    fn fetch16(&mut self, system: &mut System) -> u16 {
        let word = system.read(self.pc) as u16
                   | (system.read(self.pc.wrapping_add(1)) as u16) << 8;
        self.pc = self.pc.wrapping_add(2);
        word
    }

    fn push16(&mut self, system: &mut System, value: u16) {
        self.sp = self.sp.wrapping_sub(1);
        system.write(self.sp, (value >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        system.write(self.sp, (value & 0xFF) as u8);
    }

    fn pop16(&mut self, system: &mut System) -> u16 {
        let word = system.read(self.sp) as u16
                   | (system.read(self.sp.wrapping_add(1)) as u16) << 8;
        self.sp = self.sp.wrapping_add(2);
        word
    }

    fn compare(&mut self, value: u8) {
        self.nf = true;
        self.zf = self.a == value;
        self.hf = false; // TODO
        self.cf = self.a < value;
    }

    fn add16(&mut self, a: u16, b: u16) -> u16 {
        let result = a as u32 + b as u32;
        self.cf = result & 0xFFFF0000 != 0;
        self.nf = false;
        self.hf = false; // TODO
        result as u16
    }

    fn inc(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.zf = result == 0;
        self.nf = false;
        self.hf = false; // TODO
        result
    }

    fn dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.zf = result == 0;
        self.nf = true;
        self.hf = false; // TODO
        result
    }

    fn add(&mut self, value: u8) {
        let result = self.a as u16 + value as u16;
        self.a = result as u8;
        self.zf = self.a == 0;
        self.nf = false;
        self.hf = false; // TODO
        self.cf = result & 0xFF00 != 0;
    }

    fn and(&mut self, value: u8) {
        self.a &= value;
        self.zf = self.a == 0;
        self.nf = false;
        self.hf = true;
        self.cf = false;
    }

    fn or(&mut self, value: u8) {
        self.a |= value;
        self.zf = self.a == 0;
        self.nf = false;
        self.hf = false;
        self.cf = false;
    }

    pub fn execute_next(&mut self, system: &mut System) {
        if self.ime {
            let interrupt_enable = system.read(0xFFFF);
            let interrupt_flags = system.read(0xFF0F);
            let interrupt_requests = (interrupt_enable & interrupt_flags) & 0x1F;

            if interrupt_requests != 0 {
                let interrupt_number = interrupt_requests.trailing_zeros();
                system.write(0xFF0F, interrupt_flags & !(1 << interrupt_number));
                
                self.push16(system, self.pc);
                self.pc = 0x40 + 8 * interrupt_number as u16;
                self.ime = false;
                self.halted = false;
            }
        }

        if self.halted {
            return;
        }

        match self.fetch8(system) {
            0x00 => (),
            0x10 | 0x76 => self.halted = true,
            0x2F => {
                self.a = !self.a;
                self.nf = true;
                self.hf = true;
            },
            0x37 => {
                self.nf = false;
                self.hf = false;
                self.cf = true;
            },
            0x3F => {
                self.nf = false;
                self.hf = false;
                self.cf = !self.cf;
            },
            0xF3 => self.ime = false,
            0xFB => self.ime = true,

            // 16-bit immediate loads (LD)
            0x01 => {
                let word = self.fetch16(system);
                self.set_bc(word);
            },
            0x11 => {
                let word = self.fetch16(system);
                self.set_de(word);
            },
            0x21 => {
                let word = self.fetch16(system);
                self.set_hl(word);
            },
            0x31 => self.sp = self.fetch16(system),

            // 8-bit immediate loads (LD)
            0x06 => self.b = self.fetch8(system),
            0x0E => self.c = self.fetch8(system),
            0x16 => self.d = self.fetch8(system),
            0x1E => self.e = self.fetch8(system),
            0x26 => self.h = self.fetch8(system),
            0x2E => self.l = self.fetch8(system),
            0x36 => {
                let byte = self.fetch8(system);
                system.write(self.hl(), byte);
            },
            0x3E => self.a = self.fetch8(system),

            // 8-bit memory stores (LD)
            0x02 => system.write(self.bc(), self.a),
            0x12 => system.write(self.de(), self.a),
            0x22 => system.write(self.hl_inc(), self.a),
            0x32 => system.write(self.hl_dec(), self.a),
            0x70 => system.write(self.hl(), self.b),
            0x71 => system.write(self.hl(), self.c),
            0x72 => system.write(self.hl(), self.d),
            0x73 => system.write(self.hl(), self.e),
            0x74 => system.write(self.hl(), self.h),
            0x75 => system.write(self.hl(), self.l),
            0x77 => system.write(self.hl(), self.a),
            0xE0 => {
                let addr = 0xFF00 | self.fetch8(system) as u16;
                system.write(addr, self.a);
            },
            0xE2 => system.write(0xFF00 | self.c as u16, self.a),
            0xEA => {
                let addr = self.fetch16(system);
                system.write(addr, self.a);
            },

            // 8-bit memory loads (LD)
            0x0A => self.a = system.read(self.bc()),
            0x1A => self.a = system.read(self.de()),
            0x2A => self.a = system.read(self.hl_inc()),
            0x3A => self.a = system.read(self.hl_dec()),
            0x46 => self.b = system.read(self.hl()),
            0x4E => self.c = system.read(self.hl()),
            0x56 => self.d = system.read(self.hl()),
            0x5E => self.e = system.read(self.hl()),
            0x66 => self.h = system.read(self.hl()),
            0x6E => self.l = system.read(self.hl()),
            0x7E => self.a = system.read(self.hl()),
            0xF0 => {
                let addr = 0xFF00 | self.fetch8(system) as u16;
                self.a = system.read(addr);
            }
            0xF2 => self.a = system.read(0xFF00 | self.c as u16),
            0xFA => {
                let addr = self.fetch16(system);
                self.a = system.read(addr);
            },

            // 8-bit register-register loads (LD)
            0x47 => self.b = self.a,
            0x78 => self.a = self.b,

            // Jump relative (JR)
            0x18 => {
                let offset = self.fetch8(system) as i8 as u16;
                self.pc = self.pc.wrapping_add(offset);
            },
            0x20 => {
                let offset = self.fetch8(system) as i8 as u16;
                if !self.zf {
                    self.pc = self.pc.wrapping_add(offset);
                }
            },
            0x28 => {
                let offset = self.fetch8(system) as i8 as u16;
                if self.zf {
                    self.pc = self.pc.wrapping_add(offset);
                }
            },
            0x30 => {
                let offset = self.fetch8(system) as i8 as u16;
                if !self.cf {
                    self.pc = self.pc.wrapping_add(offset);
                }
            },
            0x38 => {
                let offset = self.fetch8(system) as i8 as u16;
                if self.cf {
                    self.pc = self.pc.wrapping_add(offset);
                }
            },

            // Jump absolute (JP)
            0xC2 => if !self.zf { self.pc = self.fetch16(system) },
            0xC3 => self.pc = self.fetch16(system),
            0xCA => if self.zf { self.pc = self.fetch16(system) },
            0xD2 => if !self.cf { self.pc = self.fetch16(system) },
            0xDA => if self.cf { self.pc = self.fetch16(system) },

            // Call subroutine (CALL)
            0xC4 => {
                let addr = self.fetch16(system);
                if !self.zf {
                    self.push16(system, self.pc);
                    self.pc = addr;
                }
            },
            0xCC => {
                let addr = self.fetch16(system);
                if self.zf {
                    self.push16(system, self.pc);
                    self.pc = addr;
                }
            },
            0xCD => {
                let addr = self.fetch16(system);
                self.push16(system, self.pc);
                self.pc = addr;
            },
            0xD4 => {
                let addr = self.fetch16(system);
                if !self.cf {
                    self.push16(system, self.pc);
                    self.pc = addr;
                }
            },
            0xDC => {
                let addr = self.fetch16(system);
                if self.cf {
                    self.push16(system, self.pc);
                    self.pc = addr;
                }
            },

            // Return from subroutine (RET)
            0xC0 => if !self.zf { self.pc = self.pop16(system) },
            0xC8 => if self.zf { self.pc = self.pop16(system) },
            0xC9 => self.pc = self.pop16(system),
            0xD0 => if !self.cf { self.pc = self.pop16(system) },
            0xD8 => if self.cf { self.pc = self.pop16(system) },
            0xD9 => {
                self.pc = self.pop16(system);
                self.ime = true;
            },

            // Compare (CP)
            0xB8 => self.compare(self.b),
            0xB9 => self.compare(self.c),
            0xBA => self.compare(self.d),
            0xBB => self.compare(self.e),
            0xBC => self.compare(self.h),
            0xBD => self.compare(self.l),
            0xBE => self.compare(system.read(self.hl())),
            0xBF => self.compare(self.a),
            0xFE => {
                let byte = self.fetch8(system);
                self.compare(byte);
            },

            // 16-bit arithmetic (INC/DEC/ADD)
            0x03 => self.set_bc(self.bc().wrapping_add(1)),
            0x0B => self.set_bc(self.bc().wrapping_sub(1)),
            0x13 => self.set_de(self.de().wrapping_add(1)),
            0x1B => self.set_de(self.de().wrapping_sub(1)),
            0x23 => self.set_hl(self.hl().wrapping_add(1)),
            0x2B => self.set_hl(self.hl().wrapping_sub(1)),
            0x33 => self.sp = self.sp.wrapping_add(1),
            0x3B => self.sp = self.sp.wrapping_sub(1),
            0x09 => {
                let result = self.add16(self.hl(), self.bc());
                self.set_hl(result);
            },
            0x19 => {
                let result = self.add16(self.hl(), self.de());
                self.set_hl(result);
            },
            0x29 => {
                let result = self.add16(self.hl(), self.hl());
                self.set_hl(result);
            },
            0x39 => {
                let result = self.add16(self.hl(), self.sp);
                self.set_hl(result);
            },
            0xE8 => {
                let offset = self.fetch8(system) as i8 as u16;
                self.sp = self.add16(self.sp, offset);
                self.zf = false;
            },

            // 8-bit increment and decrement (INC/DEC)
            0x04 => self.b = self.inc(self.b),
            0x05 => self.b = self.dec(self.b),
            0x0C => self.c = self.inc(self.c),
            0x0D => self.c = self.dec(self.c),
            0x14 => self.d = self.inc(self.d),
            0x15 => self.d = self.dec(self.d),
            0x1C => self.e = self.inc(self.e),
            0x1D => self.e = self.dec(self.e),
            0x24 => self.h = self.inc(self.h),
            0x25 => self.h = self.dec(self.h),
            0x2C => self.l = self.inc(self.l),
            0x2D => self.l = self.dec(self.l),
            0x34 => {
                let result = self.inc(system.read(self.hl()));
                system.write(self.hl(), result);
            },
            0x35 => {
                let result = self.dec(system.read(self.hl()));
                system.write(self.hl(), result);
            },
            0x3C => self.a = self.inc(self.a),
            0x3D => self.a = self.dec(self.a),

            // 8-bit addition (ADD/ADC)
            0x80 => self.add(self.b),

            // Bitwise OR (OR)
            0xB1 => self.or(self.c),

            // Bitwise AND (AND)
            0xE6 => {
                let byte = self.fetch8(system);
                self.and(byte);
            },

            opc => unimplemented!("opcode 0x{:02x} at 0x{:04x} -- {:?}", opc, self.pc.wrapping_sub(1), self),
        }
    }
}
