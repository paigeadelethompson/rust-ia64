//! Interrupt and Exception Handling
//! 
//! This module implements the IA-64 interrupt and exception handling system,
//! including hardware interrupts, software interrupts, faults, and traps.

use crate::EmulatorError;

/// Interrupt vector numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InterruptVector {
    /// External interrupt
    ExtInt = 0,
    /// Virtual memory fault
    VirtualMemoryFault = 1,
    /// Instruction TLB fault
    InstructionTLBFault = 2,
    /// Data TLB fault
    DataTLBFault = 3,
    /// Alternate instruction TLB fault
    AltInstructionTLBFault = 4,
    /// Alternate data TLB fault
    AltDataTLBFault = 5,
    /// Data nested TLB fault
    DataNestedTLBFault = 6,
    /// Instruction key miss fault
    InstructionKeyMissFault = 7,
    /// Data key miss fault
    DataKeyMissFault = 8,
    /// Dirty-bit fault
    DirtyBitFault = 9,
    /// Instruction access bit fault
    InstructionAccessBitFault = 10,
    /// Data access bit fault
    DataAccessBitFault = 11,
    /// Break instruction fault
    BreakFault = 12,
    /// External reset
    ExternalReset = 13,
    /// NaT consumption fault
    NatConsumptionFault = 14,
    /// Reserved register/field fault
    ReservedRegisterFault = 15,
    /// Disabled FP-register fault
    DisabledFPRegisterFault = 16,
    /// Unimplemented data address fault
    UnimplementedDataAddressFault = 17,
    /// Privileged operation fault
    PrivilegedOperationFault = 18,
    /// Disabled instruction set transition fault
    DisabledISATransitionFault = 19,
    /// Illegal operation fault
    IllegalOperationFault = 20,
    /// Illegal dependency fault
    IllegalDependencyFault = 21,
    /// Debug fault
    DebugFault = 22,
    /// Unaligned reference fault
    UnalignedReferenceFault = 23,
    /// Unsupported data reference fault
    UnsupportedDataReferenceFault = 24,
    /// Floating-point fault
    FPFault = 25,
    /// Floating-point trap
    FPTrap = 26,
    /// Lower-privilege transfer trap
    LowerPrivilegeTransferTrap = 27,
    /// Taken branch trap
    TakenBranchTrap = 28,
    /// Single step trap
    SingleStepTrap = 29,
}

/// Interrupt state information
#[derive(Debug, Clone)]
pub struct InterruptState {
    /// Interrupt vector number
    pub vector: InterruptVector,
    /// Instruction pointer where interrupt occurred
    pub ip: u64,
    /// Processor status register at time of interrupt
    pub psr: u64,
    /// Instruction bundle that caused the interrupt
    pub bundle: [u8; 16],
    /// Additional interrupt-specific information
    pub info: u64,
}

/// Interrupt handler table entry
#[derive(Debug, Clone)]
struct HandlerEntry {
    /// Handler function address
    address: u64,
    /// Minimum privilege level required
    min_privilege: u8,
    /// Whether handler is enabled
    enabled: bool,
}

/// Interrupt handler table
#[derive(Debug)]
pub struct InterruptTable {
    /// Handler entries indexed by vector number
    handlers: Vec<HandlerEntry>,
}

impl InterruptTable {
    /// Create new interrupt table
    pub fn new() -> Self {
        let mut handlers = Vec::with_capacity(32);
        for _ in 0..32 {
            handlers.push(HandlerEntry {
                address: 0,
                min_privilege: 0,
                enabled: false,
            });
        }
        Self { handlers }
    }

    /// Register interrupt handler
    pub fn register_handler(&mut self, vector: InterruptVector, address: u64, min_privilege: u8) -> Result<(), EmulatorError> {
        let idx = vector as usize;
        if idx >= self.handlers.len() {
            return Err(EmulatorError::ExecutionError(format!(
                "Invalid interrupt vector: {}", idx
            )));
        }

        self.handlers[idx] = HandlerEntry {
            address,
            min_privilege,
            enabled: true,
        };

        Ok(())
    }

    /// Enable/disable handler
    pub fn set_handler_enabled(&mut self, vector: InterruptVector, enabled: bool) -> Result<(), EmulatorError> {
        let idx = vector as usize;
        if idx >= self.handlers.len() {
            return Err(EmulatorError::ExecutionError(format!(
                "Invalid interrupt vector: {}", idx
            )));
        }

        self.handlers[idx].enabled = enabled;
        Ok(())
    }

    /// Get handler address
    pub fn get_handler_address(&self, vector: InterruptVector) -> Result<Option<u64>, EmulatorError> {
        let idx = vector as usize;
        if idx >= self.handlers.len() {
            return Err(EmulatorError::ExecutionError(format!(
                "Invalid interrupt vector: {}", idx
            )));
        }

        let handler = &self.handlers[idx];
        if handler.enabled {
            Ok(Some(handler.address))
        } else {
            Ok(None)
        }
    }
}

