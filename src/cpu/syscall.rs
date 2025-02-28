//! System Call Interface
//!
//! This module implements the IA-64 system call interface, handling transitions
//! between user and kernel mode, parameter passing, and system service dispatching.

use super::Cpu;
use crate::EmulatorError;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::hash::Hash;

/// System call numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u64)]
pub enum SyscallNumber {
    /// Exit the current process
    Exit = 1,
    /// Create a new process
    Fork = 2,
    /// Read from a file descriptor
    Read = 3,
    /// Write to a file descriptor
    Write = 4,
    /// Open a file
    Open = 5,
    /// Close a file descriptor
    Close = 6,
    /// Wait for a child process
    WaitPid = 7,
    /// Execute a program
    Execve = 11,
    /// Change current directory
    ChDir = 12,
    /// Get current time
    Time = 13,
    /// Create a directory
    MkDir = 14,
    /// Remove a directory
    RmDir = 15,
    /// Change program break
    Break = 17,
    /// Get process ID
    GetPid = 20,
    /// Mount a filesystem
    Mount = 21,
    /// Unmount a filesystem
    Unmount = 22,
    /// Set user ID
    SetUid = 23,
    /// Get user ID
    GetUid = 24,
    /// Get current time of day
    GetTimeOfDay = 78,
    /// Map memory pages
    Mmap = 90,
    /// Unmap memory pages
    Munmap = 91,
    /// Truncate a file
    Truncate = 92,
    /// Truncate a file by descriptor
    Ftruncate = 93,
    /// Create a socket
    Socket = 97,
    /// Connect to a socket
    Connect = 98,
    /// Accept a connection
    Accept = 99,
    /// Send data through socket
    Send = 100,
    /// Receive data from socket
    Recv = 101,
    /// Shutdown a socket
    Shutdown = 102,
}

impl TryFrom<u64> for SyscallNumber {
    type Error = EmulatorError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Exit),
            2 => Ok(Self::Fork),
            3 => Ok(Self::Read),
            4 => Ok(Self::Write),
            5 => Ok(Self::Open),
            6 => Ok(Self::Close),
            7 => Ok(Self::WaitPid),
            11 => Ok(Self::Execve),
            12 => Ok(Self::ChDir),
            13 => Ok(Self::Time),
            14 => Ok(Self::MkDir),
            15 => Ok(Self::RmDir),
            17 => Ok(Self::Break),
            20 => Ok(Self::GetPid),
            21 => Ok(Self::Mount),
            22 => Ok(Self::Unmount),
            23 => Ok(Self::SetUid),
            24 => Ok(Self::GetUid),
            78 => Ok(Self::GetTimeOfDay),
            90 => Ok(Self::Mmap),
            91 => Ok(Self::Munmap),
            92 => Ok(Self::Truncate),
            93 => Ok(Self::Ftruncate),
            97 => Ok(Self::Socket),
            98 => Ok(Self::Connect),
            99 => Ok(Self::Accept),
            100 => Ok(Self::Send),
            101 => Ok(Self::Recv),
            102 => Ok(Self::Shutdown),
            _ => Err(EmulatorError::ExecutionError(format!(
                "Invalid system call number: {}",
                value
            ))),
        }
    }
}

/// System call parameter registers
pub const SYSCALL_PARAM_REGS: [usize; 8] = [32, 33, 34, 35, 36, 37, 38, 39];

/// System call return registers
pub const SYSCALL_RETURN_REGS: [usize; 2] = [8, 9];

/// System call context
#[derive(Debug, Clone)]
pub struct SyscallContext {
    /// System call number
    pub number: SyscallNumber,
    /// Parameters
    pub params: [u64; 8],
    /// Return values
    pub returns: [u64; 2],
    /// Error code
    pub error: Option<u64>,
}

impl SyscallContext {
    /// Create new system call context
    pub fn new(number: SyscallNumber) -> Self {
        Self {
            number,
            params: [0; 8],
            returns: [0; 2],
            error: None,
        }
    }

    /// Set parameter value
    pub fn set_param(&mut self, index: usize, value: u64) {
        if index < self.params.len() {
            self.params[index] = value;
        }
    }

