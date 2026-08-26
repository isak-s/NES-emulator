use crate::{op_codes};
use super::addressing_modes::AddressingMode;

pub struct CpuFlags;
impl CpuFlags {
    /// # Status Register https://www.nesdev.org/wiki/Status_flags
    ///  7 N --- Negative Flag
    ///  6 V --- Overflow Flag
    ///  5 _ --- Nothing
    ///  4 B --- Break command
    ///  3 D --- Decimal mode (Not used on NES)
    ///  2 I --- Interrupt Disable
    ///  1 Z --- Zero Flag
    ///  0 C --- Carry Flag
    pub const NEGATIVE: u8 =            0b1000_0000;
    pub const OVERFLOW: u8 =            0b0100_0000;
    pub const NOTHING_BIT: u8 =         0b0010_0000;   // Completely unused
    pub const BREAK_COMMAND: u8 =       0b0001_0000;  // break is done with return instead
    pub const DECIMAL_MODE_UNUSED: u8 = 0b0000_1000;
    pub const INTERRUPT_DISABLE: u8 =   0b0000_0100;
    pub const ZERO_FLAG: u8 =           0b0000_0010;
    pub const CARRY_FLAG: u8 =          0b0000_0001;
}

pub struct CPU {
    pub register_a: u8, // also called accumulator
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0x10000], // FFFF
    // With the 6502, the stack is always on page one ($100-$1FF) and works top down.
    stack_pointer: u8,
}

const STACK_START: u16 = 0x100;
// Stack end is added to stack start to get the real stack end.
const STACK_END: u8 = 0xFD;

