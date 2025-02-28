use crate::cpu::{PSRFlags, PSR};
use crate::EmulatorError;

/// Number of control registers
pub const NUM_CR: usize = 128;

/// Control register indices
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CRIndex {
    /// Processor Status Register
    PSR = 0,
    /// Interval Timer Match Register
    ITM = 1,
    /// Interval Timer Vector
    ITV = 2,
    /// Page Table Address
    PTA = 8,
    /// Interruption Status Register
    ISR = 16,
    /// Interruption Processor Status Register
    IPSR = 17,
    /// Interruption Faulting Address
    IFA = 18,
    /// Interruption TLB Insertion Register
    ITIR = 19,
    /// Interruption Instruction Previous Address
    IIPA = 20,
    /// Interruption Function State
    IFS = 21,
    /// Interruption Immediate
    IIM = 22,
    /// Interruption Hash Address
    IHA = 23,
    /// Interruption Vector Address
    IVA = 24,
    /// Page Table Status Register
    PTS = 25,
    /// TLB Purge History
    TPHA = 26,
    /// External Interrupt Vector Register
    XIVA = 27,
    /// Local ID
    LID = 64,
    /// Task Priority Register
    TPR = 65,
    /// External Interrupt Request Register 0
    IRR0 = 66,
    /// External Interrupt Request Register 1
    IRR1 = 67,
    /// External Interrupt Request Register 2
    IRR2 = 68,
    /// External Interrupt Request Register 3
    IRR3 = 69,
    /// Interval Timer Configuration Register
    ITC = 72,
    /// Performance Monitor Vector
    PMV = 73,
    /// Corrected Machine Check Vector
    CMCV = 74,
    /// Local Redirection Register 0
    LRR0 = 80,
    /// Local Redirection Register 1
    LRR1 = 81,
}

impl CRIndex {
    /// Try to create from raw bits
    pub fn from_bits(bits: u8) -> Option<Self> {
        match bits {
            // SAFETY: These transmutes are safe because:
            // 1. The bit patterns are validated by the match arms
            // 2. The enum variants are repr(u64) and can hold these values
            // 3. The ranges are non-overlapping and exhaustive
            0..=2 => Some(unsafe { std::mem::transmute::<u8, CRIndex>(bits) }),
            8 => Some(Self::PTA),
            16..=27 => Some(unsafe { std::mem::transmute::<u8, CRIndex>(bits) }),
            64..=69 => Some(unsafe { std::mem::transmute::<u8, CRIndex>(bits) }),
            72..=74 => Some(unsafe { std::mem::transmute::<u8, CRIndex>(bits) }),
            80..=81 => Some(unsafe { std::mem::transmute::<u8, CRIndex>(bits) }),
            _ => None,
        }
    }
}

/// Control register file
#[derive(Debug)]
pub struct CRFile {
    /// Register values
    registers: [u64; NUM_CR],
}

impl Default for CRFile {
    fn default() -> Self {
        Self::new()
    }
}

impl CRFile {
    /// Create new register file
    pub fn new() -> Self {
        Self {
            registers: [0; NUM_CR],
        }
    }

    /// Read register value
    pub fn read(&self, index: CRIndex) -> u64 {
        self.registers[index as usize]
    }

    /// Write register value
    pub fn write(&mut self, index: CRIndex, value: u64) -> Result<(), EmulatorError> {
        // Some registers have reserved fields that must be preserved
        match index {
            CRIndex::PSR => {
                // Preserve reserved fields in PSR
                let mask = 0x0000_FFFF_FFFF_FFFF;
                let preserved = self.registers[index as usize] & !mask;
                self.registers[index as usize] = preserved | (value & mask);
            }
            CRIndex::TPR => {
                // TPR only uses low 16 bits
                let mask = 0x0000_0000_0000_FFFF;
                let preserved = self.registers[index as usize] & !mask;
                self.registers[index as usize] = preserved | (value & mask);
            }
            _ => {
                self.registers[index as usize] = value;
            }
        }
        Ok(())
    }

    /// Get processor status register
    pub fn get_psr(&self) -> u64 {
        self.read(CRIndex::PSR)
    }

    /// Get interruption status register
    pub fn get_isr(&self) -> u64 {
        self.read(CRIndex::ISR)
    }

    /// Get task priority register
    pub fn get_tpr(&self) -> u64 {
        self.read(CRIndex::TPR)
    }

    /// Get external interrupt request registers
    pub fn get_irr(&self) -> [u64; 4] {
        [
            self.read(CRIndex::IRR0),
            self.read(CRIndex::IRR1),
            self.read(CRIndex::IRR2),
            self.read(CRIndex::IRR3),
        ]
    }

    /// Returns the raw bits of the control register
    pub fn bits(&self) -> u64 {
        self.registers[0]
    }

    /// Checks if the control register contains the specified flags
    pub fn contains(&self, flags: PSRFlags) -> bool {
        self.bits() & flags.bits() == flags.bits()
    }

    /// Updates the control register using the provided function
    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(u64) -> u64,
    {
        self.registers[0] = f(self.registers[0]);
    }

    /// Creates a new CRFile from raw bits, truncating any excess bits
    pub fn from_bits_truncate(bits: u64) -> Self {
        let mut cr = Self::new();
        cr.registers[0] = bits;
        cr
    }

    /// Sets or clears the specified flags in the control register
    pub fn set(&mut self, flags: PSRFlags, value: bool) {
        if value {
            self.registers[0] |= flags.bits();
        } else {
            self.registers[0] &= !flags.bits();
        }
    }
}

impl From<PSR> for CRFile {
    fn from(psr: PSR) -> Self {
        Self::from_bits_truncate(psr.bits())
    }
}
