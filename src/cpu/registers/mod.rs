/// Application Register module
pub mod ar;
/// Control Register module
pub mod cr;
/// Region Register module
pub mod rr;
/// Protection Key Register module
pub mod pkr;
/// Data Breakpoint Register module
pub mod dbr;
/// Data Debug Register module
pub mod ddr;

pub use ar::{ARFile, AR};
pub use cr::{CRFile, CRIndex};
pub use rr::{RRFile, RegionFields};
pub use pkr::{PKRFile, KeyFields};
pub use dbr::{DBRFile, BreakFields, BreakAccessType};
pub use ddr::{DDRFile, DataFields};

/// Register state for IA-64 CPU
#[derive(Debug)]
pub struct RegisterState {
    /// Application registers
    pub ar: ARFile,
    /// Control registers
    pub cr: CRFile,
    /// Region registers
    pub rr: RRFile,
    /// Protection key registers
    pub pkr: PKRFile,
    /// Debug break registers
    pub dbr: DBRFile,
    /// Debug data registers
    pub ddr: DDRFile,
}

impl RegisterState {
    /// Create new register state
    pub fn new() -> Self {
        Self {
            ar: ARFile::new(),
            cr: CRFile::new(),
            rr: RRFile::new(),
            pkr: PKRFile::new(),
            dbr: DBRFile::new(),
            ddr: DDRFile::new(),
        }
    }
} 