const PROGRAM_START: u16 = 0x600;

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 0x10000], // FFFF
            stack_pointer: STACK_END,
        }
    }
    // should self be mut?
    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            // treats last bit as a sign bit
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            // does not treat last bit as sign bit
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }

            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }

            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }

            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }

            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);
                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }

            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | lo as u16;
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }


            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode)
            }
        }
    }

    pub fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }
    // the nes cpu uses little endian addressing. [lower 8 bits : higher 8 bits]
    // make self mut later
    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    // the stack works top down
    fn stack_push(&mut self, value: u8) {
        println!("                  PUSHING {:2X?} to 0x{:4X?}", value, STACK_START + self.stack_pointer as u16);
        self.mem_write((STACK_START as u16) + self.stack_pointer as u16, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let stored = self.mem_read((STACK_START as u16) + self.stack_pointer as u16);
        println!("                  POPPING {:2X?} from 0x{:4X?}", stored, STACK_START + self.stack_pointer as u16);
        stored
    }

    fn stack_push_u16(&mut self, value: u16) {
        let hi = (value >> 8) as u8;
        let lo = (value & 0xff) as u8;
        self.stack_push(hi);
        self.stack_push(lo);

        println!("                  AS {:4X?}", value);
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;
        println!("                  AS {:4X?}", (hi << 8) | lo);
        (hi << 8) | lo
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;
        self.stack_pointer = STACK_END;

        self.program_counter = self.mem_read_u16(0xFFFC)
    }
    // this is the nes version:
//    pub fn load(&mut self, program: Vec<u8>) {
        //self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);  // load the entire program after 0x8000
        //self.mem_write_u16(0xFFFC, 0x8000); // save a reference to the start of the code in 0xffc
    //}

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x0600..(0x0600 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x600);//0x0600);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status |= CpuFlags::ZERO_FLAG;
        } else {
            self.status &= !CpuFlags::ZERO_FLAG;
        }
        if result & 0b1000_0000 != 0 {
            self.status |= CpuFlags::NEGATIVE;
        } else {
            self.status &= !CpuFlags::NEGATIVE;
        }
    }

    fn set_register_a(&mut self, new_val: u8) {
        self.register_a = new_val;
        self.update_zero_and_negative_flags(self.register_a);
    }
    fn set_register_x(&mut self, new_val: u8) {
        self.register_x = new_val;
        self.update_zero_and_negative_flags(self.register_x);
    }
    fn set_register_y(&mut self, new_val: u8) {
        self.register_y = new_val;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn _adc(&mut self, operand: i16) {
        let sum: i16 = (self.register_a as i16)
            + operand as i16
            + (self.status & CpuFlags::CARRY_FLAG) as i16;

        // carry check
        self.status &= !CpuFlags::CARRY_FLAG; // clear carry
        if sum > 0xff {
            self.status |= CpuFlags::CARRY_FLAG; // set carry
        }

        let res = sum as u8;
        // overflow check
        self.status &= !CpuFlags::OVERFLOW;
        if (operand as u8 ^ res) & (res ^ self.register_a) & 0x80 != 0 {
            self.status |= CpuFlags::OVERFLOW;
        }

        self.set_register_a(res as u8);
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let operand = self.mem_read(addr);
        self._adc(operand as i16);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let operand = self.mem_read(addr);
        self.set_register_a(self.register_a & operand);
    }

    fn asl(&mut self, mode: &AddressingMode) {
        // arithmetic shift left one bit
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        if data & CpuFlags::NEGATIVE != 0 { // if bit 7 is set
            self.status = self.status | CpuFlags::CARRY_FLAG;
        } else {
            self.status = self.status & !CpuFlags::CARRY_FLAG;
        }

        let res = data << 1;

        self.mem_write(addr, res);
        self.update_zero_and_negative_flags(res);
    }

    fn asl_accumulator(&mut self) {
        if self.register_a & CpuFlags::NEGATIVE != 0 { // if bit 7 is set
            self.status = self.status | CpuFlags::CARRY_FLAG;
        } else {
            self.status = self.status & !CpuFlags::CARRY_FLAG;
        }
        self.set_register_a(self.register_a << 1);
    }

    fn branch(&mut self, condition: bool) {
        if condition {
            // relative can be both pos and neg. Treat as signed!!!
            let relative = self.mem_read(self.program_counter) as i8;
            let jump_addr = self.program_counter
                .wrapping_add(1) // one or two?
                .wrapping_add(relative as u16);  // why u16???
            self.program_counter = jump_addr;
        }
    }

    fn bcc(&mut self) {
        self.branch(self.status & CpuFlags::CARRY_FLAG == 0);
    }

    fn bcs(&mut self) {
        self.branch(self.status & CpuFlags::CARRY_FLAG != 0);
    }

    fn beq(&mut self) {
        // use CMP instruction before, and then branch based on status
        self.branch(self.status & CpuFlags::ZERO_FLAG != 0);
    }
    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let operand = self.mem_read(addr);

        let res = self.register_a & operand;
        if res == 0 {
            self.status |= CpuFlags::ZERO_FLAG;
        } else {
            self.status &= !CpuFlags::ZERO_FLAG;
        }
        // negative and overflow bits
        let stat_mask = operand & 0b1100_0000; // why not from res?
        self.status = (self.status & 0b0011_1111) | stat_mask
    }

    fn bmi(&mut self) {
        self.branch(self.status & CpuFlags::NEGATIVE != 0);
    }

    fn bne(&mut self) {
        // use cmp instruction before
        self.branch(self.status & CpuFlags::ZERO_FLAG == 0);
    }

    fn bpl(&mut self) {
        self.branch(self.status & CpuFlags::NEGATIVE == 0);
    }

    fn bvc(&mut self) {
        self.branch(self.status & CpuFlags::OVERFLOW == 0);
    }

    fn bvs(&mut self) {
        self.branch(self.status & CpuFlags::OVERFLOW != 0);
    }

    fn clc(&mut self) {
        self.status &= !CpuFlags::CARRY_FLAG;
    }

    fn cld(&mut self) {
        self.status &= !CpuFlags::DECIMAL_MODE_UNUSED;
    }

    fn cli(&mut self) {
        self.status &= !CpuFlags::INTERRUPT_DISABLE;
    }

    fn clv(&mut self) {
        self.status &= !CpuFlags::OVERFLOW;
    }

    fn _cmp(&mut self, mode: &AddressingMode, register: u8) {
        let addr = self.get_operand_address(mode);
        let operand = self.mem_read(addr);
        // accumulator - operand
        let res = register.wrapping_sub(operand);

        self.status &= !CpuFlags::CARRY_FLAG; // clear carry flag
        if register >= operand {
            self.status |= CpuFlags::CARRY_FLAG;
        }
        self.update_zero_and_negative_flags(res);
    }

    fn cmp(&mut self, mode:&AddressingMode) {
        self._cmp(mode, self.register_a);
    }

    fn cpx(&mut self, mode: &AddressingMode) {
        self._cmp(mode, self.register_x);
    }

    fn cpy(&mut self, mode: &AddressingMode) {
        self._cmp(mode, self.register_y);
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let operand = self.mem_read(addr);
        let res = operand.wrapping_sub(1);
        self.mem_write(addr, res);
        self.update_zero_and_negative_flags(res);
    }

    fn dex(&mut self) {
        self.set_register_x(self.register_x.wrapping_sub(1));
    }

    fn dey(&mut self) {
        self.set_register_y(self.register_y.wrapping_sub(1));
    }

    fn eor(&mut self, mode: &AddressingMode) {
        // This is xor. In the specification its called eor for some reason
        let addr = self.get_operand_address(mode);
        let operand = self.mem_read(addr);

        self.set_register_a(self.register_a ^ operand);
        self.update_zero_and_negative_flags(self.register_a);
    }


    fn inc(&mut self, mode: &AddressingMode) {
        // possibly make helper function modify_memory_content that applies a function passed in
        let addr = self.get_operand_address(mode);
        let operand = self.mem_read(addr);
        let res = operand.wrapping_add(1);
        self.mem_write(addr, res);
        self.update_zero_and_negative_flags(res);
    }

    fn inx(&mut self) {
        self.set_register_x(self.register_x.wrapping_add(1));
    }

    fn iny(&mut self) {
        self.set_register_y(self.register_y.wrapping_add(1));
    }

    fn jmp_absolute_with_bug(&mut self, mode: &AddressingMode) {
        // An original 6502 has does not correctly fetch the target address if the indirect vector falls on a page boundary
        let addr = self.get_operand_address(mode);

        let lo = self.mem_read(addr);
        let hi_addr = if addr & 0x00FF == 0x00FF {
            addr & 0xFF00
        } else {addr + 1};
        let hi = self.mem_read(hi_addr);

        self.program_counter = ((hi as u16) << 8) | lo as u16;
    }

    fn jsr(&mut self) {
        println!("          JSR: pc before operand = {:04X}", self.program_counter);

        self.stack_push_u16(self.program_counter.wrapping_add(1));

        let destination = self.get_operand_address(&AddressingMode::Absolute);

        self.program_counter = destination;

        println!("          JSR: pc after = {:04X}", self.program_counter);
    }
