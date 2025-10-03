/// TN5250 Protocol Constants and Codes
/// 
/// This module contains a direct port of the original lib5250 codes5250.h
/// header file, translating all the 5250 protocol constants, command codes,
/// orders, error codes, and field attributes from C to Rust.
/// 
/// Source: lib5250/codes5250.h from the tn5250 project
/// Copyright (C) 2000-2008 Michael Madore
/// Miscellaneous constants




/// 5250 Protocol Commands
pub const CMD_CLEAR_UNIT: u8 = 0x40;
pub const CMD_CLEAR_UNIT_ALTERNATE: u8 = 0x20;
pub const CMD_CLEAR_FORMAT_TABLE: u8 = 0x50;
pub const CMD_WRITE_TO_DISPLAY: u8 = 0x11;
pub const CMD_WRITE_ERROR_CODE: u8 = 0x21;
pub const CMD_WRITE_ERROR_CODE_WINDOW: u8 = 0x22;
pub const CMD_READ_INPUT_FIELDS: u8 = 0x42;
pub const CMD_READ_MDT_FIELDS: u8 = 0x52;
pub const CMD_READ_MDT_FIELDS_ALT: u8 = 0x82;
pub const CMD_READ_SCREEN_IMMEDIATE: u8 = 0x62;
pub const CMD_READ_SCREEN_EXTENDED: u8 = 0x64;
pub const CMD_READ_SCREEN_PRINT: u8 = 0x66;
pub const CMD_READ_SCREEN_PRINT_EXTENDED: u8 = 0x68;
pub const CMD_READ_SCREEN_PRINT_GRID: u8 = 0x6A;
pub const CMD_READ_SCREEN_PRINT_EXT_GRID: u8 = 0x6C;
pub const CMD_READ_IMMEDIATE: u8 = 0x72;
pub const CMD_READ_IMMEDIATE_ALT: u8 = 0x83;
pub const CMD_SAVE_SCREEN: u8 = 0x02;
pub const CMD_SAVE_PARTIAL_SCREEN: u8 = 0x03;
pub const CMD_RESTORE_SCREEN: u8 = 0x12;
pub const CMD_RESTORE_PARTIAL_SCREEN: u8 = 0x13;
pub const CMD_ROLL: u8 = 0x23;
pub const CMD_WRITE_STRUCTURED_FIELD: u8 = 0xF3;

/// 5250 Protocol Orders
pub const SOH: u8 = 0x01;  // Start of header
pub const RA: u8 = 0x02;   // Repeat to address
pub const EA: u8 = 0x03;   // Erase to Address on 5494 (FIXME: not implemented)

pub const SBA: u8 = 0x11;  // Set buffer address

pub const IC: u8 = 0x13;   // Insert cursor


pub const SF: u8 = 0x1D;   // Start of field

/// Write to display structured field types
pub const DEFINE_SELECTION_FIELD: u8 = 0x50;
pub const CREATE_WINDOW: u8 = 0x51;
pub const UNREST_WIN_CURS_MOVE: u8 = 0x52;
pub const DEFINE_SCROLL_BAR_FIELD: u8 = 0x53;
pub const WRITE_DATA: u8 = 0x54;
pub const PROGRAMMABLE_MOUSE_BUT: u8 = 0x55;
pub const REM_GUI_SEL_FIELD: u8 = 0x58;
pub const REM_GUI_WINDOW: u8 = 0x59;
pub const REM_GUI_SCROLL_BAR_FIELD: u8 = 0x5B;
pub const REM_ALL_GUI_CONSTRUCTS: u8 = 0x5F;
pub const DRAW_ERASE_GRID_LINES: u8 = 0x60;
pub const CLEAR_GRID_LINE_BUFFER: u8 = 0x61;

/// Write structured field types
pub const DEFINE_AUDIT_WINDOW_TABLE: u8 = 0x30;
pub const DEFINE_COMMAND_KEY_FUNCTION: u8 = 0x31;
pub const READ_TEXT_SCREEN: u8 = 0x32;
pub const DEFINE_PENDING_OPERATIONS: u8 = 0x33;
pub const DEFINE_TEXT_SCREEN_FORMAT: u8 = 0x34;
pub const DEFINE_SCALE_TIME: u8 = 0x35;
pub const WRITE_TEXT_SCREEN: u8 = 0x36;
pub const DEFINE_SPECIAL_CHARACTERS: u8 = 0x37;
pub const PENDING_DATA: u8 = 0x38;
pub const DEFINE_OPERATOR_ERROR_MSGS: u8 = 0x39;
pub const DEFINE_PITCH_TABLE: u8 = 0x3A;
pub const DEFINE_FAKE_DP_CMD_KEY_FUNC: u8 = 0x3B;
pub const PASS_THROUGH: u8 = 0x3F;
pub const SF_5250_QUERY: u8 = 0x70;
pub const SF_5250_QUERY_STATION_STATE: u8 = 0x72;

