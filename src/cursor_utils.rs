//! Cursor position validation utilities for TN5250R terminal emulation
//!
//! This module provides centralized cursor position validation logic to ensure
//! cursor movements stay within terminal bounds and prevent security issues.

use crate::terminal::{TERMINAL_WIDTH, TERMINAL_HEIGHT};

/// Validate cursor position using 1-based coordinates (5250 protocol standard)
/// 
/// # Arguments
/// * `row` - The row position (1-based, 1 <= row <= 24)
/// * `col` - The column position (1-based, 1 <= col <= 80)
/// 
/// # Returns
/// * `Ok(())` if the position is valid
/// * `Err(String)` with error description if invalid
/// 
/// # Examples
/// ```
/// use tn5250r::cursor_utils::validate_cursor_position;
/// 
/// assert!(validate_cursor_position(1, 1).is_ok());     // Top-left corner
/// assert!(validate_cursor_position(24, 80).is_ok());   // Bottom-right corner
/// assert!(validate_cursor_position(0, 1).is_err());    // Invalid row
/// assert!(validate_cursor_position(1, 0).is_err());    // Invalid column
/// assert!(validate_cursor_position(25, 1).is_err());   // Row out of bounds
/// assert!(validate_cursor_position(1, 81).is_err());   // Column out of bounds
/// ```
pub fn validate_cursor_position(row: usize, col: usize) -> Result<(), String> {
    if row == 0 || col == 0 {
        return Err(format!("Invalid cursor position: ({}, {}) - coordinates must be 1-based", row, col));
    }
    
    if row > TERMINAL_HEIGHT {
        return Err(format!("Invalid cursor position: ({}, {}) - row exceeds terminal height ({})", row, col, TERMINAL_HEIGHT));
    }
    
    if col > TERMINAL_WIDTH {
        return Err(format!("Invalid cursor position: ({}, {}) - column exceeds terminal width ({})", row, col, TERMINAL_WIDTH));
    }
    
    Ok(())
}

/// Validate cursor bounds using 0-based coordinates (internal buffer indexing)
/// 
/// # Arguments
/// * `x` - The column position (0-based, 0 <= x < 80)
/// * `y` - The row position (0-based, 0 <= y < 24)
/// 
/// # Returns
/// * `Ok(())` if the position is within bounds
/// * `Err(String)` with error description if out of bounds
/// 
/// # Examples
/// ```
/// use tn5250r::cursor_utils::validate_cursor_bounds;
/// 
/// assert!(validate_cursor_bounds(0, 0).is_ok());      // Top-left corner
/// assert!(validate_cursor_bounds(79, 23).is_ok());    // Bottom-right corner
/// assert!(validate_cursor_bounds(80, 0).is_err());    // Column out of bounds
/// assert!(validate_cursor_bounds(0, 24).is_err());    // Row out of bounds
/// ```
pub fn validate_cursor_bounds(x: usize, y: usize) -> Result<(), String> {
    if x >= TERMINAL_WIDTH {
        return Err(format!("Cursor position exceeds bounds: ({}, {}) - column >= {}", y, x, TERMINAL_WIDTH));
    }
    
    if y >= TERMINAL_HEIGHT {
        return Err(format!("Cursor position exceeds bounds: ({}, {}) - row >= {}", y, x, TERMINAL_HEIGHT));
    }
    
    Ok(())
}

/// Clamp cursor position to valid terminal bounds (0-based coordinates)
/// 
/// This function ensures cursor positions stay within terminal bounds by
/// clamping out-of-bounds values to the nearest valid position.
/// 
/// # Arguments
/// * `x` - The column position to clamp
/// * `y` - The row position to clamp
/// 
/// # Returns
/// A tuple (x, y) with clamped coordinates
/// 
/// # Examples
/// ```
/// use tn5250r::cursor_utils::clamp_cursor_position;
/// 
/// assert_eq!(clamp_cursor_position(100, 50), (79, 23)); // Clamp to max bounds
/// assert_eq!(clamp_cursor_position(40, 12), (40, 12));  // Already valid
/// ```
pub fn clamp_cursor_position(x: usize, y: usize) -> (usize, usize) {
    let clamped_x = x.min(TERMINAL_WIDTH - 1);
    let clamped_y = y.min(TERMINAL_HEIGHT - 1);
    (clamped_x, clamped_y)
}