//    fn jsr(&mut self) {
        //// jump to subroutine (function). Pushes the address (minus one) of the return point to the stack.
        //self.stack_push_u16(self.program_counter); // .wrapping_add(2).wrapping_sub(1));
        //let addr = self.get_operand_address(&AddressingMode::Absolute);
        //let destination = self.mem_read_u16(addr); // u16 since this is not relative. Just address
        //self.program_counter = destination;
    //}

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.set_register_a(value);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.set_register_x(value);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.set_register_y(value);
    }

    fn lsr(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let stat_mask = value & CpuFlags::CARRY_FLAG;
        self.status = (self.status & !CpuFlags::CARRY_FLAG) | stat_mask;

        let res = value >> 1;
        self.mem_write(addr, res);
        self.update_zero_and_negative_flags(res);
    }

    fn lsr_accumulator(&mut self) {
        let stat_mask = self.register_a & CpuFlags::CARRY_FLAG;
        self.status = (self.status & !CpuFlags::CARRY_FLAG) | stat_mask;
        self.set_register_a(self.register_a >> 1);
    }

    fn nop(&mut self) {
        // no operation
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mask = self.mem_read(addr);
        self.set_register_a(self.register_a | mask);
    }

    // With the 6502, the stack is always on page one ($100-$1FF) and works top down.
    fn pha(&mut self) {
        // push accumulator to stack
        // decrement the stack pointer one byte
        self.stack_push(self.register_a);
    }

    fn php(&mut self) {
        self.stack_push(self.status | 0b0011_0000);
    }

    fn pla(&mut self) {
        let stored = self.stack_pop();
        self.set_register_a(stored);
    }

    fn plp(&mut self) {
        let stored = self.stack_pop();
        self.status = (stored & !CpuFlags::BREAK_COMMAND) | CpuFlags::NOTHING_BIT;
    }

    fn rol(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let operand = self.mem_read(addr);

        let bit_7 = operand & 0b1000_0000;
        let shifted = operand << 1;
        let carry_inserted = shifted | (self.status & CpuFlags::CARRY_FLAG);

        self.mem_write(addr, carry_inserted);
        self.update_zero_and_negative_flags(carry_inserted);
        self.status = (self.status & !CpuFlags::CARRY_FLAG) | (bit_7 >> 7)
    }

    fn rol_accumulator(&mut self) {
        let bit_7 = self.register_a & 0b1000_0000;
        let shifted = self.register_a << 1;
        let carry_inserted = shifted | (self.status & CpuFlags::CARRY_FLAG);

        self.set_register_a(carry_inserted);
        self.status = (self.status & !CpuFlags::CARRY_FLAG) | (bit_7 >> 7)
    }

    fn ror(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let operand = self.mem_read(addr);

        let bit_0 = operand & 0b0000_0001;
        let shifted = operand >> 1;
        let carry_inserted = shifted | ((self.status & CpuFlags::CARRY_FLAG) << 7);

        self.mem_write(addr, carry_inserted);
        self.update_zero_and_negative_flags(carry_inserted);
        self.status = (self.status & !CpuFlags::CARRY_FLAG) | bit_0;
    }

    fn ror_accumulator(&mut self) {
        let bit_0 = self.register_a & 0b0000_0001;
        let shifted = self.register_a >> 1;
        let carry_inserted = shifted | ((self.status & CpuFlags::CARRY_FLAG) << 7);

        self.status = (self.status & !CpuFlags::CARRY_FLAG) | bit_0;
        self.set_register_a(carry_inserted);
    }

    fn rti(&mut self) {
        // return from interrupt. Fetch processor flags and pc
        self.plp();
        self.program_counter = self.stack_pop_u16();
    }

    fn rts(&mut self) {
        // return from subroutine
        let popped = self.stack_pop_u16().wrapping_add(1);
        println!("        popped is {:04X?}", popped);
        self.program_counter = popped;
        //popped = self.stack_pop_u16();
        // println!("popped {:04X?}", popped);
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let operand = self.mem_read(addr);

        let carry = self.status & CpuFlags::CARRY_FLAG;
        let value = operand ^ 0xFF;

        let sum = self.register_a as u16
            + value as u16
            + carry as u16;

        let result = sum as u8;

        if sum > 0xFF {
            self.status |= CpuFlags::CARRY_FLAG;
        } else {
            self.status &= !CpuFlags::CARRY_FLAG;
        }

        if ((self.register_a ^ result) & (self.register_a ^ operand) & 0x80) != 0 {
            self.status |= CpuFlags::OVERFLOW;
        } else {
            self.status &= !CpuFlags::OVERFLOW;
        }

        self.set_register_a(result);
    }

    fn sec(&mut self) {
        self.status |= CpuFlags::CARRY_FLAG;
    }

    fn sed(&mut self) {
        self.status |= CpuFlags::DECIMAL_MODE_UNUSED;
    }

    fn sei(&mut self) {
        self.status |= CpuFlags::INTERRUPT_DISABLE;
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }

    fn tax(&mut self) {
        self.set_register_x(self.register_a);
    }

    fn tay(&mut self) {
        self.set_register_y(self.register_a);
    }

    fn tsx(&mut self) {
        self.set_register_x(self.stack_pointer);
    }

    fn txa(&mut self) {
        self.set_register_a(self.register_x);
    }

    fn txs(&mut self) {
        // transfer x to stack pointer
        self.stack_pointer = self.register_x;
    }

    fn tya(&mut self) {
        self.set_register_a(self.register_y);
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where F: FnMut(&mut CPU), {
        let ref opcodes = *op_codes::CPU_CODES_MAP;

        loop {

            callback(self);

            let code = self.memory[self.program_counter as usize];

            let op_code = opcodes
                .get(&code)
                .expect(&format!("Opcode {:x} not recognized!", code));

            println!("pc is at {:X?}", self.program_counter as u16);
            println!("running inst {} with opcode {:x}", &op_code.name, code);

            self.program_counter += 1;
            let pc_state_before_inst_execution = self.program_counter;

            match code {

                0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => {
                    self.adc(&op_code.addressing_mode);
                }

                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
                    self.and(&op_code.addressing_mode);
                }

                0x06 | 0x16 | 0x0e | 0x1e => {
                    self.asl(&op_code.addressing_mode);
                }

                0x0A => self.asl_accumulator(),


                0x90 => self.bcc(),

                0xB0 => self.bcs(),

                0xF0 => self.beq(),

                0x24 | 0x2C => self.bit(&op_code.addressing_mode),

                0x30 => self.bmi(),

                0xD0 => self.bne(),

                0x10 => self.bpl(),

                0x50 => self.bvc(),

                0x70 => self.bvs(),

                0x18 => self.clc(),

                0xD8 => self.cld(),

                0x58 => self.cli(),

                0xB8 => self.clv(),

                0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => {
                    self.cmp(&op_code.addressing_mode);
                },

                0xE0 | 0xE4 | 0xEC => self.cpx(&op_code.addressing_mode),

                0xC0 | 0xC4 | 0xCC => self.cpy(&op_code.addressing_mode),

                0xC6 | 0xD6 | 0xCE | 0xDE => self.dec(&op_code.addressing_mode),

                0xCA => self.dex(),

                0x88 => self.dey(),

                0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => {
                    self.eor(&op_code.addressing_mode);
                },

                0xE6 | 0xF6 | 0xEE | 0xFE => self.inc(&op_code.addressing_mode),

                0xE8 => self.inx(),

                0xC8 => self.iny(),

                0x4C => {
                    let mem_address = self.mem_read_u16(self.program_counter);
                    self.program_counter = mem_address;
                },

                0x6C => self.jmp_absolute_with_bug(&op_code.addressing_mode),

                0x20 => self.jsr(),

                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                    self.lda(&op_code.addressing_mode);
                }

                0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => {
                    self.ldx(&op_code.addressing_mode)
                }

                0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => {
                    self.ldy(&op_code.addressing_mode);
                }

                0x46 | 0x56 | 0x4E | 0x5E => {
                    self.lsr(&op_code.addressing_mode)
                },

                0x4A => self.lsr_accumulator(),

                0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => {
                    self.ora(&op_code.addressing_mode)
                }

                0x48 => self.pha(),

                0x08 => self.php(),

                0x68 => self.pla(),

                0x28 => self.plp(),


                0x2A => self.rol_accumulator(),

                0x26 | 0x36 | 0x2E | 0x3E => self.rol(&op_code.addressing_mode),

                0x6A => self.ror_accumulator(),

                0x66 | 0x76 | 0x6E | 0x7E => {
                    self.ror(&op_code.addressing_mode);
                }

                0x40 => self.rti(),

                0x60 => self.rts(),

                0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 => {
                    self.sbc(&op_code.addressing_mode);
                },

                0x38 => self.sec(),

                0xF8 => self.sed(),

                0x78 => self.sei(),

                0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => {
                    self.sta(&op_code.addressing_mode);
                },

                0x86 | 0x96 | 0x8E => {
                    self.stx(&op_code.addressing_mode)
                },

                0x84 | 0x94 | 0x8C => {
                    self.sty(&op_code.addressing_mode);
                },

                0xAA => self.tax(),

                0xA8 => self.tay(),

                0xBA => self.tsx(),

                0x8A => self.txa(),

                0x9a => self.txs(),

                0x98 => self.tya(),

                // fuck EA sports. Do nothing
                0xEA => self.nop(),

                // brk force interrupt
                0x00 => return,

                _ => todo!(),
            }
            if pc_state_before_inst_execution == self.program_counter {
                self.program_counter += (op_code.bytes - 1) as u16
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10)
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        // put 10 in reg a then copy to reg x
        cpu.load_and_run(vec![0xa9, 0x0a, 0xaa, 0x00]);
        // set register a to 10

        assert_eq!(cpu.register_x, 10);
    }
    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        // cpu.register_x = 0xff;
        // everything resets when running load and run
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }
   #[test]
   fn test_lda_from_memory() {
       let mut cpu = CPU::new();
       cpu.mem_write(0x10, 0x55);

       cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

       assert_eq!(cpu.register_a, 0x55);
   }

   #[test]
   fn test_and_immediate_with_all_zeroes() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x29, 0x00]);
    assert_eq!(cpu.register_a, 0x00);
    assert_eq!(cpu.status, 0b0000_0010);
   }

   #[test]
   fn test_and_immediate_all_ones() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0b1111_1111, 0x29, 0b1111_1111, 0x00]);
    assert_eq!(cpu.register_a, 0b1111_1111);
    assert_eq!(cpu.status, 0b1000_0000);
   }
    #[test]
   fn test_and_immediate_alternating_bits() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0b0101_0101, 0x29, 0b1010_1010, 0x00]);
    assert_eq!(cpu.register_a, 0b0000_0000);
    assert_eq!(cpu.status, 0b0000_0010);
   }
   #[test]
   fn test_asl_zero_page_one_bit() {
    let mut cpu = CPU::new();
    // put 5 in register a, store in memory, use as zero page for shift
    cpu.load_and_run(vec![0xa9, 0b0000_0001, 0x85, 0x00, 0x06, 0x00, 0xa5, 0x00, 0x00]);
    assert_eq!(cpu.register_a, 0b0000_0010);
    assert_eq!(cpu.status, 0b0000_0000);
   }
   #[test]
   fn test_asl_accumulator_shifts_one_bit_left() {
    let mut cpu = CPU::new();
    // put 5 in register a, store in memory, use as zero page for shift
    cpu.load_and_run(vec![0xa9, 0b0000_0001, 0x0A, 0x00]);
    assert_eq!(cpu.register_a, 0b0000_0010);
   }
   #[test]
   fn test_asl_accumulator_carry_flag_set_to_old_bit_7_from_zero_to_one() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0b1000_0001, 0x0A, 0x00]);
    assert_eq!(cpu.status, CpuFlags::CARRY_FLAG);
   }
   #[test]
   fn test_asl_accumulator_carry_flag_set_to_old_bit_7_from_one_to_zero() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0b1000_0001, 0x0A, 0x0A, 0x00]);
    assert_eq!(cpu.status, 0b0000_0000);
   }

   #[test]
   fn test_bcc_carry_is_clear_jump_to_exit_skipping_instructions() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x90, 2, 0xa9, 0b0000_0001, 0x00]);
    assert_eq!(cpu.register_a, 0b0000_0000);
   }

   #[test]
   fn test_jsr_sets_correct_pc_address() {
    let mut cpu = CPU::new();
    //cpu.load_and_run(vec![0x20, 2, 0x02, 0x00]);
    //assert_eq!(PROGRAM_START, cpu.program_counter);

    //let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x20, 1, 0]);

    assert_eq!(cpu.program_counter, 2);
   }

   #[test]
   fn test_jsr_puts_return_address_on_stack() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x20, 0b0000_0000, 0b_1000_0000]);
    let before_jsr = cpu.program_counter;
    cpu.run();
    let r = cpu.stack_pop_u16();
    assert_eq!(before_jsr, r);
   }

   #[test]
   fn test_push_to_stack_then_pop() {
    let mut cpu = CPU::new();
    cpu.reset();

    cpu.stack_push(69);

    assert_eq!(69, cpu.stack_pop());
   }

   #[test]
   fn test_push_u16_to_stack_then_pop() {
    let mut cpu = CPU::new();
    cpu.reset();

    cpu.stack_push_u16(69);

    assert_eq!(69, cpu.stack_pop_u16());
   }

   #[test]
   fn test_adc_with_overflow() {
    let mut cpu = CPU::new();
    cpu.reset();
    cpu.register_a = 100;
    let operand = 50;
    cpu._adc(operand);
    assert_eq!(cpu.status, CpuFlags::OVERFLOW | CpuFlags::NEGATIVE,
            "status is {:#b}, wanted is {:#b}",
            cpu.status,
            CpuFlags::OVERFLOW | CpuFlags::NEGATIVE);
   }

   #[test]
   fn test_adc_with_carry() {
    let mut cpu = CPU::new();
    cpu.reset();
    cpu.register_a = 200;
    let operand = 100;
    cpu._adc(operand);
    assert_eq!(cpu.status, CpuFlags::CARRY_FLAG,
            "status is {:#b}, wanted is {:#b}",
            cpu.status,
            CpuFlags::CARRY_FLAG);
    }

    #[test]
    fn test_adc_with_overflow_and_carry() {
        let mut cpu = CPU::new();
        cpu.reset();
        cpu.register_a = 145;
        let operand = 145;
        cpu._adc(operand);
        assert_eq!(cpu.status, CpuFlags::CARRY_FLAG | CpuFlags::OVERFLOW,
            "status is {:#b}, wanted is {:#b}",
            cpu.status,
            CpuFlags::CARRY_FLAG | CpuFlags::OVERFLOW);
    }
}