// Additional structured field types from protocol-level processing
pub const SF_QUERY_COMMAND: u8 = 0x84;
pub const SF_SET_REPLY_MODE: u8 = 0x85;
pub const SF_ERASE_RESET: u8 = 0x5B;
pub const SF_DEFINE_PENDING_OPERATIONS: u8 = 0x80;
pub const SF_ENABLE_COMMAND_RECOGNITION: u8 = 0x82;
pub const SF_REQUEST_TIMESTAMP_INTERVAL: u8 = 0x83;
pub const SF_DEFINE_ROLL_DIRECTION: u8 = 0x86;
pub const SF_SET_MONITOR_MODE: u8 = 0x87;
pub const SF_CANCEL_RECOVERY: u8 = 0x88;
pub const SF_CREATE_CHANGE_EXTENDED_ATTRIBUTE: u8 = 0xC1;
pub const SF_SET_EXTENDED_ATTRIBUTE_LIST: u8 = 0xCA;
pub const SF_READ_TEXT: u8 = 0xD2;
pub const SF_DEFINE_EXTENDED_ATTRIBUTE: u8 = 0xD3;
pub const SF_DEFINE_NAMED_LOGICAL_UNIT: u8 = 0x7E;

/// Operator Error Codes
/// See 5494 User's Guide (GA27-3960-03) 2.3.4
pub const ERR_DONT_KNOW: u8 = 0x01;
pub const ERR_BYPASS_FIELD: u8 = 0x04;
pub const ERR_NO_FIELD: u8 = 0x05;
pub const ERR_INVALID_SYSREQ: u8 = 0x06;
pub const ERR_MANDATORY_ENTRY: u8 = 0x07;
pub const ERR_ALPHA_ONLY: u8 = 0x08;
pub const ERR_NUMERIC_ONLY: u8 = 0x09;
pub const ERR_DIGITS_ONLY: u8 = 0x10;
pub const ERR_LAST_SIGNED: u8 = 0x11;
pub const ERR_NO_ROOM: u8 = 0x12;
pub const ERR_MANADATORY_FILL: u8 = 0x14;
pub const ERR_CHECK_DIGIT: u8 = 0x15;
pub const ERR_NOT_SIGNED: u8 = 0x16;
pub const ERR_EXIT_NOT_VALID: u8 = 0x18;
pub const ERR_DUP_NOT_ENABLED: u8 = 0x19;
pub const ERR_NO_FIELD_EXIT: u8 = 0x20;
pub const ERR_NO_INPUT: u8 = 0x26;
pub const ERR_BAD_CHAR: u8 = 0x27;

/// Error Messages
pub const MSG_DONT_KNOW: &str = "Keyboard overrun.";
pub const MSG_BYPASS_FIELD: &str = "Entry of data not allowed in this input/output field.";
pub const MSG_NO_FIELD: &str = "Cursor in protected area of display.";
pub const MSG_INVALID_SYSREQ: &str = "Key pressed following System Request key was not valid.";
pub const MSG_MANDATORY_ENTRY: &str = "Mandatory data entry field. Must have data entered.";
pub const MSG_ALPHA_ONLY: &str = "Field requires alphabetic characters.";
pub const MSG_NUMERIC_ONLY: &str = "Field requires numeric characters.";
pub const MSG_DIGITS_ONLY: &str = "Only characters 0 through 9 allowed.";
pub const MSG_LAST_SIGNED: &str = "Key for sign position of field not valid.";
pub const MSG_NO_ROOM: &str = "No room to insert data.";
pub const MSG_MANADATORY_FILL: &str = "Mandatory fill field. Must fill to exit.";
pub const MSG_CHECK_DIGIT: &str = "Modulo 10 or 11 check digit error.";
pub const MSG_NOT_SIGNED: &str = "Field Minus key not valid in field.";
pub const MSG_EXIT_NOT_VALID: &str = "The key used to exit field not valid.";
pub const MSG_DUP_NOT_ENABLED: &str = "Duplicate key or Field Mark key not allowed in field.";
pub const MSG_NO_FIELD_EXIT: &str = "Enter key not allowed in field.";
pub const MSG_NO_INPUT: &str = "Field- entry not allowed.";
pub const MSG_BAD_CHAR: &str = "Cannot use undefined key.";
pub const MSG_NO_HELP: &str = "No help text is available.";

