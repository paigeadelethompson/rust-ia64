use crate::EmulatorError;

/// Number of protection key registers
pub const NUM_PKR: usize = 16;

/// Protection key fields
#[derive(Debug, Clone, Copy)]
pub struct KeyFields {
    /// Key value
    pub key: u32,
    /// Valid bit
    pub v: bool,
    /// Write disable
    pub wd: bool,
    /// Read disable
    pub rd: bool,
    /// Execute disable
    pub xd: bool,
}

impl KeyFields {
    /// Create from raw bits
    pub fn from_bits(bits: u64) -> Self {
        Self {
            key: (bits & 0xFFFF_FFFF) as u32,
            v: ((bits >> 32) & 1) != 0,
            wd: ((bits >> 33) & 1) != 0,
            rd: ((bits >> 34) & 1) != 0,
            xd: ((bits >> 35) & 1) != 0,
        }
    }

    /// Convert to raw bits
    pub fn to_bits(&self) -> u64 {
        (self.key as u64)
            | ((self.v as u64) << 32)
            | ((self.wd as u64) << 33)
            | ((self.rd as u64) << 34)
            | ((self.xd as u64) << 35)
    }

    /// Check if key allows read access
    pub fn can_read(&self) -> bool {
        self.v && !self.rd
    }

    /// Check if key allows write access
    pub fn can_write(&self) -> bool {
        self.v && !self.wd
    }

    /// Check if key allows execute access
    pub fn can_execute(&self) -> bool {
        self.v && !self.xd
    }
}

/// Protection key register file
#[derive(Debug)]
pub struct PKRFile {
    /// Register values
    regs: [u64; NUM_PKR],
}

impl Default for PKRFile {
    fn default() -> Self {
        Self::new()
    }
}

impl PKRFile {
    /// Create new register file
    pub fn new() -> Self {
        Self { regs: [0; NUM_PKR] }
    }

    /// Read register value
    pub fn read(&self, index: usize) -> Result<KeyFields, EmulatorError> {
        if index >= NUM_PKR {
            return Err(EmulatorError::RegisterError(format!(
                "Invalid protection key register index: {}",
                index
            )));
        }
        Ok(KeyFields::from_bits(self.regs[index]))
    }

    /// Write register value
    pub fn write(&mut self, index: usize, fields: KeyFields) -> Result<(), EmulatorError> {
        if index >= NUM_PKR {
            return Err(EmulatorError::RegisterError(format!(
                "Invalid protection key register index: {}",
                index
            )));
        }
        self.regs[index] = fields.to_bits();
        Ok(())
    }

    /// Find key by value
    pub fn find_key(&self, key: u32) -> Option<usize> {
        for i in 0..NUM_PKR {
            if let Ok(fields) = self.read(i) {
                if fields.v && fields.key == key {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Check if a key allows read access
    pub fn check_read(&self, key: u32) -> bool {
        self.find_key(key)
            .map(|i| self.read(i).map(|f| f.can_read()).unwrap_or(false))
            .unwrap_or(false)
    }

    /// Check if a key allows write access
    pub fn check_write(&self, key: u32) -> bool {
        self.find_key(key)
            .map(|i| self.read(i).map(|f| f.can_write()).unwrap_or(false))
            .unwrap_or(false)
    }

    /// Check if a key allows execute access
    pub fn check_execute(&self, key: u32) -> bool {
        self.find_key(key)
            .map(|i| self.read(i).map(|f| f.can_execute()).unwrap_or(false))
            .unwrap_or(false)
    }

    /// Invalidate a key
    pub fn invalidate(&mut self, key: u32) -> Result<(), EmulatorError> {
        if let Some(index) = self.find_key(key) {
            let mut fields = self.read(index)?;
            fields.v = false;
            self.write(index, fields)?;
        }
        Ok(())
    }

    /// Add a new key
    pub fn add_key(&mut self, key: u32, wd: bool, rd: bool, xd: bool) -> Result<(), EmulatorError> {
        // First try to find an invalid entry
        let mut target_index = None;
        for i in 0..NUM_PKR {
            if let Ok(fields) = self.read(i) {
                if !fields.v {
                    target_index = Some(i);
                    break;
                }
            }
        }

        let index = target_index.ok_or_else(|| {
            EmulatorError::RegisterError("No free protection key registers".to_string())
        })?;

        self.write(
            index,
            KeyFields {
                key,
                v: true,
                wd,
                rd,
                xd,
            },
        )
    }
}