/// Interrupt controller state
#[derive(Debug)]
pub struct InterruptController {
    /// Interrupt table
    table: InterruptTable,
    /// Pending interrupts
    pending: Vec<InterruptState>,
    /// Currently executing interrupt
    current: Option<InterruptState>,
    /// Interrupt nesting level
    nesting_level: u32,
    /// Whether interrupts are enabled
    interrupts_enabled: bool,
}

impl InterruptController {
    /// Create new interrupt controller
    pub fn new() -> Self {
        Self {
            table: InterruptTable::new(),
            pending: Vec::new(),
            current: None,
            nesting_level: 0,
            interrupts_enabled: false,
        }
    }

    /// Register interrupt handler
    pub fn register_handler(&mut self, vector: InterruptVector, address: u64, min_privilege: u8) -> Result<(), EmulatorError> {
        self.table.register_handler(vector, address, min_privilege)
    }

    /// Enable/disable interrupts
    pub fn set_interrupts_enabled(&mut self, enabled: bool) {
        self.interrupts_enabled = enabled;
    }

    /// Raise interrupt
    pub fn raise_interrupt(&mut self, state: InterruptState) {
        self.pending.push(state);
    }

    /// Check and handle pending interrupts
    pub fn check_interrupts(&mut self) -> Option<u64> {
        if !self.interrupts_enabled || self.pending.is_empty() {
            return None;
        }

        // Get highest priority pending interrupt
        if let Some(state) = self.pending.pop() {
            // Save current state if this is a nested interrupt
            if self.nesting_level > 0 {
                if let Some(current) = self.current.take() {
                    self.pending.push(current);
                }
            }

            // Get handler address and check privilege
            if let Ok(Some(handler_addr)) = self.table.get_handler_address(state.vector) {
                // Get handler entry
                let idx = state.vector as usize;
                if idx < self.table.handlers.len() {
                    let handler = &self.table.handlers[idx];
                    
                    // Check if handler is enabled and privilege level is sufficient
                    if handler.enabled && (state.psr >> 32) & 0x3 >= handler.min_privilege as u64 {
                        self.current = Some(state);
                        self.nesting_level += 1;
                        return Some(handler_addr);
                    }
                }
            }
        }

        None
    }

    /// Return from interrupt
    pub fn return_from_interrupt(&mut self) -> Option<u64> {
        if self.nesting_level == 0 {
            return None;
        }

        self.nesting_level -= 1;
        self.current = None;

        // Restore previous interrupt state if any
        if !self.pending.is_empty() {
            if let Some(state) = self.pending.pop() {
                if let Ok(Some(handler_addr)) = self.table.get_handler_address(state.vector) {
                    self.current = Some(state);
                    return Some(handler_addr);
                }
            }
        }

        None
    }

    /// Get current interrupt state
    pub fn current_interrupt(&self) -> Option<&InterruptState> {
        self.current.as_ref()
    }

    /// Get interrupt nesting level
    pub fn nesting_level(&self) -> u32 {
        self.nesting_level
    }

    /// Clear all pending interrupts
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interrupt_table_creation() {
        let table = InterruptTable::new();
        assert_eq!(table.handlers.len(), 32);
        for handler in &table.handlers {
            assert_eq!(handler.address, 0);
            assert_eq!(handler.min_privilege, 0);
            assert!(!handler.enabled);
        }
    }

    #[test]
    fn test_interrupt_registration() {
        let mut controller = InterruptController::new();
        
        // Register handler
        assert!(controller.register_handler(
            InterruptVector::ExtInt,
            0x1000,
            0
        ).is_ok());
        
        // Check handler address
        assert_eq!(
            controller.table.get_handler_address(InterruptVector::ExtInt).unwrap(),
            Some(0x1000)
        );

        // Try registering invalid vector
        assert!(controller.register_handler(
            InterruptVector::SingleStepTrap,
            0x2000,
            0
        ).is_ok());
    }

    #[test]
    fn test_interrupt_enable_disable() {
        let mut controller = InterruptController::new();
        
        // Register and enable handler
        assert!(controller.register_handler(
            InterruptVector::ExtInt,
            0x1000,
            0
        ).is_ok());
        
        assert!(controller.table.set_handler_enabled(InterruptVector::ExtInt, true).is_ok());
        assert_eq!(
            controller.table.get_handler_address(InterruptVector::ExtInt).unwrap(),
            Some(0x1000)
        );
        
        // Disable handler
        assert!(controller.table.set_handler_enabled(InterruptVector::ExtInt, false).is_ok());
        assert_eq!(
            controller.table.get_handler_address(InterruptVector::ExtInt).unwrap(),
            None
        );
    }

