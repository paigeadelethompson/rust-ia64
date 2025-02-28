//! CPU implementation for IA-64 architecture
//! 
//! This module implements the CPU state and operations for the IA-64 architecture,
//! including register management and instruction execution.

use crate::EmulatorError;
use crate::cpu::alat::ALAT;
use crate::cpu::interrupts::{InterruptController, InterruptVector, InterruptState};
use crate::cpu::syscall::{SyscallManager, SyscallNumber, SyscallContext};
use crate::cpu::registers::RegisterState;
use crate::cpu::rse::{RSE, RSEConfig, RSEMode};
use crate::memory::Memory;
use crate::cpu::registers::{ARFile, CRFile, RRFile, PKRFile, DBRFile, DDRFile};

pub mod alat;
pub mod interrupts;
pub mod syscall;
pub mod instructions;
/// Register management module containing implementations for various register types
/// including general purpose registers, floating point registers, predicate registers,
/// branch registers, application registers, control registers, region registers,
/// protection key registers, debug break registers, and data debug registers.
pub mod registers;
pub mod rse;

/// Number of general purpose registers in IA-64
pub const NUM_GR: usize = 128;
/// Number of floating point registers in IA-64
pub const NUM_FR: usize = 128;
/// Number of predicate registers in IA-64
pub const NUM_PR: usize = 64;
/// Number of branch registers in IA-64
pub const NUM_BR: usize = 8;

/// Processor status register flags
#[derive(Debug, Clone, Copy)]
pub struct PSR(u64);

impl PSR {
    /// Create empty PSR
    pub fn empty() -> Self {
        Self(0)
    }

    /// Get raw bits
    pub fn bits(&self) -> u64 {
        self.0
    }

    /// Create from raw bits
    pub fn from_bits_truncate(bits: u64) -> Self {
        Self(bits)
    }

    /// Check if contains flag
    pub fn contains(&self, flag: PSRFlags) -> bool {
        self.0 & (flag as u64) != 0
    }

    /// Set flag
    pub fn set(&mut self, flag: PSRFlags, value: bool) {
        if value {
            self.0 |= flag as u64;
        } else {
            self.0 &= !(flag as u64);
        }
    }
}

/// Individual PSR flags
#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub enum PSRFlags {
    /// User mask
    UM = 1 << 0,
    /// Secure bit
    SECURE = 1 << 1,
    /// Register stack engine enable
    RSE = 1 << 2,
    /// Big-endian memory access enable
    BE = 1 << 3,
    /// Data access fault disable
    DFLT = 1 << 4,
    /// Instruction access fault disable
    IFLT = 1 << 5,
    /// Performance monitor enable
    PME = 1 << 6,
    /// Interrupt collection
    IC = 1 << 13,
    /// Interrupt enable
    I = 1 << 14,
    /// Data debug fault disable
    DD = 1 << 39,
    /// Instruction debug fault disable
    ID = 1 << 40,
}

impl PSRFlags {
    /// Returns the raw bits of the flag
    pub fn bits(self) -> u64 {
        self as u64
    }
}

/// CPU state for IA-64
#[derive(Debug)]
pub struct Cpu {
    /// General registers (r0-r127)
    pub gr: [u64; NUM_GR],
    /// Floating point registers (f0-f127)
    pub fr: [u64; NUM_FR],
    /// Predicate registers (p0-p63)
    pub pr: [bool; NUM_PR],
    /// Branch registers (b0-b7)
    pub br: [u64; NUM_BR],
    /// Instruction pointer
    pub ip: u64,
    /// Previous function state
    pub pfs: u64,
    /// Current frame marker
    pub cfm: u64,
    /// User mask
    pub user_mask: u64,
    /// System registers
    pub system_regs: RegisterState,
    /// ALAT
    pub alat: ALAT,
    /// Interrupt controller
    pub interrupt_ctrl: InterruptController,
    /// Syscall manager
    pub syscall_mgr: SyscallManager,
    /// Register Stack Engine
    pub rse: RSE,
    /// Memory
    pub memory: Memory,
}

