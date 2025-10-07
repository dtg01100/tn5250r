//! Comprehensive error handling for TN5250R
//! 
//! This module provides structured error types, recovery mechanisms, and detailed error reporting
//! for robust production operation of the TN5250R terminal emulator.

use std::fmt;
use std::io;
use std::error::Error as StdError;

/// Top-level error type for TN5250R operations
#[derive(Debug)]
pub enum TN5250Error {
    /// Network connection errors
    Network(NetworkError),
    /// Telnet protocol negotiation errors  
    Telnet(TelnetError),
    /// 5250 protocol parsing errors
    Protocol(ProtocolError),
    /// Terminal emulation errors
    Terminal(TerminalError),
    /// Field management errors
    Field(FieldError),
    /// Buffer management errors
    Buffer(BufferError),
    /// Configuration errors
    Config(ConfigError),
    /// Recovery operation errors
    Recovery(RecoveryError),
}

/// Network connection related errors
#[derive(Debug)]
pub enum NetworkError {
    /// Connection refused by remote host
    ConnectionRefused { host: String, port: u16 },
    /// Connection timeout
    Timeout { host: String, port: u16, timeout_seconds: u64 },
    /// DNS resolution failure
    DnsResolution { host: String },
    /// Network unreachable
    NetworkUnreachable { host: String },
    /// Connection lost during operation
    ConnectionLost { reason: String },
    /// Invalid network address
    InvalidAddress { address: String },
    /// SSL/TLS errors for secure connections
    SslError { message: String },
}

/// Telnet protocol negotiation errors
#[derive(Debug)]
pub enum TelnetError {
    /// Invalid IAC command sequence
    InvalidCommand { command: Vec<u8> },
    /// Negotiation timeout
    NegotiationTimeout { option: u8, timeout_ms: u64 },
    /// Option negotiation failed
    OptionNegotiationFailed { option: u8, reason: String },
    /// Required option not supported by server
    RequiredOptionUnsupported { option: u8 },
    /// Malformed subnegotiation data
    MalformedSubnegotiation { option: u8, data: Vec<u8> },
    /// Protocol state machine error
    StateMachineError { current_state: String, invalid_transition: String },
    /// Buffer pool exhaustion during negotiation
    BufferPoolExhausted { pool_type: String },
}

/// 5250 protocol parsing errors
#[derive(Debug)]
pub enum ProtocolError {
    /// Invalid 5250 command code
    InvalidCommandCode { code: u8 },
    /// Incomplete data stream
    IncompleteData { expected: usize, received: usize },
    /// EBCDIC conversion error
    EbcdicConversion { byte: u8, context: String },
    /// Invalid structured field
    InvalidStructuredField { field_id: u8, reason: String },
    /// Cursor positioning error
    InvalidCursorPosition { row: usize, col: usize },
    /// Screen buffer overflow
    ScreenBufferOverflow { position: usize, buffer_size: usize },
    /// Invalid field attributes
    InvalidFieldAttribute { attribute: u8 },
    /// Device identification error
    DeviceIdError { message: String },
    /// Unsupported protocol requested
    UnsupportedProtocol { protocol: String, reason: String },
    /// Protocol mismatch between configured and detected
    ProtocolMismatch { configured: String, detected: String },
    /// Protocol switch operation failed
    ProtocolSwitchFailed { from: String, to: String, reason: String },
    /// Invalid protocol configuration
    InvalidProtocolConfiguration { parameter: String, value: String, reason: String },
}

/// Terminal emulation errors
#[derive(Debug)]
pub enum TerminalError {
    /// Screen size mismatch
    ScreenSizeMismatch { expected: (usize, usize), actual: (usize, usize) },
    /// Character set conversion error
    CharsetConversion { char: char, target_charset: String },
    /// Display rendering error
    DisplayRender { message: String },
    /// Input processing error
    InputProcessing { input: String, reason: String },
    /// Function key mapping error
    FunctionKeyMapping { key_code: u32 },
    /// Terminal state corruption
    StateCorruption { component: String, details: String },
}

/// Field management errors
#[derive(Debug)]
pub enum FieldError {
    /// Field not found at position
    FieldNotFound { row: usize, col: usize },
    /// Invalid field type
    InvalidFieldType { field_type: String },
    /// Field validation failure
    ValidationFailure { field_id: usize, message: String },
    /// Field input out of bounds
    InputOutOfBounds { field_id: usize, input_length: usize, max_length: usize },
    /// Required field empty
    RequiredFieldEmpty { field_id: usize, field_name: String },
    /// Field format error
    FormatError { field_id: usize, expected_format: String, actual_input: String },
    /// Field continuation error
    ContinuationError { field_id: usize, reason: String },
}

