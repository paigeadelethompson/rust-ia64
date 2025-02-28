use crate::EmulatorError;

/// Number of application registers
pub const NUM_AR: usize = 128;

/// Application Register numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AR {
    /// Kernel Register 1
    KR1 = 1,
    /// Kernel Register 2
    KR2 = 2,
    /// Kernel Register 3
    KR3 = 3,
    /// Kernel Register 4
    KR4 = 4,
    /// Kernel Register 5
    KR5 = 5,
    /// Kernel Register 6
    KR6 = 6,
    /// Kernel Register 7
    KR7 = 7,

    /// RSE Configuration Register
    RSC = 16,
    /// Backing Store Pointer
    BSP = 17,
    /// Backing Store Pointer for Memory Stores
    BSPSTORE = 18,
    /// RSE NaT Collection Register
    RNAT = 19,
    /// Compare and Exchange Compare Value Register
    CCV = 32,
    /// User NaT Collection Register
    UNAT = 36,
    /// Floating-point Status Register
    FPSR = 40,
    /// Interval Time Counter Register
    ITC = 44,

    /// Performance Data Register 1
    PFD1 = 65,
    /// Performance Data Register 2
    PFD2 = 66,
    /// Performance Data Register 3
    PFD3 = 67,
    /// Performance Data Register 4
    PFD4 = 68,
    /// Performance Data Register 5
    PFD5 = 69,
    /// Performance Data Register 6
    PFD6 = 70,
    /// Performance Data Register 7
    PFD7 = 71,
    /// Performance Data Register 8
    PFD8 = 72,
    /// Performance Data Register 9
    PFD9 = 73,
    /// Performance Data Register 10
    PFD10 = 74,
    /// Performance Data Register 11
    PFD11 = 75,
    /// Performance Data Register 12
    PFD12 = 76,
    /// Performance Data Register 13
    PFD13 = 77,
    /// Performance Data Register 14
    PFD14 = 78,
    /// Performance Data Register 15
    PFD15 = 79,
    /// Performance Data Register 16
    PFD16 = 80,
    /// Performance Data Register 17
    PFD17 = 81,

    /// Performance Counter Register 1
    PFC1 = 89,
    /// Performance Counter Register 2
    PFC2 = 90,
    /// Performance Counter Register 3
    PFC3 = 91,
    /// Performance Counter Register 4
    PFC4 = 92,
    /// Performance Counter Register 5
    PFC5 = 93,
    /// Performance Counter Register 6
    PFC6 = 94,
    /// Performance Counter Register 7
    PFC7 = 95,

    /// CPUID Register 1
    CPUID1 = 97,
    /// CPUID Register 2
    CPUID2 = 98,
    /// CPUID Register 3
    CPUID3 = 99,
    /// CPUID Register 4
    CPUID4 = 100,
}

impl AR {
    /// Try to create from raw bits
    pub fn from_bits(bits: u8) -> Option<Self> {
        match bits {
            0..=7 => Some(unsafe { std::mem::transmute(bits) }),
            16..=19 => Some(unsafe { std::mem::transmute(bits) }),
            32 => Some(Self::CCV),
            36 => Some(Self::UNAT),
            40 => Some(Self::FPSR),
            44 => Some(Self::ITC),
            64..=81 => Some(unsafe { std::mem::transmute(bits) }),
            88..=95 => Some(unsafe { std::mem::transmute(bits) }),
            96..=100 => Some(unsafe { std::mem::transmute(bits) }),
            _ => None,
        }
    }
}

/// Application register file
#[derive(Debug)]
pub struct ARFile {
    /// Register values
    regs: [u64; NUM_AR],
}

impl Default for ARFile {
    fn default() -> Self {
        Self::new()
    }
}

impl ARFile {
    /// Create new register file
    pub fn new() -> Self {
        Self { regs: [0; NUM_AR] }
    }

    /// Read register value
    pub fn read(&self, index: AR) -> Result<u64, EmulatorError> {
        match index {
            AR::CPUID1 | AR::CPUID2 | AR::CPUID3 | AR::CPUID4 => Err(EmulatorError::RegisterError(
                "CPUID registers are read-only".to_string(),
            )),
            _ => Ok(self.regs[index as usize]),
        }
    }

    /// Write register value
    pub fn write(&mut self, index: AR, value: u64) -> Result<(), EmulatorError> {
        match index {
            AR::CPUID1 | AR::CPUID2 | AR::CPUID3 | AR::CPUID4 => Err(EmulatorError::RegisterError(
                "CPUID registers are read-only".to_string(),
            )),
            _ => {
                self.regs[index as usize] = value;
                Ok(())
            }
        }
    }

    /// Get RSE configuration
    pub fn get_rse_config(&self) -> u64 {
        self.read(AR::RSC).unwrap()
    }

    /// Get backing store pointer
    pub fn get_bsp(&self) -> u64 {
        self.read(AR::BSP).unwrap()
    }

    /// Get UNAT collection register
    pub fn get_unat(&self) -> u64 {
        self.read(AR::UNAT).unwrap()
    }

    /// Get floating-point status register
    pub fn get_fpsr(&self) -> u64 {
        self.read(AR::FPSR).unwrap()
    }
}
