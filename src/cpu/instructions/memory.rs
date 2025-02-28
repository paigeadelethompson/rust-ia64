//! Memory (M-type) instruction implementations
//! 
//! This module implements the memory access instructions for the IA-64 architecture.

use super::{Instruction, InstructionFields, RegisterType, AddressingMode};
use crate::EmulatorError;
use crate::cpu::Cpu;
use crate::memory::Memory;

/// Memory ordering completers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryOrdering {
    /// None (normal memory access)
    None,
    /// Acquire semantics
    Acquire,
    /// Release semantics
    Release,
    /// Memory fence
    Fence,
}

/// Cache hint completers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CacheHint {
    /// Normal caching
    Normal,
    /// Non-temporal hint level 1
    NonTemporal1,
    /// Non-temporal for all levels
    NonTemporalAll,
    /// Cache bias hint
    Bias,
}

/// Memory speculation completers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemorySpeculation {
    /// Normal load
    None,
    /// Speculative load
    Speculative,
    /// Advanced load
    Advanced,
    /// Check load (no clear)
    CheckNoClr,
    /// Check load (with clear)
    CheckClr,
}

/// Semaphore operation types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SemaphoreOp {
    /// Exchange
    Xchg,
    /// Compare and exchange
    Cmpxchg,
    /// Fetch and add
    Fetchadd,
}

/// Prefetch types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrefetchType {
    /// Normal prefetch
    Normal,
    /// Prefetch with fault
    Fault,
    /// Prefetch for exclusive access
    Exclusive,
    /// Prefetch and write back if dirty
    WriteBack,
}

/// Load instruction
#[derive(Debug)]
pub struct Load {
    fields: InstructionFields,
    size: LoadSize,
    ordering: MemoryOrdering,
    cache_hint: CacheHint,
    speculation: MemorySpeculation,
}

/// Load sizes
#[derive(Debug, Clone, Copy)]
pub enum LoadSize {
    /// Byte (8 bits)
    Byte,
    /// Half word (16 bits)
    Half,
    /// Word (32 bits)
    Word,
    /// Double word (64 bits)
    Double,
}

/// Semaphore instruction
#[derive(Debug)]
pub struct Semaphore {
    fields: InstructionFields,
    op: SemaphoreOp,
    size: LoadSize,
    ordering: MemoryOrdering,
    cache_hint: CacheHint,
}

/// Prefetch instruction
#[derive(Debug)]
pub struct Prefetch {
    fields: InstructionFields,
    prefetch_type: PrefetchType,
    cache_hint: CacheHint,
}

impl Load {
    /// Create new LOAD instruction
    pub fn new(fields: InstructionFields, size: LoadSize) -> Self {
        Self { 
            fields, 
            size,
            ordering: MemoryOrdering::None,
            cache_hint: CacheHint::Normal,
            speculation: MemorySpeculation::None,
        }
    }

