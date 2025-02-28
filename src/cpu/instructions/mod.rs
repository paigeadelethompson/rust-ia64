//! CPU instruction implementations
//!
//! This module contains implementations of the IA-64 instruction set.

use crate::cpu::Cpu;
use crate::memory::Memory;
use crate::EmulatorError;

pub mod alu;
pub mod branch;
pub mod float;
pub mod memory;
pub mod system;

/// Common trait for all instructions
pub trait Instruction {
    /// Execute the instruction
    fn execute(&self, cpu: &mut Cpu, memory: &mut Memory) -> Result<(), EmulatorError>;
}

/// Instruction completion type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionType {
    /// Instruction completed normally
    Normal,
    /// Instruction caused a branch
    Branch(u64), // Target address
    /// Instruction caused an exception
    Exception(Exception),
}

/// Exception types that can occur during instruction execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exception {
    /// Invalid operation
    InvalidOperation,
    /// Illegal instruction
    IllegalInstruction,
    /// Privileged operation
    PrivilegedOperation,
    /// Memory access fault
    MemoryFault,
    /// Floating point exception
    FloatingPoint,
    /// Debug exception
    Debug,
}

/// Register types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterType {
    /// General register
    GR(u8),
    /// Floating point register
    FR(u8),
    /// Predicate register
    PR(u8),
    /// Branch register
    BR(u8),
    /// Address register
    AR(u8),
    /// Control register
    CR(u8),
    /// Return register
    RR(u8),
    /// Predicate key register
    PKR(u8),
    /// Data buffer register
    DBR(u8),
    /// Data descriptor register
    DDR(u8),
}

/// Addressing modes
#[derive(Debug, Clone, Copy)]
pub enum AddressingMode {
    /// Register indirect
    Indirect(u8),
    /// Register indirect with offset
    IndirectOffset(u8, i64),
    /// Register indirect with index
    IndirectIndex(u8, u8),
    /// Absolute address
    Absolute(u64),
}

/// Common instruction format fields
#[derive(Debug, Clone)]
pub struct InstructionFields {
    /// Predicate register
    pub qp: u8,
    /// Major opcode
    pub major_op: u8,
    /// Source registers
    pub sources: Vec<RegisterType>,
    /// Destination registers
    pub destinations: Vec<RegisterType>,
    /// Immediate value if present
    pub immediate: Option<i64>,
    /// Addressing mode if present
    pub addressing: Option<AddressingMode>,
}

impl InstructionFields {
    /// Create new instruction fields
    pub fn new(
        qp: u8,
        major_op: u8,
        sources: Vec<RegisterType>,
        destinations: Vec<RegisterType>,
        immediate: Option<i64>,
        addressing: Option<AddressingMode>,
    ) -> Self {
        Self {
            qp,
            major_op,
            sources,
            destinations,
            immediate,
            addressing,
        }
    }
}

impl RegisterType {
    /// Gets the register number from the register type
    pub fn get_reg_num(&self) -> usize {
        match self {
            RegisterType::GR(n) => *n as usize,
            RegisterType::FR(n) => *n as usize,
            RegisterType::PR(n) => *n as usize,
            RegisterType::BR(n) => *n as usize,
            RegisterType::AR(n) => *n as usize,
            RegisterType::CR(n) => *n as usize,
            RegisterType::RR(n) => *n as usize,
            RegisterType::PKR(n) => *n as usize,
            RegisterType::DBR(n) => *n as usize,
            RegisterType::DDR(n) => *n as usize,
        }
    }
}