impl Default for Cpu {
    fn default() -> Self {
        let mut cpu = Self {
            gr: [0; NUM_GR],
            fr: [0; NUM_FR],
            pr: [false; NUM_PR],
            br: [0; NUM_BR],
            ip: 0,
            pfs: 0,
            cfm: 0,
            user_mask: 0,
            system_regs: RegisterState::new(),
            alat: ALAT::new(),
            interrupt_ctrl: InterruptController::new(),
            syscall_mgr: SyscallManager::new(),
            rse: RSE::new(),
            memory: Memory::new(),
        };
        cpu.syscall_mgr.init_default_handlers();
        cpu
    }
}

impl Cpu {
    /// Create a new CPU instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset the CPU state
    pub fn reset(&mut self) -> Result<(), EmulatorError> {
        // Reset registers
        self.gr = [0; NUM_GR];
        self.fr = [0; NUM_FR];
        self.pr = [false; NUM_PR];
        self.br = [0; NUM_BR];
        
        // Reset instruction pointer
        self.ip = 0;
        
        // Reset current frame marker
        self.cfm = 0;
        
        // Reset system registers
        self.system_regs.cr = PSR::empty().into();
        
        Ok(())
    }

    /// Get the value of a general register
    pub fn get_gr(&self, reg: usize) -> Result<u64, EmulatorError> {
        if reg >= NUM_GR {
            return Err(EmulatorError::CpuStateError(
                format!("Invalid general register index: {}", reg)
            ));
        }
        Ok(self.gr[reg])
    }

    /// Set the value of a general register
    pub fn set_gr(&mut self, reg: usize, value: u64) -> Result<(), EmulatorError> {
        if reg >= NUM_GR {
            return Err(EmulatorError::CpuStateError(
                format!("Invalid general register index: {}", reg)
            ));
        }
        // r0 is always 0 in IA-64
        if reg != 0 {
            self.gr[reg] = value;
        }
        Ok(())
    }

    /// Get the value of a floating point register
    pub fn get_fr(&self, reg: usize) -> Result<f64, EmulatorError> {
        if reg >= NUM_FR {
            return Err(EmulatorError::CpuStateError(
                format!("Invalid floating point register index: {}", reg)
            ));
        }
        Ok(f64::from_bits(self.fr[reg]))
    }

    /// Set the value of a floating point register
    pub fn set_fr(&mut self, reg: usize, value: f64) -> Result<(), EmulatorError> {
        if reg >= NUM_FR {
            return Err(EmulatorError::CpuStateError(
                format!("Invalid floating point register index: {}", reg)
            ));
        }
        self.fr[reg] = value.to_bits();
        Ok(())
    }

    /// Get the value of a predicate register
    pub fn get_pr(&self, reg: usize) -> Result<bool, EmulatorError> {
        if reg >= NUM_PR {
            return Err(EmulatorError::CpuStateError(
                format!("Invalid predicate register index: {}", reg)
            ));
        }
        Ok(self.pr[reg])
    }

    /// Set the value of a predicate register
    pub fn set_pr(&mut self, reg: usize, value: bool) -> Result<(), EmulatorError> {
        if reg >= NUM_PR {
            return Err(EmulatorError::CpuStateError(
                format!("Invalid predicate register index: {}", reg)
            ));
        }
        self.pr[reg] = value;
        Ok(())
    }

    /// Get the value of a branch register
    pub fn get_br(&self, reg: usize) -> Result<u64, EmulatorError> {
        if reg >= NUM_BR {
            return Err(EmulatorError::CpuStateError(
                format!("Invalid branch register index: {}", reg)
            ));
        }
        Ok(self.br[reg])
    }

    /// Set the value of a branch register
    pub fn set_br(&mut self, reg: usize, value: u64) -> Result<(), EmulatorError> {
        if reg >= NUM_BR {
            return Err(EmulatorError::CpuStateError(
                format!("Invalid branch register index: {}", reg)
            ));
        }
        self.br[reg] = value;
        Ok(())
    }

    /// Add entry to ALAT
    pub fn alat_add_entry(&mut self, address: u64, size: u64, register: u32, is_integer: bool) -> Result<(), EmulatorError> {
        self.alat.add_entry(address, size, register, is_integer)
    }

