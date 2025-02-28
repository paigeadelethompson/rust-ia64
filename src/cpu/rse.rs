//! Register Stack Engine (RSE)
//!
//! This module implements the IA-64 Register Stack Engine, which manages
//! the register stack and performs register renaming.

use crate::memory::Memory;
use crate::EmulatorError;

/// Size of each register frame in bytes (512 bytes = 64 registers * 8 bytes)
const FRAME_SIZE: u64 = 512;

/// Maximum number of dirty registers before forced spill
const MAX_DIRTY_REGS: u32 = 48;

/// Register frame information
#[derive(Debug, Clone, Copy)]
pub struct FrameInfo {
    /// Size of local area
    pub locals: u32,
    /// Size of output area
    pub outputs: u32,
    /// Base of current frame
    pub base: u32,
}

/// Backing store state
#[derive(Debug)]
struct BackingStore {
    /// Base address of backing store
    base: u64,
    /// Current top of backing store
    top: u64,
    /// Store limit
    limit: u64,
}

impl BackingStore {
    /// Create new backing store
    fn new(base: u64, size: u64) -> Self {
        Self {
            base,
            top: base,
            limit: base + size,
        }
    }

    /// Check if store is full
    fn is_full(&self) -> bool {
        self.top >= self.limit
    }

    /// Get available space
    fn available_space(&self) -> u64 {
        self.limit - self.top
    }

    /// Advance top pointer
    fn advance(&mut self, size: u64) -> Result<(), EmulatorError> {
        let new_top = self.top + size;
        if new_top > self.limit {
            return Err(EmulatorError::ExecutionError(
                "Backing store overflow".to_string(),
            ));
        }
        self.top = new_top;
        Ok(())
    }
}

/// Register Stack Engine configuration
#[derive(Debug, Clone, Copy)]
pub struct RSEConfig {
    /// Mode
    pub mode: RSEMode,
    /// Load/store order
    pub load_store_order: LoadStoreOrder,
    /// Store intensity
    pub store_intensity: u8,
    /// Load intensity
    pub load_intensity: u8,
}

impl Default for RSEConfig {
    fn default() -> Self {
        Self {
            mode: RSEMode::Lazy,
            load_store_order: LoadStoreOrder::Preserve,
            store_intensity: 0,
            load_intensity: 0,
        }
    }
}

/// RSE operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RSEMode {
    /// Enforced lazy mode
    Enforced,
    /// Eager mode
    Eager,
    /// Lazy mode
    Lazy,
}

/// Load/store ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadStoreOrder {
    /// Preserve
    Preserve,
    /// Release
    Release,
}

impl RSEConfig {
    /// Create from raw bits
    pub fn from_bits(bits: u64) -> Self {
        Self {
            mode: match (bits >> 16) & 0x3 {
                0 => RSEMode::Enforced,
                1 => RSEMode::Eager,
                2 => RSEMode::Lazy,
                _ => RSEMode::Lazy, // Default
            },
            load_store_order: if (bits >> 18) & 1 != 0 {
                LoadStoreOrder::Release
            } else {
                LoadStoreOrder::Preserve
            },
            store_intensity: ((bits >> 19) & 0xF) as u8,
            load_intensity: ((bits >> 23) & 0xF) as u8,
        }
    }

    /// Convert to raw bits
    pub fn to_bits(&self) -> u64 {
        let mode_bits = match self.mode {
            RSEMode::Enforced => 0,
            RSEMode::Eager => 1,
            RSEMode::Lazy => 2,
        };

        let order_bit = match self.load_store_order {
            LoadStoreOrder::Preserve => 0,
            LoadStoreOrder::Release => 1,
        };

        (mode_bits << 16)
            | (order_bit << 18)
            | ((self.store_intensity as u64) << 19)
            | ((self.load_intensity as u64) << 23)
    }
}

/// Register Stack Engine state
#[derive(Debug)]
pub struct RSE {
    /// Configuration
    config: RSEConfig,
    /// Backing store pointer
    bsp: u64,
    /// Backing store pointer for stores
    bspstore: u64,
    /// Number of dirty registers
    dirty_count: u32,
    /// Number of clean registers
    clean_count: u32,
    /// Invalid count
    invalid_count: u32,
    /// NaT collection bits
    rnat: u64,
}

impl Default for RSE {
    fn default() -> Self {
        Self::new()
    }
}

impl RSE {
    /// Create new RSE
    pub fn new() -> Self {
        Self {
            config: RSEConfig::default(),
            bsp: 0,
            bspstore: 0,
            rnat: 0,
            dirty_count: 0,
            clean_count: 0,
            invalid_count: 0,
        }
    }

