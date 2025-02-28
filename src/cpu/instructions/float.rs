//! Floating-point (F-type) instruction implementations
//! 
//! This module implements the floating-point instructions for the IA-64 architecture.

use super::{Instruction, InstructionFields, RegisterType};
use crate::EmulatorError;
use crate::cpu::Cpu;
use crate::memory::Memory;

/// Floating-point add instruction
#[derive(Debug)]
pub struct FAdd {
    fields: InstructionFields,
}

impl FAdd {
    /// Create new FADD instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }
}

impl Instruction for FAdd {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source registers
        let src1 = match self.fields.sources[0] {
            RegisterType::FR(reg) => cpu.get_fr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        let src2 = match self.fields.sources[1] {
            RegisterType::FR(reg) => cpu.get_fr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Perform floating-point addition
        let result = src1 + src2;

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::FR(reg) => cpu.set_fr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Floating-point subtract instruction
#[derive(Debug)]
pub struct FSub {
    fields: InstructionFields,
}

impl FSub {
    /// Create new FSUB instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }
}

impl Instruction for FSub {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source registers
        let src1 = match self.fields.sources[0] {
            RegisterType::FR(reg) => cpu.get_fr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        let src2 = match self.fields.sources[1] {
            RegisterType::FR(reg) => cpu.get_fr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Perform floating-point subtraction
        let result = src1 - src2;

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::FR(reg) => cpu.set_fr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Floating-point multiply instruction
#[derive(Debug)]
pub struct FMul {
    fields: InstructionFields,
}

impl FMul {
    /// Create new FMUL instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }
}

impl Instruction for FMul {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source registers
        let src1 = match self.fields.sources[0] {
            RegisterType::FR(reg) => cpu.get_fr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        let src2 = match self.fields.sources[1] {
            RegisterType::FR(reg) => cpu.get_fr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Perform floating-point multiplication
        let result = src1 * src2;

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::FR(reg) => cpu.set_fr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Floating-point divide instruction
#[derive(Debug)]
pub struct FDiv {
    fields: InstructionFields,
}

impl FDiv {
    /// Create new FDIV instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }
}

impl Instruction for FDiv {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source registers
        let src1 = match self.fields.sources[0] {
            RegisterType::FR(reg) => cpu.get_fr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        let src2 = match self.fields.sources[1] {
            RegisterType::FR(reg) => cpu.get_fr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Check for division by zero
        if src2 == 0.0 {
            return Err(EmulatorError::ExecutionError("Division by zero".to_string()));
        }

        // Perform floating-point division
        let result = src1 / src2;

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::FR(reg) => cpu.set_fr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{Memory, Permissions};
    use std::f64;

    fn setup_test() -> (Cpu, Memory, InstructionFields) {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();
        memory.map(0x1000, 4096, Permissions::ReadWriteExecute).unwrap();
        
        // Initialize predicate registers
        cpu.set_pr(0, true).unwrap(); // Set p0 to true by default
        
        let fields = InstructionFields {
            qp: 0,
            major_op: 0,
            sources: vec![RegisterType::FR(1), RegisterType::FR(2)],
            destinations: vec![RegisterType::FR(3)],
            immediate: None,
            addressing: None,
        };
        (cpu, memory, fields)
    }

    #[test]
    fn test_fadd() {
        let (mut cpu, mut memory, fields) = setup_test();
        let fadd = FAdd::new(fields);

        // Test basic addition
        cpu.set_fr(1, 3.14).unwrap();
        cpu.set_fr(2, 2.86).unwrap();
        fadd.execute(&mut cpu, &mut memory).unwrap();
        assert!((cpu.get_fr(3).unwrap() - 6.0).abs() < f64::EPSILON);

        // Test with negative numbers
        cpu.set_fr(1, -1.5).unwrap();
        cpu.set_fr(2, -2.5).unwrap();
        fadd.execute(&mut cpu, &mut memory).unwrap();
        assert!((cpu.get_fr(3).unwrap() - (-4.0)).abs() < f64::EPSILON);

        // Test with infinity
        cpu.set_fr(1, f64::INFINITY).unwrap();
        cpu.set_fr(2, 1.0).unwrap();
        fadd.execute(&mut cpu, &mut memory).unwrap();
        assert!(cpu.get_fr(3).unwrap().is_infinite());
    }

    #[test]
    fn test_fsub() {
        let (mut cpu, mut memory, fields) = setup_test();
        let fsub = FSub::new(fields);

        // Test basic subtraction
        cpu.set_fr(1, 5.0).unwrap();
        cpu.set_fr(2, 3.0).unwrap();
        fsub.execute(&mut cpu, &mut memory).unwrap();
        assert!((cpu.get_fr(3).unwrap() - 2.0).abs() < f64::EPSILON);

        // Test with negative numbers
        cpu.set_fr(1, -1.5).unwrap();
        cpu.set_fr(2, 2.5).unwrap();
        fsub.execute(&mut cpu, &mut memory).unwrap();
        assert!((cpu.get_fr(3).unwrap() - (-4.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fmul() {
        let (mut cpu, mut memory, fields) = setup_test();
        let fmul = FMul::new(fields);

        // Test basic multiplication
        cpu.set_fr(1, 2.5).unwrap();
        cpu.set_fr(2, 4.0).unwrap();
        fmul.execute(&mut cpu, &mut memory).unwrap();
        assert!((cpu.get_fr(3).unwrap() - 10.0).abs() < f64::EPSILON);

        // Test with zero
        cpu.set_fr(1, 1.5).unwrap();
        cpu.set_fr(2, 0.0).unwrap();
        fmul.execute(&mut cpu, &mut memory).unwrap();
        assert!((cpu.get_fr(3).unwrap() - 0.0).abs() < f64::EPSILON);

        // Test with infinity
        cpu.set_fr(1, f64::INFINITY).unwrap();
        cpu.set_fr(2, 2.0).unwrap();
        fmul.execute(&mut cpu, &mut memory).unwrap();
        assert!(cpu.get_fr(3).unwrap().is_infinite());
    }

    #[test]
    fn test_fdiv() {
        let (mut cpu, mut memory, fields) = setup_test();
        let fdiv = FDiv::new(fields);

        // Test basic division
        cpu.set_fr(1, 10.0).unwrap();
        cpu.set_fr(2, 2.0).unwrap();
        fdiv.execute(&mut cpu, &mut memory).unwrap();
        assert!((cpu.get_fr(3).unwrap() - 5.0).abs() < f64::EPSILON);

        // Test division by zero
        cpu.set_fr(1, 1.0).unwrap();
        cpu.set_fr(2, 0.0).unwrap();
        assert!(fdiv.execute(&mut cpu, &mut memory).is_err());

        // Test with infinity
        cpu.set_fr(1, f64::INFINITY).unwrap();
        cpu.set_fr(2, 2.0).unwrap();
        fdiv.execute(&mut cpu, &mut memory).unwrap();
        assert!(cpu.get_fr(3).unwrap().is_infinite());
    }

    #[test]
    fn test_predicated_execution() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        fields.qp = 1;
        let fadd = FAdd::new(fields);

        // Set initial values
        cpu.set_fr(1, 2.0).unwrap();
        cpu.set_fr(2, 3.0).unwrap();
        cpu.set_fr(3, 0.0).unwrap();

        // Test with false predicate
        cpu.set_pr(1, false).unwrap();
        fadd.execute(&mut cpu, &mut memory).unwrap();
        assert!((cpu.get_fr(3).unwrap() - 0.0).abs() < f64::EPSILON);

        // Test with true predicate
        cpu.set_pr(1, true).unwrap();
        fadd.execute(&mut cpu, &mut memory).unwrap();
        assert!((cpu.get_fr(3).unwrap() - 5.0).abs() < f64::EPSILON);
    }
} 