    /// Check if register has valid ALAT entry
    pub fn alat_check_register(&self, register: u32, is_integer: bool) -> bool {
        self.alat.check_register(register, is_integer)
    }

    /// Invalidate overlapping ALAT entries
    pub fn alat_invalidate_overlap(&mut self, address: u64, size: u64) {
        self.alat.invalidate_overlap(address, size)
    }

    /// Clear all ALAT entries
    pub fn alat_clear(&mut self) {
        self.alat.clear()
    }

    /// Get ALAT entry information
    pub fn alat_get_entry_info(&self, register: u32, is_integer: bool) -> Option<(u64, u64, alat::EntryState)> {
        self.alat.get_entry_info(register, is_integer)
    }

    /// Remove ALAT entry
    pub fn alat_remove_entry(&mut self, register: u32, is_integer: bool) {
        self.alat.remove_entry(register, is_integer)
    }

    /// Purge old ALAT entries
    pub fn alat_purge_old_entries(&mut self) {
        self.alat.purge_old_entries()
    }

    /// Register interrupt handler
    pub fn register_interrupt_handler(&mut self, vector: InterruptVector, address: u64, min_privilege: u8) -> Result<(), EmulatorError> {
        self.interrupt_ctrl.register_handler(vector, address, min_privilege)
    }

    /// Enable/disable interrupts
    pub fn set_interrupts_enabled(&mut self, enabled: bool) {
        self.interrupt_ctrl.set_interrupts_enabled(enabled);
        self.system_regs.cr.set(PSRFlags::I, enabled);
    }

    /// Raise interrupt
    pub fn raise_interrupt(&mut self, vector: InterruptVector, info: u64) {
        let state = InterruptState {
            vector,
            ip: self.ip,
            psr: self.system_regs.cr.get_psr(),
            bundle: [0; 16], // Current bundle would be filled in by decoder
            info,
        };
        self.interrupt_ctrl.raise_interrupt(state);
    }

    /// Check and handle pending interrupts
    pub fn check_interrupts(&mut self) -> Option<u64> {
        // Only check if interrupts are enabled in PSR
        if !self.system_regs.cr.contains(PSRFlags::I) {
            return None;
        }

        if let Some(handler_addr) = self.interrupt_ctrl.check_interrupts() {
            // Switch to privileged mode
            self.system_regs.cr.set(PSRFlags::I, false); // Disable interrupts
            self.system_regs.cr.set(PSRFlags::IC, true); // Set interrupt collection
            
            // Return handler address
            Some(handler_addr)
        } else {
            None
        }
    }

    /// Return from interrupt
    pub fn return_from_interrupt(&mut self) -> Result<(), EmulatorError> {
        // Get current interrupt state
        let state = match self.interrupt_ctrl.current_interrupt() {
            Some(s) => s.clone(),
            None => return Err(EmulatorError::ExecutionError("No interrupt to return from".to_string())),
        };

        // Restore saved state
        self.system_regs.cr = PSR::from_bits_truncate(state.psr).into();
        
        // Get next handler or return to interrupted code
        let next_ip = self.interrupt_ctrl.return_from_interrupt()
            .unwrap_or(state.ip);
            
        // Update instruction pointer
        self.ip = next_ip;
        
        Ok(())
    }

    /// Get current interrupt state
    pub fn current_interrupt(&self) -> Option<&InterruptState> {
        self.interrupt_ctrl.current_interrupt()
    }

    /// Get interrupt nesting level
    pub fn interrupt_nesting_level(&self) -> u32 {
        self.interrupt_ctrl.nesting_level()
    }

    /// Clear all pending interrupts
    pub fn clear_pending_interrupts(&mut self) {
        self.interrupt_ctrl.clear_pending();
    }