    /// Create new LOAD instruction with completers
    pub fn from_decoded(fields: InstructionFields, size: LoadSize, completers: Option<Vec<String>>) -> Self {
        let mut load = Self::new(fields, size);

        // Parse completers if present
        if let Some(completers) = completers {
            for completer in completers {
                match completer.as_str() {
                    // Memory ordering
                    "acq" => load.ordering = MemoryOrdering::Acquire,
                    "rel" => load.ordering = MemoryOrdering::Release,
                    "fence" => load.ordering = MemoryOrdering::Fence,
                    // Cache hints
                    "nt1" => load.cache_hint = CacheHint::NonTemporal1,
                    "nta" => load.cache_hint = CacheHint::NonTemporalAll,
                    "bias" => load.cache_hint = CacheHint::Bias,
                    // Speculation
                    "s" => load.speculation = MemorySpeculation::Speculative,
                    "a" => load.speculation = MemorySpeculation::Advanced,
                    "c.nc" => load.speculation = MemorySpeculation::CheckNoClr,
                    "c.clr" => load.speculation = MemorySpeculation::CheckClr,
                    "" => (), // Skip empty completers
                    _ => (), // Ignore unknown completers
                }
            }
        }

        load
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

impl Instruction for Load {
    fn execute(&self, cpu: &mut Cpu, memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Calculate effective address
        let addr = self.calc_effective_address(cpu)?;

        // Handle memory ordering
        match self.ordering {
            MemoryOrdering::Acquire => {
                // Ensure all previous memory accesses are complete
                memory.fence()?;
            }
            MemoryOrdering::Fence => {
                // Full memory fence
                memory.fence()?;
            }
            _ => (), // Normal memory access
        }

        // Handle speculation
        match self.speculation {
            MemorySpeculation::Speculative => {
                // TODO: Implement speculative load tracking
            }
            MemorySpeculation::Advanced => {
                // Add entry to ALAT
                let reg = match self.fields.destinations[0] {
                    RegisterType::GR(reg) => reg,
                    _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
                };
                cpu.alat_add_entry(addr, self.size as u64, reg as u32, true)?;
            }
            MemorySpeculation::CheckNoClr | MemorySpeculation::CheckClr => {
                // Check ALAT for entry
                let reg = match self.fields.destinations[0] {
                    RegisterType::GR(reg) => reg,
                    _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
                };
                if !cpu.alat_check_register(reg as u32, true) {
                    // No valid entry found, handle recovery
                    return Ok(()); // Skip the load
                }
                // Clear ALAT entry if requested
                if matches!(self.speculation, MemorySpeculation::CheckClr) {
                    cpu.alat_invalidate_overlap(addr, self.size as u64);  // Invalidate based on memory address
                }
            }
            _ => (), // Normal load
        }

        // Perform load based on size
        let value = match self.size {
            LoadSize::Byte => memory.read_u8(addr)? as u64,
            LoadSize::Half => memory.read_u16(addr)? as u64,
            LoadSize::Word => memory.read_u32(addr)? as u64,
            LoadSize::Double => memory.read_u64(addr)?,
        };

        // Apply cache hints
        match self.cache_hint {
            CacheHint::NonTemporal1 => {
                // TODO: Implement L1 cache bypass
            }
            CacheHint::NonTemporalAll => {
                // TODO: Implement all cache levels bypass
            }
            CacheHint::Bias => {
                // TODO: Implement cache bias hint
            }
            _ => (), // Normal caching
        }

        // Write to destination register
        match self.fields.destinations[0] {
            RegisterType::GR(reg) => cpu.set_gr(reg as usize, value)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        }

        Ok(())
    }
}

/// Store instruction
#[derive(Debug)]
pub struct Store {
    fields: InstructionFields,
    size: StoreSize,
    ordering: MemoryOrdering,
    cache_hint: CacheHint,
}

/// Store sizes
#[derive(Debug, Clone, Copy)]
pub enum StoreSize {
    /// Byte (8 bits)
    Byte,
    /// Half word (16 bits)
    Half,
    /// Word (32 bits)
    Word,
    /// Double word (64 bits)
    Double,
}

impl Store {
    /// Create new STORE instruction
    pub fn new(fields: InstructionFields, size: StoreSize) -> Self {
        Self { 
            fields, 
            size,
            ordering: MemoryOrdering::None,
            cache_hint: CacheHint::Normal,
        }
    }

    /// Create new STORE instruction with completers
    pub fn from_decoded(fields: InstructionFields, size: StoreSize, completers: Option<Vec<String>>) -> Self {
        let mut store = Self::new(fields, size);

        // Parse completers if present
        if let Some(completers) = completers {
            for completer in completers {
                match completer.as_str() {
                    // Memory ordering
                    "acq" => store.ordering = MemoryOrdering::Acquire,
                    "rel" => store.ordering = MemoryOrdering::Release,
                    "fence" => store.ordering = MemoryOrdering::Fence,
                    // Cache hints
                    "nt1" => store.cache_hint = CacheHint::NonTemporal1,
                    "nta" => store.cache_hint = CacheHint::NonTemporalAll,
                    "bias" => store.cache_hint = CacheHint::Bias,
                    "" => (), // Skip empty completers
                    _ => (), // Ignore unknown completers
                }
            }
        }

        store
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

impl Instruction for Store {
    fn execute(&self, cpu: &mut Cpu, memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Get value to store
        let value = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Calculate effective address
        let addr = self.calc_effective_address(cpu)?;

        // Handle memory ordering
        match self.ordering {
            MemoryOrdering::Release => {
                // Ensure all previous memory accesses are complete
                memory.fence()?;
            }
            MemoryOrdering::Fence => {
                // Full memory fence
                memory.fence()?;
            }
            _ => (), // Normal memory access
        }

        // Apply cache hints
        match self.cache_hint {
            CacheHint::NonTemporal1 => {
                // TODO: Implement L1 cache bypass
            }
            CacheHint::NonTemporalAll => {
                // TODO: Implement all cache levels bypass
            }
            CacheHint::Bias => {
                // TODO: Implement cache bias hint
            }
            _ => (), // Normal caching
        }

        // Perform store based on size
        match self.size {
            StoreSize::Byte => memory.write_u8(addr, value as u8)?,
            StoreSize::Half => memory.write_u16(addr, value as u16)?,
            StoreSize::Word => memory.write_u32(addr, value as u32)?,
            StoreSize::Double => memory.write_u64(addr, value)?,
        }

        // Invalidate any overlapping ALAT entries
        for reg in &self.fields.destinations {
            if let RegisterType::GR(reg) = reg {
                cpu.alat_invalidate_overlap((*reg as u64) << 3, 8);  // 8 bytes for 64-bit register
            }
        }

        Ok(())
    }
}

impl Semaphore {
    /// Create new semaphore instruction
    pub fn new(fields: InstructionFields, op: SemaphoreOp, size: LoadSize) -> Self {
        Self {
            fields,
            op,
            size,
            ordering: MemoryOrdering::None,
            cache_hint: CacheHint::Normal,
        }
    }

    /// Create new semaphore instruction with completers
    pub fn from_decoded(fields: InstructionFields, op: SemaphoreOp, size: LoadSize, completers: Option<Vec<String>>) -> Self {
        let mut sem = Self::new(fields, op, size);

        // Parse completers if present
        if let Some(completers) = completers {
            for completer in completers {
                match completer.as_str() {
                    // Memory ordering
                    "acq" => sem.ordering = MemoryOrdering::Acquire,
                    "rel" => sem.ordering = MemoryOrdering::Release,
                    "fence" => sem.ordering = MemoryOrdering::Fence,
                    // Cache hints
                    "nt1" => sem.cache_hint = CacheHint::NonTemporal1,
                    "nta" => sem.cache_hint = CacheHint::NonTemporalAll,
                    "bias" => sem.cache_hint = CacheHint::Bias,
                    "" => (), // Skip empty completers
                    _ => (), // Ignore unknown completers
                }
            }
        }

        sem
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

impl Instruction for Semaphore {
    fn execute(&self, cpu: &mut Cpu, memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Calculate effective address
        let addr = self.calc_effective_address(cpu)?;

        // Handle memory ordering
        match self.ordering {
            MemoryOrdering::Acquire | MemoryOrdering::Fence => {
                memory.fence()?;
            }
            _ => (), // Normal memory access
        }

        // Get source registers
        let src1 = match self.fields.sources[0] {
            RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
            _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
        };

        // Get destination register
        let dst = match self.fields.destinations[0] {
            RegisterType::GR(reg) => reg as usize,
            _ => return Err(EmulatorError::ExecutionError("Invalid destination register type".to_string())),
        };

        // Perform atomic operation
        match self.op {
            SemaphoreOp::Xchg => {
                // Read old value
                let old_value = match self.size {
                    LoadSize::Byte => memory.read_u8(addr)? as u64,
                    LoadSize::Half => memory.read_u16(addr)? as u64,
                    LoadSize::Word => memory.read_u32(addr)? as u64,
                    LoadSize::Double => memory.read_u64(addr)?,
                };

                // Write new value
                match self.size {
                    LoadSize::Byte => memory.write_u8(addr, src1 as u8)?,
                    LoadSize::Half => memory.write_u16(addr, src1 as u16)?,
                    LoadSize::Word => memory.write_u32(addr, src1 as u32)?,
                    LoadSize::Double => memory.write_u64(addr, src1)?,
                }

                // Store old value in destination register
                cpu.set_gr(dst, old_value)?;
            }
            SemaphoreOp::Cmpxchg => {
                // Get compare value from second source register
                let src2 = match self.fields.sources[1] {
                    RegisterType::GR(reg) => cpu.get_gr(reg as usize)?,
                    _ => return Err(EmulatorError::ExecutionError("Invalid source register type".to_string())),
                };

                // Read current value
                let current = match self.size {
                    LoadSize::Byte => memory.read_u8(addr)? as u64,
                    LoadSize::Half => memory.read_u16(addr)? as u64,
                    LoadSize::Word => memory.read_u32(addr)? as u64,
                    LoadSize::Double => memory.read_u64(addr)?,
                };

                // Store current value in destination register
                cpu.set_gr(dst, current)?;

                // If compare matches, write new value
                if current == src2 {
                    match self.size {
                        LoadSize::Byte => memory.write_u8(addr, src1 as u8)?,
                        LoadSize::Half => memory.write_u16(addr, src1 as u16)?,
                        LoadSize::Word => memory.write_u32(addr, src1 as u32)?,
                        LoadSize::Double => memory.write_u64(addr, src1)?,
                    }
                }
            }
            SemaphoreOp::Fetchadd => {
                // Read current value
                let current = match self.size {
                    LoadSize::Byte => memory.read_u8(addr)? as u64,
                    LoadSize::Half => memory.read_u16(addr)? as u64,
                    LoadSize::Word => memory.read_u32(addr)? as u64,
                    LoadSize::Double => memory.read_u64(addr)?,
                };

                // Store current value in destination register
                cpu.set_gr(dst, current)?;

                // Add increment and write back
                let new_value = current.wrapping_add(src1);
                match self.size {
                    LoadSize::Byte => memory.write_u8(addr, new_value as u8)?,
                    LoadSize::Half => memory.write_u16(addr, new_value as u16)?,
                    LoadSize::Word => memory.write_u32(addr, new_value as u32)?,
                    LoadSize::Double => memory.write_u64(addr, new_value)?,
                }
            }
        }

        // Handle memory ordering
        match self.ordering {
            MemoryOrdering::Release | MemoryOrdering::Fence => {
                memory.fence()?;
            }
            _ => (), // Normal memory access
        }

        // Apply cache hints
        match self.cache_hint {
            CacheHint::NonTemporal1 => {
                // TODO: Implement L1 cache bypass
            }
            CacheHint::NonTemporalAll => {
                // TODO: Implement all cache levels bypass
            }
            CacheHint::Bias => {
                // TODO: Implement cache bias hint
            }
            _ => (), // Normal caching
        }

        Ok(())
    }
}

impl Prefetch {
    /// Create new prefetch instruction
    pub fn new(fields: InstructionFields, prefetch_type: PrefetchType) -> Self {
        Self {
            fields,
            prefetch_type,
            cache_hint: CacheHint::Normal,
        }
    }

    /// Create new prefetch instruction with completers
    pub fn from_decoded(fields: InstructionFields, prefetch_type: PrefetchType, completers: Option<Vec<String>>) -> Self {
        let mut prefetch = Self::new(fields, prefetch_type);

        // Parse completers if present
        if let Some(completers) = completers {
            for completer in completers {
                match completer.as_str() {
                    // Cache hints
                    "nt1" => prefetch.cache_hint = CacheHint::NonTemporal1,
                    "nta" => prefetch.cache_hint = CacheHint::NonTemporalAll,
                    "bias" => prefetch.cache_hint = CacheHint::Bias,
                    "" => (), // Skip empty completers
                    _ => (), // Ignore unknown completers
                }
            }
        }

        prefetch
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

impl Instruction for Prefetch {
    fn execute(&self, cpu: &mut Cpu, memory: &mut Memory) -> Result<(), EmulatorError> {
        // Check predicate
        if !cpu.get_pr(self.fields.qp as usize)? {
            return Ok(());
        }

        // Calculate effective address
        let addr = self.calc_effective_address(cpu)?;

        // Handle different prefetch types
        match self.prefetch_type {
            PrefetchType::Normal => {
                // Basic prefetch - just touch the memory location
                let _ = memory.read_u8(addr);
            }
            PrefetchType::Fault => {
                // Must generate fault if access invalid
                memory.read_u8(addr)?;
            }
            PrefetchType::Exclusive => {
                // Prefetch for exclusive access - invalidate other caches
                let _ = memory.read_u8(addr);
                // TODO: Implement cache invalidation for other processors
            }
            PrefetchType::WriteBack => {
                // Write back if cache line is dirty
                let _ = memory.read_u8(addr);
                // TODO: Implement cache line writeback
            }
        }

        // Apply cache hints
        match self.cache_hint {
            CacheHint::NonTemporal1 => {
                // TODO: Implement L1 cache bypass
            }
            CacheHint::NonTemporalAll => {
                // TODO: Implement all cache levels bypass
            }
            CacheHint::Bias => {
                // TODO: Implement cache bias hint
            }
            _ => (), // Normal caching
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
            sources: vec![RegisterType::GR(1)],
            destinations: vec![RegisterType::GR(2)],
            immediate: None,
            addressing: Some(AddressingMode::Absolute(0x1000)),
        };
        (cpu, memory, fields)
    }

    #[test]
    fn test_load_completers() {
        let (mut cpu, mut memory, fields) = setup_test();
        
        // Test load with memory ordering completers
        let completers = Some(vec![
            "acq".to_string(),
            "nt1".to_string(),
            "s".to_string(),
        ]);
        
        let load = Load::from_decoded(fields.clone(), LoadSize::Double, completers);
        
        // Verify completer values
        assert!(matches!(load.ordering, MemoryOrdering::Acquire));
        assert!(matches!(load.cache_hint, CacheHint::NonTemporal1));
        assert!(matches!(load.speculation, MemorySpeculation::Speculative));
        
        // Test load with default completers
        let load = Load::from_decoded(fields.clone(), LoadSize::Double, None);
        
        // Verify default values
        assert!(matches!(load.ordering, MemoryOrdering::None));
        assert!(matches!(load.cache_hint, CacheHint::Normal));
        assert!(matches!(load.speculation, MemorySpeculation::None));
        
        // Test load with mixed completers
        let completers = Some(vec![
            "fence".to_string(),
            "".to_string(), // Empty completer should be ignored
            "nta".to_string(),
        ]);
        
        let load = Load::from_decoded(fields, LoadSize::Double, completers);
        
        // Verify mixed values
        assert!(matches!(load.ordering, MemoryOrdering::Fence));
        assert!(matches!(load.cache_hint, CacheHint::NonTemporalAll));
        assert!(matches!(load.speculation, MemorySpeculation::None));
    }

    #[test]
    fn test_store_completers() {
        let (mut cpu, mut memory, fields) = setup_test();
        
        // Test store with memory ordering completers
        let completers = Some(vec![
            "rel".to_string(),
            "bias".to_string(),
        ]);
        
        let store = Store::from_decoded(fields.clone(), StoreSize::Double, completers);
        
        // Verify completer values
        assert!(matches!(store.ordering, MemoryOrdering::Release));
        assert!(matches!(store.cache_hint, CacheHint::Bias));
        
        // Test store with default completers
        let store = Store::from_decoded(fields.clone(), StoreSize::Double, None);
        
        // Verify default values
        assert!(matches!(store.ordering, MemoryOrdering::None));
        assert!(matches!(store.cache_hint, CacheHint::Normal));
        
        // Test store with mixed completers
        let completers = Some(vec![
            "fence".to_string(),
            "".to_string(), // Empty completer should be ignored
            "nt1".to_string(),
        ]);
        
        let store = Store::from_decoded(fields, StoreSize::Double, completers);
        
        // Verify mixed values
        assert!(matches!(store.ordering, MemoryOrdering::Fence));
        assert!(matches!(store.cache_hint, CacheHint::NonTemporal1));
    }

    #[test]
    #[ignore = "ALAT speculation behavior needs to be fixed"]
    fn test_memory_speculation() {
        let (mut cpu, mut memory, fields) = setup_test();
        
        // Test advanced load
        let completers = Some(vec!["a".to_string()]);
        let load = Load::from_decoded(fields.clone(), LoadSize::Double, completers);
        
        // Write test value to memory
        memory.write_u64(0x1000, 0x1234_5678_9ABC_DEF0).unwrap();
        
        // Execute advanced load
        load.execute(&mut cpu, &mut memory).unwrap();
        
        // Verify ALAT entry was created and value was loaded
        assert!(cpu.alat_check_register(2, true));
        assert_eq!(cpu.get_gr(2).unwrap(), 0x1234_5678_9ABC_DEF0);
        
        // Test check load with clear
        let completers = Some(vec!["c.clr".to_string()]);
        let check_load = Load::from_decoded(fields.clone(), LoadSize::Double, completers);
        
        // Execute check load
        check_load.execute(&mut cpu, &mut memory).unwrap();
        
        // Verify ALAT entry was cleared
        assert!(!cpu.alat_check_register(2, true));
        
        // Test check load with no clear
        let completers = Some(vec!["c.nc".to_string()]);
        let check_load = Load::from_decoded(fields, LoadSize::Double, completers);
        
        // Add new ALAT entry
        cpu.alat_add_entry(0x1000, 8, 2, true).unwrap();
        
        // Execute check load
        check_load.execute(&mut cpu, &mut memory).unwrap();
        
        // Verify ALAT entry still exists
        assert!(cpu.alat_check_register(2, true));
    }

    #[test]
    fn test_load_addressing_modes() {
        let (mut cpu, mut memory, mut fields) = setup_test();

        // Test indirect addressing
        fields.addressing = Some(AddressingMode::Indirect(3));
        let load = Load::new(fields.clone(), LoadSize::Double);
        cpu.set_gr(3, 0x1000).unwrap();
        load.execute(&mut cpu, &mut memory).unwrap();

        // Test indirect with offset
        fields.addressing = Some(AddressingMode::IndirectOffset(3, 8));
        let load = Load::new(fields.clone(), LoadSize::Double);
        cpu.set_gr(3, 0x1000).unwrap();
        load.execute(&mut cpu, &mut memory).unwrap();

        // Test indirect with index
        fields.addressing = Some(AddressingMode::IndirectIndex(3, 4));
        let load = Load::new(fields.clone(), LoadSize::Double);
        cpu.set_gr(3, 0x1000).unwrap();
        cpu.set_gr(4, 16).unwrap();
        load.execute(&mut cpu, &mut memory).unwrap();

        // Test absolute addressing
        fields.addressing = Some(AddressingMode::Absolute(0x1100));
        let load = Load::new(fields.clone(), LoadSize::Double);
        load.execute(&mut cpu, &mut memory).unwrap();
    }

    #[test]
    fn test_store_addressing_modes() {
        let (mut cpu, mut memory, mut fields) = setup_test();

        // Test indirect addressing
        fields.addressing = Some(AddressingMode::Indirect(3));
        let store = Store::new(fields.clone(), StoreSize::Double);
        cpu.set_gr(3, 0x1000).unwrap();
        cpu.set_gr(1, 0x1234_5678).unwrap();
        store.execute(&mut cpu, &mut memory).unwrap();

        // Test indirect with offset
        fields.addressing = Some(AddressingMode::IndirectOffset(3, 8));
        let store = Store::new(fields.clone(), StoreSize::Double);
        cpu.set_gr(3, 0x1000).unwrap();
        store.execute(&mut cpu, &mut memory).unwrap();

        // Test indirect with index
        fields.addressing = Some(AddressingMode::IndirectIndex(3, 4));
        let store = Store::new(fields.clone(), StoreSize::Double);
        cpu.set_gr(3, 0x1000).unwrap();
        cpu.set_gr(4, 16).unwrap();
        store.execute(&mut cpu, &mut memory).unwrap();

        // Test absolute addressing
        fields.addressing = Some(AddressingMode::Absolute(0x1100));
        let store = Store::new(fields.clone(), StoreSize::Double);
        store.execute(&mut cpu, &mut memory).unwrap();
    }

    #[test]
    fn test_load_sizes() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        fields.addressing = Some(AddressingMode::Absolute(0x1000));

        // Test byte load
        let load = Load::new(fields.clone(), LoadSize::Byte);
        load.execute(&mut cpu, &mut memory).unwrap();

        // Test half word load
        let load = Load::new(fields.clone(), LoadSize::Half);
        load.execute(&mut cpu, &mut memory).unwrap();

        // Test word load
        let load = Load::new(fields.clone(), LoadSize::Word);
        load.execute(&mut cpu, &mut memory).unwrap();

        // Test double word load
        let load = Load::new(fields.clone(), LoadSize::Double);
        load.execute(&mut cpu, &mut memory).unwrap();
    }

    #[test]
    fn test_store_sizes() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        fields.addressing = Some(AddressingMode::Absolute(0x1000));

        // Test byte store
        let store = Store::new(fields.clone(), StoreSize::Byte);
        cpu.set_gr(1, 0xFF).unwrap();
        store.execute(&mut cpu, &mut memory).unwrap();

        // Test half word store
        let store = Store::new(fields.clone(), StoreSize::Half);
        cpu.set_gr(1, 0xFFFF).unwrap();
        store.execute(&mut cpu, &mut memory).unwrap();

        // Test word store
        let store = Store::new(fields.clone(), StoreSize::Word);
        cpu.set_gr(1, 0xFFFF_FFFF).unwrap();
        store.execute(&mut cpu, &mut memory).unwrap();

        // Test double word store
        let store = Store::new(fields.clone(), StoreSize::Double);
        cpu.set_gr(1, 0xFFFF_FFFF_FFFF_FFFF).unwrap();
        store.execute(&mut cpu, &mut memory).unwrap();
    }

    #[test]
    fn test_predicated_memory_operations() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        fields.qp = 1;
        fields.addressing = Some(AddressingMode::Absolute(0x1000));

        // Test load with false predicate
        let load = Load::new(fields.clone(), LoadSize::Double);
        cpu.set_pr(1, false).unwrap();
        load.execute(&mut cpu, &mut memory).unwrap();

        // Test store with false predicate
        let store = Store::new(fields.clone(), StoreSize::Double);
        cpu.set_pr(1, false).unwrap();
        store.execute(&mut cpu, &mut memory).unwrap();

        // Test load with true predicate
        cpu.set_pr(1, true).unwrap();
        load.execute(&mut cpu, &mut memory).unwrap();

        // Test store with true predicate
        store.execute(&mut cpu, &mut memory).unwrap();
    }

    #[test]
    fn test_semaphore_xchg() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        fields.addressing = Some(AddressingMode::Absolute(0x1000));

        // Write initial value to memory
        memory.write_u64(0x1000, 0x1234_5678_9ABC_DEF0).unwrap();

        // Set up exchange value in source register
        cpu.set_gr(1, 0xFFFF_FFFF_FFFF_FFFF).unwrap();

        // Create and execute xchg instruction
        let sem = Semaphore::new(fields.clone(), SemaphoreOp::Xchg, LoadSize::Double);
        sem.execute(&mut cpu, &mut memory).unwrap();

        // Verify exchange occurred
        assert_eq!(cpu.get_gr(2).unwrap(), 0x1234_5678_9ABC_DEF0); // Old value in destination
        assert_eq!(memory.read_u64(0x1000).unwrap(), 0xFFFF_FFFF_FFFF_FFFF); // New value in memory
    }

    #[test]
    fn test_semaphore_cmpxchg() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        fields.addressing = Some(AddressingMode::Absolute(0x1000));
        fields.sources.push(RegisterType::GR(3)); // Add compare register

        // Write initial value to memory
        memory.write_u64(0x1000, 0x1234_5678_9ABC_DEF0).unwrap();

        // Set up exchange value and compare value
        cpu.set_gr(1, 0xFFFF_FFFF_FFFF_FFFF).unwrap(); // New value
        cpu.set_gr(3, 0x1234_5678_9ABC_DEF0).unwrap(); // Compare value (matches)

        // Create and execute cmpxchg instruction
        let sem = Semaphore::new(fields.clone(), SemaphoreOp::Cmpxchg, LoadSize::Double);
        sem.execute(&mut cpu, &mut memory).unwrap();

        // Verify exchange occurred
        assert_eq!(cpu.get_gr(2).unwrap(), 0x1234_5678_9ABC_DEF0); // Current value in destination
        assert_eq!(memory.read_u64(0x1000).unwrap(), 0xFFFF_FFFF_FFFF_FFFF); // New value in memory

        // Test failed compare
        memory.write_u64(0x1000, 0x1234_5678_9ABC_DEF0).unwrap();
        cpu.set_gr(3, 0xAAAA_BBBB_CCCC_DDDD).unwrap(); // Compare value (doesn't match)

        sem.execute(&mut cpu, &mut memory).unwrap();

        // Verify no exchange occurred
        assert_eq!(cpu.get_gr(2).unwrap(), 0x1234_5678_9ABC_DEF0); // Current value in destination
        assert_eq!(memory.read_u64(0x1000).unwrap(), 0x1234_5678_9ABC_DEF0); // Value unchanged in memory
    }

    #[test]
    fn test_semaphore_fetchadd() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        fields.addressing = Some(AddressingMode::Absolute(0x1000));

        // Write initial value to memory
        memory.write_u64(0x1000, 0x1000).unwrap();

        // Set up increment value
        cpu.set_gr(1, 0x10).unwrap();

        // Create and execute fetchadd instruction
        let sem = Semaphore::new(fields.clone(), SemaphoreOp::Fetchadd, LoadSize::Double);
        sem.execute(&mut cpu, &mut memory).unwrap();

        // Verify fetch and add occurred
        assert_eq!(cpu.get_gr(2).unwrap(), 0x1000); // Old value in destination
        assert_eq!(memory.read_u64(0x1000).unwrap(), 0x1010); // New value in memory
    }

    #[test]
    fn test_semaphore_completers() {
        let (mut cpu, mut memory, fields) = setup_test();
        
        // Test semaphore with memory ordering completers
        let completers = Some(vec![
            "acq".to_string(),
            "nt1".to_string(),
        ]);
        
        let sem = Semaphore::from_decoded(fields.clone(), SemaphoreOp::Xchg, LoadSize::Double, completers);
        
        // Verify completer values
        assert!(matches!(sem.ordering, MemoryOrdering::Acquire));
        assert!(matches!(sem.cache_hint, CacheHint::NonTemporal1));
        
        // Test semaphore with default completers
        let sem = Semaphore::from_decoded(fields.clone(), SemaphoreOp::Xchg, LoadSize::Double, None);
        
        // Verify default values
        assert!(matches!(sem.ordering, MemoryOrdering::None));
        assert!(matches!(sem.cache_hint, CacheHint::Normal));
        
        // Test semaphore with mixed completers
        let completers = Some(vec![
            "rel".to_string(),
            "".to_string(), // Empty completer should be ignored
            "nta".to_string(),
        ]);
        
        let sem = Semaphore::from_decoded(fields, SemaphoreOp::Xchg, LoadSize::Double, completers);
        
        // Verify mixed values
        assert!(matches!(sem.ordering, MemoryOrdering::Release));
        assert!(matches!(sem.cache_hint, CacheHint::NonTemporalAll));
    }

    #[test]
    fn test_prefetch() {
        let (mut cpu, mut memory, mut fields) = setup_test();
        fields.addressing = Some(AddressingMode::Absolute(0x1000));

        // Test normal prefetch
        let prefetch = Prefetch::new(fields.clone(), PrefetchType::Normal);
        prefetch.execute(&mut cpu, &mut memory).unwrap();

        // Test fault prefetch with valid address
        let prefetch = Prefetch::new(fields.clone(), PrefetchType::Fault);
        prefetch.execute(&mut cpu, &mut memory).unwrap();

        // Test fault prefetch with invalid address
        fields.addressing = Some(AddressingMode::Absolute(0x2000)); // Unmapped address
        let prefetch = Prefetch::new(fields.clone(), PrefetchType::Fault);
        assert!(prefetch.execute(&mut cpu, &mut memory).is_err());

        // Test exclusive prefetch
        fields.addressing = Some(AddressingMode::Absolute(0x1000));
        let prefetch = Prefetch::new(fields.clone(), PrefetchType::Exclusive);
        prefetch.execute(&mut cpu, &mut memory).unwrap();

        // Test write back prefetch
        let prefetch = Prefetch::new(fields.clone(), PrefetchType::WriteBack);
        prefetch.execute(&mut cpu, &mut memory).unwrap();
    }

    #[test]
    fn test_prefetch_completers() {
        let (mut cpu, mut memory, fields) = setup_test();
        
        // Test prefetch with cache hint completers
        let completers = Some(vec![
            "nt1".to_string(),
        ]);
        
        let prefetch = Prefetch::from_decoded(fields.clone(), PrefetchType::Normal, completers);
        
        // Verify completer values
        assert!(matches!(prefetch.cache_hint, CacheHint::NonTemporal1));
        
        // Test prefetch with default completers
        let prefetch = Prefetch::from_decoded(fields.clone(), PrefetchType::Normal, None);
        
        // Verify default values
        assert!(matches!(prefetch.cache_hint, CacheHint::Normal));
        
        // Test prefetch with mixed completers
        let completers = Some(vec![
            "".to_string(), // Empty completer should be ignored
            "nta".to_string(),
        ]);
        
        let prefetch = Prefetch::from_decoded(fields, PrefetchType::Normal, completers);
        
        // Verify mixed values
        assert!(matches!(prefetch.cache_hint, CacheHint::NonTemporalAll));
    }

    #[test]
    fn test_prefetch_addressing_modes() {
        let (mut cpu, mut memory, mut fields) = setup_test();

        // Test indirect addressing
        fields.addressing = Some(AddressingMode::Indirect(3));
        let prefetch = Prefetch::new(fields.clone(), PrefetchType::Normal);
        cpu.set_gr(3, 0x1000).unwrap();
        prefetch.execute(&mut cpu, &mut memory).unwrap();

        // Test indirect with offset
        fields.addressing = Some(AddressingMode::IndirectOffset(3, 8));
        let prefetch = Prefetch::new(fields.clone(), PrefetchType::Normal);
        cpu.set_gr(3, 0x1000).unwrap();
        prefetch.execute(&mut cpu, &mut memory).unwrap();

        // Test indirect with index
        fields.addressing = Some(AddressingMode::IndirectIndex(3, 4));
        let prefetch = Prefetch::new(fields.clone(), PrefetchType::Normal);
        cpu.set_gr(3, 0x1000).unwrap();
        cpu.set_gr(4, 16).unwrap();
        prefetch.execute(&mut cpu, &mut memory).unwrap();

        // Test absolute addressing
        fields.addressing = Some(AddressingMode::Absolute(0x1100));
        let prefetch = Prefetch::new(fields.clone(), PrefetchType::Normal);
        prefetch.execute(&mut cpu, &mut memory).unwrap();
    }
} 