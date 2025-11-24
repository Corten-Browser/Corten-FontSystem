//! TrueType Hinting Support
//!
//! This module provides parsing for TrueType hinting tables:
//! - cvt (Control Value Table): Global hint values
//! - fpgm (Font Program): Instructions executed once when font is loaded
//! - prep (Control Value Program): Instructions executed when font size changes

use crate::ParseError;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

/// Control Value Table (cvt)
///
/// Contains a list of values that can be referenced by hinting instructions.
/// These values are typically used for stem widths, x-height, cap-height, etc.
#[derive(Debug, Clone, PartialEq)]
pub struct CvtTable {
    /// Control values in FUnits
    pub values: Vec<i16>,
}

impl CvtTable {
    /// Parse cvt table from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() % 2 != 0 {
            return Err(ParseError::CorruptedData(
                "cvt table size must be even".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let num_values = data.len() / 2;
        let mut values = Vec::with_capacity(num_values);

        for _ in 0..num_values {
            values.push(cursor.read_i16::<BigEndian>()?);
        }

        Ok(CvtTable { values })
    }

    /// Get the number of control values
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the table is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Get a control value by index
    pub fn get(&self, index: usize) -> Option<i16> {
        self.values.get(index).copied()
    }
}

/// Font Program (fpgm)
///
/// Contains TrueType instructions that are executed once when the font is loaded.
/// These typically define custom functions used by glyph hints.
#[derive(Debug, Clone, PartialEq)]
pub struct FpgmTable {
    /// Raw instruction bytecode
    pub instructions: Vec<u8>,
}

impl FpgmTable {
    /// Parse fpgm table from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        Ok(FpgmTable {
            instructions: data.to_vec(),
        })
    }

    /// Get the number of instruction bytes
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    /// Check if the program is empty
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    /// Decode instructions into human-readable form
    pub fn decode_instructions(&self) -> Vec<HintInstruction> {
        decode_instructions(&self.instructions)
    }
}

/// Control Value Program (prep)
///
/// Contains TrueType instructions executed whenever the font size or
/// transformation changes. This typically sets up the graphics state.
#[derive(Debug, Clone, PartialEq)]
pub struct PrepTable {
    /// Raw instruction bytecode
    pub instructions: Vec<u8>,
}

impl PrepTable {
    /// Parse prep table from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        Ok(PrepTable {
            instructions: data.to_vec(),
        })
    }

    /// Get the number of instruction bytes
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    /// Check if the program is empty
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    /// Decode instructions into human-readable form
    pub fn decode_instructions(&self) -> Vec<HintInstruction> {
        decode_instructions(&self.instructions)
    }
}

/// Maximum Profile (maxp) table
///
/// Contains memory requirements for the font including hinting stack sizes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaxpHintingInfo {
    /// Maximum points in a non-composite glyph
    pub max_points: u16,
    /// Maximum contours in a non-composite glyph
    pub max_contours: u16,
    /// Maximum points in a composite glyph
    pub max_composite_points: u16,
    /// Maximum contours in a composite glyph
    pub max_composite_contours: u16,
    /// Maximum zones (twilight + glyph)
    pub max_zones: u16,
    /// Maximum points in twilight zone
    pub max_twilight_points: u16,
    /// Maximum storage area locations
    pub max_storage: u16,
    /// Maximum function definitions (FDEF)
    pub max_function_defs: u16,
    /// Maximum instruction definitions (IDEF)
    pub max_instruction_defs: u16,
    /// Maximum stack elements
    pub max_stack_elements: u16,
    /// Maximum size of instructions
    pub max_size_of_instructions: u16,
    /// Maximum components for composite glyphs
    pub max_component_elements: u16,
    /// Maximum recursion depth for composites
    pub max_component_depth: u16,
}

