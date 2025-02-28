use crate::EmulatorError;

/// Number of debug data registers
pub const NUM_DDR: usize = 8;

/// Debug data register fields
#[derive(Debug, Clone, Copy)]
pub struct DataFields {
    /// Data value
    pub data: u64,
    /// Data mask
    pub mask: u64,
}

impl DataFields {
    /// Create from raw bits
    pub fn from_bits(bits: u64) -> Self {
        Self {
            data: bits,
            mask: (bits >> 56) & 0xFF,
        }
    }

    /// Convert to raw bits
    pub fn to_bits(&self) -> u64 {
        self.data | (self.mask << 56)
    }

    /// Check if value matches data pattern
    pub fn matches(&self, value: u64) -> bool {
        let mask = !(self.mask << 56);
        (value & mask) == (self.data & mask)
    }
}

/// Debug data register file
#[derive(Debug)]
pub struct DDRFile {
    /// Register values
    regs: [u64; NUM_DDR],
}

impl Default for DDRFile {
    fn default() -> Self {
        Self::new()
    }
}

impl DDRFile {
    /// Create new register file
    pub fn new() -> Self {
        Self { regs: [0; NUM_DDR] }
    }

    /// Read register value
    pub fn read(&self, index: usize) -> Result<DataFields, EmulatorError> {
        if index >= NUM_DDR {
            return Err(EmulatorError::RegisterError(format!(
                "Invalid debug data register index: {}",
                index
            )));
        }
        Ok(DataFields::from_bits(self.regs[index]))
    }

    /// Write register value
    pub fn write(&mut self, index: usize, fields: DataFields) -> Result<(), EmulatorError> {
        if index >= NUM_DDR {
            return Err(EmulatorError::RegisterError(format!(
                "Invalid debug data register index: {}",
                index
            )));
        }
        self.regs[index] = fields.to_bits();
        Ok(())
    }

    /// Check if any data register matches value
    pub fn check_match(&self, value: u64) -> bool {
        for i in 0..NUM_DDR {
            if let Ok(fields) = self.read(i) {
                if fields.matches(value) {
                    return true;
                }
            }
        }
        false
    }

    /// Set a new data match value
    pub fn set_match(&mut self, data: u64, mask: u64) -> Result<(), EmulatorError> {
        // Find first unused register
        let mut target_index = None;
        for i in 0..NUM_DDR {
            if let Ok(fields) = self.read(i) {
                if fields.mask == 0 {
                    target_index = Some(i);
                    break;
                }
            }
        }

        let index = target_index.ok_or_else(|| {
            EmulatorError::RegisterError("No free debug data registers".to_string())
        })?;

        self.write(index, DataFields { data, mask })
    }

    /// Clear a data match
    pub fn clear_match(&mut self, index: usize) -> Result<(), EmulatorError> {
        if index >= NUM_DDR {
            return Err(EmulatorError::RegisterError(format!(
                "Invalid debug data register index: {}",
                index
            )));
        }

        let mut fields = self.read(index)?;
        fields.mask = 0;
        self.write(index, fields)
    }
}
