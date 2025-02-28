//! Rust IA-64 Emulator
//! 
//! This library provides emulation of the Intel IA-64 (Itanium) architecture.
//! It includes CPU state management, memory management, and instruction decoding/execution.

#![deny(missing_docs)]

pub mod cpu;
pub mod memory;
pub mod decoder;

use std::error::Error;
use std::fmt;

/// Main error type for the emulator
#[derive(Debug)]
pub enum EmulatorError {
    /// Error during instruction execution
    ExecutionError(String),
    /// Error during instruction decoding
    DecodeError(String),
    /// Error during memory access
    MemoryError(String),
    /// Error in CPU state
    CpuStateError(String),
    /// Memory access is not properly aligned
    InvalidAlignment,
    /// Memory regions overlap
    MemoryOverlap,
    /// Invalid system call number
    InvalidSyscall,
    /// No system call context available
    NoSyscallContext,
    /// Error when accessing or manipulating registers
    RegisterError(String),
    /// Error related to Register Stack Engine operations
    RSEError(String),
    /// Error when attempting to execute privileged instructions in user mode
    PrivilegeViolation,
}

impl fmt::Display for EmulatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmulatorError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            EmulatorError::DecodeError(msg) => write!(f, "Decode error: {}", msg),
            EmulatorError::MemoryError(msg) => write!(f, "Memory error: {}", msg),
            EmulatorError::CpuStateError(msg) => write!(f, "CPU state error: {}", msg),
            EmulatorError::InvalidAlignment => write!(f, "Invalid alignment"),
            EmulatorError::MemoryOverlap => write!(f, "Memory overlap"),
            EmulatorError::InvalidSyscall => write!(f, "Invalid syscall"),
            EmulatorError::NoSyscallContext => write!(f, "No syscall context"),
            EmulatorError::RegisterError(msg) => write!(f, "Register error: {}", msg),
            EmulatorError::RSEError(msg) => write!(f, "RSE error: {}", msg),
            EmulatorError::PrivilegeViolation => write!(f, "Privilege violation"),
        }
    }
}

impl Error for EmulatorError {}

/// Result type for emulator operations
pub type EmulatorResult<T> = Result<T, EmulatorError>; 