/// Buffer management errors
#[derive(Debug)]
pub enum BufferError {
    /// Buffer pool allocation failure
    AllocationFailure { requested_size: usize, pool_type: String },
    /// Buffer size exceeded
    SizeExceeded { size: usize, max_size: usize },
    /// Buffer corruption detected
    Corruption { buffer_id: String, checksum_mismatch: bool },
    /// Concurrent access violation
    ConcurrentAccessViolation { buffer_id: String },
    /// Memory pressure
    MemoryPressure { current_usage: usize, max_usage: usize },
}

/// Configuration errors
#[derive(Debug)]
pub enum ConfigError {
    /// Invalid configuration parameter
    InvalidParameter { parameter: String, value: String, reason: String },
    /// Missing required configuration
    MissingRequired { parameter: String },
    /// Configuration file error
    FileError { path: String, error: String },
    /// Environment variable error
    EnvironmentError { variable: String, error: String },
    /// Version compatibility error
    VersionMismatch { expected: String, actual: String },
}

/// Recovery operation errors
#[derive(Debug)]
pub enum RecoveryError {
    /// Recovery attempt failed
    RecoveryFailed { attempt: u32, max_attempts: u32, reason: String },
    /// Recovery timeout
    RecoveryTimeout { operation: String, timeout_seconds: u64 },
    /// Recovery strategy not available
    StrategyUnavailable { strategy: String, context: String },
    /// Recovery state inconsistent
    StateInconsistent { expected_state: String, actual_state: String },
}

// Implement Display for all error types
impl fmt::Display for TN5250Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TN5250Error::Network(err) => write!(f, "Network error: {err}"),
            TN5250Error::Telnet(err) => write!(f, "Telnet error: {err}"),
            TN5250Error::Protocol(err) => write!(f, "Protocol error: {err}"),
            TN5250Error::Terminal(err) => write!(f, "Terminal error: {err}"),
            TN5250Error::Field(err) => write!(f, "Field error: {err}"),
            TN5250Error::Buffer(err) => write!(f, "Buffer error: {err}"),
            TN5250Error::Config(err) => write!(f, "Configuration error: {err}"),
            TN5250Error::Recovery(err) => write!(f, "Recovery error: {err}"),
        }
    }
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::ConnectionRefused { host, port } => 
                write!(f, "Connection refused to {host}:{port}"),
            NetworkError::Timeout { host, port, timeout_seconds } => 
                write!(f, "Connection timeout to {host}:{port} after {timeout_seconds}s"),
            NetworkError::DnsResolution { host } => 
                write!(f, "DNS resolution failed for {host}"),
            NetworkError::NetworkUnreachable { host } => 
                write!(f, "Network unreachable to {host}"),
            NetworkError::ConnectionLost { reason } => 
                write!(f, "Connection lost: {reason}"),
            NetworkError::InvalidAddress { address } => 
                write!(f, "Invalid network address: {address}"),
            NetworkError::SslError { message } => 
                write!(f, "SSL/TLS error: {message}"),
        }
    }
}

