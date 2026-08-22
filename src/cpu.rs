use crate::op_codes;
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
    pub const NOTHING_BIT: u8 =         0b0010_0000;
    pub const BREAK_COMMAND: u8 =       0b0001_0000;
    pub const DECIMAL_MODE_UNUSED: u8 = 0b0000_1000;
    pub const INTERRUPT_DISABLE: u8 =   0b0000_0100;
    pub const ZERO_FLAG: u8 =           0b0000_0010;
    pub const CARRY_FLAG: u8 =          0b0000_0001;
}

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0x10000], // FFFF
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 0x10000], // FFFF
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
                let deref_base = (hi as u16) | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }


            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode)
            }
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }
    // the nes cpu uses little endian addressing. [lower 8 bits : higher 8 bits]
    // make self mut later
    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | lo
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;

        self.program_counter = self.mem_read_u16(0xFFFC)
    }
    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);  // load the entire program after 0x8000
        self.mem_write_u16(0xFFFC, 0x8000); // save a reference to the start of the code in 0xffc
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }
        if result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
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

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.set_register_a(value);
    }

    fn tax(&mut self) {
        self.set_register_x(self.register_a);
    }

    fn inx(&mut self) {
        self.set_register_x(self.register_x.wrapping_add(1));
    }

    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let operand = self.mem_read(addr);
        self.set_register_a(self.register_a & operand);
    }

    fn branch(&mut self, condition: bool) {
        if condition {
            // relative can be both pos and neg. Treat as signed!!!
            let relative = self.mem_read(self.program_counter) as i8;
            let jump_addr = self.program_counter
                .wrapping_add(1)
                .wrapping_add(relative as u16);
            self.program_counter = jump_addr;
        }
    }

    fn bcc(&mut self) {
        self.branch(self.status & CpuFlags::CARRY_FLAG == 0);
    }

    fn asl(&mut self, mode: &AddressingMode) {
        // arithmetic shift left
        if self.register_a & CpuFlags::NEGATIVE != 0 { // if bit 7 is set
            self.status = self.status | CpuFlags::CARRY_FLAG;
        } else {
            self.status = self.status & CpuFlags::CARRY_FLAG.reverse_bits();
        }
        let addr = self.get_operand_address(mode);
        let shift_amnt = self.mem_read(addr);
        self.set_register_a(self.register_a << shift_amnt);
    }

    fn asl_accumulator(&mut self) {
        if self.register_a & CpuFlags::NEGATIVE != 0 { // if bit 7 is set
            self.status = self.status | CpuFlags::CARRY_FLAG;
        } else {
            self.status = self.status & CpuFlags::CARRY_FLAG.reverse_bits();
        }
        self.set_register_a(self.register_a << 1);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    //pub fn interpret(&mut self, program: Vec<u8>) {}

    pub fn run(&mut self) {
        let ref opcodes = *op_codes::CPU_CODES_MAP;
        loop {
            let code = self.memory[self.program_counter as usize];

            let op_code = opcodes
                .get(&code)
                .expect(&format!("Opcode {:x} not recognized!", code));

            println!("running inst {} with opcode {:x}", &op_code.name, code);
            self.program_counter += 1;
            let pc_state_before_inst_execution = self.program_counter;

            match code {
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                    self.lda(&op_code.addressing_mode);
                }
                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
                    self.and(&op_code.addressing_mode);
                }
                0x06 | 0x16 | 0x0e | 0x1e => {
                    self.asl(&op_code.addressing_mode);
                }
                0x0A => {
                    self.asl_accumulator();
                }

                0x85 => {
                    self.sta(&op_code.addressing_mode);
                }

                0x90 => self.bcc(),

                0xAA => self.tax(),

                0xE8 => self.inx(),

                0x00 => return,

                _ => todo!(),
            }
            println!("{}", self.program_counter);
            if pc_state_before_inst_execution == self.program_counter {
                self.program_counter += (op_code.bytes - 1) as u16
            }
            println!("{}", self.program_counter);
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

   // todo test all addressing modes for lda

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
    cpu.load_and_run(vec![0xa9, 0b0000_0001, 0x85, 0x00, 0x06, 0x00, 0x00]);
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
}