/// DBCS Support (Japan)
pub const ERR_DBCS_WRONG_TYPE: u8 = 0x60;
pub const ERR_SBCS_WRONG_TYPE: u8 = 0x61;
pub const MSG_DBCS_WRONG_TYPE: &str = "Field requires alphanumeric characters.";
pub const MSG_SBCS_WRONG_TYPE: &str = "Field requires double-byte characters.";

/// Data Stream Negative Response Codes
/// From Data Stream Negative Responses (SC30-3533-04) 13.4
pub const DSNR_RESEQ_ERR: u8 = 0x03;
pub const DSNR_INVCURSPOS: u8 = 0x22;
pub const DSNR_RAB4WSA: u8 = 0x23;
pub const DSNR_INVSFA: u8 = 0x26;
pub const DSNR_FLDEOD: u8 = 0x28;
pub const DSNR_FMTOVF: u8 = 0x29;
pub const DSNR_WRTEOD: u8 = 0x2A;
pub const DSNR_SOHLEN: u8 = 0x2B;
pub const DSNR_ROLLPARM: u8 = 0x2C;
pub const DSNR_NO_ESC: u8 = 0x31;
pub const DSNR_INV_WECW: u8 = 0x32;
pub const DSNR_UNKNOWN: i8 = -1;

/// Error Messages for Data Stream Negative Responses
pub const EMSG_RESEQ_ERR: &str = "Format table resequencing error.";
pub const EMSG_INVCURSPOS: &str = "Write to display order row/col address is not valid";
pub const EMSG_RAB4WSA: &str = "Repeat to Address less than the current WS address.";
pub const EMSG_INVSFA: &str = "Start-of-field order address not valid";
pub const EMSG_FLDEOD: &str = "Field extends past the end of the display.";
pub const EMSG_FMTOVF: &str = "Format table overflow.";
pub const EMSG_WRTEOD: &str = "Attempted to write past the end of display.";
pub const EMSG_SOHLEN: &str = "Start-of-header length not valid.";
pub const EMSG_ROLLPARM: &str = "Invalid ROLL command parameter.";
pub const EMSG_NO_ESC: &str = "No escape code was found where it was expected.";
pub const EMSG_INV_WECW: &str = "Invalid row/col address on WEC TO WINDOW command.";

/// Field Attributes
/// C.f. 5494 Functions Reference (SC30-3533-04), Section 15.6.12.3.
/// Bits 0-2 always set to 001 to identify as an attribute byte.
pub const ATTR_5250_GREEN: u8 = 0x20;   // Default
pub const ATTR_5250_WHITE: u8 = 0x22;
pub const ATTR_5250_NONDISP: u8 = 0x27; // Nondisplay
pub const ATTR_5250_RED: u8 = 0x28;
pub const ATTR_5250_TURQ: u8 = 0x30;
pub const ATTR_5250_YELLOW: u8 = 0x32;
pub const ATTR_5250_PINK: u8 = 0x38;
pub const ATTR_5250_BLUE: u8 = 0x3A;

pub const ATTR_5250_NORMAL: u8 = ATTR_5250_GREEN;

/// Keyboard / Error handling states
pub const TN5250_KEYSTATE_UNLOCKED: u8 = 0;
pub const TN5250_KEYSTATE_LOCKED: u8 = 1;
pub const TN5250_KEYSTATE_HARDWARE: u8 = 2;
pub const TN5250_KEYSTATE_PREHELP: u8 = 3;
pub const TN5250_KEYSTATE_POSTHELP: u8 = 4;

