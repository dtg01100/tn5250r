/// TN3270 Protocol Constants and Codes
///
/// This module contains the IBM 3270 protocol constants, command codes,
/// order codes, AID (Attention Identifier) keys, and field attributes
/// as specified in RFC 1205 and RFC 2355.
///
/// # References
/// - RFC 1205: 5250 Telnet Interface
/// - RFC 2355: TN3270 Enhancements
/// - IBM 3270 Data Stream Programmer's Reference (GA23-0059)
/// 3270 Command Codes
///
/// These are the primary commands sent from the host to the terminal
pub const CMD_WRITE: u8 = 0x01;              // Write command
pub const CMD_ERASE_WRITE: u8 = 0x05;        // Erase/Write command
pub const CMD_ERASE_WRITE_ALTERNATE: u8 = 0x0D; // Erase/Write Alternate
pub const CMD_READ_BUFFER: u8 = 0x02;        // Read Buffer command
pub const CMD_READ_MODIFIED: u8 = 0x06;      // Read Modified command
pub const CMD_READ_MODIFIED_ALL: u8 = 0x0E;  // Read Modified All command
pub const CMD_ERASE_ALL_UNPROTECTED: u8 = 0x0F; // Erase All Unprotected
pub const CMD_WRITE_STRUCTURED_FIELD: u8 = 0x11; // Write Structured Field

/// 3270 Order Codes
/// These are embedded in the data stream to control formatting
pub const ORDER_SF: u8 = 0x1D;    // Start Field
pub const ORDER_SFE: u8 = 0x29;   // Start Field Extended
pub const ORDER_SBA: u8 = 0x11;   // Set Buffer Address
pub const ORDER_SA: u8 = 0x28;    // Set Attribute
pub const ORDER_MF: u8 = 0x2C;    // Modify Field
pub const ORDER_IC: u8 = 0x13;    // Insert Cursor
pub const ORDER_PT: u8 = 0x05;    // Program Tab
pub const ORDER_RA: u8 = 0x3C;    // Repeat to Address
pub const ORDER_EUA: u8 = 0x12;   // Erase Unprotected to Address
pub const ORDER_GE: u8 = 0x08;    // Graphic Escape

/// Write Control Character (WCC) Bits
/// Used with Write and Erase/Write commands
pub const WCC_RESET: u8 = 0x40;           // Reset bit
pub const WCC_ALARM: u8 = 0x04;           // Sound alarm
pub const WCC_RESTORE: u8 = 0x02;         // Restore keyboard
pub const WCC_RESET_MDT: u8 = 0x01;       // Reset MDT bits

/// AID (Attention Identifier) Keys
/// Sent from terminal to host to identify which key was pressed
pub const AID_NO_AID: u8 = 0x60;          // No AID generated
pub const AID_STRUCTURED_FIELD: u8 = 0x88; // Structured field
pub const AID_READ_PARTITION: u8 = 0x61;   // Read partition
pub const AID_TRIGGER: u8 = 0x7F;          // Trigger action

// Function keys
pub const AID_PF1: u8 = 0xF1;
pub const AID_PF2: u8 = 0xF2;
pub const AID_PF3: u8 = 0xF3;
pub const AID_PF4: u8 = 0xF4;
pub const AID_PF5: u8 = 0xF5;
pub const AID_PF6: u8 = 0xF6;
pub const AID_PF7: u8 = 0xF7;
pub const AID_PF8: u8 = 0xF8;
pub const AID_PF9: u8 = 0xF9;
pub const AID_PF10: u8 = 0x7A;
pub const AID_PF11: u8 = 0x7B;
pub const AID_PF12: u8 = 0x7C;
pub const AID_PF13: u8 = 0xC1;
pub const AID_PF14: u8 = 0xC2;
pub const AID_PF15: u8 = 0xC3;
pub const AID_PF16: u8 = 0xC4;
pub const AID_PF17: u8 = 0xC5;
pub const AID_PF18: u8 = 0xC6;
pub const AID_PF19: u8 = 0xC7;
pub const AID_PF20: u8 = 0xC8;
pub const AID_PF21: u8 = 0xC9;
pub const AID_PF22: u8 = 0x4A;
pub const AID_PF23: u8 = 0x4B;
pub const AID_PF24: u8 = 0x4C;