impl MaxpHintingInfo {
    /// Parse hinting info from maxp table data
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 32 {
            return Err(ParseError::CorruptedData(
                "maxp table too short for hinting info".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);

        // Skip version (4 bytes) and numGlyphs (2 bytes)
        let version = cursor.read_u32::<BigEndian>()?;
        let _num_glyphs = cursor.read_u16::<BigEndian>()?;

        // Version 0.5 (CFF fonts) doesn't have hinting info
        if version == 0x00005000 {
            return Ok(MaxpHintingInfo {
                max_points: 0,
                max_contours: 0,
                max_composite_points: 0,
                max_composite_contours: 0,
                max_zones: 0,
                max_twilight_points: 0,
                max_storage: 0,
                max_function_defs: 0,
                max_instruction_defs: 0,
                max_stack_elements: 0,
                max_size_of_instructions: 0,
                max_component_elements: 0,
                max_component_depth: 0,
            });
        }

        // Version 1.0 (TrueType) has full hinting info
        let max_points = cursor.read_u16::<BigEndian>()?;
        let max_contours = cursor.read_u16::<BigEndian>()?;
        let max_composite_points = cursor.read_u16::<BigEndian>()?;
        let max_composite_contours = cursor.read_u16::<BigEndian>()?;
        let max_zones = cursor.read_u16::<BigEndian>()?;
        let max_twilight_points = cursor.read_u16::<BigEndian>()?;
        let max_storage = cursor.read_u16::<BigEndian>()?;
        let max_function_defs = cursor.read_u16::<BigEndian>()?;
        let max_instruction_defs = cursor.read_u16::<BigEndian>()?;
        let max_stack_elements = cursor.read_u16::<BigEndian>()?;
        let max_size_of_instructions = cursor.read_u16::<BigEndian>()?;
        let max_component_elements = cursor.read_u16::<BigEndian>()?;
        let max_component_depth = cursor.read_u16::<BigEndian>()?;

        Ok(MaxpHintingInfo {
            max_points,
            max_contours,
            max_composite_points,
            max_composite_contours,
            max_zones,
            max_twilight_points,
            max_storage,
            max_function_defs,
            max_instruction_defs,
            max_stack_elements,
            max_size_of_instructions,
            max_component_elements,
            max_component_depth,
        })
    }
}

/// TrueType hinting instruction
#[derive(Debug, Clone, PartialEq)]
pub enum HintInstruction {
    // Push instructions
    /// Push bytes
    PushB(Vec<u8>),
    /// Push words
    PushW(Vec<i16>),
    /// Push N bytes
    NPushB(Vec<u8>),
    /// Push N words
    NPushW(Vec<i16>),

    // Stack manipulation
    /// Duplicate top of stack
    Dup,
    /// Pop top of stack
    Pop,
    /// Clear stack
    Clear,
    /// Swap top two elements
    Swap,
    /// Get stack depth
    Depth,
    /// Copy indexed element
    CIndex,
    /// Move indexed element to top
    MIndex,
    /// Roll top 3 elements
    Roll,

    // Comparison
    /// Less than
    Lt,
    /// Less than or equal
    LtEq,
    /// Greater than
    Gt,
    /// Greater than or equal
    GtEq,
    /// Equal
    Eq,
    /// Not equal
    NEq,

    // Logical
    /// Logical AND
    And,
    /// Logical OR
    Or,
    /// Logical NOT
    Not,

    // Arithmetic
    /// Add
    Add,
    /// Subtract
    Sub,
    /// Divide
    Div,
    /// Multiply
    Mul,
    /// Absolute value
    Abs,
    /// Negate
    Neg,
    /// Floor
    Floor,
    /// Ceiling
    Ceiling,
    /// Maximum
    Max,
    /// Minimum
    Min,

    // Control flow
    /// If
    If,
    /// Else
    Else,
    /// End if
    Eif,
    /// Jump relative
    JmpR,
    /// Jump relative on true
    JrOT,
    /// Jump relative on false
    JrOF,

    // Function definition
    /// Function definition
    FDef,
    /// End function definition
    EndF,
    /// Call function
    Call,
    /// Loop and call
    LoopCall,
    /// Instruction definition
    IDef,