/// Keyboard Error Source Codes
pub const TN5250_KBDSRC_NONE: u16 = 0x0000;            // No Error
pub const TN5250_KBDSRC_INVALID_CMD: u16 = 0x0003;     // Bad key following CMD key
pub const TN5250_KBDSRC_DATA_DISALLOWED: u16 = 0x0004; // Keyboard in MSR field
pub const TN5250_KBDSRC_PROTECT: u16 = 0x0005;         // Cursor in protected area
pub const TN5250_KBDSRC_ALPHAONLY: u16 = 0x0008;       // Field Requires Alpha
pub const TN5250_KBDSRC_NUMONLY: u16 = 0x0009;         // Field Requires Numeric
pub const TN5250_KBDSRC_ONLY09: u16 = 0x0010;          // Only chars 0-9 allowed
pub const TN5250_KBDSRC_SIGNPOS: u16 = 0x0011;         // Sign position invalid
pub const TN5250_KBDSRC_NOROOM: u16 = 0x0012;          // No room for insert
pub const TN5250_KBDSRC_FLDM_DISALLOWED: u16 = 0x0016; // Field- Not Allowed
pub const TN5250_KBDSRC_FER: u16 = 0x0018;             // Field Exit Required
pub const TN5250_KBDSRC_DUP_DISALLOWED: u16 = 0x0019;  // Dup Key Not Allowed

/// Helper function to get error message for a given error code
pub fn get_error_message(error_code: u8) -> Option<&'static str> {
    match error_code {
        ERR_DONT_KNOW => Some(MSG_DONT_KNOW),
        ERR_BYPASS_FIELD => Some(MSG_BYPASS_FIELD),
        ERR_NO_FIELD => Some(MSG_NO_FIELD),
        ERR_INVALID_SYSREQ => Some(MSG_INVALID_SYSREQ),
        ERR_MANDATORY_ENTRY => Some(MSG_MANDATORY_ENTRY),
        ERR_ALPHA_ONLY => Some(MSG_ALPHA_ONLY),
        ERR_NUMERIC_ONLY => Some(MSG_NUMERIC_ONLY),
        ERR_DIGITS_ONLY => Some(MSG_DIGITS_ONLY),
        ERR_LAST_SIGNED => Some(MSG_LAST_SIGNED),
        ERR_NO_ROOM => Some(MSG_NO_ROOM),
        ERR_MANADATORY_FILL => Some(MSG_MANADATORY_FILL),
        ERR_CHECK_DIGIT => Some(MSG_CHECK_DIGIT),
        ERR_NOT_SIGNED => Some(MSG_NOT_SIGNED),
        ERR_EXIT_NOT_VALID => Some(MSG_EXIT_NOT_VALID),
        ERR_DUP_NOT_ENABLED => Some(MSG_DUP_NOT_ENABLED),
        ERR_NO_FIELD_EXIT => Some(MSG_NO_FIELD_EXIT),
        ERR_NO_INPUT => Some(MSG_NO_INPUT),
        ERR_BAD_CHAR => Some(MSG_BAD_CHAR),
        ERR_DBCS_WRONG_TYPE => Some(MSG_DBCS_WRONG_TYPE),
        ERR_SBCS_WRONG_TYPE => Some(MSG_SBCS_WRONG_TYPE),
        _ => None,
    }
}

/// Enum representation of 5250 protocol commands for type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandCode {
    ClearUnit = CMD_CLEAR_UNIT as isize,
    ClearUnitAlternate = CMD_CLEAR_UNIT_ALTERNATE as isize,
    ClearFormatTable = CMD_CLEAR_FORMAT_TABLE as isize,
    WriteToDisplay = CMD_WRITE_TO_DISPLAY as isize,
    WriteErrorCode = CMD_WRITE_ERROR_CODE as isize,
    WriteErrorCodeWindow = CMD_WRITE_ERROR_CODE_WINDOW as isize,
    ReadInputFields = CMD_READ_INPUT_FIELDS as isize,
    ReadMdtFields = CMD_READ_MDT_FIELDS as isize,
    ReadMdtFieldsAlt = CMD_READ_MDT_FIELDS_ALT as isize,
    ReadScreenImmediate = CMD_READ_SCREEN_IMMEDIATE as isize,
    ReadScreenExtended = CMD_READ_SCREEN_EXTENDED as isize,
    ReadScreenPrint = CMD_READ_SCREEN_PRINT as isize,
    ReadScreenPrintExtended = CMD_READ_SCREEN_PRINT_EXTENDED as isize,
    ReadScreenPrintGrid = CMD_READ_SCREEN_PRINT_GRID as isize,
    ReadScreenPrintExtGrid = CMD_READ_SCREEN_PRINT_EXT_GRID as isize,
    ReadImmediate = CMD_READ_IMMEDIATE as isize,
    ReadImmediateAlt = CMD_READ_IMMEDIATE_ALT as isize,
    SaveScreen = CMD_SAVE_SCREEN as isize,
    SavePartialScreen = CMD_SAVE_PARTIAL_SCREEN as isize,
    RestoreScreen = CMD_RESTORE_SCREEN as isize,
    RestorePartialScreen = CMD_RESTORE_PARTIAL_SCREEN as isize,
    Roll = CMD_ROLL as isize,
    WriteStructuredField = CMD_WRITE_STRUCTURED_FIELD as isize,
}