/// Convert 1-based cursor position to 0-based coordinates
/// 
/// # Arguments
/// * `row` - The row position (1-based)
/// * `col` - The column position (1-based)
/// 
/// # Returns
/// A tuple (x, y) with 0-based coordinates, or None if input is invalid
pub fn cursor_1based_to_0based(row: usize, col: usize) -> Option<(usize, usize)> {
    if row == 0 || col == 0 {
        return None;
    }
    Some((col - 1, row - 1))
}

/// Convert 0-based coordinates to 1-based cursor position
/// 
/// # Arguments
/// * `x` - The column position (0-based)
/// * `y` - The row position (0-based)
/// 
/// # Returns
/// A tuple (row, col) with 1-based coordinates
pub fn cursor_0based_to_1based(x: usize, y: usize) -> (usize, usize) {
    (y + 1, x + 1)
}

/// Log security warning for invalid cursor position attempt
/// 
/// This function provides consistent security logging for cursor validation failures.
/// 
/// # Arguments
/// * `row` - The attempted row position
/// * `col` - The attempted column position
/// * `context` - Additional context for the log message
pub fn log_invalid_cursor_attempt(row: usize, col: usize, context: &str) {
    eprintln!("SECURITY: Invalid cursor position ({}, {}) - {} - out of bounds", row, col, context);
}

/// Log security warning for cursor bounds violation
/// 
/// # Arguments
/// * `x` - The attempted column position (0-based)
/// * `y` - The attempted row position (0-based)
/// * `context` - Additional context for the log message
pub fn log_cursor_bounds_violation(x: usize, y: usize, context: &str) {
    eprintln!("SECURITY: Attempted to access outside terminal bounds at ({}, {}) - {}", y, x, context);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_cursor_position() {
        // Valid positions
        assert!(validate_cursor_position(1, 1).is_ok());
        assert!(validate_cursor_position(24, 80).is_ok());
        assert!(validate_cursor_position(12, 40).is_ok());
        
        // Invalid positions - zero coordinates
        assert!(validate_cursor_position(0, 1).is_err());
        assert!(validate_cursor_position(1, 0).is_err());
        assert!(validate_cursor_position(0, 0).is_err());
        
        // Invalid positions - out of bounds
        assert!(validate_cursor_position(25, 1).is_err());
        assert!(validate_cursor_position(1, 81).is_err());
        assert!(validate_cursor_position(100, 200).is_err());
    }
    
    #[test]
    fn test_validate_cursor_bounds() {
        // Valid bounds
        assert!(validate_cursor_bounds(0, 0).is_ok());
        assert!(validate_cursor_bounds(79, 23).is_ok());
        assert!(validate_cursor_bounds(40, 12).is_ok());
        
        // Invalid bounds
        assert!(validate_cursor_bounds(80, 0).is_err());
        assert!(validate_cursor_bounds(0, 24).is_err());
        assert!(validate_cursor_bounds(100, 200).is_err());
    }
    
    #[test]
    fn test_clamp_cursor_position() {
        assert_eq!(clamp_cursor_position(0, 0), (0, 0));
        assert_eq!(clamp_cursor_position(79, 23), (79, 23));
        assert_eq!(clamp_cursor_position(100, 50), (79, 23));
        assert_eq!(clamp_cursor_position(40, 12), (40, 12));
    }
    
    #[test]
    fn test_coordinate_conversion() {
        // 1-based to 0-based
        assert_eq!(cursor_1based_to_0based(1, 1), Some((0, 0)));
        assert_eq!(cursor_1based_to_0based(24, 80), Some((79, 23)));
        assert_eq!(cursor_1based_to_0based(0, 1), None);
        assert_eq!(cursor_1based_to_0based(1, 0), None);
        
        // 0-based to 1-based
        assert_eq!(cursor_0based_to_1based(0, 0), (1, 1));
        assert_eq!(cursor_0based_to_1based(79, 23), (24, 80));
    }
}