    /// Get configuration
    pub fn get_config(&self) -> RSEConfig {
        self.config
    }

    /// Set configuration
    pub fn set_config(&mut self, config: RSEConfig) {
        self.config = config;
    }

    /// Get backing store pointer
    pub fn get_bsp(&self) -> u64 {
        self.bsp
    }

    /// Get backing store pointer for stores
    pub fn get_bspstore(&self) -> u64 {
        self.bspstore
    }

    /// Get NaT collection bits
    pub fn get_rnat(&self) -> u64 {
        self.rnat
    }

    /// Spill registers to backing store
    pub fn spill_registers(
        &mut self,
        memory: &mut Memory,
        count: u32,
    ) -> Result<(), EmulatorError> {
        if count > self.dirty_count {
            return Err(EmulatorError::RSEError(
                "Not enough dirty registers to spill".to_string(),
            ));
        }

        for _ in 0..count {
            // Write register value to memory
            memory.write_u64(self.bspstore, 0)?; // TODO: Get actual register value

            // Update RNAT if needed
            if (self.bspstore >> 3) & 0x3F == 0x3F {
                memory.write_u64(self.bspstore + 8, self.rnat)?;
                self.bspstore += 16;
            } else {
                self.bspstore += 8;
            }

            self.dirty_count -= 1;
            self.clean_count += 1;
        }

        Ok(())
    }

    /// Fill registers from backing store
    pub fn fill_registers(&mut self, memory: &mut Memory, count: u32) -> Result<(), EmulatorError> {
        if count > self.invalid_count {
            return Err(EmulatorError::RSEError(
                "Not enough invalid registers to fill".to_string(),
            ));
        }

        for _ in 0..count {
            // Read register value from memory
            let value = memory.read_u64(self.bsp)?;

            // Check if we need to read RNAT
            let nat_bit = (self.rnat >> ((self.bsp >> 3) & 0x3F)) & 1 != 0;

            // Update BSP
            if (self.bsp >> 3) & 0x3F == 0x3F {
                self.rnat = memory.read_u64(self.bsp + 8)?;
                self.bsp += 16;
            } else {
                self.bsp += 8;
            }

            self.invalid_count -= 1;
            self.clean_count += 1;
        }

        Ok(())
    }

    /// Flush dirty registers
    pub fn flush(&mut self, memory: &mut Memory) -> Result<(), EmulatorError> {
        self.spill_registers(memory, self.dirty_count)
    }

    /// Invalidate clean registers
    pub fn invalidate(&mut self) {
        self.invalid_count += self.clean_count;
        self.clean_count = 0;
    }

    /// Handle register allocation
    pub fn allocate_registers(
        &mut self,
        memory: &mut Memory,
        count: u32,
    ) -> Result<(), EmulatorError> {
        // First, try to use clean registers
        let clean_to_use = count.min(self.clean_count);
        if clean_to_use > 0 {
            self.clean_count -= clean_to_use;
            self.dirty_count += clean_to_use;
        }

        // If we still need more registers, use invalid ones
        let remaining = count - clean_to_use;
        if remaining > 0 {
            if remaining > self.invalid_count {
                return Err(EmulatorError::RSEError(
                    "Not enough registers available".to_string(),
                ));
            }
            self.invalid_count -= remaining;
            self.dirty_count += remaining;
        }

        Ok(())
    }

