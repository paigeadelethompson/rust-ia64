//! Instruction bundle implementation
//! 
//! This module implements the IA-64 instruction bundle format and decoding.

use crate::EmulatorError;

/// Bundle template types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BundleTemplate {
    /// MII: Memory + I-unit + I-unit
    MII,
    /// MIB: Memory + I-unit + B-unit
    MIB,
    /// MMI: Memory + Memory + I-unit
    MMI,
    /// MMB: Memory + Memory + B-unit
    MMB,
    /// MFI: Memory + F-unit + I-unit
    MFI,
    /// MFB: Memory + F-unit + B-unit
    MFB,
}

impl BundleTemplate {
    /// Create from template bits
    pub fn from_bits(bits: u8) -> Result<Self, EmulatorError> {
        match bits {
            0b00000 => Ok(BundleTemplate::MII),
            0b00001 => Ok(BundleTemplate::MIB),
            0b00010 => Ok(BundleTemplate::MMI),
            0b00011 => Ok(BundleTemplate::MMB),
            0b00100 => Ok(BundleTemplate::MFI),
            0b00101 => Ok(BundleTemplate::MFB),
            _ => Err(EmulatorError::DecodeError("Invalid bundle template".to_string())),
        }
    }
}

/// Instruction slot types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlotType {
    /// M-unit (memory)
    M,
    /// I-unit (integer)
    I,
    /// B-unit (branch)
    B,
    /// F-unit (floating point)
    F,
}

/// Instruction bundle
#[derive(Debug)]
pub struct Bundle {
    /// Raw bundle data
    pub data: u128,
    /// Bundle template
    pub template: BundleTemplate,
    /// Instruction slots
    pub slots: [Option<u64>; 3],
}

impl Bundle {
    /// Create new bundle
    pub fn new(data: u128) -> Result<Self, EmulatorError> {
        // Extract template bits (bits 1-5)
        // Note: We need to mask first, then shift right to get the correct value
        let template_bits = ((data & (0x1Fu128 << 1)) >> 1) as u8;
        let template = BundleTemplate::from_bits(template_bits)?;

        let mut slots = [None; 3];
        
        // Extract instruction slots (41 bits each)
        // Note: IA-64 uses little-endian byte order
        let slot_mask = (1u64 << 41) - 1;
        slots[0] = Some((data >> 6) as u64 & slot_mask);  // Start at bit 6 (after template bits)
        slots[1] = Some((data >> 46) as u64 & slot_mask); // Start at bit 46
        slots[2] = Some((data >> 87) as u64 & slot_mask); // Start at bit 87

        Ok(Self {
            data,
            template,
            slots,
        })
    }

    /// Get slot type
    pub fn get_slot_type(&self, slot: usize) -> Result<SlotType, EmulatorError> {
        if slot >= 3 {
            return Err(EmulatorError::DecodeError("Invalid slot index".to_string()));
        }

        match (self.template, slot) {
            (BundleTemplate::MII, 0) => Ok(SlotType::M),
            (BundleTemplate::MII, _) => Ok(SlotType::I),
            (BundleTemplate::MIB, 0) => Ok(SlotType::M),
            (BundleTemplate::MIB, 1) => Ok(SlotType::I),
            (BundleTemplate::MIB, _) => Ok(SlotType::B),
            (BundleTemplate::MMI, 2) => Ok(SlotType::I),
            (BundleTemplate::MMI, _) => Ok(SlotType::M),
            (BundleTemplate::MMB, 2) => Ok(SlotType::B),
            (BundleTemplate::MMB, _) => Ok(SlotType::M),
            (BundleTemplate::MFI, 0) => Ok(SlotType::M),
            (BundleTemplate::MFI, 1) => Ok(SlotType::F),
            (BundleTemplate::MFI, _) => Ok(SlotType::I),
            (BundleTemplate::MFB, 0) => Ok(SlotType::M),
            (BundleTemplate::MFB, 1) => Ok(SlotType::F),
            (BundleTemplate::MFB, _) => Ok(SlotType::B),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_template() {
        assert_eq!(BundleTemplate::from_bits(0b00000).unwrap(), BundleTemplate::MII);
        assert_eq!(BundleTemplate::from_bits(0b00001).unwrap(), BundleTemplate::MIB);
        assert_eq!(BundleTemplate::from_bits(0b00010).unwrap(), BundleTemplate::MMI);
        assert_eq!(BundleTemplate::from_bits(0b00011).unwrap(), BundleTemplate::MMB);
        assert_eq!(BundleTemplate::from_bits(0b00100).unwrap(), BundleTemplate::MFI);
        assert_eq!(BundleTemplate::from_bits(0b00101).unwrap(), BundleTemplate::MFB);
        assert!(BundleTemplate::from_bits(0b11111).is_err());
    }

    #[test]
    fn test_bundle_creation() {
        // Create MII bundle (template bits 0b00000)
        let data = 0x0123456789ABCDEF0123456789ABCD00_u128;
        let bundle = Bundle::new(data).unwrap();
        assert_eq!(bundle.template, BundleTemplate::MII);
        assert!(bundle.slots[0].is_some());
        assert!(bundle.slots[1].is_some());
        assert!(bundle.slots[2].is_some());
    }

    #[test]
    fn test_slot_types() {
        // Test MII bundle slot types (template bits 0b00000)
        let data = 0x0123456789ABCDEF0123456789ABCD00_u128;
        let bundle = Bundle::new(data).unwrap();
        assert_eq!(bundle.get_slot_type(0).unwrap(), SlotType::M);
        assert_eq!(bundle.get_slot_type(1).unwrap(), SlotType::I);
        assert_eq!(bundle.get_slot_type(2).unwrap(), SlotType::I);

        // Test invalid slot index
        assert!(bundle.get_slot_type(3).is_err());
    }

    #[test]
    fn test_bundle_extraction() {
        // Create a bundle with template bits set to 0b00000 (MII template)
        // and each 41-bit slot filled with 1s
        let mut data = 0u128;
        let slot_mask = (1u128 << 41) - 1;
        
        // Set up each 41-bit slot with all 1s
        // Note: We need to be careful not to overlap with the template bits
        // The template bits are at bits 1-5, so we need to ensure our slots
        // don't affect those bits
        data |= slot_mask << 6;  // First slot (starts after template bits)
        data |= slot_mask << 46; // Second slot
        data |= slot_mask << 87; // Third slot
        
        let bundle = Bundle::new(data).unwrap();

        // Each slot should be 41 bits of all 1s
        let expected = (1u64 << 41) - 1;
        assert_eq!(bundle.slots[0].unwrap(), expected);
        assert_eq!(bundle.slots[1].unwrap(), expected);
        assert_eq!(bundle.slots[2].unwrap(), expected);
        
        // Verify that we got the correct template
        assert_eq!(bundle.template, BundleTemplate::MII);
    }

    #[test]
    fn test_invalid_template() {
        // Create bundle with invalid template bits
        let data = 0x1F_u128 << 1; // Set template bits to 0b11111
        assert!(Bundle::new(data).is_err());
    }
} 