// Program attention keys
pub const AID_PA1: u8 = 0x6C;
pub const AID_PA2: u8 = 0x6E;
pub const AID_PA3: u8 = 0x6B;

// Special keys
pub const AID_CLEAR: u8 = 0x6D;
pub const AID_ENTER: u8 = 0x7D;
pub const AID_SYSREQ: u8 = 0xF0;

/// Field Attribute Byte Bits
/// Used in Start Field (SF) order
pub const ATTR_PROTECTED: u8 = 0x20;      // Bit 5: Protected field
pub const ATTR_NUMERIC: u8 = 0x10;        // Bit 4: Numeric field
pub const ATTR_DISPLAY: u8 = 0x0C;        // Bits 2-3: Display attributes
pub const ATTR_RESERVED: u8 = 0x02;       // Bit 1: Reserved
pub const ATTR_MDT: u8 = 0x01;            // Bit 0: Modified Data Tag

/// Display Attribute Values (bits 2-3 of field attribute)
pub const DISPLAY_NORMAL: u8 = 0x00;      // Normal display
pub const DISPLAY_INTENSIFIED: u8 = 0x08; // Intensified display
pub const DISPLAY_HIDDEN: u8 = 0x0C;      // Non-display (hidden)

/// Extended Field Attribute Types (for SFE order)
pub const XA_ALL: u8 = 0x00;              // All character attributes
pub const XA_3270: u8 = 0xC0;             // 3270 field attribute
pub const XA_VALIDATION: u8 = 0xC1;       // Field validation
pub const XA_OUTLINING: u8 = 0xC2;        // Field outlining
pub const XA_HIGHLIGHTING: u8 = 0x41;     // Highlighting
pub const XA_FOREGROUND: u8 = 0x42;       // Foreground color
pub const XA_CHARSET: u8 = 0x43;          // Character set
pub const XA_BACKGROUND: u8 = 0x45;       // Background color
pub const XA_TRANSPARENCY: u8 = 0x46;     // Transparency

/// Color Attribute Values
pub const COLOR_DEFAULT: u8 = 0x00;       // Default color
pub const COLOR_BLUE: u8 = 0xF1;          // Blue
pub const COLOR_RED: u8 = 0xF2;           // Red
pub const COLOR_PINK: u8 = 0xF3;          // Pink
pub const COLOR_GREEN: u8 = 0xF4;         // Green
pub const COLOR_TURQUOISE: u8 = 0xF5;     // Turquoise
pub const COLOR_YELLOW: u8 = 0xF6;        // Yellow
pub const COLOR_WHITE: u8 = 0xF7;         // White
pub const COLOR_BLACK: u8 = 0xF8;         // Black
pub const COLOR_DEEP_BLUE: u8 = 0xF9;     // Deep blue
pub const COLOR_ORANGE: u8 = 0xFA;        // Orange
pub const COLOR_PURPLE: u8 = 0xFB;        // Purple
pub const COLOR_PALE_GREEN: u8 = 0xFC;    // Pale green
pub const COLOR_PALE_TURQUOISE: u8 = 0xFD; // Pale turquoise
pub const COLOR_GREY: u8 = 0xFE;          // Grey
pub const COLOR_NEUTRAL: u8 = 0xFF;       // Neutral (white)

/// Highlighting Attribute Values
pub const HIGHLIGHT_DEFAULT: u8 = 0x00;   // Default (normal)
pub const HIGHLIGHT_NORMAL: u8 = 0xF0;    // Normal
pub const HIGHLIGHT_BLINK: u8 = 0xF1;     // Blink
pub const HIGHLIGHT_REVERSE: u8 = 0xF2;   // Reverse video
pub const HIGHLIGHT_UNDERSCORE: u8 = 0xF4; // Underscore

/// Validation Attribute Values
pub const VALIDATION_MANDATORY_FILL: u8 = 0x04;    // Mandatory fill
pub const VALIDATION_MANDATORY_ENTRY: u8 = 0x02;   // Mandatory entry
pub const VALIDATION_TRIGGER: u8 = 0x01;           // Trigger field

