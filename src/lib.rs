//! # Rust IA-64 Emulator
//!
//! This is an emulator for the Intel Itanium (IA-64) architecture written in Rust.
//! The emulator aims to provide accurate emulation of IA-64 instructions and system
//! behavior.
//!
//! ## Features
//!
//! - Full IA-64 instruction set emulation
//! - Register stack engine (RSE)
//! - Memory management and caching
//! - System call handling
//! - Basic I/O operations
//!
//! ## Usage
//!
//! The emulator can be used as a library or as a standalone binary. Here's a basic
//! example of using the emulator:
//!
//! ```rust,no_run
//! use rust_ia64::{Cpu, Memory};
//!
//! let mut cpu = Cpu::new();
//! let mut memory = Memory::new();
//!
//! // Load program into memory...
//! // Execute program...
//! ```
//!
//! ## Architecture
//!
//! The emulator is organized into several main components:
//!
//! - CPU core (`cpu` module)
//! - Memory management (`memory` module)
//! - Instruction decoder (`decoder` module)
//! - System call interface (`syscall` module)
//!
//! Each component is designed to be modular and testable, allowing for easy
//! maintenance and extension of functionality.

#![deny(missing_docs)]

pub mod cpu;
pub mod decoder;
pub mod memory;

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