    /// Get parameter value
    pub fn get_param(&self, index: usize) -> Option<u64> {
        if index < self.params.len() {
            Some(self.params[index])
        } else {
            None
        }
    }

    /// Set return value
    pub fn set_return(&mut self, index: usize, value: u64) {
        if index < self.returns.len() {
            self.returns[index] = value;
        }
    }

    /// Set error code
    pub fn set_error(&mut self, error: u64) {
        self.error = Some(error);
    }
}

/// Type alias for syscall handler function
type SyscallHandler =
    Box<dyn Fn(&mut Cpu, &mut SyscallContext) -> Result<(), EmulatorError> + Send + Sync>;

/// Syscall handler registry
#[derive(Default)]
pub struct SyscallRegistry {
    /// Registered syscall handlers
    handlers: HashMap<SyscallNumber, SyscallHandler>,
}

impl std::fmt::Debug for SyscallRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyscallRegistry")
            .field("handlers", &format!("<{} handlers>", self.handlers.len()))
            .finish()
    }
}

impl SyscallRegistry {
    /// Create new syscall registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a syscall handler
    pub fn register(&mut self, number: SyscallNumber, handler: SyscallHandler) {
        self.handlers.insert(number, handler);
    }

    /// Get a syscall handler
    pub fn get(&self, number: SyscallNumber) -> Option<&SyscallHandler> {
        self.handlers.get(&number)
    }
}

/// System call manager
pub struct SyscallManager {
    handlers: HashMap<SyscallNumber, SyscallHandler>,
    pub(crate) current: Option<SyscallContext>,
}

impl fmt::Debug for SyscallManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyscallManager")
            .field("current", &self.current)
            .field("handlers", &format!("<{} handlers>", self.handlers.len()))
            .finish()
    }
}

impl SyscallManager {
    /// Create new system call manager
    pub fn new() -> Self {
        let mut manager = Self {
            handlers: HashMap::new(),
            current: None,
        };
        manager.register_default_handlers();
        manager
    }

    fn register_default_handlers(&mut self) {
        self.register_handler(SyscallNumber::Exit, Self::handle_exit);
        self.register_handler(SyscallNumber::Write, Self::handle_write);
        self.register_handler(SyscallNumber::Read, Self::handle_read);
        self.register_handler(SyscallNumber::GetPid, Self::handle_getpid);
    }

    /// Register a handler for a system call
    ///
    /// # Arguments
    /// * `number` - The system call number to register the handler for
    /// * `handler` - The handler function to call when the system call is executed
    pub fn register_handler<F>(&mut self, number: SyscallNumber, handler: F)
    where
        F: Fn(&mut Cpu, &mut SyscallContext) -> Result<(), EmulatorError> + Send + Sync + 'static,
    {
        self.handlers.insert(number, Box::new(handler));
    }