impl CommandCode {
    /// Convert a byte value to a CommandCode enum
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            CMD_CLEAR_UNIT => Some(Self::ClearUnit),
            CMD_CLEAR_UNIT_ALTERNATE => Some(Self::ClearUnitAlternate),
            CMD_CLEAR_FORMAT_TABLE => Some(Self::ClearFormatTable),
            CMD_WRITE_TO_DISPLAY => Some(Self::WriteToDisplay),
            CMD_WRITE_ERROR_CODE => Some(Self::WriteErrorCode),
            CMD_WRITE_ERROR_CODE_WINDOW => Some(Self::WriteErrorCodeWindow),
            CMD_READ_INPUT_FIELDS => Some(Self::ReadInputFields),
            CMD_READ_MDT_FIELDS => Some(Self::ReadMdtFields),
            CMD_READ_MDT_FIELDS_ALT => Some(Self::ReadMdtFieldsAlt),
            CMD_READ_SCREEN_IMMEDIATE => Some(Self::ReadScreenImmediate),
            CMD_READ_SCREEN_EXTENDED => Some(Self::ReadScreenExtended),
            CMD_READ_SCREEN_PRINT => Some(Self::ReadScreenPrint),
            CMD_READ_SCREEN_PRINT_EXTENDED => Some(Self::ReadScreenPrintExtended),
            CMD_READ_SCREEN_PRINT_GRID => Some(Self::ReadScreenPrintGrid),
            CMD_READ_SCREEN_PRINT_EXT_GRID => Some(Self::ReadScreenPrintExtGrid),
            CMD_READ_IMMEDIATE => Some(Self::ReadImmediate),
            CMD_READ_IMMEDIATE_ALT => Some(Self::ReadImmediateAlt),
            CMD_SAVE_SCREEN => Some(Self::SaveScreen),
            CMD_SAVE_PARTIAL_SCREEN => Some(Self::SavePartialScreen),
            CMD_RESTORE_SCREEN => Some(Self::RestoreScreen),
            CMD_RESTORE_PARTIAL_SCREEN => Some(Self::RestorePartialScreen),
            CMD_ROLL => Some(Self::Roll),
            CMD_WRITE_STRUCTURED_FIELD => Some(Self::WriteStructuredField),
            _ => None,
        }
    }

    /// Convert CommandCode enum to byte value
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Enum representation of 5250 protocol orders
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderCode {
    StartOfHeader = SOH as isize,
    RepeatToAddress = RA as isize,
    EraseToAddress = EA as isize,
    SetBufferAddress = SBA as isize,
    InsertCursor = IC as isize,
    StartOfField = SF as isize,
}

impl OrderCode {
    /// Convert a byte value to an OrderCode enum
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            SOH => Some(Self::StartOfHeader),
            RA => Some(Self::RepeatToAddress),
            EA => Some(Self::EraseToAddress),
            SBA => Some(Self::SetBufferAddress),
            IC => Some(Self::InsertCursor),
            SF => Some(Self::StartOfField),
            _ => None,
        }
    }

    /// Convert OrderCode enum to byte value
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_code_conversion() {
        assert_eq!(CommandCode::from_u8(CMD_WRITE_TO_DISPLAY), Some(CommandCode::WriteToDisplay));
        assert_eq!(CommandCode::WriteToDisplay.to_u8(), CMD_WRITE_TO_DISPLAY);
        assert_eq!(CommandCode::from_u8(0xFF), None);
    }

    #[test]
    fn test_order_code_conversion() {
        assert_eq!(OrderCode::from_u8(SOH), Some(OrderCode::StartOfHeader));
        assert_eq!(OrderCode::StartOfHeader.to_u8(), SOH);
        assert_eq!(OrderCode::from_u8(0xFF), None);
    }

    #[test]
    fn test_error_messages() {
        assert_eq!(get_error_message(ERR_NO_FIELD), Some(MSG_NO_FIELD));
        assert_eq!(get_error_message(0xFF), None);
    }

    #[test]
    fn test_field_attributes() {
        assert_eq!(ATTR_5250_NORMAL, ATTR_5250_GREEN);
        assert_eq!(ATTR_5250_GREEN, 0x20);
    }
}