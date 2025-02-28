/// Application Register module
pub mod ar;
/// Control Register module
pub mod cr;
/// Data Breakpoint Register module
pub mod dbr;
/// Data Debug Register module
pub mod ddr;
/// Protection Key Register module
pub mod pkr;
/// Region Register module
pub mod rr;

pub use ar::{ARFile, AR};
pub use cr::{CRFile, CRIndex};
pub use dbr::{BreakAccessType, BreakFields, DBRFile};
pub use ddr::{DDRFile, DataFields};
pub use pkr::{KeyFields, PKRFile};
pub use rr::{RRFile, RegionFields};

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

impl Default for RegisterState {
    fn default() -> Self {
        Self::new()
    }
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