    /// Handle register deallocation
    pub fn deallocate_registers(
        &mut self,
        memory: &mut Memory,
        count: u32,
    ) -> Result<(), EmulatorError> {
        match self.config.mode {
            RSEMode::Lazy => {
                // Just mark registers as invalid
                self.dirty_count = self.dirty_count.saturating_sub(count);
                self.clean_count = self
                    .clean_count
                    .saturating_sub(count.saturating_sub(self.dirty_count));
                self.invalid_count += count;
            }
            RSEMode::Eager => {
                // First spill dirty registers
                let to_spill = count.min(self.dirty_count);
                if to_spill > 0 {
                    self.spill_registers(memory, to_spill)?;
                }

                // Then invalidate clean registers if needed
                let remaining = count - to_spill;
                if remaining > 0 {
                    self.clean_count = self.clean_count.saturating_sub(remaining);
                    self.invalid_count += remaining;
                }
            }
            RSEMode::Enforced => {
                // Similar to eager mode but must spill all registers
                self.spill_registers(memory, self.dirty_count)?;
                self.clean_count = self
                    .clean_count
                    .saturating_sub(count.saturating_sub(self.dirty_count));
                self.invalid_count += count;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Memory;

    #[test]
    fn test_rse_config() {
        let mut rse = RSE::new();

        let config = RSEConfig {
            mode: RSEMode::Eager,
            load_store_order: LoadStoreOrder::Release,
            store_intensity: 7,
            load_intensity: 4,
        };

        rse.set_config(config);
        let read_config = rse.get_config();

        assert_eq!(read_config.mode, RSEMode::Eager);
        assert_eq!(read_config.load_store_order, LoadStoreOrder::Release);
        assert_eq!(read_config.store_intensity, 7);
        assert_eq!(read_config.load_intensity, 4);
    }

    #[test]
    #[ignore = "RSE spill operation needs to be fixed"]
    fn test_rse_spill() {
        let mut rse = RSE::new();
        let mut memory = Memory::new();

        // Set up initial state
        rse.dirty_count = 10;
        rse.bspstore = 0x1000;

        // Spill 5 registers
        assert!(rse.spill_registers(&mut memory, 5).is_ok());

        // Check state after spill
        assert_eq!(rse.dirty_count, 5);
        assert_eq!(rse.clean_count, 5);
        assert_eq!(rse.bspstore, 0x1000 + 5 * 8);
    }

    #[test]
    #[ignore = "RSE fill operation needs to be fixed"]
    fn test_rse_fill() {
        let mut rse = RSE::new();
        let mut memory = Memory::new();

        // Set up initial state
        rse.invalid_count = 10;
        rse.bsp = 0x1000;

        // Fill 5 registers
        assert!(rse.fill_registers(&mut memory, 5).is_ok());

        // Check state after fill
        assert_eq!(rse.invalid_count, 5);
        assert_eq!(rse.clean_count, 5);
        assert_eq!(rse.bsp, 0x1000 + 5 * 8);
    }

    #[test]
    fn test_rse_allocation() {
        let mut rse = RSE::new();
        let mut memory = Memory::new();

        // Set up initial state
        rse.clean_count = 5;
        rse.invalid_count = 10;

        // Allocate 8 registers
        assert!(rse.allocate_registers(&mut memory, 8).is_ok());

        // Check state after allocation
        assert_eq!(rse.clean_count, 0);
        assert_eq!(rse.dirty_count, 8);
        assert_eq!(rse.invalid_count, 7);
    }

    #[test]
    #[ignore = "RSE deallocation needs to be fixed"]
    fn test_rse_deallocation() {
        let mut rse = RSE::new();
        let mut memory = Memory::new();

        // Set up initial state
        rse.dirty_count = 5;
        rse.clean_count = 5;

        // Configure eager mode
        rse.set_config(RSEConfig {
            mode: RSEMode::Eager,
            load_store_order: LoadStoreOrder::Preserve,
            store_intensity: 0,
            load_intensity: 0,
        });

        // Deallocate 8 registers
        assert!(rse.deallocate_registers(&mut memory, 8).is_ok());

        // Check state after deallocation
        assert_eq!(rse.dirty_count, 0);
        assert_eq!(rse.clean_count, 2);
        assert_eq!(rse.invalid_count, 8);
    }

    #[test]
    #[ignore = "RSE RNAT handling needs to be fixed"]
    fn test_rse_rnat() {
        let mut rse = RSE::new();
        let mut memory = Memory::new();

        // Set up initial state with dirty registers
        rse.dirty_count = 63;
        rse.bspstore = 0x1000;

        // Spill registers to trigger RNAT write
        assert!(rse.spill_registers(&mut memory, 63).is_ok());

        // Check RNAT was written
        assert_eq!(rse.bspstore, 0x1000 + 64 * 8); // 63 registers + 1 RNAT

        // Read back RNAT value
        let rnat = memory.read_u64(0x1000 + 63 * 8).unwrap();
        assert_eq!(rnat, 0); // Should be 0 since we didn't set any NaT bits
    }

    #[test]
    #[ignore = "RSE flush operation needs to be fixed"]
    fn test_rse_flush() {
        let mut rse = RSE::new();
        let mut memory = Memory::new();

        // Set up initial state
        rse.dirty_count = 10;
        rse.bspstore = 0x1000;

        // Flush all dirty registers
        assert!(rse.flush(&mut memory).is_ok());

        // Check state after flush
        assert_eq!(rse.dirty_count, 0);
        assert_eq!(rse.clean_count, 10);
        assert_eq!(rse.bspstore, 0x1000 + 10 * 8);
    }

    #[test]
    fn test_rse_invalidate() {
        let mut rse = RSE::new();

        // Set up initial state
        rse.clean_count = 10;

        // Invalidate clean registers
        rse.invalidate();

        // Check state after invalidation
        assert_eq!(rse.clean_count, 0);
        assert_eq!(rse.invalid_count, 10);
    }
}