    // Point operations
    /// Set vectors to axis
    SvTCa(u8),
    /// Projection vector to axis
    SpvTCa(u8),
    /// Freedom vector to axis
    SfvTCa(u8),
    /// Set dual projection vector to line
    SDPvTL(bool),
    /// Set projection vector to line
    SPvTL(bool),
    /// Set freedom vector to line
    SfvTL(bool),
    /// Set freedom vector to projection vector
    SfvTPv,
    /// Get projection vector
    GPv,
    /// Get freedom vector
    GFv,
    /// Set reference point 0
    SRP0,
    /// Set reference point 1
    SRP1,
    /// Set reference point 2
    SRP2,
    /// Set zone pointer 0
    SZP0,
    /// Set zone pointer 1
    SZP1,
    /// Set zone pointer 2
    SZP2,
    /// Set all zone pointers
    SZPs,

    // Point movement
    /// Move direct absolute point
    MDAP(bool),
    /// Move indirect absolute point
    MIAP(bool),
    /// Move direct relative point
    MDRP(u8),
    /// Move indirect relative point
    MIRP(u8),
    /// Align relative point
    AlignRP,
    /// Interpolate point
    IP,
    /// Move stack indirect relative point
    MSIRP(bool),
    /// Align points
    AlignPts,
    /// Untouch point
    UTP,
    /// Interpolate untouched points through outline
    IUP(u8),
    /// Shift point
    SHP(u8),
    /// Shift contour
    SHC(u8),
    /// Shift zone
    SHZ(u8),
    /// Shift point by a pixel amount
    ShpIX,

    // Measurement
    /// Measure distance
    MD(bool),
    /// Get coordinate
    GC(bool),
    /// Set coordinate from stack
    SCFS,
    /// Measure PPem
    MpPem,
    /// Measure point size
    MPS,

    // CVT and storage
    /// Read CVT entry
    RCVT,
    /// Write CVT entry in FUnits
    WCvtF,
    /// Write CVT entry in pixel units
    WCvtP,
    /// Read storage
    RS,
    /// Write storage
    WS,

    // Graphics state
    /// Set auto flip on
    FlipOn,
    /// Set auto flip off
    FlipOff,
    /// Flip point
    FlipPt,
    /// Flip range on
    FlipRgOn,
    /// Flip range off
    FlipRgOff,
    /// Set scan conversion control
    ScanCtrl,
    /// Set scan type
    ScanType,
    /// Instruction control
    InstCtrl,
    /// Set control value table cut in
    SCvTCi,
    /// Set single width cut in
    SSWCi,
    /// Set single width
    SSW,
    /// Set delta base
    SDB,
    /// Set delta shift
    SDS,
    /// Round to double grid
    RTDg,
    /// Round to grid
    RTG,
    /// Round to half grid
    RTHG,
    /// Round off
    ROff,
    /// Set round state
    SRound,
    /// Set 45 degree round state
    S45Round,
    /// Set minimum distance
    SMD,
    /// Get information
    GetInfo,

    // Delta instructions
    /// Delta exception P1
    DeltaP1,
    /// Delta exception P2
    DeltaP2,
    /// Delta exception P3
    DeltaP3,
    /// Delta exception C1
    DeltaC1,
    /// Delta exception C2
    DeltaC2,
    /// Delta exception C3
    DeltaC3,

    // Miscellaneous
    /// Debug instruction (no-op in production)
    Debug,
    /// AA (deprecated antialiasing instruction)
    AA,
    /// Sangw (deprecated)
    Sangw,