impl fmt::Display for TelnetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TelnetError::InvalidCommand { command } => 
                write!(f, "Invalid telnet command: {command:?}"),
            TelnetError::NegotiationTimeout { option, timeout_ms } => 
                write!(f, "Telnet negotiation timeout for option {option} after {timeout_ms}ms"),
            TelnetError::OptionNegotiationFailed { option, reason } => 
                write!(f, "Option {option} negotiation failed: {reason}"),
            TelnetError::RequiredOptionUnsupported { option } => 
                write!(f, "Required telnet option {option} not supported by server"),
            TelnetError::MalformedSubnegotiation { option, data } => 
                write!(f, "Malformed subnegotiation for option {option}: {data:?}"),
            TelnetError::StateMachineError { current_state, invalid_transition } => 
                write!(f, "State machine error: invalid transition '{invalid_transition}' from state '{current_state}'"),
            TelnetError::BufferPoolExhausted { pool_type } => 
                write!(f, "Buffer pool exhausted: {pool_type}"),
        }
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::InvalidCommandCode { code } =>
                write!(f, "Invalid 5250 command code: 0x{code:02X}"),
            ProtocolError::IncompleteData { expected, received } =>
                write!(f, "Incomplete data: expected {expected} bytes, received {received}"),
            ProtocolError::EbcdicConversion { byte, context } =>
                write!(f, "EBCDIC conversion error for byte 0x{byte:02X} in context: {context}"),
            ProtocolError::InvalidStructuredField { field_id, reason } =>
                write!(f, "Invalid structured field 0x{field_id:02X}: {reason}"),
            ProtocolError::InvalidCursorPosition { row, col } =>
                write!(f, "Invalid cursor position: row {row}, col {col}"),
            ProtocolError::ScreenBufferOverflow { position, buffer_size } =>
                write!(f, "Screen buffer overflow: position {position} exceeds buffer size {buffer_size}"),
            ProtocolError::InvalidFieldAttribute { attribute } =>
                write!(f, "Invalid field attribute: 0x{attribute:02X}"),
            ProtocolError::DeviceIdError { message } =>
                write!(f, "Device identification error: {message}"),
            ProtocolError::UnsupportedProtocol { protocol, reason } =>
                write!(f, "Unsupported protocol '{protocol}': {reason}"),
            ProtocolError::ProtocolMismatch { configured, detected } =>
                write!(f, "Protocol mismatch: configured for '{configured}' but detected '{detected}'"),
            ProtocolError::ProtocolSwitchFailed { from, to, reason } =>
                write!(f, "Failed to switch protocol from '{from}' to '{to}': {reason}"),
            ProtocolError::InvalidProtocolConfiguration { parameter, value, reason } =>
                write!(f, "Invalid protocol configuration: parameter '{parameter}' = '{value}': {reason}"),
        }
    }
}

impl fmt::Display for TerminalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TerminalError::ScreenSizeMismatch { expected, actual } => 
                write!(f, "Screen size mismatch: expected {}x{}, actual {}x{}", expected.0, expected.1, actual.0, actual.1),
            TerminalError::CharsetConversion { char, target_charset } => 
                write!(f, "Character '{char}' cannot be converted to charset {target_charset}"),
            TerminalError::DisplayRender { message } => 
                write!(f, "Display rendering error: {message}"),
            TerminalError::InputProcessing { input, reason } => 
                write!(f, "Input processing error for '{input}': {reason}"),
            TerminalError::FunctionKeyMapping { key_code } => 
                write!(f, "Unknown function key code: {key_code}"),
            TerminalError::StateCorruption { component, details } => 
                write!(f, "Terminal state corruption in {component}: {details}"),
        }
    }
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldError::FieldNotFound { row, col } => 
                write!(f, "No field found at position ({row}, {col})"),
            FieldError::InvalidFieldType { field_type } => 
                write!(f, "Invalid field type: {field_type}"),
            FieldError::ValidationFailure { field_id, message } => 
                write!(f, "Field {field_id} validation failed: {message}"),
            FieldError::InputOutOfBounds { field_id, input_length, max_length } => 
                write!(f, "Field {field_id} input length {input_length} exceeds maximum {max_length}"),
            FieldError::RequiredFieldEmpty { field_id, field_name } => 
                write!(f, "Required field {field_id} '{field_name}' is empty"),
            FieldError::FormatError { field_id, expected_format, actual_input } => 
                write!(f, "Field {field_id} format error: expected '{expected_format}', got '{actual_input}'"),
            FieldError::ContinuationError { field_id, reason } => 
                write!(f, "Field {field_id} continuation error: {reason}"),
        }
    }
}

impl fmt::Display for BufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BufferError::AllocationFailure { requested_size, pool_type } => 
                write!(f, "Failed to allocate {requested_size} bytes from {pool_type} pool"),
            BufferError::SizeExceeded { size, max_size } => 
                write!(f, "Buffer size {size} exceeds maximum {max_size}"),
            BufferError::Corruption { buffer_id, checksum_mismatch } => 
                write!(f, "Buffer corruption detected in {buffer_id}: checksum_mismatch={checksum_mismatch}"),
            BufferError::ConcurrentAccessViolation { buffer_id } => 
                write!(f, "Concurrent access violation on buffer {buffer_id}"),
            BufferError::MemoryPressure { current_usage, max_usage } => 
                write!(f, "Memory pressure: {current_usage} bytes used, {max_usage} bytes maximum"),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::InvalidParameter { parameter, value, reason } => 
                write!(f, "Invalid configuration parameter '{parameter}' = '{value}': {reason}"),
            ConfigError::MissingRequired { parameter } => 
                write!(f, "Missing required configuration parameter: {parameter}"),
            ConfigError::FileError { path, error } => 
                write!(f, "Configuration file error '{path}': {error}"),
            ConfigError::EnvironmentError { variable, error } => 
                write!(f, "Environment variable '{variable}' error: {error}"),
            ConfigError::VersionMismatch { expected, actual } => 
                write!(f, "Version mismatch: expected {expected}, got {actual}"),
        }
    }
}

