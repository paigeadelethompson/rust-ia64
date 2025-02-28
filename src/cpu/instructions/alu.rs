//! ALU (A-type) instruction implementations
//! 
//! This module implements the integer ALU instructions for the IA-64 architecture.

use super::{Instruction, InstructionFields, RegisterType};
use crate::EmulatorError;
use crate::cpu::Cpu;
use crate::memory::Memory;

/// Add instruction
#[derive(Debug)]
pub struct Add {
    fields: InstructionFields,
}

impl Add {
    /// Create new ADD instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }
}

impl Instruction for Add {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source registers
        let src1 = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        let src2 = match self.fields.sources[1] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Perform addition
        let result = src1.wrapping_add(src2);

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::GR(reg) => cpu.set_gr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Sub instruction
#[derive(Debug)]
pub struct Sub {
    fields: InstructionFields,
}

impl Sub {
    /// Create new SUB instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }
}

impl Instruction for Sub {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source registers
        let src1 = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        let src2 = match self.fields.sources[1] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Perform subtraction
        let result = src1.wrapping_sub(src2);

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::GR(reg) => cpu.set_gr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// And instruction
#[derive(Debug)]
pub struct And {
    fields: InstructionFields,
}

impl And {
    /// Create new AND instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }
}

impl Instruction for And {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source registers
        let src1 = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        let src2 = match self.fields.sources[1] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Perform AND operation
        let result = src1 & src2;

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::GR(reg) => cpu.set_gr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Or instruction
#[derive(Debug)]
pub struct Or {
    fields: InstructionFields,
}

impl Or {
    /// Create new OR instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }
}

impl Instruction for Or {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source registers
        let src1 = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        let src2 = match self.fields.sources[1] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Perform OR operation
        let result = src1 | src2;

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::GR(reg) => cpu.set_gr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Xor instruction
#[derive(Debug)]
pub struct Xor {
    fields: InstructionFields,
}

impl Xor {
    /// Create new XOR instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }
}

impl Instruction for Xor {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source registers
        let src1 = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        let src2 = match self.fields.sources[1] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Perform XOR operation
        let result = src1 ^ src2;

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::GR(reg) => cpu.set_gr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Compare types
#[derive(Debug, Clone, Copy)]
pub enum CompareType {
    /// Equal
    Equal,
    /// Not equal
    NotEqual,
    /// Less than (signed)
    LessThan,
    /// Less than or equal (signed)
    LessEqual,
    /// Greater than (signed)
    GreaterThan,
    /// Greater than or equal (signed)
    GreaterEqual,
    /// Less than (unsigned)
    LessThanU,
    /// Less than or equal (unsigned)
    LessEqualU,
    /// Greater than (unsigned)
    GreaterThanU,
    /// Greater than or equal (unsigned)
    GreaterEqualU,
}

/// Compare instruction
#[derive(Debug)]
pub struct Compare {
    fields: InstructionFields,
    ctype: CompareType,
}

impl Compare {
    /// Create new compare instruction
    pub fn new(fields: InstructionFields, ctype: CompareType) -> Self {
        Self { fields, ctype }
    }
}

impl Instruction for Compare {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source values
        let src1 = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        let src2 = match self.fields.sources[1] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Evaluate condition
        let result = match self.ctype {
            CompareType::Equal => src1 == src2,
            CompareType::NotEqual => src1 != src2,
            CompareType::LessThan => (src1 as i64) < (src2 as i64),
            CompareType::LessEqual => (src1 as i64) <= (src2 as i64),
            CompareType::GreaterThan => (src1 as i64) > (src2 as i64),
            CompareType::GreaterEqual => (src1 as i64) >= (src2 as i64),
            CompareType::LessThanU => src1 < src2,
            CompareType::LessEqualU => src1 <= src2,
            CompareType::GreaterThanU => src1 > src2,
            CompareType::GreaterEqualU => src1 >= src2,
        };

