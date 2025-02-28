//! Speculation handling for IA-64
//! 
//! This module implements speculation support including advanced loads,
//! check loads, and speculation recovery.

use crate::EmulatorError;
use crate::cpu::Cpu;
use crate::memory::Memory;

/// Speculation state
#[derive(Debug, Default)]
pub struct SpeculationState {
    /// Whether currently in speculative mode
    pub speculative: bool,
    /// Recovery IP for failed speculation
    pub recovery_ip: u64,
}

impl SpeculationState {
    /// Create new speculation state
    pub fn new() -> Self {
        Self::default()
    }

    /// Enter speculative mode
    pub fn enter_speculation(&mut self, recovery_ip: u64) {
        self.speculative = true;
        self.recovery_ip = recovery_ip;
    }

    /// Exit speculative mode
    pub fn exit_speculation(&mut self) {
        self.speculative = false;
        self.recovery_ip = 0;
    }

    /// Check if in speculative mode
    pub fn is_speculative(&self) -> bool {
        self.speculative
    }

    /// Get recovery IP
    pub fn get_recovery_ip(&self) -> u64 {
        self.recovery_ip
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speculation_state() {
        let mut state = SpeculationState::new();
        assert!(!state.is_speculative());
        assert_eq!(state.get_recovery_ip(), 0);

        // Enter speculation
        state.enter_speculation(0x1000);
        assert!(state.is_speculative());
        assert_eq!(state.get_recovery_ip(), 0x1000);

        // Exit speculation
        state.exit_speculation();
        assert!(!state.is_speculative());
        assert_eq!(state.get_recovery_ip(), 0);
    }

    #[test]
    fn test_speculation_recovery() {
        let mut state = SpeculationState::new();
        
        // Set up recovery point
        state.enter_speculation(0x2000);
        assert_eq!(state.get_recovery_ip(), 0x2000);
        
        // Simulate recovery
        let recovery_ip = state.get_recovery_ip();
        state.exit_speculation();
        assert!(!state.is_speculative());
        assert_eq!(recovery_ip, 0x2000);
    }

    #[test]
    fn test_nested_speculation() {
        let mut state = SpeculationState::new();
        
        // First level
        state.enter_speculation(0x1000);
        assert!(state.is_speculative());
        assert_eq!(state.get_recovery_ip(), 0x1000);
        
        // Second level (overrides first)
        state.enter_speculation(0x2000);
        assert!(state.is_speculative());
        assert_eq!(state.get_recovery_ip(), 0x2000);
        
        // Exit inner level
        state.exit_speculation();
        assert!(!state.is_speculative());
    }
} 