/// Enum representation of 3270 command codes for type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandCode {
    Write = CMD_WRITE as isize,
    EraseWrite = CMD_ERASE_WRITE as isize,
    EraseWriteAlternate = CMD_ERASE_WRITE_ALTERNATE as isize,
    ReadBuffer = CMD_READ_BUFFER as isize,
    ReadModified = CMD_READ_MODIFIED as isize,
    ReadModifiedAll = CMD_READ_MODIFIED_ALL as isize,
    EraseAllUnprotected = CMD_ERASE_ALL_UNPROTECTED as isize,
    WriteStructuredField = CMD_WRITE_STRUCTURED_FIELD as isize,
}

impl CommandCode {
    /// Convert a byte value to a CommandCode enum
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            CMD_WRITE => Some(Self::Write),
            CMD_ERASE_WRITE => Some(Self::EraseWrite),
            CMD_ERASE_WRITE_ALTERNATE => Some(Self::EraseWriteAlternate),
            CMD_READ_BUFFER => Some(Self::ReadBuffer),
            CMD_READ_MODIFIED => Some(Self::ReadModified),
            CMD_READ_MODIFIED_ALL => Some(Self::ReadModifiedAll),
            CMD_ERASE_ALL_UNPROTECTED => Some(Self::EraseAllUnprotected),
            CMD_WRITE_STRUCTURED_FIELD => Some(Self::WriteStructuredField),
            _ => None,
        }
    }

    /// Convert CommandCode enum to byte value
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Enum representation of 3270 order codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderCode {
    StartField = ORDER_SF as isize,
    StartFieldExtended = ORDER_SFE as isize,
    SetBufferAddress = ORDER_SBA as isize,
    SetAttribute = ORDER_SA as isize,
    ModifyField = ORDER_MF as isize,
    InsertCursor = ORDER_IC as isize,
    ProgramTab = ORDER_PT as isize,
    RepeatToAddress = ORDER_RA as isize,
    EraseUnprotectedToAddress = ORDER_EUA as isize,
    GraphicEscape = ORDER_GE as isize,
}

impl OrderCode {
    /// Convert a byte value to an OrderCode enum
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            ORDER_SF => Some(Self::StartField),
            ORDER_SFE => Some(Self::StartFieldExtended),
            ORDER_SBA => Some(Self::SetBufferAddress),
            ORDER_SA => Some(Self::SetAttribute),
            ORDER_MF => Some(Self::ModifyField),
            ORDER_IC => Some(Self::InsertCursor),
            ORDER_PT => Some(Self::ProgramTab),
            ORDER_RA => Some(Self::RepeatToAddress),
            ORDER_EUA => Some(Self::EraseUnprotectedToAddress),
            ORDER_GE => Some(Self::GraphicEscape),
            _ => None,
        }
    }

    /// Convert OrderCode enum to byte value
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Enum representation of AID keys
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AidKey {
    NoAid,
    Enter,
    Clear,
    PA1,
    PA2,
    PA3,
    PF1, PF2, PF3, PF4, PF5, PF6,
    PF7, PF8, PF9, PF10, PF11, PF12,
    PF13, PF14, PF15, PF16, PF17, PF18,
    PF19, PF20, PF21, PF22, PF23, PF24,
    StructuredField,
    ReadPartition,
    Trigger,
    SysReq,
}

