/// IA-64 instruction format definitions

/// A-type instruction format (ALU)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AFormat {
    /// Predicate register (qp) [0:5]
    pub predicate: u8,
    /// Major opcode [6:13]
    pub major_opcode: u8,
    /// x2 field [14:20]
    pub x2: u8,
    /// ve field [21:21]
    pub ve: bool,
    /// x4 field [22:23]
    pub x4: u8,
    /// First source register (r2) [24:30]
    pub r2: u8,
    /// First source register (r3) [31:37]
    pub r3: u8,
    /// Target register (r1) [38:44]
    pub r1: u8,
}

/// I-type instruction format (Integer)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IFormat {
    /// Predicate register (qp) [0:5]
    pub predicate: u8,
    /// Major opcode [6:13]
    pub major_opcode: u8,
    /// x2 field [14:20]
    pub x2: u8,
    /// Immediate8 [21:28]
    pub imm8: u8,
    /// First source register (r2) [29:35]
    pub r2: u8,
    /// Target register (r1) [36:42]
    pub r1: u8,
}

/// M-type instruction format (Memory)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MFormat {
    /// Predicate register (qp) [0:5]
    pub predicate: u8,
    /// Major opcode [6:13]
    pub major_opcode: u8,
    /// x2 field [14:15]
    pub x2: u8,
    /// Hint [16:17]
    pub hint: u8,
    /// x4 field [18:19]
    pub x4: u8,
    /// Base register (r3) [20:26]
    pub r3: u8,
    /// Target register (r1) [27:33]
    pub r1: u8,
    /// Immediate7 [34:40]
    pub imm7: u8,
}

/// F-type instruction format (Floating-point)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FFormat {
    /// Predicate register (qp) [0:5]
    pub predicate: u8,
    /// Major opcode [6:13]
    pub major_opcode: u8,
    /// x2 field [14:18]
    pub x2: u8,
    /// First source register (f2) [19:25]
    pub f2: u8,
    /// Second source register (f3) [26:32]
    pub f3: u8,
    /// Target register (f1) [33:39]
    pub f1: u8,
    /// sf field [40:40]
    pub sf: bool,
}

/// B-type instruction format (Branch)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BFormat {
    /// Predicate register (qp) [0:5]
    pub predicate: u8,
    /// Major opcode [6:13]
    pub major_opcode: u8,
    /// btype field [14:15]
    pub btype: u8,
    /// wh field [16:17]
    pub wh: u8,
    /// d field [18:18]
    pub d: bool,
    /// Immediate20 [19:38]
    pub imm20: u32,
    /// p field [39:40]
    pub p: u8,
}

/// X-type instruction format (Extended)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct XFormat {
    /// Major opcode [0:7]
    pub major_opcode: u8,
    /// x2 field [8:13]
    pub x2: u8,
    /// ve field [14:14]
    pub ve: bool,
    /// Immediate27 [15:41]
    pub imm27: u32,
}

/// L-type instruction format (Long immediate)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LFormat {
    /// Template [0:4]
    pub template: u8,
    /// Immediate41 [5:45]
    pub imm41: u64,
}

impl AFormat {
    /// Decodes a 64-bit instruction into an A-format instruction
    pub fn decode(bits: u64) -> Self {
        Self {
            predicate: (bits & 0x3F) as u8,
            major_opcode: ((bits >> 6) & 0xFF) as u8,
            x2: ((bits >> 14) & 0x7F) as u8,
            ve: ((bits >> 21) & 0x1) != 0,
            x4: ((bits >> 22) & 0x3) as u8,
            r2: ((bits >> 24) & 0x7F) as u8,
            r3: ((bits >> 31) & 0x7F) as u8,
            r1: ((bits >> 38) & 0x7F) as u8,
        }
    }
}

impl IFormat {
    /// Decodes a 64-bit instruction into an I-format instruction
    pub fn decode(bits: u64) -> Self {
        Self {
            predicate: (bits & 0x3F) as u8,
            major_opcode: ((bits >> 6) & 0xFF) as u8,
            x2: ((bits >> 14) & 0x7F) as u8,
            imm8: ((bits >> 21) & 0xFF) as u8,
            r2: ((bits >> 29) & 0x7F) as u8,
            r1: ((bits >> 36) & 0x7F) as u8,
        }
    }
}

impl MFormat {
    /// Decodes a 64-bit instruction into an M-format instruction
    pub fn decode(bits: u64) -> Self {
        Self {
            predicate: (bits & 0x3F) as u8,
            major_opcode: ((bits >> 6) & 0xFF) as u8,
            x2: ((bits >> 14) & 0x3) as u8,
            hint: ((bits >> 16) & 0x3) as u8,
            x4: ((bits >> 18) & 0x3) as u8,
            r3: ((bits >> 20) & 0x7F) as u8,
            r1: ((bits >> 27) & 0x7F) as u8,
            imm7: ((bits >> 34) & 0x7F) as u8,
        }
    }
}

impl FFormat {
    /// Decodes a 64-bit instruction into an F-format instruction
    pub fn decode(bits: u64) -> Self {
        Self {
            predicate: (bits & 0x3F) as u8,
            major_opcode: ((bits >> 6) & 0xFF) as u8,
            x2: ((bits >> 14) & 0x1F) as u8,
            f2: ((bits >> 19) & 0x7F) as u8,
            f3: ((bits >> 26) & 0x7F) as u8,
            f1: ((bits >> 33) & 0x7F) as u8,
            sf: ((bits >> 40) & 0x1) != 0,
        }
    }
}

impl BFormat {
    /// Decodes a 64-bit instruction into a B-format instruction
    pub fn decode(bits: u64) -> Self {
        Self {
            predicate: (bits & 0x3F) as u8,
            major_opcode: ((bits >> 6) & 0xFF) as u8,
            btype: ((bits >> 14) & 0x3) as u8,
            wh: ((bits >> 16) & 0x3) as u8,
            d: ((bits >> 18) & 0x1) != 0,
            imm20: ((bits >> 19) & 0xFFFFF) as u32,
            p: ((bits >> 39) & 0x3) as u8,
        }
    }
}

impl XFormat {
    /// Decodes a 64-bit instruction into an X-format instruction
    pub fn decode(bits: u64) -> Self {
        Self {
            major_opcode: (bits & 0xFF) as u8,
            x2: ((bits >> 8) & 0x3F) as u8,
            ve: ((bits >> 14) & 0x1) != 0,
            imm27: ((bits >> 15) & 0x7FFFFFF) as u32,
        }
    }
}

impl LFormat {
    /// Decodes a 64-bit instruction into an L-format instruction
    pub fn decode(bits: u64) -> Self {
        Self {
            template: (bits & 0x1F) as u8,
            imm41: (bits >> 5) & ((1 << 41) - 1),
        }
    }
}
