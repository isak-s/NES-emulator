use std::{collections::HashMap, sync::LazyLock};

use lazy_static::lazy_static;

use crate::addressing_modes::{AddressingMode};

pub struct OpCode {
    pub code: u8,
    pub name: &'static str,  // 3 char string, wil not change during runtime
    pub bytes: u8,
    pub cycles: u8,
    pub addressing_mode: AddressingMode
}

impl OpCode {
    const fn new(
        code: u8,
        name: &'static str,
        bytes: u8,
        cycles: u8,
        addressing_mode: AddressingMode) -> Self {
        OpCode { code, name, bytes, cycles, addressing_mode }
    }
}

lazy_static!(
    pub static ref CPU_OP_CODES: Vec<OpCode> = vec![
    OpCode::new(0x00, "BRK", 1, 7, AddressingMode::NoneAddressing),

    // BEGIN Loads
    OpCode::new(0xA9, "LDA", 2, 2, AddressingMode::Immediate),
    OpCode::new(0xA5, "LDA", 2, 3, AddressingMode::ZeroPage),
    OpCode::new(0xB5, "LDA", 2, 4, AddressingMode::ZeroPage_X),
    OpCode::new(0xAD, "LDA", 3, 4, AddressingMode::Absolute),
    OpCode::new(0xBD, "LDA", 3, 4, AddressingMode::Absolute_X), // +1 if page crossed
    OpCode::new(0xB9, "LDA", 3, 4, AddressingMode::Absolute_Y), // +1 if page crossed
    OpCode::new(0xA1, "LDA", 2, 6, AddressingMode::Indirect_X),
    OpCode::new(0x00, "LDA", 2, 5, AddressingMode::Indirect_Y), // +1 if page crossed
    // END Loads

    // Begin Stores
    OpCode::new(0x85, "STA", 2, 3, AddressingMode::ZeroPage),  // store accumilator
    OpCode::new(0xAA, "TAX", 1, 2, AddressingMode::NoneAddressing),  // transfer a to x
    // End Stores

    // BEGIN Arithmetic
    OpCode::new(0xE8, "INX", 1, 2, AddressingMode::NoneAddressing),
    // END Arithmetic
    ];

    pub static ref CPU_CODES_MAP: HashMap<u8, &'static OpCode> = {
        let mut map = HashMap::new();
        for cpu_op in &*CPU_OP_CODES {
            map.insert(cpu_op.code, cpu_op);
        }
        map
    };
);