use crate::memory::Memory;

// Flag bit positions
pub const FLAG_Z: u8 = 7;
pub const FLAG_N: u8 = 6;
pub const FLAG_H: u8 = 5;
pub const FLAG_C: u8 = 4;

#[derive(Clone, Copy)]
pub struct Register {
    val: u16,
}

impl Register {
    pub fn new(v: u16) -> Self {
        Register { val: v }
    }

    pub fn reg(&self) -> u16 {
        self.val
    }

    pub fn set_reg(&mut self, v: u16) {
        self.val = v;
    }

    pub fn hi(&self) -> u8 {
        (self.val >> 8) as u8
    }

    pub fn lo(&self) -> u8 {
        self.val as u8
    }

    pub fn set_hi(&mut self, v: u8) {
        self.val = ((v as u16) << 8) | (self.val & 0x00FF);
    }

    pub fn set_lo(&mut self, v: u8) {
        self.val = (self.val & 0xFF00) | (v as u16);
    }
}

pub struct Cpu {
    pub af: Register,
    pub bc: Register,
    pub de: Register,
    pub hl: Register,
    pub sp: u16,
    pub pc: u16,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            af: Register::new(0x01B0),
            bc: Register::new(0x0013),
            de: Register::new(0x00D8),
            hl: Register::new(0x014D),
            sp: 0xFFFE,
            pc: 0x100,
        }
    }

    fn get_flag(&self, flag: u8) -> bool {
        (self.af.lo() >> flag) & 1 == 1
    }

    fn set_flag(&mut self, flag: u8, val: bool) {
        if val {
            self.af.set_lo(self.af.lo() | (1 << flag));
        } else {
            self.af.set_lo(self.af.lo() & !(1 << flag));
        }
    }

    fn read_word(&self, mem: &Memory) -> u16 {
        let lo = mem.read_byte(self.pc) as u16;
        let hi = mem.read_byte(self.pc.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    pub fn push_word(&mut self, mem: &mut Memory, val: u16) {
        self.sp = self.sp.wrapping_sub(1);
        mem.write_byte(self.sp, (val >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        mem.write_byte(self.sp, val as u8);
    }

    fn pop_word(&mut self, mem: &Memory) -> u16 {
        let lo = mem.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        let hi = mem.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        (hi << 8) | lo
    }

    pub fn execute_next_opcode(&mut self, mem: &mut Memory) -> u32 {
        let opcode = mem.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        self.execute_opcode(opcode, mem)
    }

    fn execute_opcode(&mut self, opcode: u8, mem: &mut Memory) -> u32 {
        match opcode {
            // NOP
            0x00 => 4,

            // LD BC,nn
            0x01 => { let v = self.read_word(mem); self.pc += 2; self.bc.set_reg(v); 12 }
            // LD (BC),A
            0x02 => { mem.write_byte(self.bc.reg(), self.af.hi()); 8 }
            // INC BC
            0x03 => { self.bc.set_reg(self.bc.reg().wrapping_add(1)); 8 }
            // INC B
            0x04 => { let v = self.inc_8bit(self.bc.hi()); self.bc.set_hi(v); 4 }
            // DEC B
            0x05 => { let v = self.dec_8bit(self.bc.hi()); self.bc.set_hi(v); 4 }
            // LD B,n
            0x06 => { let n = mem.read_byte(self.pc); self.pc += 1; self.bc.set_hi(n); 8 }
            // RLCA
            0x07 => {
                let a = self.af.hi();
                let carry = a >> 7;
                let result = (a << 1) | carry;
                self.af.set_hi(result);
                self.af.set_lo(0);
                if carry == 1 { self.set_flag(FLAG_C, true); }
                4
            }
            // LD (nn),SP
            0x08 => {
                let addr = self.read_word(mem); self.pc += 2;
                mem.write_byte(addr, self.sp as u8);
                mem.write_byte(addr + 1, (self.sp >> 8) as u8);
                20
            }
            // ADD HL,BC
            0x09 => { self.add_16bit(self.bc.reg()); 8 }
            // LD A,(BC)
            0x0A => { let v = mem.read_byte(self.bc.reg()); self.af.set_hi(v); 8 }
            // DEC BC
            0x0B => { self.bc.set_reg(self.bc.reg().wrapping_sub(1)); 8 }
            // INC C
            0x0C => { let v = self.inc_8bit(self.bc.lo()); self.bc.set_lo(v); 4 }
            // DEC C
            0x0D => { let v = self.dec_8bit(self.bc.lo()); self.bc.set_lo(v); 4 }
            // LD C,n
            0x0E => { let n = mem.read_byte(self.pc); self.pc += 1; self.bc.set_lo(n); 8 }
            // RRCA
            0x0F => {
                let a = self.af.hi();
                let carry = a & 1;
                let result = (a >> 1) | (carry << 7);
                self.af.set_hi(result);
                self.af.set_lo(0);
                if carry == 1 { self.set_flag(FLAG_C, true); }
                4
            }
            // STOP
            0x10 => { self.pc += 1; 4 }
            // LD DE,nn
            0x11 => { let v = self.read_word(mem); self.pc += 2; self.de.set_reg(v); 12 }
            // LD (DE),A
            0x12 => { mem.write_byte(self.de.reg(), self.af.hi()); 8 }
            // INC DE
            0x13 => { self.de.set_reg(self.de.reg().wrapping_add(1)); 8 }
            // INC D
            0x14 => { let v = self.inc_8bit(self.de.hi()); self.de.set_hi(v); 4 }
            // DEC D
            0x15 => { let v = self.dec_8bit(self.de.hi()); self.de.set_hi(v); 4 }
            // LD D,n
            0x16 => { let n = mem.read_byte(self.pc); self.pc += 1; self.de.set_hi(n); 8 }
            // RLA
            0x17 => {
                let a = self.af.hi();
                let old_carry = if self.get_flag(FLAG_C) { 1u8 } else { 0 };
                let new_carry = a >> 7;
                let result = (a << 1) | old_carry;
                self.af.set_hi(result);
                self.af.set_lo(0);
                self.set_flag(FLAG_C, new_carry == 1);
                4
            }
            // JR n
            0x18 => {
                let n = mem.read_byte(self.pc) as i8;
                self.pc = self.pc.wrapping_add(1);
                self.pc = (self.pc as i32 + n as i32) as u16;
                12
            }
            // ADD HL,DE
            0x19 => { self.add_16bit(self.de.reg()); 8 }
            // LD A,(DE)
            0x1A => { let v = mem.read_byte(self.de.reg()); self.af.set_hi(v); 8 }
            // DEC DE
            0x1B => { self.de.set_reg(self.de.reg().wrapping_sub(1)); 8 }
            // INC E
            0x1C => { let v = self.inc_8bit(self.de.lo()); self.de.set_lo(v); 4 }
            // DEC E
            0x1D => { let v = self.dec_8bit(self.de.lo()); self.de.set_lo(v); 4 }
            // LD E,n
            0x1E => { let n = mem.read_byte(self.pc); self.pc += 1; self.de.set_lo(n); 8 }
            // RRA
            0x1F => {
                let a = self.af.hi();
                let old_carry = if self.get_flag(FLAG_C) { 1u8 } else { 0 };
                let new_carry = a & 1;
                let result = (a >> 1) | (old_carry << 7);
                self.af.set_hi(result);
                self.af.set_lo(0);
                self.set_flag(FLAG_C, new_carry == 1);
                4
            }
            // JR NZ,n
            0x20 => {
                let n = mem.read_byte(self.pc) as i8;
                self.pc = self.pc.wrapping_add(1);
                if !self.get_flag(FLAG_Z) {
                    self.pc = (self.pc as i32 + n as i32) as u16;
                    return 12;
                }
                8
            }
            // LD HL,nn
            0x21 => { let v = self.read_word(mem); self.pc += 2; self.hl.set_reg(v); 12 }
            // LD (HL+),A
            0x22 => {
                mem.write_byte(self.hl.reg(), self.af.hi());
                self.hl.set_reg(self.hl.reg().wrapping_add(1));
                8
            }
            // INC HL
            0x23 => { self.hl.set_reg(self.hl.reg().wrapping_add(1)); 8 }
            // INC H
            0x24 => { let v = self.inc_8bit(self.hl.hi()); self.hl.set_hi(v); 4 }
            // DEC H
            0x25 => { let v = self.dec_8bit(self.hl.hi()); self.hl.set_hi(v); 4 }
            // LD H,n
            0x26 => { let n = mem.read_byte(self.pc); self.pc += 1; self.hl.set_hi(n); 8 }
            // DAA
            0x27 => { self.daa(); 4 }
            // JR Z,n
            0x28 => {
                let n = mem.read_byte(self.pc) as i8;
                self.pc = self.pc.wrapping_add(1);
                if self.get_flag(FLAG_Z) {
                    self.pc = (self.pc as i32 + n as i32) as u16;
                    return 12;
                }
                8
            }
            // ADD HL,HL
            0x29 => { let v = self.hl.reg(); self.add_16bit(v); 8 }
            // LD A,(HL+)
            0x2A => {
                let v = mem.read_byte(self.hl.reg());
                self.af.set_hi(v);
                self.hl.set_reg(self.hl.reg().wrapping_add(1));
                8
            }
            // DEC HL
            0x2B => { self.hl.set_reg(self.hl.reg().wrapping_sub(1)); 8 }
            // INC L
            0x2C => { let v = self.inc_8bit(self.hl.lo()); self.hl.set_lo(v); 4 }
            // DEC L
            0x2D => { let v = self.dec_8bit(self.hl.lo()); self.hl.set_lo(v); 4 }
            // LD L,n
            0x2E => { let n = mem.read_byte(self.pc); self.pc += 1; self.hl.set_lo(n); 8 }
            // CPL
            0x2F => {
                self.af.set_hi(!self.af.hi());
                self.set_flag(FLAG_N, true);
                self.set_flag(FLAG_H, true);
                4
            }
            // JR NC,n
            0x30 => {
                let n = mem.read_byte(self.pc) as i8;
                self.pc = self.pc.wrapping_add(1);
                if !self.get_flag(FLAG_C) {
                    self.pc = (self.pc as i32 + n as i32) as u16;
                    return 12;
                }
                8
            }
            // LD SP,nn
            0x31 => { let v = self.read_word(mem); self.pc += 2; self.sp = v; 12 }
            // LD (HL-),A
            0x32 => {
                mem.write_byte(self.hl.reg(), self.af.hi());
                self.hl.set_reg(self.hl.reg().wrapping_sub(1));
                8
            }
            // INC SP
            0x33 => { self.sp = self.sp.wrapping_add(1); 8 }
            // INC (HL)
            0x34 => {
                let v = self.inc_8bit(mem.read_byte(self.hl.reg()));
                mem.write_byte(self.hl.reg(), v);
                12
            }
            // DEC (HL)
            0x35 => {
                let v = self.dec_8bit(mem.read_byte(self.hl.reg()));
                mem.write_byte(self.hl.reg(), v);
                12
            }
            // LD (HL),n
            0x36 => { let n = mem.read_byte(self.pc); self.pc += 1; mem.write_byte(self.hl.reg(), n); 12 }
            // SCF
            0x37 => {
                self.set_flag(FLAG_N, false);
                self.set_flag(FLAG_H, false);
                self.set_flag(FLAG_C, true);
                4
            }
            // JR C,n
            0x38 => {
                let n = mem.read_byte(self.pc) as i8;
                self.pc = self.pc.wrapping_add(1);
                if self.get_flag(FLAG_C) {
                    self.pc = (self.pc as i32 + n as i32) as u16;
                    return 12;
                }
                8
            }
            // ADD HL,SP
            0x39 => { self.add_16bit(self.sp); 8 }
            // LD A,(HL-)
            0x3A => {
                let v = mem.read_byte(self.hl.reg());
                self.af.set_hi(v);
                self.hl.set_reg(self.hl.reg().wrapping_sub(1));
                8
            }
            // DEC SP
            0x3B => { self.sp = self.sp.wrapping_sub(1); 8 }
            // INC A
            0x3C => { let v = self.inc_8bit(self.af.hi()); self.af.set_hi(v); 4 }
            // DEC A
            0x3D => { let v = self.dec_8bit(self.af.hi()); self.af.set_hi(v); 4 }
            // LD A,n
            0x3E => { let n = mem.read_byte(self.pc); self.pc += 1; self.af.set_hi(n); 8 }
            // CCF
            0x3F => {
                let c = !self.get_flag(FLAG_C);
                self.set_flag(FLAG_N, false);
                self.set_flag(FLAG_H, false);
                self.set_flag(FLAG_C, c);
                4
            }

            // LD B,B through LD B,A
            0x40 => 4, // LD B,B
            0x41 => { self.bc.set_hi(self.bc.lo()); 4 }
            0x42 => { self.bc.set_hi(self.de.hi()); 4 }
            0x43 => { self.bc.set_hi(self.de.lo()); 4 }
            0x44 => { self.bc.set_hi(self.hl.hi()); 4 }
            0x45 => { self.bc.set_hi(self.hl.lo()); 4 }
            0x46 => { let v = mem.read_byte(self.hl.reg()); self.bc.set_hi(v); 8 }
            0x47 => { self.bc.set_hi(self.af.hi()); 4 }

            // LD C,x
            0x48 => { self.bc.set_lo(self.bc.hi()); 4 }
            0x49 => 4, // LD C,C
            0x4A => { self.bc.set_lo(self.de.hi()); 4 }
            0x4B => { self.bc.set_lo(self.de.lo()); 4 }
            0x4C => { self.bc.set_lo(self.hl.hi()); 4 }
            0x4D => { self.bc.set_lo(self.hl.lo()); 4 }
            0x4E => { let v = mem.read_byte(self.hl.reg()); self.bc.set_lo(v); 8 }
            0x4F => { self.bc.set_lo(self.af.hi()); 4 }

            // LD D,x
            0x50 => { self.de.set_hi(self.bc.hi()); 4 }
            0x51 => { self.de.set_hi(self.bc.lo()); 4 }
            0x52 => 4, // LD D,D
            0x53 => { self.de.set_hi(self.de.lo()); 4 }
            0x54 => { self.de.set_hi(self.hl.hi()); 4 }
            0x55 => { self.de.set_hi(self.hl.lo()); 4 }
            0x56 => { let v = mem.read_byte(self.hl.reg()); self.de.set_hi(v); 8 }
            0x57 => { self.de.set_hi(self.af.hi()); 4 }

            // LD E,x
            0x58 => { self.de.set_lo(self.bc.hi()); 4 }
            0x59 => { self.de.set_lo(self.bc.lo()); 4 }
            0x5A => { self.de.set_lo(self.de.hi()); 4 }
            0x5B => 4, // LD E,E
            0x5C => { self.de.set_lo(self.hl.hi()); 4 }
            0x5D => { self.de.set_lo(self.hl.lo()); 4 }
            0x5E => { let v = mem.read_byte(self.hl.reg()); self.de.set_lo(v); 8 }
            0x5F => { self.de.set_lo(self.af.hi()); 4 }

            // LD H,x
            0x60 => { self.hl.set_hi(self.bc.hi()); 4 }
            0x61 => { self.hl.set_hi(self.bc.lo()); 4 }
            0x62 => { self.hl.set_hi(self.de.hi()); 4 }
            0x63 => { self.hl.set_hi(self.de.lo()); 4 }
            0x64 => 4, // LD H,H
            0x65 => { self.hl.set_hi(self.hl.lo()); 4 }
            0x66 => { let v = mem.read_byte(self.hl.reg()); self.hl.set_hi(v); 8 }
            0x67 => { self.hl.set_hi(self.af.hi()); 4 }

            // LD L,x
            0x68 => { self.hl.set_lo(self.bc.hi()); 4 }
            0x69 => { self.hl.set_lo(self.bc.lo()); 4 }
            0x6A => { self.hl.set_lo(self.de.hi()); 4 }
            0x6B => { self.hl.set_lo(self.de.lo()); 4 }
            0x6C => { self.hl.set_lo(self.hl.hi()); 4 }
            0x6D => 4, // LD L,L
            0x6E => { let v = mem.read_byte(self.hl.reg()); self.hl.set_lo(v); 8 }
            0x6F => { self.hl.set_lo(self.af.hi()); 4 }

            // LD (HL),x
            0x70 => { mem.write_byte(self.hl.reg(), self.bc.hi()); 8 }
            0x71 => { mem.write_byte(self.hl.reg(), self.bc.lo()); 8 }
            0x72 => { mem.write_byte(self.hl.reg(), self.de.hi()); 8 }
            0x73 => { mem.write_byte(self.hl.reg(), self.de.lo()); 8 }
            0x74 => { mem.write_byte(self.hl.reg(), self.hl.hi()); 8 }
            0x75 => { mem.write_byte(self.hl.reg(), self.hl.lo()); 8 }
            // HALT
            0x76 => { mem.halted = true; 4 }
            0x77 => { mem.write_byte(self.hl.reg(), self.af.hi()); 8 }

            // LD A,x
            0x78 => { self.af.set_hi(self.bc.hi()); 4 }
            0x79 => { self.af.set_hi(self.bc.lo()); 4 }
            0x7A => { self.af.set_hi(self.de.hi()); 4 }
            0x7B => { self.af.set_hi(self.de.lo()); 4 }
            0x7C => { self.af.set_hi(self.hl.hi()); 4 }
            0x7D => { self.af.set_hi(self.hl.lo()); 4 }
            0x7E => { let v = mem.read_byte(self.hl.reg()); self.af.set_hi(v); 8 }
            0x7F => 4, // LD A,A

            // ADD A,x
            0x80 => { let v = self.bc.hi(); self.add_8bit(v, false); 4 }
            0x81 => { let v = self.bc.lo(); self.add_8bit(v, false); 4 }
            0x82 => { let v = self.de.hi(); self.add_8bit(v, false); 4 }
            0x83 => { let v = self.de.lo(); self.add_8bit(v, false); 4 }
            0x84 => { let v = self.hl.hi(); self.add_8bit(v, false); 4 }
            0x85 => { let v = self.hl.lo(); self.add_8bit(v, false); 4 }
            0x86 => { let v = mem.read_byte(self.hl.reg()); self.add_8bit(v, false); 8 }
            0x87 => { let v = self.af.hi(); self.add_8bit(v, false); 4 }

            // ADC A,x
            0x88 => { let v = self.bc.hi(); self.add_8bit(v, true); 4 }
            0x89 => { let v = self.bc.lo(); self.add_8bit(v, true); 4 }
            0x8A => { let v = self.de.hi(); self.add_8bit(v, true); 4 }
            0x8B => { let v = self.de.lo(); self.add_8bit(v, true); 4 }
            0x8C => { let v = self.hl.hi(); self.add_8bit(v, true); 4 }
            0x8D => { let v = self.hl.lo(); self.add_8bit(v, true); 4 }
            0x8E => { let v = mem.read_byte(self.hl.reg()); self.add_8bit(v, true); 8 }
            0x8F => { let v = self.af.hi(); self.add_8bit(v, true); 4 }

            // SUB x
            0x90 => { let v = self.bc.hi(); self.sub_8bit(v, false); 4 }
            0x91 => { let v = self.bc.lo(); self.sub_8bit(v, false); 4 }
            0x92 => { let v = self.de.hi(); self.sub_8bit(v, false); 4 }
            0x93 => { let v = self.de.lo(); self.sub_8bit(v, false); 4 }
            0x94 => { let v = self.hl.hi(); self.sub_8bit(v, false); 4 }
            0x95 => { let v = self.hl.lo(); self.sub_8bit(v, false); 4 }
            0x96 => { let v = mem.read_byte(self.hl.reg()); self.sub_8bit(v, false); 8 }
            0x97 => { let v = self.af.hi(); self.sub_8bit(v, false); 4 }

            // SBC A,x
            0x98 => { let v = self.bc.hi(); self.sub_8bit(v, true); 4 }
            0x99 => { let v = self.bc.lo(); self.sub_8bit(v, true); 4 }
            0x9A => { let v = self.de.hi(); self.sub_8bit(v, true); 4 }
            0x9B => { let v = self.de.lo(); self.sub_8bit(v, true); 4 }
            0x9C => { let v = self.hl.hi(); self.sub_8bit(v, true); 4 }
            0x9D => { let v = self.hl.lo(); self.sub_8bit(v, true); 4 }
            0x9E => { let v = mem.read_byte(self.hl.reg()); self.sub_8bit(v, true); 8 }
            0x9F => { let v = self.af.hi(); self.sub_8bit(v, true); 4 }

            // AND x
            0xA0 => { let v = self.bc.hi(); self.and_8bit(v); 4 }
            0xA1 => { let v = self.bc.lo(); self.and_8bit(v); 4 }
            0xA2 => { let v = self.de.hi(); self.and_8bit(v); 4 }
            0xA3 => { let v = self.de.lo(); self.and_8bit(v); 4 }
            0xA4 => { let v = self.hl.hi(); self.and_8bit(v); 4 }
            0xA5 => { let v = self.hl.lo(); self.and_8bit(v); 4 }
            0xA6 => { let v = mem.read_byte(self.hl.reg()); self.and_8bit(v); 8 }
            0xA7 => { let v = self.af.hi(); self.and_8bit(v); 4 }

            // XOR x
            0xA8 => { let v = self.bc.hi(); self.xor_8bit(v); 4 }
            0xA9 => { let v = self.bc.lo(); self.xor_8bit(v); 4 }
            0xAA => { let v = self.de.hi(); self.xor_8bit(v); 4 }
            0xAB => { let v = self.de.lo(); self.xor_8bit(v); 4 }
            0xAC => { let v = self.hl.hi(); self.xor_8bit(v); 4 }
            0xAD => { let v = self.hl.lo(); self.xor_8bit(v); 4 }
            0xAE => { let v = mem.read_byte(self.hl.reg()); self.xor_8bit(v); 8 }
            0xAF => { let v = self.af.hi(); self.xor_8bit(v); 4 }

            // OR x
            0xB0 => { let v = self.bc.hi(); self.or_8bit(v); 4 }
            0xB1 => { let v = self.bc.lo(); self.or_8bit(v); 4 }
            0xB2 => { let v = self.de.hi(); self.or_8bit(v); 4 }
            0xB3 => { let v = self.de.lo(); self.or_8bit(v); 4 }
            0xB4 => { let v = self.hl.hi(); self.or_8bit(v); 4 }
            0xB5 => { let v = self.hl.lo(); self.or_8bit(v); 4 }
            0xB6 => { let v = mem.read_byte(self.hl.reg()); self.or_8bit(v); 8 }
            0xB7 => { let v = self.af.hi(); self.or_8bit(v); 4 }

            // CP x
            0xB8 => { let v = self.bc.hi(); self.cp_8bit(v); 4 }
            0xB9 => { let v = self.bc.lo(); self.cp_8bit(v); 4 }
            0xBA => { let v = self.de.hi(); self.cp_8bit(v); 4 }
            0xBB => { let v = self.de.lo(); self.cp_8bit(v); 4 }
            0xBC => { let v = self.hl.hi(); self.cp_8bit(v); 4 }
            0xBD => { let v = self.hl.lo(); self.cp_8bit(v); 4 }
            0xBE => { let v = mem.read_byte(self.hl.reg()); self.cp_8bit(v); 8 }
            0xBF => { let v = self.af.hi(); self.cp_8bit(v); 4 }

            // RET NZ
            0xC0 => {
                if !self.get_flag(FLAG_Z) {
                    self.pc = self.pop_word(mem);
                    return 20;
                }
                8
            }
            // POP BC
            0xC1 => { let v = self.pop_word(mem); self.bc.set_reg(v); 12 }
            // JP NZ,nn
            0xC2 => {
                let nn = self.read_word(mem); self.pc += 2;
                if !self.get_flag(FLAG_Z) { self.pc = nn; return 16; }
                12
            }
            // JP nn
            0xC3 => { let nn = self.read_word(mem); self.pc = nn; 16 }
            // CALL NZ,nn
            0xC4 => {
                let nn = self.read_word(mem); self.pc += 2;
                if !self.get_flag(FLAG_Z) {
                    self.push_word(mem, self.pc);
                    self.pc = nn;
                    return 24;
                }
                12
            }
            // PUSH BC
            0xC5 => { let v = self.bc.reg(); self.push_word(mem, v); 16 }
            // ADD A,n
            0xC6 => { let n = mem.read_byte(self.pc); self.pc += 1; self.add_8bit(n, false); 8 }
            // RST 00H
            0xC7 => { self.push_word(mem, self.pc); self.pc = 0x00; 16 }
            // RET Z
            0xC8 => {
                if self.get_flag(FLAG_Z) {
                    self.pc = self.pop_word(mem);
                    return 20;
                }
                8
            }
            // RET
            0xC9 => { self.pc = self.pop_word(mem); 16 }
            // JP Z,nn
            0xCA => {
                let nn = self.read_word(mem); self.pc += 2;
                if self.get_flag(FLAG_Z) { self.pc = nn; return 16; }
                12
            }
            // CB prefix
            0xCB => { self.execute_extended(mem) }
            // CALL Z,nn
            0xCC => {
                let nn = self.read_word(mem); self.pc += 2;
                if self.get_flag(FLAG_Z) {
                    self.push_word(mem, self.pc);
                    self.pc = nn;
                    return 24;
                }
                12
            }
            // CALL nn
            0xCD => {
                let nn = self.read_word(mem); self.pc += 2;
                self.push_word(mem, self.pc);
                self.pc = nn;
                24
            }
            // ADC A,n
            0xCE => { let n = mem.read_byte(self.pc); self.pc += 1; self.add_8bit(n, true); 8 }
            // RST 08H
            0xCF => { self.push_word(mem, self.pc); self.pc = 0x08; 16 }
            // RET NC
            0xD0 => {
                if !self.get_flag(FLAG_C) {
                    self.pc = self.pop_word(mem);
                    return 20;
                }
                8
            }
            // POP DE
            0xD1 => { let v = self.pop_word(mem); self.de.set_reg(v); 12 }
            // JP NC,nn
            0xD2 => {
                let nn = self.read_word(mem); self.pc += 2;
                if !self.get_flag(FLAG_C) { self.pc = nn; return 16; }
                12
            }
            // CALL NC,nn
            0xD4 => {
                let nn = self.read_word(mem); self.pc += 2;
                if !self.get_flag(FLAG_C) {
                    self.push_word(mem, self.pc);
                    self.pc = nn;
                    return 24;
                }
                12
            }
            // PUSH DE
            0xD5 => { let v = self.de.reg(); self.push_word(mem, v); 16 }
            // SUB n
            0xD6 => { let n = mem.read_byte(self.pc); self.pc += 1; self.sub_8bit(n, false); 8 }
            // RST 10H
            0xD7 => { self.push_word(mem, self.pc); self.pc = 0x10; 16 }
            // RET C
            0xD8 => {
                if self.get_flag(FLAG_C) {
                    self.pc = self.pop_word(mem);
                    return 20;
                }
                8
            }
            // RETI
            0xD9 => {
                self.pc = self.pop_word(mem);
                mem.interrupt_master = true;
                16
            }
            // JP C,nn
            0xDA => {
                let nn = self.read_word(mem); self.pc += 2;
                if self.get_flag(FLAG_C) { self.pc = nn; return 16; }
                12
            }
            // CALL C,nn
            0xDC => {
                let nn = self.read_word(mem); self.pc += 2;
                if self.get_flag(FLAG_C) {
                    self.push_word(mem, self.pc);
                    self.pc = nn;
                    return 24;
                }
                12
            }
            // SBC A,n
            0xDE => { let n = mem.read_byte(self.pc); self.pc += 1; self.sub_8bit(n, true); 8 }
            // RST 18H
            0xDF => { self.push_word(mem, self.pc); self.pc = 0x18; 16 }
            // LDH (n),A
            0xE0 => {
                let n = mem.read_byte(self.pc); self.pc += 1;
                mem.write_byte(0xFF00 + n as u16, self.af.hi());
                12
            }
            // POP HL
            0xE1 => { let v = self.pop_word(mem); self.hl.set_reg(v); 12 }
            // LD (0xFF00+C),A
            0xE2 => { mem.write_byte(0xFF00 + self.bc.lo() as u16, self.af.hi()); 8 }
            // PUSH HL
            0xE5 => { let v = self.hl.reg(); self.push_word(mem, v); 16 }
            // AND n
            0xE6 => { let n = mem.read_byte(self.pc); self.pc += 1; self.and_8bit(n); 8 }
            // RST 20H
            0xE7 => { self.push_word(mem, self.pc); self.pc = 0x20; 16 }
            // ADD SP,n
            0xE8 => {
                let n = mem.read_byte(self.pc) as i8 as i16;
                self.pc += 1;
                let sp = self.sp;
                self.af.set_lo(0);
                if (sp & 0xF) as i16 + (n & 0xF) > 0xF {
                    self.set_flag(FLAG_H, true);
                }
                if (sp & 0xFF) as i16 + (n & 0xFF) > 0xFF {
                    self.set_flag(FLAG_C, true);
                }
                self.sp = (sp as i16).wrapping_add(n) as u16;
                16
            }
            // JP (HL)
            0xE9 => { self.pc = self.hl.reg(); 4 }
            // LD (nn),A
            0xEA => {
                let nn = self.read_word(mem); self.pc += 2;
                mem.write_byte(nn, self.af.hi());
                16
            }
            // XOR n
            0xEE => { let n = mem.read_byte(self.pc); self.pc += 1; self.xor_8bit(n); 8 }
            // RST 28H
            0xEF => { self.push_word(mem, self.pc); self.pc = 0x28; 16 }
            // LDH A,(n)
            0xF0 => {
                let n = mem.read_byte(self.pc); self.pc += 1;
                let v = mem.read_byte(0xFF00 + n as u16);
                self.af.set_hi(v);
                12
            }
            // POP AF
            0xF1 => {
                let v = self.pop_word(mem);
                self.af.set_reg(v & 0xFFF0); // lower 4 bits of F always 0
                12
            }
            // LD A,(0xFF00+C)
            0xF2 => {
                let v = mem.read_byte(0xFF00 + self.bc.lo() as u16);
                self.af.set_hi(v);
                8
            }
            // DI
            0xF3 => { mem.pending_disable_interrupts = 2; 4 }
            // PUSH AF
            0xF5 => { let v = self.af.reg(); self.push_word(mem, v); 16 }
            // OR n
            0xF6 => { let n = mem.read_byte(self.pc); self.pc += 1; self.or_8bit(n); 8 }
            // RST 30H
            0xF7 => { self.push_word(mem, self.pc); self.pc = 0x30; 16 }
            // LD HL,SP+n
            0xF8 => {
                let n = mem.read_byte(self.pc) as i8 as i16;
                self.pc += 1;
                let sp = self.sp;
                self.af.set_lo(0);
                if (sp & 0xF) as i16 + (n & 0xF) > 0xF {
                    self.set_flag(FLAG_H, true);
                }
                if (sp & 0xFF) as i16 + (n & 0xFF) > 0xFF {
                    self.set_flag(FLAG_C, true);
                }
                self.hl.set_reg((sp as i16).wrapping_add(n) as u16);
                12
            }
            // LD SP,HL
            0xF9 => { self.sp = self.hl.reg(); 8 }
            // LD A,(nn)
            0xFA => {
                let nn = self.read_word(mem); self.pc += 2;
                let v = mem.read_byte(nn);
                self.af.set_hi(v);
                16
            }
            // EI
            0xFB => { mem.pending_enable_interrupts = 2; 4 }
            // CP n
            0xFE => { let n = mem.read_byte(self.pc); self.pc += 1; self.cp_8bit(n); 8 }
            // RST 38H
            0xFF => { self.push_word(mem, self.pc); self.pc = 0x38; 16 }

            _ => {
                eprintln!("Unhandled opcode: 0x{:02X} at PC=0x{:04X}", opcode, self.pc.wrapping_sub(1));
                4
            }
        }
    }

    fn execute_extended(&mut self, mem: &mut Memory) -> u32 {
        let opcode = mem.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);

        match opcode {
            // RLC r
            0x00 => { let v = self.rlc(self.bc.hi()); self.bc.set_hi(v); 8 }
            0x01 => { let v = self.rlc(self.bc.lo()); self.bc.set_lo(v); 8 }
            0x02 => { let v = self.rlc(self.de.hi()); self.de.set_hi(v); 8 }
            0x03 => { let v = self.rlc(self.de.lo()); self.de.set_lo(v); 8 }
            0x04 => { let v = self.rlc(self.hl.hi()); self.hl.set_hi(v); 8 }
            0x05 => { let v = self.rlc(self.hl.lo()); self.hl.set_lo(v); 8 }
            0x06 => { let v = mem.read_byte(self.hl.reg()); let r = self.rlc(v); mem.write_byte(self.hl.reg(), r); 16 }
            0x07 => { let v = self.rlc(self.af.hi()); self.af.set_hi(v); 8 }

            // RRC r
            0x08 => { let v = self.rrc(self.bc.hi()); self.bc.set_hi(v); 8 }
            0x09 => { let v = self.rrc(self.bc.lo()); self.bc.set_lo(v); 8 }
            0x0A => { let v = self.rrc(self.de.hi()); self.de.set_hi(v); 8 }
            0x0B => { let v = self.rrc(self.de.lo()); self.de.set_lo(v); 8 }
            0x0C => { let v = self.rrc(self.hl.hi()); self.hl.set_hi(v); 8 }
            0x0D => { let v = self.rrc(self.hl.lo()); self.hl.set_lo(v); 8 }
            0x0E => { let v = mem.read_byte(self.hl.reg()); let r = self.rrc(v); mem.write_byte(self.hl.reg(), r); 16 }
            0x0F => { let v = self.rrc(self.af.hi()); self.af.set_hi(v); 8 }

            // RL r
            0x10 => { let v = self.rl(self.bc.hi()); self.bc.set_hi(v); 8 }
            0x11 => { let v = self.rl(self.bc.lo()); self.bc.set_lo(v); 8 }
            0x12 => { let v = self.rl(self.de.hi()); self.de.set_hi(v); 8 }
            0x13 => { let v = self.rl(self.de.lo()); self.de.set_lo(v); 8 }
            0x14 => { let v = self.rl(self.hl.hi()); self.hl.set_hi(v); 8 }
            0x15 => { let v = self.rl(self.hl.lo()); self.hl.set_lo(v); 8 }
            0x16 => { let v = mem.read_byte(self.hl.reg()); let r = self.rl(v); mem.write_byte(self.hl.reg(), r); 16 }
            0x17 => { let v = self.rl(self.af.hi()); self.af.set_hi(v); 8 }

            // RR r
            0x18 => { let v = self.rr(self.bc.hi()); self.bc.set_hi(v); 8 }
            0x19 => { let v = self.rr(self.bc.lo()); self.bc.set_lo(v); 8 }
            0x1A => { let v = self.rr(self.de.hi()); self.de.set_hi(v); 8 }
            0x1B => { let v = self.rr(self.de.lo()); self.de.set_lo(v); 8 }
            0x1C => { let v = self.rr(self.hl.hi()); self.hl.set_hi(v); 8 }
            0x1D => { let v = self.rr(self.hl.lo()); self.hl.set_lo(v); 8 }
            0x1E => { let v = mem.read_byte(self.hl.reg()); let r = self.rr(v); mem.write_byte(self.hl.reg(), r); 16 }
            0x1F => { let v = self.rr(self.af.hi()); self.af.set_hi(v); 8 }

            // SLA r
            0x20 => { let v = self.sla(self.bc.hi()); self.bc.set_hi(v); 8 }
            0x21 => { let v = self.sla(self.bc.lo()); self.bc.set_lo(v); 8 }
            0x22 => { let v = self.sla(self.de.hi()); self.de.set_hi(v); 8 }
            0x23 => { let v = self.sla(self.de.lo()); self.de.set_lo(v); 8 }
            0x24 => { let v = self.sla(self.hl.hi()); self.hl.set_hi(v); 8 }
            0x25 => { let v = self.sla(self.hl.lo()); self.hl.set_lo(v); 8 }
            0x26 => { let v = mem.read_byte(self.hl.reg()); let r = self.sla(v); mem.write_byte(self.hl.reg(), r); 16 }
            0x27 => { let v = self.sla(self.af.hi()); self.af.set_hi(v); 8 }

            // SRA r
            0x28 => { let v = self.sra(self.bc.hi()); self.bc.set_hi(v); 8 }
            0x29 => { let v = self.sra(self.bc.lo()); self.bc.set_lo(v); 8 }
            0x2A => { let v = self.sra(self.de.hi()); self.de.set_hi(v); 8 }
            0x2B => { let v = self.sra(self.de.lo()); self.de.set_lo(v); 8 }
            0x2C => { let v = self.sra(self.hl.hi()); self.hl.set_hi(v); 8 }
            0x2D => { let v = self.sra(self.hl.lo()); self.hl.set_lo(v); 8 }
            0x2E => { let v = mem.read_byte(self.hl.reg()); let r = self.sra(v); mem.write_byte(self.hl.reg(), r); 16 }
            0x2F => { let v = self.sra(self.af.hi()); self.af.set_hi(v); 8 }

            // SWAP r
            0x30 => { let v = self.swap(self.bc.hi()); self.bc.set_hi(v); 8 }
            0x31 => { let v = self.swap(self.bc.lo()); self.bc.set_lo(v); 8 }
            0x32 => { let v = self.swap(self.de.hi()); self.de.set_hi(v); 8 }
            0x33 => { let v = self.swap(self.de.lo()); self.de.set_lo(v); 8 }
            0x34 => { let v = self.swap(self.hl.hi()); self.hl.set_hi(v); 8 }
            0x35 => { let v = self.swap(self.hl.lo()); self.hl.set_lo(v); 8 }
            0x36 => { let v = mem.read_byte(self.hl.reg()); let r = self.swap(v); mem.write_byte(self.hl.reg(), r); 16 }
            0x37 => { let v = self.swap(self.af.hi()); self.af.set_hi(v); 8 }

            // SRL r
            0x38 => { let v = self.srl(self.bc.hi()); self.bc.set_hi(v); 8 }
            0x39 => { let v = self.srl(self.bc.lo()); self.bc.set_lo(v); 8 }
            0x3A => { let v = self.srl(self.de.hi()); self.de.set_hi(v); 8 }
            0x3B => { let v = self.srl(self.de.lo()); self.de.set_lo(v); 8 }
            0x3C => { let v = self.srl(self.hl.hi()); self.hl.set_hi(v); 8 }
            0x3D => { let v = self.srl(self.hl.lo()); self.hl.set_lo(v); 8 }
            0x3E => { let v = mem.read_byte(self.hl.reg()); let r = self.srl(v); mem.write_byte(self.hl.reg(), r); 16 }
            0x3F => { let v = self.srl(self.af.hi()); self.af.set_hi(v); 8 }

            // BIT b,r - test bit b of register r
            0x40..=0x7F => {
                let bit = (opcode - 0x40) / 8;
                let reg = opcode & 0x07;
                let val = self.get_reg_val(reg, mem);
                self.test_bit(val, bit);
                if reg == 6 { 12 } else { 8 }
            }

            // RES b,r - reset bit b of register r
            0x80..=0xBF => {
                let bit = (opcode - 0x80) / 8;
                let reg = opcode & 0x07;
                let val = self.get_reg_val(reg, mem);
                let result = val & !(1 << bit);
                self.set_reg_val(reg, result, mem);
                if reg == 6 { 16 } else { 8 }
            }

            // SET b,r - set bit b of register r
            0xC0..=0xFF => {
                let bit = (opcode - 0xC0) / 8;
                let reg = opcode & 0x07;
                let val = self.get_reg_val(reg, mem);
                let result = val | (1 << bit);
                self.set_reg_val(reg, result, mem);
                if reg == 6 { 16 } else { 8 }
            }
        }
    }

    fn get_reg_val(&self, reg: u8, mem: &Memory) -> u8 {
        match reg {
            0 => self.bc.hi(),
            1 => self.bc.lo(),
            2 => self.de.hi(),
            3 => self.de.lo(),
            4 => self.hl.hi(),
            5 => self.hl.lo(),
            6 => mem.read_byte(self.hl.reg()),
            7 => self.af.hi(),
            _ => 0,
        }
    }

    fn set_reg_val(&mut self, reg: u8, val: u8, mem: &mut Memory) {
        match reg {
            0 => self.bc.set_hi(val),
            1 => self.bc.set_lo(val),
            2 => self.de.set_hi(val),
            3 => self.de.set_lo(val),
            4 => self.hl.set_hi(val),
            5 => self.hl.set_lo(val),
            6 => mem.write_byte(self.hl.reg(), val),
            7 => self.af.set_hi(val),
            _ => {}
        }
    }

    // --- ALU helpers ---

    fn add_8bit(&mut self, val: u8, add_carry: bool) {
        let a = self.af.hi();
        let carry_val: u8 = if add_carry && self.get_flag(FLAG_C) { 1 } else { 0 };
        let total = a as u16 + val as u16 + carry_val as u16;
        self.af.set_hi(total as u8);
        self.af.set_lo(0);
        if (total & 0xFF) == 0 { self.set_flag(FLAG_Z, true); }
        if (a & 0xF) + (val & 0xF) + carry_val > 0xF { self.set_flag(FLAG_H, true); }
        if total > 0xFF { self.set_flag(FLAG_C, true); }
    }

    fn sub_8bit(&mut self, val: u8, sub_carry: bool) {
        let a = self.af.hi();
        let carry_val: u8 = if sub_carry && self.get_flag(FLAG_C) { 1 } else { 0 };
        let total = val as u16 + carry_val as u16;
        let result = a.wrapping_sub(total as u8);
        self.af.set_lo(0);
        self.set_flag(FLAG_N, true);
        if result == 0 { self.set_flag(FLAG_Z, true); }
        if (a as u16) < total { self.set_flag(FLAG_C, true); }
        if (a & 0xF) < (val & 0xF) + carry_val { self.set_flag(FLAG_H, true); }
        self.af.set_hi(result);
    }

    fn and_8bit(&mut self, val: u8) {
        let result = self.af.hi() & val;
        self.af.set_hi(result);
        self.af.set_lo(0);
        if result == 0 { self.set_flag(FLAG_Z, true); }
        self.set_flag(FLAG_H, true);
    }

    fn or_8bit(&mut self, val: u8) {
        let result = self.af.hi() | val;
        self.af.set_hi(result);
        self.af.set_lo(0);
        if result == 0 { self.set_flag(FLAG_Z, true); }
    }

    fn xor_8bit(&mut self, val: u8) {
        let result = self.af.hi() ^ val;
        self.af.set_hi(result);
        self.af.set_lo(0);
        if result == 0 { self.set_flag(FLAG_Z, true); }
    }

    fn cp_8bit(&mut self, val: u8) {
        let a = self.af.hi();
        self.af.set_lo(0);
        self.set_flag(FLAG_N, true);
        if a == val { self.set_flag(FLAG_Z, true); }
        if a < val { self.set_flag(FLAG_C, true); }
        if (a & 0xF) < (val & 0xF) { self.set_flag(FLAG_H, true); }
    }

    fn inc_8bit(&mut self, val: u8) -> u8 {
        let result = val.wrapping_add(1);
        self.set_flag(FLAG_Z, result == 0);
        self.set_flag(FLAG_N, false);
        self.set_flag(FLAG_H, (val & 0xF) + 1 > 0xF);
        result
    }

    fn dec_8bit(&mut self, val: u8) -> u8 {
        let result = val.wrapping_sub(1);
        self.set_flag(FLAG_Z, result == 0);
        self.set_flag(FLAG_N, true);
        self.set_flag(FLAG_H, (val & 0xF) == 0);
        result
    }

    fn add_16bit(&mut self, val: u16) {
        let hl = self.hl.reg();
        let result = hl as u32 + val as u32;
        self.set_flag(FLAG_N, false);
        self.set_flag(FLAG_H, (hl & 0xFFF) + (val & 0xFFF) > 0xFFF);
        self.set_flag(FLAG_C, result > 0xFFFF);
        self.hl.set_reg(result as u16);
    }

    fn daa(&mut self) {
        let mut a = self.af.hi() as u16;
        if !self.get_flag(FLAG_N) {
            if self.get_flag(FLAG_H) || (a & 0xF) > 9 {
                a += 0x06;
            }
            if self.get_flag(FLAG_C) || a > 0x9F {
                a += 0x60;
            }
        } else {
            if self.get_flag(FLAG_H) {
                a = (a.wrapping_sub(6)) & 0xFF;
            }
            if self.get_flag(FLAG_C) {
                a = a.wrapping_sub(0x60);
            }
        }
        self.set_flag(FLAG_H, false);
        if a >= 0x100 { self.set_flag(FLAG_C, true); }
        a &= 0xFF;
        self.set_flag(FLAG_Z, a == 0);
        self.af.set_hi(a as u8);
    }

    // --- Rotate/shift helpers ---

    fn rlc(&mut self, val: u8) -> u8 {
        let carry = val >> 7;
        let result = (val << 1) | carry;
        self.af.set_lo(0);
        if result == 0 { self.set_flag(FLAG_Z, true); }
        if carry == 1 { self.set_flag(FLAG_C, true); }
        result
    }

    fn rrc(&mut self, val: u8) -> u8 {
        let carry = val & 1;
        let result = (val >> 1) | (carry << 7);
        self.af.set_lo(0);
        if result == 0 { self.set_flag(FLAG_Z, true); }
        if carry == 1 { self.set_flag(FLAG_C, true); }
        result
    }

    fn rl(&mut self, val: u8) -> u8 {
        let old_carry = if self.get_flag(FLAG_C) { 1u8 } else { 0 };
        let new_carry = val >> 7;
        let result = (val << 1) | old_carry;
        self.af.set_lo(0);
        if result == 0 { self.set_flag(FLAG_Z, true); }
        if new_carry == 1 { self.set_flag(FLAG_C, true); }
        result
    }

    fn rr(&mut self, val: u8) -> u8 {
        let old_carry = if self.get_flag(FLAG_C) { 1u8 } else { 0 };
        let new_carry = val & 1;
        let result = (val >> 1) | (old_carry << 7);
        self.af.set_lo(0);
        if result == 0 { self.set_flag(FLAG_Z, true); }
        if new_carry == 1 { self.set_flag(FLAG_C, true); }
        result
    }

    fn sla(&mut self, val: u8) -> u8 {
        let carry = val >> 7;
        let result = val << 1;
        self.af.set_lo(0);
        if result == 0 { self.set_flag(FLAG_Z, true); }
        if carry == 1 { self.set_flag(FLAG_C, true); }
        result
    }

    fn sra(&mut self, val: u8) -> u8 {
        let carry = val & 1;
        let result = (val >> 1) | (val & 0x80);
        self.af.set_lo(0);
        if result == 0 { self.set_flag(FLAG_Z, true); }
        if carry == 1 { self.set_flag(FLAG_C, true); }
        result
    }

    fn srl(&mut self, val: u8) -> u8 {
        let carry = val & 1;
        let result = val >> 1;
        self.af.set_lo(0);
        if result == 0 { self.set_flag(FLAG_Z, true); }
        if carry == 1 { self.set_flag(FLAG_C, true); }
        result
    }

    fn swap(&mut self, val: u8) -> u8 {
        let result = ((val & 0xF0) >> 4) | ((val & 0x0F) << 4);
        self.af.set_lo(0);
        if result == 0 { self.set_flag(FLAG_Z, true); }
        result
    }

    fn test_bit(&mut self, val: u8, bit: u8) {
        if (val >> bit) & 1 == 0 {
            self.set_flag(FLAG_Z, true);
        } else {
            self.set_flag(FLAG_Z, false);
        }
        self.set_flag(FLAG_N, false);
        self.set_flag(FLAG_H, true);
    }
}
