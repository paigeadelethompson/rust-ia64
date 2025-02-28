//! Memory Management Unit (MMU) implementation
//! 
//! This module implements the IA-64 MMU including TLB management,
//! virtual address translation, and memory protection.

use crate::EmulatorError;
use crate::memory::{Memory, Permissions};

/// Page size (4KB)
pub const PAGE_SIZE: u64 = 4096;

/// TLB entry
#[derive(Debug, Clone, Copy)]
pub struct TLBEntry {
    /// Virtual page number
    pub vpn: u64,
    /// Physical page number
    pub ppn: u64,
    /// Page permissions
    pub permissions: Permissions,
    /// Valid bit
    pub valid: bool,
}

/// Memory Management Unit
#[derive(Debug)]
pub struct MMU {
    /// TLB entries
    tlb: Vec<TLBEntry>,
    /// Page table base register
    pub ptbr: u64,
}

impl MMU {
    /// Create new MMU instance
    pub fn new() -> Self {
        Self {
            tlb: Vec::new(),
            ptbr: 0,
        }
    }

    /// Add TLB entry
    pub fn add_tlb_entry(&mut self, vpn: u64, ppn: u64, permissions: Permissions) {
        let entry = TLBEntry {
            vpn,
            ppn,
            permissions,
            valid: true,
        };
        self.tlb.push(entry);
    }

    /// Translate virtual address
    pub fn translate(&self, vaddr: u64) -> Result<u64, EmulatorError> {
        let vpn = vaddr / PAGE_SIZE;
        let offset = vaddr % PAGE_SIZE;

        // Look up in TLB
        for entry in &self.tlb {
            if entry.valid && entry.vpn == vpn {
                return Ok(entry.ppn * PAGE_SIZE + offset);
            }
        }

        Err(EmulatorError::ExecutionError("TLB miss".to_string()))
    }

    /// Check memory access permissions
    pub fn check_permissions(&self, vaddr: u64, required: Permissions) -> Result<(), EmulatorError> {
        let vpn = vaddr / PAGE_SIZE;

        // Look up in TLB
        for entry in &self.tlb {
            if entry.valid && entry.vpn == vpn {
                if entry.permissions.contains(required) {
                    return Ok(());
                } else {
                    return Err(EmulatorError::ExecutionError("Permission denied".to_string()));
                }
            }
        }

        Err(EmulatorError::ExecutionError("TLB miss".to_string()))
    }

    /// Invalidate TLB entry
    pub fn invalidate_entry(&mut self, vpn: u64) {
        for entry in &mut self.tlb {
            if entry.vpn == vpn {
                entry.valid = false;
            }
        }
    }

    /// Flush entire TLB
    pub fn flush_tlb(&mut self) {
        self.tlb.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tlb_basic() {
        let mut mmu = MMU::new();
        
        // Add TLB entry
        mmu.add_tlb_entry(0x1000, 0x2000, Permissions::ReadWrite);
        
        // Translate address
        let vaddr = 0x1000 * PAGE_SIZE + 0x123;
        let paddr = mmu.translate(vaddr).unwrap();
        assert_eq!(paddr, 0x2000 * PAGE_SIZE + 0x123);
    }

    #[test]
    fn test_tlb_permissions() {
        let mut mmu = MMU::new();
        
        // Add TLB entry with read-only permissions
        mmu.add_tlb_entry(0x1000, 0x2000, Permissions::Read);
        
        // Check permissions
        let vaddr = 0x1000 * PAGE_SIZE;
        assert!(mmu.check_permissions(vaddr, Permissions::Read).is_ok());
        assert!(mmu.check_permissions(vaddr, Permissions::Write).is_err());
    }

    #[test]
    fn test_tlb_miss() {
        let mmu = MMU::new();
        
        // Try to translate non-existent mapping
        let result = mmu.translate(0x1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_tlb_invalidation() {
        let mut mmu = MMU::new();
        
        // Add TLB entry
        mmu.add_tlb_entry(0x1000, 0x2000, Permissions::ReadWrite);
        
        // Invalidate entry
        mmu.invalidate_entry(0x1000);
        
        // Try to translate - should fail
        let result = mmu.translate(0x1000 * PAGE_SIZE);
        assert!(result.is_err());
    }

    #[test]
    fn test_tlb_flush() {
        let mut mmu = MMU::new();
        
        // Add multiple TLB entries
        mmu.add_tlb_entry(0x1000, 0x2000, Permissions::ReadWrite);
        mmu.add_tlb_entry(0x3000, 0x4000, Permissions::ReadWrite);
        
        // Flush TLB
        mmu.flush_tlb();
        
        // Try to translate - should fail
        let result1 = mmu.translate(0x1000 * PAGE_SIZE);
        let result2 = mmu.translate(0x3000 * PAGE_SIZE);
        assert!(result1.is_err());
        assert!(result2.is_err());
    }
} 