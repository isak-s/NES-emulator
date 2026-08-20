use std::sync::LazyLock;

use crate::addressing_modes::{AddressingMode};

pub struct OpCode {
    op_code: u8,
    op_name: &'static str,  // 3 char string, wil not change during runtime
    bytes: u8,
    cycles: u8,
    addressing_mode: AddressingMode
}

impl OpCode {
    const fn new(
        op_code: u8,
        op_name: &'static str,
        bytes: u8,
        cycles: u8,
        addressing_mode: AddressingMode) -> Self {
        OpCode { op_code, op_name, bytes, cycles, addressing_mode }
    }
}

pub static CPU_OP_CODES: LazyLock<Vec<OpCode>> = LazyLock::new(|| {
vec![
    OpCode::new(0x00, "BRK", 1, 7, AddressingMode::NoneAddressing),
    OpCode::new(0xAA, "TAX", 1, 2, AddressingMode::NoneAddressing),

    OpCode::new(0xA9, "LDA", 2, 2, AddressingMode::Immediate),
    OpCode::new(0xA5, "LDA", 2, 3, AddressingMode::ZeroPage),
    OpCode::new(0xB5, "LDA", 2, 4, AddressingMode::ZeroPage_X),
    OpCode::new(0xAD, "LDA", 3, 4, AddressingMode::Absolute),
    OpCode::new(0xBD, "LDA", 3, 4, AddressingMode::Absolute_X), // +1 if page crossed
    OpCode::new(0xB9, "LDA", 3, 4, AddressingMode::Absolute_Y), // +1 if page crossed
    OpCode::new(0xA1, "LDA", 2, 6, AddressingMode::Indirect_X),
    OpCode::new(0x00, "LDA", 2, 5, AddressingMode::Indirect_Y), // +1 if page crossed
]}
);