    /// Unknown instruction
    Unknown(u8),
}

/// Decode TrueType instruction bytecode
pub fn decode_instructions(bytecode: &[u8]) -> Vec<HintInstruction> {
    let mut instructions = Vec::new();
    let mut i = 0;

    while i < bytecode.len() {
        let opcode = bytecode[i];
        i += 1;

        let instruction = match opcode {
            // Push B - Push bytes
            0xB0..=0xB7 => {
                let count = (opcode - 0xB0 + 1) as usize;
                let mut values = Vec::with_capacity(count);
                for _ in 0..count {
                    if i < bytecode.len() {
                        values.push(bytecode[i]);
                        i += 1;
                    }
                }
                HintInstruction::PushB(values)
            }
            // Push W - Push words
            0xB8..=0xBF => {
                let count = (opcode - 0xB8 + 1) as usize;
                let mut values = Vec::with_capacity(count);
                for _ in 0..count {
                    if i + 1 < bytecode.len() {
                        let val = ((bytecode[i] as i16) << 8) | (bytecode[i + 1] as i16);
                        values.push(val);
                        i += 2;
                    }
                }
                HintInstruction::PushW(values)
            }
            // NPUSHB
            0x40 => {
                let count = if i < bytecode.len() {
                    bytecode[i] as usize
                } else {
                    0
                };
                i += 1;
                let mut values = Vec::with_capacity(count);
                for _ in 0..count {
                    if i < bytecode.len() {
                        values.push(bytecode[i]);
                        i += 1;
                    }
                }
                HintInstruction::NPushB(values)
            }
            // NPUSHW
            0x41 => {
                let count = if i < bytecode.len() {
                    bytecode[i] as usize
                } else {
                    0
                };
                i += 1;
                let mut values = Vec::with_capacity(count);
                for _ in 0..count {
                    if i + 1 < bytecode.len() {
                        let val = ((bytecode[i] as i16) << 8) | (bytecode[i + 1] as i16);
                        values.push(val);
                        i += 2;
                    }
                }
                HintInstruction::NPushW(values)
            }

            // Stack manipulation
            0x20 => HintInstruction::Dup,
            0x21 => HintInstruction::Pop,
            0x22 => HintInstruction::Clear,
            0x23 => HintInstruction::Swap,
            0x24 => HintInstruction::Depth,
            0x25 => HintInstruction::CIndex,
            0x26 => HintInstruction::MIndex,
            0x8A => HintInstruction::Roll,

            // Comparison
            0x50 => HintInstruction::Lt,
            0x51 => HintInstruction::LtEq,
            0x52 => HintInstruction::Gt,
            0x53 => HintInstruction::GtEq,
            0x54 => HintInstruction::Eq,
            0x55 => HintInstruction::NEq,

            // Logical
            0x5A => HintInstruction::And,
            0x5B => HintInstruction::Or,
            0x5C => HintInstruction::Not,

            // Arithmetic
            0x60 => HintInstruction::Add,
            0x61 => HintInstruction::Sub,
            0x62 => HintInstruction::Div,
            0x63 => HintInstruction::Mul,
            0x64 => HintInstruction::Abs,
            0x65 => HintInstruction::Neg,
            0x66 => HintInstruction::Floor,
            0x67 => HintInstruction::Ceiling,
            0x8B => HintInstruction::Max,
            0x8C => HintInstruction::Min,

            // Control flow
            0x58 => HintInstruction::If,
            0x1B => HintInstruction::Else,
            0x59 => HintInstruction::Eif,
            0x1C => HintInstruction::JmpR,
            0x78 => HintInstruction::JrOT,
            0x79 => HintInstruction::JrOF,

            // Function definition
            0x2C => HintInstruction::FDef,
            0x2D => HintInstruction::EndF,
            0x2B => HintInstruction::Call,
            0x2A => HintInstruction::LoopCall,
            0x89 => HintInstruction::IDef,

            // Set vectors
            0x00 => HintInstruction::SvTCa(0),
            0x01 => HintInstruction::SvTCa(1),
            0x02 => HintInstruction::SpvTCa(0),
            0x03 => HintInstruction::SpvTCa(1),
            0x04 => HintInstruction::SfvTCa(0),
            0x05 => HintInstruction::SfvTCa(1),
            0x06 => HintInstruction::SPvTL(false),
            0x07 => HintInstruction::SPvTL(true),
            0x08 => HintInstruction::SfvTL(false),
            0x09 => HintInstruction::SfvTL(true),
            0x0A => HintInstruction::SpvTCa(2), // SPVFS - Set projection vector from stack
            0x0B => HintInstruction::SfvTCa(2), // SFVFS - Set freedom vector from stack
            0x0C => HintInstruction::GPv,
            0x0D => HintInstruction::GFv,
            0x0E => HintInstruction::SfvTPv,
            0x86 => HintInstruction::SDPvTL(false),
            0x87 => HintInstruction::SDPvTL(true),

            // Reference points
            0x10 => HintInstruction::SRP0,
            0x11 => HintInstruction::SRP1,
            0x12 => HintInstruction::SRP2,

            // Zone pointers
            0x13 => HintInstruction::SZP0,
            0x14 => HintInstruction::SZP1,
            0x15 => HintInstruction::SZP2,
            0x16 => HintInstruction::SZPs,

            // Point movement
            0x2E => HintInstruction::MDAP(false),
            0x2F => HintInstruction::MDAP(true),
            0x3E => HintInstruction::MIAP(false),
            0x3F => HintInstruction::MIAP(true),
            0xC0..=0xDF => HintInstruction::MDRP(opcode - 0xC0),
            0xE0..=0xFF => HintInstruction::MIRP(opcode - 0xE0),
            0x3C => HintInstruction::AlignRP,
            0x39 => HintInstruction::IP,
            0x3A => HintInstruction::MSIRP(false),
            0x3B => HintInstruction::MSIRP(true),
            0x27 => HintInstruction::AlignPts,
            0x29 => HintInstruction::UTP,
            0x30 => HintInstruction::IUP(0),
            0x31 => HintInstruction::IUP(1),
            0x32 => HintInstruction::SHP(0),
            0x33 => HintInstruction::SHP(1),
            0x34 => HintInstruction::SHC(0),
            0x35 => HintInstruction::SHC(1),
            0x36 => HintInstruction::SHZ(0),
            0x37 => HintInstruction::SHZ(1),
            0x38 => HintInstruction::ShpIX,

            // Measurement
            0x49 => HintInstruction::MD(false),
            0x4A => HintInstruction::MD(true),
            0x46 => HintInstruction::GC(false),
            0x47 => HintInstruction::GC(true),
            0x48 => HintInstruction::SCFS,
            0x4B => HintInstruction::MpPem,
            0x4C => HintInstruction::MPS,

            // CVT and storage
            0x45 => HintInstruction::RCVT,
            0x70 => HintInstruction::WCvtP,
            0x44 => HintInstruction::WCvtF,
            0x43 => HintInstruction::RS,
            0x42 => HintInstruction::WS,

            // Graphics state
            0x4D => HintInstruction::FlipOn,
            0x4E => HintInstruction::FlipOff,
            0x80 => HintInstruction::FlipPt,
            0x81 => HintInstruction::FlipRgOn,
            0x82 => HintInstruction::FlipRgOff,
            0x85 => HintInstruction::ScanCtrl,
            0x8D => HintInstruction::ScanType,
            0x8E => HintInstruction::InstCtrl,
            0x1D => HintInstruction::SCvTCi,
            0x1E => HintInstruction::SSWCi,
            0x1F => HintInstruction::SSW,
            0x5E => HintInstruction::SDB,
            0x5F => HintInstruction::SDS,
            0x3D => HintInstruction::RTDg,
            0x18 => HintInstruction::RTG,
            0x19 => HintInstruction::RTHG,
            0x7A => HintInstruction::ROff,
            0x76 => HintInstruction::SRound,
            0x77 => HintInstruction::S45Round,
            0x1A => HintInstruction::SMD,
            0x88 => HintInstruction::GetInfo,

            // Delta instructions
            0x5D => HintInstruction::DeltaP1,
            0x71 => HintInstruction::DeltaP2,
            0x72 => HintInstruction::DeltaP3,
            0x73 => HintInstruction::DeltaC1,
            0x74 => HintInstruction::DeltaC2,
            0x75 => HintInstruction::DeltaC3,

            // Miscellaneous
            0x4F => HintInstruction::Debug,
            0x7F => HintInstruction::AA,
            0x8F => HintInstruction::Sangw,

            // Unknown
            _ => HintInstruction::Unknown(opcode),
        };

        instructions.push(instruction);
    }

    instructions
}

/// Hinting configuration and state
#[derive(Debug, Clone)]
pub struct HintingState {
    /// Control values (from cvt table, potentially modified)
    pub control_values: Vec<i32>,
    /// Storage area
    pub storage: Vec<i32>,
    /// Stack
    pub stack: Vec<i32>,
    /// Graphics state
    pub graphics_state: GraphicsState,
    /// Functions defined by FDEF
    pub functions: Vec<Vec<u8>>,
}

/// TrueType graphics state
#[derive(Debug, Clone, Default)]
pub struct GraphicsState {
    /// Auto flip on/off
    pub auto_flip: bool,
    /// Control value cut-in
    pub control_value_cut_in: f64,
    /// Delta base
    pub delta_base: i32,
    /// Delta shift
    pub delta_shift: i32,
    /// Dual projection vector
    pub dual_projection_vector: (f64, f64),
    /// Freedom vector
    pub freedom_vector: (f64, f64),
    /// Projection vector
    pub projection_vector: (f64, f64),
    /// Instruction control flags
    pub instruct_control: u8,
    /// Loop value
    pub loop_value: i32,
    /// Minimum distance
    pub minimum_distance: f64,
    /// Reference point 0
    pub rp0: u16,
    /// Reference point 1
    pub rp1: u16,
    /// Reference point 2
    pub rp2: u16,
    /// Round state
    pub round_state: RoundState,
    /// Scan control
    pub scan_control: bool,
    /// Single width cut-in
    pub single_width_cut_in: f64,
    /// Single width value
    pub single_width_value: i32,
    /// Zone pointer 0
    pub zp0: u8,
    /// Zone pointer 1
    pub zp1: u8,
    /// Zone pointer 2
    pub zp2: u8,
}

/// Round state for hinting
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum RoundState {
    /// Round to half grid
    HalfGrid,
    /// Round to grid
    #[default]
    Grid,
    /// Round to double grid
    DoubleGrid,
    /// Round down to grid
    DownToGrid,
    /// Round up to grid
    UpToGrid,
    /// Round off (no rounding)
    Off,
    /// Super round
    Super,
    /// Super 45 round
    Super45,
}

impl HintingState {
    /// Create a new hinting state with given CVT values
    pub fn new(cvt: &CvtTable, max_storage: usize, max_stack: usize) -> Self {
        HintingState {
            control_values: cvt.values.iter().map(|&v| v as i32).collect(),
            storage: vec![0; max_storage],
            stack: Vec::with_capacity(max_stack),
            graphics_state: GraphicsState::default(),
            functions: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cvt_parse_empty() {
        let result = CvtTable::parse(&[]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_cvt_parse_values() {
        let data = vec![0x00, 0x64, 0xFF, 0x9C]; // 100, -100
        let cvt = CvtTable::parse(&data).unwrap();
        assert_eq!(cvt.len(), 2);
        assert_eq!(cvt.get(0), Some(100));
        assert_eq!(cvt.get(1), Some(-100));
    }

    #[test]
    fn test_cvt_parse_odd_length() {
        let data = vec![0x00, 0x64, 0xFF]; // Odd number of bytes
        let result = CvtTable::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_fpgm_parse() {
        let data = vec![0x40, 0x02, 0x01, 0x02]; // NPUSHB with 2 bytes
        let fpgm = FpgmTable::parse(&data).unwrap();
        assert_eq!(fpgm.len(), 4);
        assert!(!fpgm.is_empty());
    }

    #[test]
    fn test_fpgm_decode() {
        let data = vec![0x20]; // DUP instruction
        let fpgm = FpgmTable::parse(&data).unwrap();
        let instructions = fpgm.decode_instructions();
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0], HintInstruction::Dup);
    }

    #[test]
    fn test_prep_parse() {
        let data = vec![0x18, 0x19]; // RTG, RTHG
        let prep = PrepTable::parse(&data).unwrap();
        assert_eq!(prep.len(), 2);
    }

    #[test]
    fn test_prep_decode() {
        let data = vec![0x18, 0x19]; // RTG, RTHG
        let prep = PrepTable::parse(&data).unwrap();
        let instructions = prep.decode_instructions();
        assert_eq!(instructions.len(), 2);
        assert_eq!(instructions[0], HintInstruction::RTG);
        assert_eq!(instructions[1], HintInstruction::RTHG);
    }

    #[test]
    fn test_decode_push_instructions() {
        // PUSHB[0] with 1 byte
        let instructions = decode_instructions(&[0xB0, 0x42]);
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0], HintInstruction::PushB(vec![0x42]));
    }

    #[test]
    fn test_decode_pushw_instructions() {
        // PUSHW[0] with 1 word
        let instructions = decode_instructions(&[0xB8, 0x00, 0x64]);
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0], HintInstruction::PushW(vec![100]));
    }