    /// Get the handler for a system call
    ///
    /// # Arguments
    /// * `number` - The system call number to get the handler for
    ///
    /// # Returns
    /// * `Some(handler)` if a handler is registered for the system call
    /// * `None` if no handler is registered for the system call
    pub fn get_handler(
        &self,
        number: SyscallNumber,
    ) -> Option<impl Fn(&mut Cpu, &mut SyscallContext) -> Result<(), EmulatorError> + '_> {
        self.handlers.get(&number).map(|h| h.as_ref())
    }

    /// Execute system call
    pub fn execute_syscall(
        &mut self,
        cpu: &mut Cpu,
        context: &mut SyscallContext,
    ) -> Result<(), EmulatorError> {
        let handler = self
            .handlers
            .get(&context.number)
            .ok_or(EmulatorError::InvalidSyscall)?;
        let handler = handler.as_ref();
        handler(cpu, context)
    }

    /// Handle exit system call
    fn handle_exit(_cpu: &mut Cpu, context: &mut SyscallContext) -> Result<(), EmulatorError> {
        // For now, just set return value to 0 (success)
        context.returns[0] = 0;
        Ok(())
    }

    /// Handle write system call
    fn handle_write(_cpu: &mut Cpu, ctx: &mut SyscallContext) -> Result<(), EmulatorError> {
        let _fd = ctx.params[0];
        let _buf = ctx.params[1];
        let count = ctx.params[2];

        // For now, just pretend we wrote all the bytes
        ctx.returns[0] = count;
        ctx.error = None;
        Ok(())
    }

    /// Handle read system call
    fn handle_read(_cpu: &mut Cpu, context: &mut SyscallContext) -> Result<(), EmulatorError> {
        // For now, just set return value to number of bytes read
        context.returns[0] = 0;
        Ok(())
    }

    /// Handle getpid system call
    fn handle_getpid(_cpu: &mut Cpu, context: &mut SyscallContext) -> Result<(), EmulatorError> {
        // For now, just return a dummy PID
        context.returns[0] = 1;
        Ok(())
    }

    /// Initialize default handlers
    pub fn init_default_handlers(&mut self) {
        self.register_handler(SyscallNumber::Exit, Self::handle_exit);
        self.register_handler(SyscallNumber::Write, Self::handle_write);
        self.register_handler(SyscallNumber::Read, Self::handle_read);
        self.register_handler(SyscallNumber::GetPid, Self::handle_getpid);
    }

    /// Begins a system call by creating a new context and loading parameters from registers
    ///
    /// # Arguments
    /// * `cpu` - Reference to the CPU state to read parameters from
    /// * `syscall_num` - The system call number
    ///
    /// # Returns
    /// * `Ok(())` if the system call was started successfully
    /// * `Err(EmulatorError::InvalidSyscall)` if the syscall number is invalid
    pub fn begin_syscall(&mut self, cpu: &Cpu, syscall_num: u64) -> Result<(), EmulatorError> {
        // Convert syscall number to enum
        let syscall =
            SyscallNumber::try_from(syscall_num).map_err(|_| EmulatorError::InvalidSyscall)?;

        // Create context
        let mut context = SyscallContext::new(syscall);

        // Get parameters from registers
        for (i, reg) in SYSCALL_PARAM_REGS.iter().enumerate() {
            context.params[i] = cpu.gr[*reg];
        }

        self.current = Some(context);
        Ok(())
    }

    /// Ends a system call by setting return values in registers
    ///
    /// # Arguments
    /// * `cpu` - Mutable reference to the CPU state to write return values to
    ///
    /// # Returns
    /// * `Ok(())` if the system call was ended successfully
    /// * `Err(EmulatorError::NoSyscallContext)` if there is no active system call
    pub fn end_syscall(&mut self, cpu: &mut Cpu) -> Result<(), EmulatorError> {
        let context = self.current.take().ok_or(EmulatorError::NoSyscallContext)?;

        // Set return value
        cpu.gr[SYSCALL_RETURN_REGS[0]] = context.returns[0];

        // Set error code if any
        if let Some(err) = context.error {
            cpu.gr[SYSCALL_RETURN_REGS[1]] = err;
        }

        Ok(())
    }
}

impl Default for SyscallManager {
    fn default() -> Self {
        let mut manager = Self::new();
        manager.init_default_handlers();
        manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_context() {
        let mut context = SyscallContext::new(SyscallNumber::Write);

        // Set parameters
        context.set_param(0, 1); // fd
        context.set_param(1, 0x1000); // buffer
        context.set_param(2, 100); // count

        // Check parameters
        assert_eq!(context.get_param(0), Some(1));
        assert_eq!(context.get_param(1), Some(0x1000));
        assert_eq!(context.get_param(2), Some(100));

        // Set return value
        context.set_return(0, 100);

        // Set error
        context.set_error(0);
    }

    #[test]
    fn test_syscall_manager() {
        let mut cpu = Cpu::new();
        let mut manager = SyscallManager::new();

        // Register handlers
        manager.init_default_handlers();

        // Begin syscall
        assert!(manager
            .begin_syscall(&cpu, SyscallNumber::Write as u64)
            .is_ok());

        // Execute syscall
        assert!(manager
            .execute_syscall(&mut cpu, &mut SyscallContext::new(SyscallNumber::Write))
            .is_ok());

        // End syscall
        assert!(manager.end_syscall(&mut cpu).is_ok());
    }
}