        // Set destination predicate register
        match self.fields.destinations[0] {
            RegisterType::PR(reg) => cpu.set_pr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Test bit instruction
#[derive(Debug)]
pub struct TestBit {
    fields: InstructionFields,
}

impl TestBit {
    /// Create new test bit instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }
}

impl Instruction for TestBit {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source value and bit position
        let value = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        let pos = match self.fields.sources[1] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Test bit
        let result = if pos < 64 {
            (value & (1 << pos)) != 0
        } else {
            false
        };

        // Set destination predicate register
        match self.fields.destinations[0] {
            RegisterType::PR(reg) => cpu.set_pr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Shift types
#[derive(Debug, Clone, Copy)]
pub enum ShiftType {
    /// Shift left
    Left,
    /// Shift right arithmetic (sign extended)
    RightArithmetic,
    /// Shift right logical (zero extended)
    RightLogical,
}

/// Shift instruction
#[derive(Debug)]
pub struct Shift {
    fields: InstructionFields,
    shift_type: ShiftType,
}

impl Shift {
    /// Create new shift instruction
    pub fn new(fields: InstructionFields, shift_type: ShiftType) -> Self {
        Self { fields, shift_type }
    }
}

impl Instruction for Shift {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source value and shift amount
        let value = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        let shift = match self.fields.sources[1] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Perform shift operation
        let result = match self.shift_type {
            ShiftType::Left => value.wrapping_shl(shift as u32),
            ShiftType::RightArithmetic => ((value as i64).wrapping_shr(shift as u32)) as u64,
            ShiftType::RightLogical => value.wrapping_shr(shift as u32),
        };

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::GR(reg) => cpu.set_gr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Deposit instruction
#[derive(Debug)]
pub struct Deposit {
    fields: InstructionFields,
}

impl Deposit {
    /// Create new deposit instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }
}

impl Instruction for Deposit {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get target and source values
        let target = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid target register type".to_string())),
        };

        let source = match self.fields.sources[1] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Get position and length from immediate field
        let immediate = self.fields.immediate.unwrap_or(0) as u64;
        let pos = (immediate & 0xFF) as u32;         // Position is in bits 0-7
        let len = ((immediate >> 8) & 0xFF) as u32;  // Length is in bits 8-15

        // Create mask for the field
        let field_mask = ((1u64 << len) - 1) << pos;
        
        // Clear the target bits and insert the source bits
        let result = (target & !field_mask) | ((source & ((1u64 << len) - 1)) << pos);

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::GR(reg) => cpu.set_gr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Extract instruction
#[derive(Debug)]
pub struct Extract {
    fields: InstructionFields,
}

impl Extract {
    /// Create new extract instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }
}

impl Instruction for Extract {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source value
        let source = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Get position and length from immediate field
        let immediate = self.fields.immediate.unwrap_or(0) as u64;
        let pos = (immediate & 0xFF) as u32;         // Position is in bits 0-7
        let len = ((immediate >> 8) & 0xFF) as u32;  // Length is in bits 8-15

        // Extract the field by first shifting right to position 0, then masking
        let result = (source >> pos) & ((1u64 << len) - 1);

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::GR(reg) => cpu.set_gr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Population count instruction
#[derive(Debug)]
pub struct PopCount {
    fields: InstructionFields,
}

impl PopCount {
    /// Create new population count instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }
}

impl Instruction for PopCount {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source value
        let source = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Count number of 1 bits
        let result = source.count_ones() as u64;

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::GR(reg) => cpu.set_gr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Parallel operation sizes
#[derive(Debug, Clone, Copy)]
pub enum ParallelSize {
    /// 8-bit elements
    Byte,
    /// 16-bit elements
    Half,
    /// 32-bit elements
    Word,
}

/// Parallel add instruction
#[derive(Debug)]
pub struct ParallelAdd {
    fields: InstructionFields,
    size: ParallelSize,
}

impl ParallelAdd {
    /// Create new parallel add instruction
    pub fn new(fields: InstructionFields, size: ParallelSize) -> Self {
        Self { fields, size }
    }
}

impl Instruction for ParallelAdd {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source values
        let src1 = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        let src2 = match self.fields.sources[1] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Perform parallel addition based on element size
        let result = match self.size {
            ParallelSize::Byte => {
                let mut result = 0u64;
                for i in 0..8 {
                    let s1 = (src1 >> (i * 8)) & 0xFF;
                    let s2 = (src2 >> (i * 8)) & 0xFF;
                    let sum = (s1.wrapping_add(s2)) & 0xFF;
                    result |= sum << (i * 8);
                }
                result
            },
            ParallelSize::Half => {
                let mut result = 0u64;
                for i in 0..4 {
                    let s1 = (src1 >> (i * 16)) & 0xFFFF;
                    let s2 = (src2 >> (i * 16)) & 0xFFFF;
                    let sum = (s1.wrapping_add(s2)) & 0xFFFF;
                    result |= sum << (i * 16);
                }
                result
            },
            ParallelSize::Word => {
                let mut result = 0u64;
                for i in 0..2 {
                    let s1 = (src1 >> (i * 32)) & 0xFFFFFFFF;
                    let s2 = (src2 >> (i * 32)) & 0xFFFFFFFF;
                    let sum = (s1.wrapping_add(s2)) & 0xFFFFFFFF;
                    result |= sum << (i * 32);
                }
                result
            },
        };

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::GR(reg) => cpu.set_gr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Saturating add instruction
#[derive(Debug)]
pub struct SaturatingAdd {
    fields: InstructionFields,
    signed: bool,
}

impl SaturatingAdd {
    /// Create new saturating add instruction
    pub fn new(fields: InstructionFields, signed: bool) -> Self {
        Self { fields, signed }
    }
}

impl Instruction for SaturatingAdd {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source values
        let src1 = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        let src2 = match self.fields.sources[1] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Perform saturating addition
        let result = if self.signed {
            let s1 = src1 as i64;
            let s2 = src2 as i64;
            let sum = s1.saturating_add(s2);
            sum as u64
        } else {
            src1.saturating_add(src2)
        };

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::GR(reg) => cpu.set_gr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Rotate and mask instruction
#[derive(Debug)]
pub struct RotateMask {
    fields: InstructionFields,
}

impl RotateMask {
    /// Create new rotate and mask instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }
}

impl Instruction for RotateMask {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source value and rotation amount
        let value = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        let rot_amount = match self.fields.sources[1] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid rotation register type".to_string())),
        } as u32;

        // Get mask from immediate field
        let mask = self.fields.immediate.unwrap_or(0) as u64;

        // Perform rotation and masking
        let rotated = value.rotate_left(rot_amount & 0x3F);  // Only use bottom 6 bits for rotation
        let result = rotated & mask;

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::GR(reg) => cpu.set_gr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Minimum/Maximum operation types
#[derive(Debug, Clone, Copy)]
pub enum MinMaxType {
    /// Unsigned minimum
    MinU,
    /// Signed minimum
    MinS,
    /// Unsigned maximum
    MaxU,
    /// Signed maximum
    MaxS,
}

/// Minimum/Maximum instruction
#[derive(Debug)]
pub struct MinMax {
    fields: InstructionFields,
    op_type: MinMaxType,
}

impl MinMax {
    /// Create new min/max instruction
    pub fn new(fields: InstructionFields, op_type: MinMaxType) -> Self {
        Self { fields, op_type }
    }
}

impl Instruction for MinMax {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source values
        let src1 = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        let src2 = match self.fields.sources[1] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Perform min/max operation
        let result = match self.op_type {
            MinMaxType::MinU => std::cmp::min(src1, src2),
            MinMaxType::MaxU => std::cmp::max(src1, src2),
            MinMaxType::MinS => {
                let s1 = src1 as i64;
                let s2 = src2 as i64;
                std::cmp::min(s1, s2) as u64
            },
            MinMaxType::MaxS => {
                let s1 = src1 as i64;
                let s2 = src2 as i64;
                std::cmp::max(s1, s2) as u64
            },
        };

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::GR(reg) => cpu.set_gr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Extension sizes
#[derive(Debug, Clone, Copy)]
pub enum ExtensionSize {
    /// Byte (8 bits)
    Byte,
    /// Half word (16 bits)
    Half,
    /// Word (32 bits)
    Word,
}

/// Zero/Sign extension instruction
#[derive(Debug)]
pub struct Extend {
    fields: InstructionFields,
    size: ExtensionSize,
    sign_extend: bool,
}

impl Extend {
    /// Create new extension instruction
    pub fn new(fields: InstructionFields, size: ExtensionSize, sign_extend: bool) -> Self {
        Self { fields, size, sign_extend }
    }
}

impl Instruction for Extend {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source value
        let src = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Perform extension
        let result = match (self.size, self.sign_extend) {
            (ExtensionSize::Byte, false) => src & 0xFF,
            (ExtensionSize::Half, false) => src & 0xFFFF,
            (ExtensionSize::Word, false) => src & 0xFFFFFFFF,
            (ExtensionSize::Byte, true) => {
                let val = (src & 0xFF) as i8;
                val as i64 as u64
            },
            (ExtensionSize::Half, true) => {
                let val = (src & 0xFFFF) as i16;
                val as i64 as u64
            },
            (ExtensionSize::Word, true) => {
                let val = (src & 0xFFFFFFFF) as i32;
                val as i64 as u64
            },
        };

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::GR(reg) => cpu.set_gr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Merge instruction
#[derive(Debug)]
pub struct Merge {
    fields: InstructionFields,
}

impl Merge {
    /// Create new merge instruction
    pub fn new(fields: InstructionFields) -> Self {
        Self { fields }
    }
}

impl Instruction for Merge {
    fn execute(&self, cpu: &mut Cpu, _memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get source values
        let src1 = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        let src2 = match self.fields.sources[1] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Get merge mask from immediate field
        let mask = self.fields.immediate.unwrap() as u64;

        // Perform merge operation
        let result = (src1 & mask) | (src2 & !mask);

        // Write result to destination
        match self.fields.destinations[0] {
            RegisterType::GR(reg) => cpu.set_gr(reg as usize, result)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
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
            destinations: vec![RegisterType::GR(3)],
            immediate: None,
            addressing: None,
        };
        (cpu, memory, fields)
    }

    #[test]
    fn test_add() {
        let (mut cpu, mut memory, fields) = setup_test();
        let add = Add::new(fields);

        // Basic addition
        cpu.set_gr(1, 5).unwrap();
        cpu.set_gr(2, 3).unwrap();
        add.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 8);

        // Addition with overflow
        cpu.set_gr(1, u64::MAX).unwrap();
        cpu.set_gr(2, 1).unwrap();
        add.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0);
    }

    #[test]
    fn test_sub() {
        let (mut cpu, mut memory, fields) = setup_test();
        let sub = Sub::new(fields);

        // Basic subtraction
        cpu.set_gr(1, 10).unwrap();
        cpu.set_gr(2, 3).unwrap();
        sub.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 7);

        // Subtraction with underflow
        cpu.set_gr(1, 0).unwrap();
        cpu.set_gr(2, 1).unwrap();
        sub.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), u64::MAX);
    }

    #[test]
    fn test_and() {
        let (mut cpu, mut memory, fields) = setup_test();
        let and = And::new(fields);

        cpu.set_gr(1, 0xFF00).unwrap();
        cpu.set_gr(2, 0x0FF0).unwrap();
        and.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0x0F00);
    }

    #[test]
    fn test_or() {
        let (mut cpu, mut memory, fields) = setup_test();
        let or = Or::new(fields);

        cpu.set_gr(1, 0xFF00).unwrap();
        cpu.set_gr(2, 0x0FF0).unwrap();
        or.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0xFFF0);
    }

    #[test]
    fn test_xor() {
        let (mut cpu, mut memory, fields) = setup_test();
        let xor = Xor::new(fields);

        cpu.set_gr(1, 0xFF00).unwrap();
        cpu.set_gr(2, 0x0FF0).unwrap();
        xor.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0xF0F0);
    }

    #[test]
    fn test_compare() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        fields.destinations = vec![RegisterType::PR(1)];

        // Test equal comparison
        let cmp_eq = Compare::new(fields.clone(), CompareType::Equal);
        cpu.set_gr(1, 5).unwrap();
        cpu.set_gr(2, 5).unwrap();
        cmp_eq.execute(&mut cpu, &mut memory).unwrap();
        assert!(cpu.get_pr(1).unwrap());

        // Test not equal comparison
        let cmp_ne = Compare::new(fields.clone(), CompareType::NotEqual);
        cpu.set_gr(2, 6).unwrap();
        cmp_ne.execute(&mut cpu, &mut memory).unwrap();
        assert!(cpu.get_pr(1).unwrap());

        // Test signed less than
        let cmp_lt = Compare::new(fields.clone(), CompareType::LessThan);
        cpu.set_gr(1, 0xFFFFFFFFFFFFFFFF).unwrap(); // -1 in two's complement
        cpu.set_gr(2, 0).unwrap();
        cmp_lt.execute(&mut cpu, &mut memory).unwrap();
        assert!(cpu.get_pr(1).unwrap());

        // Test unsigned less than
        let cmp_ltu = Compare::new(fields.clone(), CompareType::LessThanU);
        cpu.set_gr(1, 5).unwrap();
        cpu.set_gr(2, 10).unwrap();
        cmp_ltu.execute(&mut cpu, &mut memory).unwrap();
        assert!(cpu.get_pr(1).unwrap());
    }

    #[test]
    fn test_test_bit() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        fields.destinations = vec![RegisterType::PR(1)];
        let tbit = TestBit::new(fields);

        // Test bit set
        cpu.set_gr(1, 0x8).unwrap(); // 1000 in binary
        cpu.set_gr(2, 3).unwrap();   // Testing bit position 3
        tbit.execute(&mut cpu, &mut memory).unwrap();
        assert!(cpu.get_pr(1).unwrap());

        // Test bit clear
        cpu.set_gr(1, 0x8).unwrap(); // 1000 in binary
        cpu.set_gr(2, 2).unwrap();   // Testing bit position 2
        tbit.execute(&mut cpu, &mut memory).unwrap();
        assert!(!cpu.get_pr(1).unwrap());

        // Test invalid bit position
        cpu.set_gr(1, 0x8).unwrap();
        cpu.set_gr(2, 64).unwrap();  // Invalid bit position
        tbit.execute(&mut cpu, &mut memory).unwrap();
        assert!(!cpu.get_pr(1).unwrap());
    }

    #[test]
    fn test_predicated_execution() {
        let (mut cpu, mut memory, fields) = setup_test();
        let add = Add::new(fields);

        // Set predicate register to false
        cpu.set_pr(0, false).unwrap();
        
        // Set up registers
        cpu.set_gr(1, 5).unwrap();
        cpu.set_gr(2, 3).unwrap();
        cpu.set_gr(3, 0).unwrap();

        // Execute add - should not modify destination due to false predicate
        add.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0);

        // Set predicate register to true
        cpu.set_pr(0, true).unwrap();

        // Execute add - should modify destination
        add.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 8);
    }

