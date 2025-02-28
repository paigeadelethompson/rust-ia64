use crate::EmulatorError;

/// Number of debug break registers
pub const NUM_DBR: usize = 8;

/// Debug break register fields
#[derive(Debug, Clone, Copy)]
pub struct BreakFields {
    /// Break address
    pub addr: u64,
    /// Mask
    pub mask: u64,
    /// Read break enable
    pub r: bool,
    /// Write break enable
    pub w: bool,
    /// Execute break enable
    pub x: bool,
    /// Privilege level mask
    pub plm: u8,
    /// Ignore mask
    pub ig: bool,
}

impl BreakFields {
    /// Create from raw bits
    pub fn from_bits(bits: u64) -> Self {
        Self {
            addr: bits & 0xFFFF_FFFF_FFFF_F000, // 4K aligned
            mask: (bits >> 48) & 0xFF,
            r: ((bits >> 56) & 1) != 0,
            w: ((bits >> 57) & 1) != 0,
            x: ((bits >> 58) & 1) != 0,
            plm: ((bits >> 59) & 0xF) as u8,
            ig: ((bits >> 63) & 1) != 0,
        }
    }

    /// Convert to raw bits
    pub fn to_bits(&self) -> u64 {
        (self.addr & 0xFFFF_FFFF_FFFF_F000) |
        ((self.mask as u64) << 48) |
        ((self.r as u64) << 56) |
        ((self.w as u64) << 57) |
        ((self.x as u64) << 58) |
        ((self.plm as u64) << 59) |
        ((self.ig as u64) << 63)
    }

    /// Check if address matches break condition
    pub fn matches(&self, addr: u64, pl: u8, access_type: BreakAccessType) -> bool {
        // Check privilege level
        if (self.plm & (1 << pl)) == 0 {
            return false;
        }

        // Check access type
        match access_type {
            BreakAccessType::Read => if !self.r { return false; },
            BreakAccessType::Write => if !self.w { return false; },
            BreakAccessType::Execute => if !self.x { return false; },
        }

        // Check address match with mask
        let mask = !((self.mask as u64) << 48);
        (addr & mask) == (self.addr & mask)
    }
}

/// Type of access for break matching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakAccessType {
    /// Memory read
    Read,
    /// Memory write
    Write,
    /// Instruction execute
    Execute,
}

/// Debug break register file
#[derive(Debug)]
pub struct DBRFile {
    /// Register values
    regs: [u64; NUM_DBR],
}

impl DBRFile {
    /// Create new register file
    pub fn new() -> Self {
        Self {
            regs: [0; NUM_DBR],
        }
    }

    /// Read register value
    pub fn read(&self, index: usize) -> Result<BreakFields, EmulatorError> {
        if index >= NUM_DBR {
            return Err(EmulatorError::RegisterError(
                format!("Invalid debug break register index: {}", index)
            ));
        }
        Ok(BreakFields::from_bits(self.regs[index]))
    }

    /// Write register value
    pub fn write(&mut self, index: usize, fields: BreakFields) -> Result<(), EmulatorError> {
        if index >= NUM_DBR {
            return Err(EmulatorError::RegisterError(
                format!("Invalid debug break register index: {}", index)
            ));
        }
        self.regs[index] = fields.to_bits();
        Ok(())
    }

    /// Check if any breakpoint matches
    pub fn check_break(&self, addr: u64, pl: u8, access_type: BreakAccessType) -> bool {
        for i in 0..NUM_DBR {
            if let Ok(fields) = self.read(i) {
                if fields.matches(addr, pl, access_type) {
                    return true;
                }
            }
        }
        false
    }

    /// Set a new breakpoint
    pub fn set_break(&mut self, addr: u64, mask: u64, r: bool, w: bool, x: bool, plm: u8) -> Result<(), EmulatorError> {
        // Find first unused register
        let mut target_index = None;
        for i in 0..NUM_DBR {
            if let Ok(fields) = self.read(i) {
                if !fields.r && !fields.w && !fields.x {
                    target_index = Some(i);
                    break;
                }
            }
        }

        let index = target_index.ok_or_else(|| EmulatorError::RegisterError(
            "No free debug break registers".to_string()
        ))?;

        self.write(index, BreakFields {
            addr,
            mask,
            r,
            w,
            x,
            plm,
            ig: false,
        })
    }

    /// Clear a breakpoint
    pub fn clear_break(&mut self, index: usize) -> Result<(), EmulatorError> {
        if index >= NUM_DBR {
            return Err(EmulatorError::RegisterError(
                format!("Invalid debug break register index: {}", index)
            ));
        }

        let mut fields = self.read(index)?;
        fields.r = false;
        fields.w = false;
        fields.x = false;
        self.write(index, fields)
    }
} 