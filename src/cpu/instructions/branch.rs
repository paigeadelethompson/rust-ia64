//! Branch (B-type) instruction implementations
//! 
//! This module implements the branch instructions for the IA-64 architecture.

use super::{Instruction, InstructionFields, RegisterType};
use crate::EmulatorError;
use crate::cpu::Cpu;
use crate::memory::Memory;

/// Branch types
#[derive(Debug, Clone, Copy)]
pub enum BranchType {
    /// Unconditional branch
    Unconditional,
    /// Branch if equal
    Equal,
    /// Branch if not equal
    NotEqual,
    /// Branch if less than
    LessThan,
    /// Branch if less than or equal
    LessEqual,
    /// Branch if greater than
    GreaterThan,
    /// Branch if greater than or equal
    GreaterEqual,
}

/// Branch prediction type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BranchPrediction {
    /// Static prediction take branch
    StaticTake,
    /// Static prediction not taken
    StaticNotTaken,
    /// Dynamic prediction take branch
    DynamicTake,
    /// Dynamic prediction not taken
    DynamicNotTaken,
}

/// Branch RSE behavior
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BranchRSE {
    /// Normal RSE operation
    Normal,
    /// Clear RSE on branch
    Clear,
}

/// Branch importance
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BranchImportance {
    /// Normal branch
    Normal,
    /// Important branch (for branch trace)
    Important,
}

/// Branch register stack impact
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BranchRegisters {
    /// Few registers to save
    Few,
    /// Many registers to save
    Many,
}

/// Branch instruction
#[derive(Debug)]
pub struct Branch {
    fields: InstructionFields,
    branch_type: BranchType,
    prediction: BranchPrediction,
    rse_behavior: BranchRSE,
    importance: BranchImportance,
    registers: BranchRegisters,
}

impl Branch {
    /// Create new branch instruction
    pub fn new(
        fields: InstructionFields, 
        branch_type: BranchType,
        prediction: BranchPrediction,
        rse_behavior: BranchRSE,
        importance: BranchImportance,
        registers: BranchRegisters,
    ) -> Self {
        Self { 
            fields, 
            branch_type,
            prediction,
            rse_behavior,
            importance,
            registers,
        }
    }

    /// Create new branch instruction from decoded instruction
    pub fn from_decoded(fields: InstructionFields, branch_type: BranchType, completers: Option<Vec<String>>) -> Self {
        // Default values
        let mut prediction = BranchPrediction::StaticTake;
        let mut rse_behavior = BranchRSE::Normal;
        let mut importance = BranchImportance::Normal;
        let mut registers = BranchRegisters::Few;

        // Parse completers if present
        if let Some(completers) = completers {
            for completer in completers {
                match completer.as_str() {
                    "sptk" => prediction = BranchPrediction::StaticTake,
                    "spnt" => prediction = BranchPrediction::StaticNotTaken,
                    "dptk" => prediction = BranchPrediction::DynamicTake,
                    "dpnt" => prediction = BranchPrediction::DynamicNotTaken,
                    "clr" => rse_behavior = BranchRSE::Clear,
                    "imp" => importance = BranchImportance::Important,
                    "few" => registers = BranchRegisters::Few,
                    "many" => registers = BranchRegisters::Many,
                    "" => (), // Skip empty completers
                    _ => (), // Ignore unknown completers
                }
            }
        }

        Self::new(fields, branch_type, prediction, rse_behavior, importance, registers)
    }

    /// Calculate branch target address
    fn calc_target(&self, cpu: &Cpu) -> Result<u64, EmulatorError> {
        match &self.fields.immediate {
            Some(offset) => {
                // IP-relative branch
                Ok(cpu.ip.wrapping_add(*offset as u64))
            }
            None => {
                // Register-indirect branch
                match self.fields.sources[0] {
                    RegisterType::BR(reg) => cpu.get_br(reg as usize),
                    _ => Err(EmulatorError::ExecutionError(
                        "Invalid branch target register type".to_string()
                    )),
                }
            }
        }
    }

