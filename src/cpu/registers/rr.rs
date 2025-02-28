use crate::EmulatorError;

/// Number of region registers
pub const NUM_RR: usize = 8;

/// Region register fields
#[derive(Debug, Clone, Copy)]
pub struct RegionFields {
    /// Virtual Region ID
    pub rid: u64,
    /// Page size
    pub ps: u8,
    /// Virtual Region Enable
    pub ve: bool,
}

impl RegionFields {
    /// Create from raw bits
    pub fn from_bits(bits: u64) -> Self {
        Self {
            rid: bits & 0xFFFF_FFFF_FFFF_FFFF,
            ps: ((bits >> 56) & 0xFF) as u8,
            ve: (bits >> 63) != 0,
        }
    }

    /// Convert to raw bits
    pub fn to_bits(&self) -> u64 {
        self.rid |
        ((self.ps as u64) << 56) |
        ((self.ve as u64) << 63)
    }
}

/// Region register file
#[derive(Debug)]
pub struct RRFile {
    /// Register values
    regs: [u64; NUM_RR],
}

impl RRFile {
    /// Create new register file
    pub fn new() -> Self {
        Self {
            regs: [0; NUM_RR],
        }
    }

    /// Read register value
    pub fn read(&self, index: usize) -> Result<RegionFields, EmulatorError> {
        if index >= NUM_RR {
            return Err(EmulatorError::RegisterError(
                format!("Invalid region register index: {}", index)
            ));
        }
        Ok(RegionFields::from_bits(self.regs[index]))
    }

    /// Write register value
    pub fn write(&mut self, index: usize, fields: RegionFields) -> Result<(), EmulatorError> {
        if index >= NUM_RR {
            return Err(EmulatorError::RegisterError(
                format!("Invalid region register index: {}", index)
            ));
        }
        self.regs[index] = fields.to_bits();
        Ok(())
    }

    /// Get virtual region ID for a region
    pub fn get_rid(&self, index: usize) -> Result<u64, EmulatorError> {
        Ok(self.read(index)?.rid)
    }

    /// Get page size for a region
    pub fn get_ps(&self, index: usize) -> Result<u8, EmulatorError> {
        Ok(self.read(index)?.ps)
    }

    /// Check if region is enabled
    pub fn is_enabled(&self, index: usize) -> Result<bool, EmulatorError> {
        Ok(self.read(index)?.ve)
    }

    /// Enable/disable a region
    pub fn set_enabled(&mut self, index: usize, enabled: bool) -> Result<(), EmulatorError> {
        let mut fields = self.read(index)?;
        fields.ve = enabled;
        self.write(index, fields)
    }

    /// Set page size for a region
    pub fn set_ps(&mut self, index: usize, ps: u8) -> Result<(), EmulatorError> {
        let mut fields = self.read(index)?;
        fields.ps = ps;
        self.write(index, fields)
    }

    /// Set virtual region ID for a region
    pub fn set_rid(&mut self, index: usize, rid: u64) -> Result<(), EmulatorError> {
        let mut fields = self.read(index)?;
        fields.rid = rid;
        self.write(index, fields)
    }
} 