impl AidKey {
    /// Convert a byte value to an AidKey enum
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            AID_NO_AID => Some(Self::NoAid),
            AID_ENTER => Some(Self::Enter),
            AID_CLEAR => Some(Self::Clear),
            AID_PA1 => Some(Self::PA1),
            AID_PA2 => Some(Self::PA2),
            AID_PA3 => Some(Self::PA3),
            AID_PF1 => Some(Self::PF1),
            AID_PF2 => Some(Self::PF2),
            AID_PF3 => Some(Self::PF3),
            AID_PF4 => Some(Self::PF4),
            AID_PF5 => Some(Self::PF5),
            AID_PF6 => Some(Self::PF6),
            AID_PF7 => Some(Self::PF7),
            AID_PF8 => Some(Self::PF8),
            AID_PF9 => Some(Self::PF9),
            AID_PF10 => Some(Self::PF10),
            AID_PF11 => Some(Self::PF11),
            AID_PF12 => Some(Self::PF12),
            AID_PF13 => Some(Self::PF13),
            AID_PF14 => Some(Self::PF14),
            AID_PF15 => Some(Self::PF15),
            AID_PF16 => Some(Self::PF16),
            AID_PF17 => Some(Self::PF17),
            AID_PF18 => Some(Self::PF18),
            AID_PF19 => Some(Self::PF19),
            AID_PF20 => Some(Self::PF20),
            AID_PF21 => Some(Self::PF21),
            AID_PF22 => Some(Self::PF22),
            AID_PF23 => Some(Self::PF23),
            AID_PF24 => Some(Self::PF24),
            AID_STRUCTURED_FIELD => Some(Self::StructuredField),
            AID_READ_PARTITION => Some(Self::ReadPartition),
            AID_TRIGGER => Some(Self::Trigger),
            AID_SYSREQ => Some(Self::SysReq),
            _ => None,
        }
    }

    /// Convert AidKey enum to byte value
    pub fn to_u8(self) -> u8 {
        match self {
            Self::NoAid => AID_NO_AID,
            Self::Enter => AID_ENTER,
            Self::Clear => AID_CLEAR,
            Self::PA1 => AID_PA1,
            Self::PA2 => AID_PA2,
            Self::PA3 => AID_PA3,
            Self::PF1 => AID_PF1,
            Self::PF2 => AID_PF2,
            Self::PF3 => AID_PF3,
            Self::PF4 => AID_PF4,
            Self::PF5 => AID_PF5,
            Self::PF6 => AID_PF6,
            Self::PF7 => AID_PF7,
            Self::PF8 => AID_PF8,
            Self::PF9 => AID_PF9,
            Self::PF10 => AID_PF10,
            Self::PF11 => AID_PF11,
            Self::PF12 => AID_PF12,
            Self::PF13 => AID_PF13,
            Self::PF14 => AID_PF14,
            Self::PF15 => AID_PF15,
            Self::PF16 => AID_PF16,
            Self::PF17 => AID_PF17,
            Self::PF18 => AID_PF18,
            Self::PF19 => AID_PF19,
            Self::PF20 => AID_PF20,
            Self::PF21 => AID_PF21,
            Self::PF22 => AID_PF22,
            Self::PF23 => AID_PF23,
            Self::PF24 => AID_PF24,
            Self::StructuredField => AID_STRUCTURED_FIELD,
            Self::ReadPartition => AID_READ_PARTITION,
            Self::Trigger => AID_TRIGGER,
            Self::SysReq => AID_SYSREQ,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_code_conversion() {
        assert_eq!(CommandCode::from_u8(CMD_WRITE), Some(CommandCode::Write));
        assert_eq!(CommandCode::Write.to_u8(), CMD_WRITE);
        assert_eq!(CommandCode::from_u8(0xFF), None);
    }

    #[test]
    fn test_order_code_conversion() {
        assert_eq!(OrderCode::from_u8(ORDER_SF), Some(OrderCode::StartField));
        assert_eq!(OrderCode::StartField.to_u8(), ORDER_SF);
        assert_eq!(OrderCode::from_u8(0xFF), None);
    }

    #[test]
    fn test_aid_key_conversion() {
        assert_eq!(AidKey::from_u8(AID_ENTER), Some(AidKey::Enter));
        assert_eq!(AidKey::Enter.to_u8(), AID_ENTER);
        assert_eq!(AidKey::from_u8(AID_PF1), Some(AidKey::PF1));
        assert_eq!(AidKey::PF1.to_u8(), AID_PF1);
    }

    #[test]
    fn test_field_attribute_bits() {
        let protected_numeric = ATTR_PROTECTED | ATTR_NUMERIC;
        assert_eq!(protected_numeric & ATTR_PROTECTED, ATTR_PROTECTED);
        assert_eq!(protected_numeric & ATTR_NUMERIC, ATTR_NUMERIC);
    }
}