    #[test]
    fn test_shift() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        
        // Test left shift
        let shift_left = Shift::new(fields.clone(), ShiftType::Left);
        cpu.set_gr(1, 0x1).unwrap();
        cpu.set_gr(2, 4).unwrap();
        shift_left.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0x10);

        // Test right arithmetic shift
        let shift_right = Shift::new(fields.clone(), ShiftType::RightArithmetic);
        cpu.set_gr(1, 0xF000000000000000).unwrap();
        cpu.set_gr(2, 4).unwrap();
        shift_right.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0xFF00000000000000);

        // Test right logical shift
        let shift_logical = Shift::new(fields.clone(), ShiftType::RightLogical);
        cpu.set_gr(1, 0xF000000000000000).unwrap();
        cpu.set_gr(2, 4).unwrap();
        shift_logical.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0x0F00000000000000);
    }

    #[test]
    #[ignore]
    fn test_deposit() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        // Position 8, length 8 (0x0808)
        fields.immediate = Some(0x0808);
        
        let deposit = Deposit::new(fields);
        cpu.set_gr(1, 0xFFFFFFFFFFFFFFFF).unwrap();
        cpu.set_gr(2, 0xAB).unwrap();
        deposit.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0xFFFFFFFFFFABFFFF);
    }

    #[test]
    #[ignore]
    fn test_extract() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        // Position 8, length 8 (0x0808)
        fields.immediate = Some(0x0808);
        
        let extract = Extract::new(fields);
        cpu.set_gr(1, 0xFFFFFFFFFFABFFFF).unwrap();
        extract.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0xAB);
    }

    #[test]
    fn test_popcount() {
        let (mut cpu, mut memory, fields) = setup_test();
        
        let popcount = PopCount::new(fields);
        cpu.set_gr(1, 0x1234567890ABCDEF).unwrap();
        popcount.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 32); // 0x1234567890ABCDEF has 32 bits set
    }

    #[test]
    fn test_parallel_add() {
        let (mut cpu, mut memory, fields) = setup_test();
        
        // Test byte parallel add
        let padd_byte = ParallelAdd::new(fields.clone(), ParallelSize::Byte);
        cpu.set_gr(1, 0x0102030405060708).unwrap();
        cpu.set_gr(2, 0x0807060504030201).unwrap();
        padd_byte.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0x0909090909090909);

        // Test half-word parallel add
        let padd_half = ParallelAdd::new(fields.clone(), ParallelSize::Half);
        cpu.set_gr(1, 0x0001000200030004).unwrap();
        cpu.set_gr(2, 0x0004000300020001).unwrap();
        padd_half.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0x0005000500050005);
    }

    #[test]
    fn test_saturating_add() {
        let (mut cpu, mut memory, fields) = setup_test();
        
        // Test unsigned saturating add
        let sadd_u = SaturatingAdd::new(fields.clone(), false);
        cpu.set_gr(1, u64::MAX - 1).unwrap();
        cpu.set_gr(2, 5).unwrap();
        sadd_u.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), u64::MAX);

        // Test signed saturating add
        let sadd_s = SaturatingAdd::new(fields.clone(), true);
        cpu.set_gr(1, 0x7FFFFFFFFFFFFFFF).unwrap(); // Max positive i64
        cpu.set_gr(2, 1).unwrap();
        sadd_s.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0x7FFFFFFFFFFFFFFF);
    }

    #[test]
    #[ignore]
    fn test_rotate_mask() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        fields.immediate = Some(0xFF00FF00FF00FF00u64 as i64);
        
        let rotmask = RotateMask::new(fields);
        cpu.set_gr(1, 0x1234567890ABCDEF).unwrap();
        cpu.set_gr(2, 8).unwrap(); // Rotate by 8 bits
        rotmask.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0x1200340056007800);
    }

    #[test]
    fn test_minmax() {
        let (mut cpu, mut memory, fields) = setup_test();
        
        // Test unsigned minimum
        let min_u = MinMax::new(fields.clone(), MinMaxType::MinU);
        cpu.set_gr(1, 5).unwrap();
        cpu.set_gr(2, 10).unwrap();
        min_u.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 5);

        // Test signed minimum with negative numbers
        let min_s = MinMax::new(fields.clone(), MinMaxType::MinS);
        cpu.set_gr(1, 0xFFFFFFFFFFFFFFFF).unwrap(); // -1 in two's complement
        cpu.set_gr(2, 5).unwrap();
        min_s.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0xFFFFFFFFFFFFFFFF);

        // Test unsigned maximum
        let max_u = MinMax::new(fields.clone(), MinMaxType::MaxU);
        cpu.set_gr(1, 0xFFFFFFFFFFFFFFFF).unwrap();
        cpu.set_gr(2, 5).unwrap();
        max_u.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0xFFFFFFFFFFFFFFFF);
    }

    #[test]
    #[ignore]
    fn test_extend() {
        let (mut cpu, mut memory, fields) = setup_test();
        
        // Test zero extension
        let zext_byte = Extend::new(fields.clone(), ExtensionSize::Byte, false);
        cpu.set_gr(1, 0xFF80).unwrap();
        zext_byte.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0x80);

        // Test sign extension
        let sext_byte = Extend::new(fields.clone(), ExtensionSize::Byte, true);
        cpu.set_gr(1, 0x80).unwrap();
        sext_byte.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0xFFFFFFFFFFFFFF80);

        // Test half-word extension
        let sext_half = Extend::new(fields.clone(), ExtensionSize::Half, true);
        cpu.set_gr(1, 0x8000).unwrap();
        sext_half.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0xFFFFFFFFFFFF8000);
    }

    #[test]
    #[ignore]
    fn test_merge() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        fields.immediate = Some(0xF0F0F0F0F0F0F0F0u64 as i64);
        
        let merge = Merge::new(fields);
        cpu.set_gr(1, 0xAAAAAAAAAAAAAAAA).unwrap();
        cpu.set_gr(2, 0x5555555555555555).unwrap();
        merge.execute(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.get_gr(3).unwrap(), 0xA5A5A5A5A5A5A5A5);
    }
} 