    /// Check branch condition
    fn check_condition(&self, cpu: &Cpu) -> Result<bool, EmulatorError> {
        match self.branch_type {
            BranchType::Unconditional => Ok(true),
            BranchType::Equal => {
                let src1 = match self.fields.sources[0] {
                    RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
                    _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
                };
                let src2 = match self.fields.sources[1] {
                    RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
                    _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
                };
                Ok(src1 == src2)
            }
            BranchType::NotEqual => {
                let src1 = match self.fields.sources[0] {
                    RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
                    _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
                };
                let src2 = match self.fields.sources[1] {
                    RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
                    _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
                };
                Ok(src1 != src2)
            }
            BranchType::LessThan => {
                let src1 = match self.fields.sources[0] {
                    RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
                    _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
                };
                let src2 = match self.fields.sources[1] {
                    RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
                    _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
                };
                Ok((src1 as i64) < (src2 as i64))
            }
            BranchType::LessEqual => {
                let src1 = match self.fields.sources[0] {
                    RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
                    _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
                };
                let src2 = match self.fields.sources[1] {
                    RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
                    _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
                };
                Ok((src1 as i64) <= (src2 as i64))
            }
            BranchType::GreaterThan => {
                let src1 = match self.fields.sources[0] {
                    RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
                    _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
                };
                let src2 = match self.fields.sources[1] {
                    RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
                    _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
                };
                Ok((src1 as i64) > (src2 as i64))
            }
            BranchType::GreaterEqual => {
                let src1 = match self.fields.sources[0] {
                    RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
                    _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
                };
                let src2 = match self.fields.sources[1] {
                    RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
                    _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
                };
                Ok((src1 as i64) >= (src2 as i64))
            }
        }
    }
}