    /// Initialize CPU state
    pub fn init(&mut self) -> Result<(), EmulatorError> {
        // Initialize registers
        self.gr = [0; NUM_GR];
        self.fr = [0; NUM_FR];
        self.pr = [false; NUM_PR];
        self.br = [0; NUM_BR];

        // Initialize special registers
        self.ip = 0;
        self.system_regs.cr = PSR::empty().into();

        // Initialize ALAT
        self.alat = ALAT::new();

        // Initialize interrupt controller
        self.interrupt_ctrl = InterruptController::new();

        // Initialize syscall manager
        self.syscall_mgr = SyscallManager::default();

        Ok(())
    }

    /// Execute system call
    pub fn do_syscall(&mut self, syscall_num: u64) -> Result<(), EmulatorError> {
        // Begin syscall
        let mut syscall_mgr = std::mem::take(&mut self.syscall_mgr);
        syscall_mgr.begin_syscall(self, syscall_num)?;

        // Execute syscall
        let mut context = syscall_mgr.current.take()
            .ok_or(EmulatorError::NoSyscallContext)?;
        syscall_mgr.execute_syscall(self, &mut context)?;
        syscall_mgr.current = Some(context);

        // End syscall
        syscall_mgr.end_syscall(self)?;
        self.syscall_mgr = syscall_mgr;
        Ok(())
    }

    /// Register system call handler
    pub fn register_syscall_handler(&mut self, number: SyscallNumber, handler: impl Fn(&mut Cpu, &mut SyscallContext) -> Result<(), EmulatorError> + Send + Sync + 'static) {
        self.syscall_mgr.register_handler(number, handler);
    }

    /// Get current system call context
    pub fn get_syscall_context(&self) -> Option<&SyscallContext> {
        self.syscall_mgr.current.as_ref()
    }

    /// Get processor status register
    pub fn get_psr(&self) -> u64 {
        self.system_regs.cr.get_psr()
    }

    /// Get interruption status register
    pub fn get_isr(&self) -> u64 {
        self.system_regs.cr.get_isr()
    }

    /// Get RSE configuration
    pub fn get_rse_config(&self) -> RSEConfig {
        self.rse.get_config()
    }

    /// Set RSE configuration
    pub fn set_rse_config(&mut self, config: RSEConfig) {
        self.rse.set_config(config);
    }

    /// Get RSE backing store pointer
    pub fn get_rse_bsp(&self) -> u64 {
        self.rse.get_bsp()
    }

    /// Get RSE backing store pointer for stores
    pub fn get_rse_bspstore(&self) -> u64 {
        self.rse.get_bspstore()
    }

    /// Get RSE NaT collection bits
    pub fn get_rse_rnat(&self) -> u64 {
        self.rse.get_rnat()
    }

    /// Allocate registers in current frame
    pub fn allocate_registers(&mut self, memory: &mut Memory, count: u32) -> Result<(), EmulatorError> {
        self.rse.allocate_registers(memory, count)
    }

    /// Deallocate registers from current frame
    pub fn deallocate_registers(&mut self, memory: &mut Memory, count: u32) -> Result<(), EmulatorError> {
        self.rse.deallocate_registers(memory, count)
    }

    /// Flush RSE
    pub fn flush_rse(&mut self, memory: &mut Memory) -> Result<(), EmulatorError> {
        self.rse.flush(memory)
    }

    /// Handle branch with alloc
    pub fn branch_with_alloc(&mut self, memory: &mut Memory, sof: u32, sol: u32, sor: u32) -> Result<(), EmulatorError> {
        let old_sof = (self.cfm >> 0 & 0x7F) as u32;
        let to_allocate = sof.saturating_sub(old_sof);
        let to_deallocate = old_sof.saturating_sub(sof);

        if to_allocate > 0 {
            self.rse.allocate_registers(memory, to_allocate)?;
        } else if to_deallocate > 0 {
            self.rse.deallocate_registers(memory, to_deallocate)?;
        }

        self.cfm = ((sof as u64) << 0) |
                  ((sol as u64) << 7) |
                  ((sor as u64) << 14);
        Ok(())
    }