    #[test]
    fn test_decode_npushb() {
        let instructions = decode_instructions(&[0x40, 0x02, 0x0A, 0x14]);
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0], HintInstruction::NPushB(vec![0x0A, 0x14]));
    }

    #[test]
    fn test_decode_arithmetic() {
        let data = vec![0x60, 0x61, 0x62, 0x63]; // ADD, SUB, DIV, MUL
        let instructions = decode_instructions(&data);
        assert_eq!(instructions.len(), 4);
        assert_eq!(instructions[0], HintInstruction::Add);
        assert_eq!(instructions[1], HintInstruction::Sub);
        assert_eq!(instructions[2], HintInstruction::Div);
        assert_eq!(instructions[3], HintInstruction::Mul);
    }

    #[test]
    fn test_decode_control_flow() {
        let data = vec![0x58, 0x1B, 0x59]; // IF, ELSE, EIF
        let instructions = decode_instructions(&data);
        assert_eq!(instructions.len(), 3);
        assert_eq!(instructions[0], HintInstruction::If);
        assert_eq!(instructions[1], HintInstruction::Else);
        assert_eq!(instructions[2], HintInstruction::Eif);
    }

    #[test]
    fn test_decode_function_defs() {
        let data = vec![0x2C, 0x2D, 0x2B]; // FDEF, ENDF, CALL
        let instructions = decode_instructions(&data);
        assert_eq!(instructions.len(), 3);
        assert_eq!(instructions[0], HintInstruction::FDef);
        assert_eq!(instructions[1], HintInstruction::EndF);
        assert_eq!(instructions[2], HintInstruction::Call);
    }

    #[test]
    fn test_decode_mdrp_mirp() {
        let data = vec![0xC0, 0xC1, 0xE0, 0xE1];
        let instructions = decode_instructions(&data);
        assert_eq!(instructions.len(), 4);
        assert_eq!(instructions[0], HintInstruction::MDRP(0));
        assert_eq!(instructions[1], HintInstruction::MDRP(1));
        assert_eq!(instructions[2], HintInstruction::MIRP(0));
        assert_eq!(instructions[3], HintInstruction::MIRP(1));
    }

    #[test]
    fn test_maxp_hinting_info_short_data() {
        let result = MaxpHintingInfo::parse(&[0, 0, 0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_graphics_state_default() {
        let gs = GraphicsState::default();
        assert!(!gs.auto_flip);
        assert_eq!(gs.rp0, 0);
        assert_eq!(gs.round_state, RoundState::Grid);
    }

    #[test]
    fn test_hinting_state_new() {
        let cvt = CvtTable {
            values: vec![100, 200, 300],
        };
        let state = HintingState::new(&cvt, 32, 256);
        assert_eq!(state.control_values.len(), 3);
        assert_eq!(state.storage.len(), 32);
        assert!(state.stack.is_empty());
    }

    #[test]
    fn test_round_state_variants() {
        let _ = RoundState::HalfGrid;
        let _ = RoundState::Grid;
        let _ = RoundState::DoubleGrid;
        let _ = RoundState::DownToGrid;
        let _ = RoundState::UpToGrid;
        let _ = RoundState::Off;
        let _ = RoundState::Super;
        let _ = RoundState::Super45;
    }

    #[test]
    fn test_unknown_instruction() {
        let instructions = decode_instructions(&[0x0F]);
        assert_eq!(instructions.len(), 1);
        // 0x0F is ISECT, which we map to Unknown in this decoder
        matches!(instructions[0], HintInstruction::Unknown(0x0F));
    }
}