impl Instruction for Branch {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Check branch condition
        if self.check_condition(cpu)? {
            // Calculate target address
            let target = self.calc_target(cpu)?;

            // Handle RSE behavior
            if self.rse_behavior == BranchRSE::Clear {
                // TODO: Implement RSE clear operation
            }

            // Update branch register if specified
            if let Some(RegisterType::BR(reg)) = self.fields.destinations.get(0) {
                cpu.set_br(*reg as usize, cpu.ip.wrapping_add(16))?; // Save return address
            }

            // Update branch prediction information
            match self.prediction {
                BranchPrediction::StaticTake | BranchPrediction::DynamicTake => {
                    // TODO: Update branch predictor state
                }
                BranchPrediction::StaticNotTaken | BranchPrediction::DynamicNotTaken => {
                    // TODO: Update branch predictor state
                }
            }

            // Handle register stack impact
            match self.registers {
                BranchRegisters::Few => {
                    // TODO: Optimize register stack save for few registers
                }
                BranchRegisters::Many => {
                    // TODO: Full register stack save
                }
            }

            // Update IP
            cpu.ip = target;

            // Handle branch importance
            if self.importance == BranchImportance::Important {
                // TODO: Add to branch trace buffer
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{Memory, Permissions};

    fn setup_test() -> (Cpu, Memory, InstructionFields) {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();
        memory.map(0x1000, 4096, Permissions::ReadWriteExecute).unwrap();
        
        // Initialize predicate registers
        cpu.set_pr(0, true).unwrap(); // Set p0 to true by default
        
        let fields = InstructionFields {
            qp: 0,
            major_op: 0,
            sources: vec![RegisterType::GR(1), RegisterType::GR(2)],
            destinations: vec![RegisterType::BR(3)],
            immediate: Some(16), // Default offset of 16 bytes
            addressing: None,
        };
        (cpu, memory, fields)
    }

    #[test]
    fn test_unconditional_branch() {
        let (mut cpu, mut memory, fields) = setup_test();
        let branch = Branch::new(fields, BranchType::Unconditional, BranchPrediction::StaticTake, BranchRSE::Normal, BranchImportance::Normal, BranchRegisters::Few);

        cpu.ip = 0x1000;
        branch.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.ip, 0x1010); // Should branch to IP + 16
        assert_eq!(cpu.get_br(3).unwrap(), 0x1010); // Return address should be IP + 16
    }

    #[test]
    fn test_conditional_branch_equal_taken() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        fields.sources = vec![RegisterType::GR(1), RegisterType::GR(2)];
        let branch = Branch::new(fields, BranchType::Equal, BranchPrediction::StaticTake, BranchRSE::Normal, BranchImportance::Normal, BranchRegisters::Few);

        cpu.ip = 0x1000;
        cpu.set_gr(1, 42).unwrap();
        cpu.set_gr(2, 42).unwrap();
        branch.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.ip, 0x1010); // Should branch when equal
    }

    #[test]
    fn test_conditional_branch_equal_not_taken() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        fields.sources = vec![RegisterType::GR(1), RegisterType::GR(2)];
        let branch = Branch::new(fields, BranchType::Equal, BranchPrediction::StaticNotTaken, BranchRSE::Normal, BranchImportance::Normal, BranchRegisters::Few);

        cpu.ip = 0x1000;
        cpu.set_gr(1, 42).unwrap();
        cpu.set_gr(2, 43).unwrap();
        branch.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.ip, 0x1000); // Should not branch when not equal
    }

    #[test]
    fn test_conditional_branch_less_than() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        fields.sources = vec![RegisterType::GR(1), RegisterType::GR(2)];
        let branch = Branch::new(fields, BranchType::LessThan, BranchPrediction::StaticTake, BranchRSE::Normal, BranchImportance::Normal, BranchRegisters::Few);

        // Test signed comparison
        cpu.ip = 0x1000;
        cpu.set_gr(1, u64::MAX).unwrap(); // -1 in two's complement
        cpu.set_gr(2, 0).unwrap();
        branch.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.ip, 0x1010); // Should branch when less than

        // Reset IP and test not taken case
        cpu.ip = 0x1000;
        cpu.set_gr(1, 5).unwrap();
        cpu.set_gr(2, 3).unwrap();
        branch.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.ip, 0x1000); // Should not branch when not less than
    }

    #[test]
    fn test_predicated_branch() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        fields.qp = 1; // Use p1 for predication
        let branch = Branch::new(fields, BranchType::Unconditional, BranchPrediction::StaticTake, BranchRSE::Normal, BranchImportance::Normal, BranchRegisters::Few);

        cpu.ip = 0x1000;
        cpu.set_pr(1, false).unwrap();
        branch.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.ip, 0x1000); // Should not branch when predicate is false

        cpu.set_pr(1, true).unwrap();
        branch.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.ip, 0x1010); // Should branch when predicate is true
    }

    #[test]
    fn test_register_indirect_branch() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        fields.immediate = None;
        fields.sources = vec![RegisterType::BR(1)];
        let branch = Branch::new(fields, BranchType::Unconditional, BranchPrediction::StaticTake, BranchRSE::Normal, BranchImportance::Normal, BranchRegisters::Few);

        cpu.ip = 0x1000;
        cpu.set_br(1, 0x2000).unwrap();
        branch.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.ip, 0x2000); // Should branch to target in BR1
        assert_eq!(cpu.get_br(3).unwrap(), 0x1010); // Return address should be IP + 16
    }

    #[test]
    fn test_branch_completers() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        
        // Test branch with all completers
        let completers = Some(vec![
            "dptk".to_string(),
            "clr".to_string(),
            "imp".to_string(),
            "many".to_string(),
        ]);
        
        let branch = Branch::from_decoded(fields.clone(), BranchType::Unconditional, completers);
        
        // Verify completer values
        assert!(matches!(branch.prediction, BranchPrediction::DynamicTake));
        assert!(matches!(branch.rse_behavior, BranchRSE::Clear));
        assert!(matches!(branch.importance, BranchImportance::Important));
        assert!(matches!(branch.registers, BranchRegisters::Many));
        
        // Test branch with default completers
        let branch = Branch::from_decoded(fields.clone(), BranchType::Unconditional, None);
        
        // Verify default values
        assert!(matches!(branch.prediction, BranchPrediction::StaticTake));
        assert!(matches!(branch.rse_behavior, BranchRSE::Normal));
        assert!(matches!(branch.importance, BranchImportance::Normal));
        assert!(matches!(branch.registers, BranchRegisters::Few));
        
        // Test branch with partial completers
        let completers = Some(vec![
            "spnt".to_string(),
            "".to_string(), // Empty completer should be ignored
            "imp".to_string(),
        ]);
        
        let branch = Branch::from_decoded(fields, BranchType::Unconditional, completers);
        
        // Verify mixed values
        assert!(matches!(branch.prediction, BranchPrediction::StaticNotTaken));
        assert!(matches!(branch.rse_behavior, BranchRSE::Normal));
        assert!(matches!(branch.importance, BranchImportance::Important));
        assert!(matches!(branch.registers, BranchRegisters::Few));
    }

    #[test]
    fn test_branch_execution_with_completers() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        fields.sources = vec![RegisterType::GR(1), RegisterType::GR(2)];
        
        // Test branch with RSE clear
        let completers = Some(vec!["sptk".to_string(), "clr".to_string()]);
        let branch = Branch::from_decoded(fields.clone(), BranchType::Equal, completers);
        
        cpu.ip = 0x1000;
        cpu.set_gr(1, 42).unwrap();
        cpu.set_gr(2, 42).unwrap();
        branch.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.ip, 0x1010); // Should branch when equal
        
        // Test branch with importance flag
        let completers = Some(vec!["dptk".to_string(), "imp".to_string()]);
        let branch = Branch::from_decoded(fields, BranchType::Equal, completers);
        
        cpu.ip = 0x1000;
        cpu.set_gr(1, 42).unwrap();
        cpu.set_gr(2, 43).unwrap();
        branch.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.ip, 0x1000); // Should not branch when not equal
    }
} 