    /// Handle return
    pub fn handle_return(&mut self, memory: &mut Memory) -> Result<(), EmulatorError> {
        // Get previous frame state from PFS
        let prev_sof = (self.pfs >> 0) & 0x7F;
        let prev_sol = (self.pfs >> 7) & 0x7F;
        let prev_sor = (self.pfs >> 14) & 0x7F;

        // Get current frame size
        let curr_sof = (self.cfm >> 0) & 0x7F;

        // Deallocate current frame
        self.deallocate_registers(memory, curr_sof as u32)?;

        // Restore previous frame
        self.cfm = ((prev_sof as u64) << 0) |
                   ((prev_sol as u64) << 7) |
                   ((prev_sor as u64) << 14);

        Ok(())
    }

    /// Check memory protection key
    pub fn check_protection_key(&self, key: u32, read: bool, write: bool, execute: bool) -> bool {
        if read && !self.system_regs.pkr.check_read(key) {
            return false;
        }
        if write && !self.system_regs.pkr.check_write(key) {
            return false;
        }
        if execute && !self.system_regs.pkr.check_execute(key) {
            return false;
        }
        true
    }

    /// Check debug breakpoint
    pub fn check_breakpoint(&self, addr: u64, pl: u8, access_type: registers::dbr::BreakAccessType) -> bool {
        self.system_regs.dbr.check_break(addr, pl, access_type)
    }

    /// Check debug data match
    pub fn check_data_match(&self, value: u64) -> bool {
        self.system_regs.ddr.check_match(value)
    }

    /// Get region ID for virtual address
    pub fn get_region_id(&self, addr: u64) -> Result<u64, EmulatorError> {
        let region = (addr >> 61) as usize;
        self.system_regs.rr.get_rid(region)
    }

    /// Get page size for virtual address
    pub fn get_page_size(&self, addr: u64) -> Result<u8, EmulatorError> {
        let region = (addr >> 61) as usize;
        self.system_regs.rr.get_ps(region)
    }

    /// Check if region is enabled
    pub fn is_region_enabled(&self, addr: u64) -> Result<bool, EmulatorError> {
        let region = (addr >> 61) as usize;
        self.system_regs.rr.is_enabled(region)
    }

    /// Updates the frame markers for the current frame
    pub fn update_frame_markers(&mut self, sof: u32, sol: u32, sor: u32) -> Result<(), EmulatorError> {
        // Validate parameters
        if sof < sol || sol < sor {
            return Err(EmulatorError::CpuStateError("Invalid frame marker values".to_string()));
        }
        
        // Update frame markers
        self.cfm = ((sof as u64) << 0) | ((sol as u64) << 7) | ((sor as u64) << 14);
        Ok(())
    }

    /// Restores CPU state from a saved processor state
    pub fn restore_state(&mut self, state: &ProcessorState) -> Result<(), EmulatorError> {
        self.gr = state.gr;
        self.fr = state.fr;
        self.pr = state.pr;
        self.br = state.br;
        self.ip = state.ip;
        self.cfm = state.cfm;
        self.system_regs.cr = CRFile::from_bits_truncate(state.psr);
        Ok(())
    }
}

/// Represents the complete processor state that can be saved and restored
pub struct ProcessorState {
    /// General registers
    pub gr: [u64; NUM_GR],
    /// Floating-point registers
    pub fr: [u64; NUM_FR],
    /// Predicate registers
    pub pr: [bool; NUM_PR],
    /// Branch registers
    pub br: [u64; NUM_BR],
    /// Instruction pointer
    pub ip: u64,
    /// Current frame marker
    pub cfm: u64,
    /// Processor status register
    pub psr: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::syscall::SyscallNumber;

    #[test]
    fn test_syscall() {
        let mut cpu = Cpu::default();

        // Set up syscall parameters
        let syscall_num = SyscallNumber::Write as u64;
        let fd = 1; // stdout
        let buf = 0x1000; // buffer address
        let count = 100; // bytes to write

        // Set up registers
        cpu.gr[32] = fd;
        cpu.gr[33] = buf;
        cpu.gr[34] = count;

        // Execute syscall
        cpu.do_syscall(syscall_num).unwrap();

        // Check return value
        assert_eq!(cpu.gr[8], count);
        assert_eq!(cpu.gr[9], 0); // no error
    }
} 