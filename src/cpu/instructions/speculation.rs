//! Speculation instructions
//! 
//! This module implements data and control speculation instructions for the IA-64
//! architecture, including advanced load, check, and recovery operations.

use super::{Instruction, InstructionFields, CompletionType, RegisterType, AddressingMode};
use crate::EmulatorError;
use crate::cpu::Cpu;
use crate::memory::Memory;

/// Advanced load instruction
#[derive(Debug)]
pub struct AdvancedLoad {
    fields: InstructionFields,
}

impl AdvancedLoad {
    /// Create new advanced load instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }

    /// Calculate effective address
    fn calc_effective_address(&self, cpu: &Cpu) -> Result<u64, EmulatorError> {
        match self.fields.addressing.unwrap() {
            AddressingMode::Indirect(reg) => {
                cpu.get_gr(reg as usize)
            },
            AddressingMode::IndirectOffset(reg, offset) => {
                let base = cpu.get_gr(reg as usize)?;
                Ok(base.wrapping_add(offset as u64))
            },
            AddressingMode::IndirectIndex(base, index) => {
                let base_val = cpu.get_gr(base as usize)?;
                let index_val = cpu.get_gr(index as usize)?;
                Ok(base_val.wrapping_add(index_val))
            },
            AddressingMode::Absolute(addr) => Ok(addr),
        }
    }
}

impl Instruction for AdvancedLoad {
    fn execute(&self, cpu: &mut Cpu, memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Calculate effective address
        let addr = self.calc_effective_address(cpu)?;

        // Perform speculative load
        let value = memory.read_u64(addr)?;

        // Get destination register
        let (reg, is_integer) = match self.fields.destinations[0] {
            RegisterType::GR(reg) => (reg, true),
            RegisterType::FR(reg) => (reg, false),
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        };

        // Add entry to ALAT
        cpu.alat_add_entry(addr, 8, reg, is_integer)?;

        // Store value in destination register
        match self.fields.destinations[0] {
            RegisterType::GR(reg) => cpu.set_gr(reg as usize, value)?,
            RegisterType::FR(reg) => cpu.set_fr(reg as usize, value)?,
            _ => unreachable!(),
        }

        Ok(())
    }

    fn latency(&self) -> u32 {
        2 // Advanced loads typically take 2 cycles
    }
}

/// Check load instruction
#[derive(Debug)]
pub struct CheckLoad {
    fields: InstructionFields,
}

impl CheckLoad {
    /// Create new check load instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }
}

impl Instruction for CheckLoad {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get register to check
        let (reg, is_integer) = match self.fields.sources[0] {
            RegisterType::GR(reg) => (reg, true),
            RegisterType::FR(reg) => (reg, false),
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Check ALAT for entry
        if !cpu.alat_check_register(reg, is_integer) {
            // No valid entry found, branch to recovery code
            if let Some(recovery_addr) = self.fields.recovery_addr {
                cpu.ip = recovery_addr;
            }
        }

        Ok(())
    }

    fn latency(&self) -> u32 {
        1 // Check operations are typically fast
    }
}

/// Recovery code branch instruction
#[derive(Debug)]
pub struct RecoveryBranch {
    fields: InstructionFields,
}

impl RecoveryBranch {
    /// Create new recovery branch instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }
}

impl Instruction for RecoveryBranch {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get recovery code address
        let target = match self.fields.sources[0] {
            RegisterType::BR(reg) => cpu.get_br(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Branch to recovery code
        cpu.ip = target;

        Ok(())
    }

    fn latency(&self) -> u32 {
        2 // Branch operations typically take 2 cycles
    }
}

/// Clear ALAT instruction
#[derive(Debug)]
pub struct ClearAlat {
    fields: InstructionFields,
}

impl ClearAlat {
    /// Create new clear ALAT instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }
}

impl Instruction for ClearAlat {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Clear all ALAT entries
        cpu.alat_clear();

        Ok(())
    }

    fn latency(&self) -> u32 {
        1 // Clear operations are typically fast
    }
}

/// Store with ALAT update instruction
#[derive(Debug)]
pub struct StoreUpdate {
    fields: InstructionFields,
}

impl StoreUpdate {
    /// Create new store update instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }

    /// Calculate effective address
    fn calc_effective_address(&self, cpu: &Cpu) -> Result<u64, EmulatorError> {
        match self.fields.addressing.unwrap() {
            AddressingMode::Indirect(reg) => {
                cpu.get_gr(reg as usize)
            },
            AddressingMode::IndirectOffset(reg, offset) => {
                let base = cpu.get_gr(reg as usize)?;
                Ok(base.wrapping_add(offset as u64))
            },
            AddressingMode::IndirectIndex(base, index) => {
                let base_val = cpu.get_gr(base as usize)?;
                let index_val = cpu.get_gr(index as usize)?;
                Ok(base_val.wrapping_add(index_val))
            },
            AddressingMode::Absolute(addr) => Ok(addr),
        }
    }
}

impl Instruction for StoreUpdate {
    fn execute(&self, cpu: &mut Cpu, memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Calculate effective address
        let addr = self.calc_effective_address(cpu)?;

        // Get value to store
        let value = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            RegisterType::FR(reg) => cpu.get_fr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Perform store
        memory.write_u64(addr, value)?;

        // Invalidate any overlapping ALAT entries
        cpu.alat_invalidate_overlap(addr, 8);

        Ok(())
    }

    fn latency(&self) -> u32 {
        2 // Store operations typically take 2 cycles
    }
} 