    #[test]
    fn test_interrupt_handling() {
        let mut controller = InterruptController::new();
        
        // Register handlers
        assert!(controller.register_handler(
            InterruptVector::ExtInt,
            0x1000,
            0
        ).is_ok());
        
        assert!(controller.register_handler(
            InterruptVector::DebugFault,
            0x2000,
            0
        ).is_ok());
        
        // Enable interrupts
        controller.set_interrupts_enabled(true);
        
        // Raise interrupts
        controller.raise_interrupt(InterruptState {
            vector: InterruptVector::ExtInt,
            ip: 0x100,
            psr: 0,
            bundle: [0; 16],
            info: 0,
        });
        
        controller.raise_interrupt(InterruptState {
            vector: InterruptVector::DebugFault,
            ip: 0x200,
            psr: 0,
            bundle: [0; 16],
            info: 0,
        });
        
        // Check interrupt handling
        assert_eq!(controller.check_interrupts(), Some(0x2000));
        assert_eq!(controller.nesting_level(), 1);
        
        assert_eq!(controller.check_interrupts(), Some(0x1000));
        assert_eq!(controller.nesting_level(), 2);
        
        // Return from interrupts
        assert_eq!(controller.return_from_interrupt(), Some(0x2000));
        assert_eq!(controller.nesting_level(), 1);
        
        assert_eq!(controller.return_from_interrupt(), None);
        assert_eq!(controller.nesting_level(), 0);
    }

    #[test]
    fn test_interrupt_privilege_levels() {
        let mut controller = InterruptController::new();
        
        // Register handler with minimum privilege level
        assert!(controller.register_handler(
            InterruptVector::ExtInt,
            0x1000,
            2
        ).is_ok());
        
        controller.set_interrupts_enabled(true);
        
        // Try to handle interrupt with insufficient privilege
        controller.raise_interrupt(InterruptState {
            vector: InterruptVector::ExtInt,
            ip: 0x100,
            psr: 0, // Privilege level 0
            bundle: [0; 16],
            info: 0,
        });
        
        assert_eq!(controller.check_interrupts(), None);
        
        // Try with sufficient privilege
        controller.raise_interrupt(InterruptState {
            vector: InterruptVector::ExtInt,
            ip: 0x100,
            psr: 2 << 32, // Privilege level 2
            bundle: [0; 16],
            info: 0,
        });
        
        assert_eq!(controller.check_interrupts(), Some(0x1000));
    }

    #[test]
    fn test_interrupt_state_management() {
        let mut controller = InterruptController::new();
        
        // Register handler
        assert!(controller.register_handler(
            InterruptVector::ExtInt,
            0x1000,
            0
        ).is_ok());
        
        controller.set_interrupts_enabled(true);
        
        // Raise interrupt
        let state = InterruptState {
            vector: InterruptVector::ExtInt,
            ip: 0x100,
            psr: 0,
            bundle: [0; 16],
            info: 42,
        };
        controller.raise_interrupt(state.clone());
        
        // Handle interrupt
        assert_eq!(controller.check_interrupts(), Some(0x1000));
        
        // Check current state
        let current = controller.current_interrupt().unwrap();
        assert_eq!(current.vector, state.vector);
        assert_eq!(current.ip, state.ip);
        assert_eq!(current.psr, state.psr);
        assert_eq!(current.info, state.info);
    }

    #[test]
    fn test_nested_interrupts() {
        let mut controller = InterruptController::new();
        
        // Register handlers
        assert!(controller.register_handler(
            InterruptVector::ExtInt,
            0x1000,
            0
        ).is_ok());
        
        assert!(controller.register_handler(
            InterruptVector::DebugFault,
            0x2000,
            0
        ).is_ok());
        
        controller.set_interrupts_enabled(true);
        
        // Raise first interrupt
        controller.raise_interrupt(InterruptState {
            vector: InterruptVector::ExtInt,
            ip: 0x100,
            psr: 0,
            bundle: [0; 16],
            info: 0,
        });
        
        // Handle first interrupt
        assert_eq!(controller.check_interrupts(), Some(0x1000));
        assert_eq!(controller.nesting_level(), 1);
        
        // Raise nested interrupt
        controller.raise_interrupt(InterruptState {
            vector: InterruptVector::DebugFault,
            ip: 0x200,
            psr: 0,
            bundle: [0; 16],
            info: 0,
        });
        
        // Handle nested interrupt
        assert_eq!(controller.check_interrupts(), Some(0x2000));
        assert_eq!(controller.nesting_level(), 2);
        
        // Return from nested interrupt
        assert_eq!(controller.return_from_interrupt(), Some(0x1000));
        assert_eq!(controller.nesting_level(), 1);
        
        // Return from first interrupt
        assert_eq!(controller.return_from_interrupt(), None);
        assert_eq!(controller.nesting_level(), 0);
    }
} 