impl fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecoveryError::RecoveryFailed { attempt, max_attempts, reason } => 
                write!(f, "Recovery failed on attempt {attempt}/{max_attempts}: {reason}"),
            RecoveryError::RecoveryTimeout { operation, timeout_seconds } => 
                write!(f, "Recovery timeout for operation '{operation}' after {timeout_seconds}s"),
            RecoveryError::StrategyUnavailable { strategy, context } => 
                write!(f, "Recovery strategy '{strategy}' unavailable in context: {context}"),
            RecoveryError::StateInconsistent { expected_state, actual_state } => 
                write!(f, "Recovery state inconsistent: expected '{expected_state}', found '{actual_state}'"),
        }
    }
}

// Implement StdError trait
impl StdError for TN5250Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            TN5250Error::Network(err) => Some(err),
            TN5250Error::Telnet(err) => Some(err),
            TN5250Error::Protocol(err) => Some(err),
            TN5250Error::Terminal(err) => Some(err),
            TN5250Error::Field(err) => Some(err),
            TN5250Error::Buffer(err) => Some(err),
            TN5250Error::Config(err) => Some(err),
            TN5250Error::Recovery(err) => Some(err),
        }
    }
}

// Implement StdError for all error types
impl StdError for NetworkError {}
impl StdError for TelnetError {}
impl StdError for ProtocolError {}
impl StdError for TerminalError {}
impl StdError for FieldError {}
impl StdError for BufferError {}
impl StdError for ConfigError {}
impl StdError for RecoveryError {}

// From implementations for easy error conversion
impl From<NetworkError> for TN5250Error {
    fn from(err: NetworkError) -> Self {
        TN5250Error::Network(err)
    }
}

impl From<TelnetError> for TN5250Error {
    fn from(err: TelnetError) -> Self {
        TN5250Error::Telnet(err)
    }
}

impl From<ProtocolError> for TN5250Error {
    fn from(err: ProtocolError) -> Self {
        TN5250Error::Protocol(err)
    }
}

impl From<TerminalError> for TN5250Error {
    fn from(err: TerminalError) -> Self {
        TN5250Error::Terminal(err)
    }
}

impl From<FieldError> for TN5250Error {
    fn from(err: FieldError) -> Self {
        TN5250Error::Field(err)
    }
}

impl From<BufferError> for TN5250Error {
    fn from(err: BufferError) -> Self {
        TN5250Error::Buffer(err)
    }
}

impl From<ConfigError> for TN5250Error {
    fn from(err: ConfigError) -> Self {
        TN5250Error::Config(err)
    }
}

impl From<RecoveryError> for TN5250Error {
    fn from(err: RecoveryError) -> Self {
        TN5250Error::Recovery(err)
    }
}

// Convert from standard IO errors
impl From<io::Error> for TN5250Error {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::ConnectionRefused => TN5250Error::Network(NetworkError::ConnectionRefused {
                host: "unknown".to_string(),
                port: 0,
            }),
            io::ErrorKind::TimedOut => TN5250Error::Network(NetworkError::Timeout {
                host: "unknown".to_string(),
                port: 0,
                timeout_seconds: 30,
            }),
            io::ErrorKind::ConnectionAborted | io::ErrorKind::ConnectionReset => {
                TN5250Error::Network(NetworkError::ConnectionLost {
                    reason: err.to_string(),
                })
            },
            _ => TN5250Error::Network(NetworkError::ConnectionLost {
                reason: format!("IO Error: {err}"),
            }),
        }
    }
}

/// Result type alias for TN5250R operations
pub type TN5250Result<T> = Result<T, TN5250Error>;

/// Specialized result types for different components
pub type NetworkResult<T> = Result<T, NetworkError>;
pub type TelnetResult<T> = Result<T, TelnetError>;
pub type ProtocolResult<T> = Result<T, ProtocolError>;
pub type TerminalResult<T> = Result<T, TerminalError>;
pub type FieldResult<T> = Result<T, FieldError>;
pub type BufferResult<T> = Result<T, BufferError>;
pub type ConfigResult<T> = Result<T, ConfigError>;
pub type RecoveryResult<T> = Result<T, RecoveryError>;