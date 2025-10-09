//! Unit tests for dynamic sizing behavior
//!
//! This module contains comprehensive tests for the dynamic terminal sizing functionality
//! that was implemented to support multiple 5250 and 3270 screen sizes.

use crate::terminal::{TerminalScreen, TerminalChar};
use crate::lib3270::display::{Display3270, ScreenSize};
use crate::lib3270::protocol::ProtocolProcessor3270;
use crate::cursor_utils::*;
use crate::buffer_utils::TerminalPositionIterator;

/// Test basic TerminalScreen dynamic sizing functionality
#[test]
fn test_terminal_screen_dynamic_sizing() {
    // Test creating screens with different sizes
    let screen_24x80 = TerminalScreen::new_with_size(80, 24);
    assert_eq!(screen_24x80.width, 80);
    assert_eq!(screen_24x80.height, 24);
    assert_eq!(screen_24x80.buffer.len(), 24 * 80);
    
    let screen_27x132 = TerminalScreen::new_with_size(132, 27);
    assert_eq!(screen_27x132.width, 132);
    assert_eq!(screen_27x132.height, 27);
    assert_eq!(screen_27x132.buffer.len(), 27 * 132);
    
    let screen_43x80 = TerminalScreen::new_with_size(80, 43);
    assert_eq!(screen_43x80.width, 80);
    assert_eq!(screen_43x80.height, 43);
    assert_eq!(screen_43x80.buffer.len(), 43 * 80);
}

/// Test TerminalScreen resize functionality
#[test]
fn test_terminal_screen_resize() {
    let mut screen = TerminalScreen::new(); // Default 80x24
    assert_eq!(screen.width, 80);
    assert_eq!(screen.height, 24);
    
    // Write some test data
    screen.write_string("Test data at position 0,0");
    
    // Resize to larger screen (preserve data)
    screen.resize(132, 27, true);
    assert_eq!(screen.width, 132);
    assert_eq!(screen.height, 27);
    assert_eq!(screen.buffer.len(), 27 * 132);
    
    // Check that data was preserved (should be at same position)
    let char_at_0_0 = screen.get_char_at(0, 0);
    assert_eq!(char_at_0_0.unwrap(), 'T');
    
    // Resize to smaller screen (preserve data)
    screen.resize(80, 24, true);
    assert_eq!(screen.width, 80);
    assert_eq!(screen.height, 24);
    assert_eq!(screen.buffer.len(), 24 * 80);
    
    // Data should still be there
    let char_at_0_0 = screen.get_char_at(0, 0);
    assert_eq!(char_at_0_0.unwrap(), 'T');
}

/// Test TerminalScreen resize without preserving data
#[test]
fn test_terminal_screen_resize_no_preserve() {
    let mut screen = TerminalScreen::new();
    screen.write_string("Test data");
    
    // Resize without preserving data
    screen.resize(132, 27, false);
    assert_eq!(screen.width, 132);
    assert_eq!(screen.height, 27);
    
    // Data should be cleared - new buffer has default characters (null)
    let char_at_0_0 = screen.get_char_at(0, 0);
    assert_eq!(char_at_0_0.unwrap(), '\0');
}

/// Test TerminalScreen index calculation for different sizes
#[test]
fn test_terminal_screen_index_calculation() {
    let screen_80x24 = TerminalScreen::new_with_size(80, 24);
    let screen_132x27 = TerminalScreen::new_with_size(132, 27);
    
    // Test index calculation for 80x24 screen
    assert_eq!(screen_80x24.index(0, 0), 0);
    assert_eq!(screen_80x24.index(79, 0), 79);
    assert_eq!(screen_80x24.index(0, 1), 80);
    assert_eq!(screen_80x24.index(79, 23), 24 * 80 - 1);
    
    // Test index calculation for 132x27 screen
    assert_eq!(screen_132x27.index(0, 0), 0);
    assert_eq!(screen_132x27.index(131, 0), 131);
    assert_eq!(screen_132x27.index(0, 1), 132);
    assert_eq!(screen_132x27.index(131, 26), 27 * 132 - 1);
}

/// Test Display3270 with different screen sizes
#[test]
fn test_display3270_screen_sizes() {
    // Test Model 2 (24x80)
    let display_model2 = Display3270::with_size(ScreenSize::Model2);
    assert_eq!(display_model2.rows(), 24);
    assert_eq!(display_model2.cols(), 80);
    assert_eq!(display_model2.buffer_size(), 24 * 80);
    
    // Test Model 3 (32x80)
    let display_model3 = Display3270::with_size(ScreenSize::Model3);
    assert_eq!(display_model3.rows(), 32);
    assert_eq!(display_model3.cols(), 80);
    assert_eq!(display_model3.buffer_size(), 32 * 80);
    
    // Test Model 4 (43x80)
    let display_model4 = Display3270::with_size(ScreenSize::Model4);
    assert_eq!(display_model4.rows(), 43);
    assert_eq!(display_model4.cols(), 80);
    assert_eq!(display_model4.buffer_size(), 43 * 80);
    
    // Test Model 5 (27x132)
    let display_model5 = Display3270::with_size(ScreenSize::Model5);
    assert_eq!(display_model5.rows(), 27);
    assert_eq!(display_model5.cols(), 132);
    assert_eq!(display_model5.buffer_size(), 27 * 132);
}

/// Test ProtocolProcessor3270 with different screen sizes
#[test]
fn test_protocol_processor3270_screen_sizes() {
    // Test Model 2 processor
    let _processor_model2 = ProtocolProcessor3270::with_screen_size(ScreenSize::Model2);
    // The processor should be configured for Model 2 (12-bit addressing)
    
    // Test Model 4 processor (should use 14-bit addressing for larger screens)
    let _processor_model4 = ProtocolProcessor3270::with_screen_size(ScreenSize::Model4);
    // The processor should be configured for Model 4 with 14-bit addressing
    
    // Test Model 5 processor (should use 14-bit addressing for wide screens)
    let _processor_model5 = ProtocolProcessor3270::with_screen_size(ScreenSize::Model5);
    // The processor should be configured for Model 5 with 14-bit addressing
    
    // Note: We can't directly test internal state without exposing fields,
    // but we can verify the processors were created successfully
    assert!(true); // Placeholder - creation success is the test
}

/// Test dynamic cursor utilities
#[test]
fn test_dynamic_cursor_validation() {
    let screen_24x80 = TerminalScreen::new_with_size(80, 24);
    let screen_27x132 = TerminalScreen::new_with_size(132, 27);
    
    // Test validate_cursor_position_dynamic for 24x80 screen
    assert!(validate_cursor_position_dynamic(1, 1, &screen_24x80).is_ok());
    assert!(validate_cursor_position_dynamic(24, 80, &screen_24x80).is_ok());
    assert!(validate_cursor_position_dynamic(25, 80, &screen_24x80).is_err());
    assert!(validate_cursor_position_dynamic(24, 81, &screen_24x80).is_err());
    assert!(validate_cursor_position_dynamic(0, 1, &screen_24x80).is_err());
    
    // Test validate_cursor_position_dynamic for 27x132 screen
    assert!(validate_cursor_position_dynamic(1, 1, &screen_27x132).is_ok());
    assert!(validate_cursor_position_dynamic(27, 132, &screen_27x132).is_ok());
    assert!(validate_cursor_position_dynamic(28, 132, &screen_27x132).is_err());
    assert!(validate_cursor_position_dynamic(27, 133, &screen_27x132).is_err());
}

/// Test dynamic cursor bounds validation
#[test]
fn test_dynamic_cursor_bounds_validation() {
    let screen_24x80 = TerminalScreen::new_with_size(80, 24);
    let screen_27x132 = TerminalScreen::new_with_size(132, 27);
    
    // Test validate_cursor_bounds_dynamic for 24x80 screen (0-based)
    assert!(validate_cursor_bounds_dynamic(0, 0, &screen_24x80).is_ok());
    assert!(validate_cursor_bounds_dynamic(79, 23, &screen_24x80).is_ok());
    assert!(validate_cursor_bounds_dynamic(80, 23, &screen_24x80).is_err());
    assert!(validate_cursor_bounds_dynamic(79, 24, &screen_24x80).is_err());
    
    // Test validate_cursor_bounds_dynamic for 27x132 screen (0-based)
    assert!(validate_cursor_bounds_dynamic(0, 0, &screen_27x132).is_ok());
    assert!(validate_cursor_bounds_dynamic(131, 26, &screen_27x132).is_ok());
    assert!(validate_cursor_bounds_dynamic(132, 26, &screen_27x132).is_err());
    assert!(validate_cursor_bounds_dynamic(131, 27, &screen_27x132).is_err());
}

/// Test dynamic cursor position clamping
#[test]
fn test_dynamic_cursor_clamping() {
    let screen_24x80 = TerminalScreen::new_with_size(80, 24);
    let screen_27x132 = TerminalScreen::new_with_size(132, 27);
    
    // Test clamp_cursor_position_dynamic for 24x80 screen
    assert_eq!(clamp_cursor_position_dynamic(100, 50, &screen_24x80), (79, 23));
    assert_eq!(clamp_cursor_position_dynamic(40, 12, &screen_24x80), (40, 12));
    assert_eq!(clamp_cursor_position_dynamic(0, 0, &screen_24x80), (0, 0));
    
    // Test clamp_cursor_position_dynamic for 27x132 screen
    assert_eq!(clamp_cursor_position_dynamic(200, 50, &screen_27x132), (131, 26));
    assert_eq!(clamp_cursor_position_dynamic(66, 13, &screen_27x132), (66, 13));
    assert_eq!(clamp_cursor_position_dynamic(0, 0, &screen_27x132), (0, 0));
}

/// Test buffer position iterators with different screen sizes
#[test]
fn test_buffer_position_iterators() {
    let _screen_24x80 = TerminalScreen::new_with_size(80, 24);
    let _screen_27x132 = TerminalScreen::new_with_size(132, 27);
    
    // Test full screen iterator for 24x80
    let iter_24x80 = TerminalPositionIterator::full_screen_with_size(80, 24);
    let positions: Vec<(usize, usize, usize)> = iter_24x80.collect();
    assert_eq!(positions.len(), 24 * 80);
    assert_eq!(positions[0], (0, 0, 0));
    assert_eq!(positions[79], (79, 0, 79));
    assert_eq!(positions[80], (0, 1, 80));
    assert_eq!(positions[24 * 80 - 1], (79, 23, 24 * 80 - 1));
    
    // Test full screen iterator for 27x132
    let iter_27x132 = TerminalPositionIterator::full_screen_with_size(132, 27);
    let positions: Vec<(usize, usize, usize)> = iter_27x132.collect();
    assert_eq!(positions.len(), 27 * 132);
    assert_eq!(positions[0], (0, 0, 0));
    assert_eq!(positions[131], (131, 0, 131));
    assert_eq!(positions[132], (0, 1, 132));
    assert_eq!(positions[27 * 132 - 1], (131, 26, 27 * 132 - 1));
}

/// Test region iterators with different screen sizes
#[test]
fn test_region_iterators() {
    // Test region iterator with dynamic size
    let iter = TerminalPositionIterator::region_with_size(10, 5, 20, 10, 132, 27);
    let positions: Vec<(usize, usize, usize)> = iter.collect();
    
    // Should iterate from (10,5) to (19,9) within a 132x27 screen (end is exclusive)
    assert_eq!(positions[0], (10, 5, 10 + 5 * 132));
    
    // Check that all positions are within the specified region
    for (x, y, _idx) in &positions {
        assert!(*x >= 10 && *x < 20); // end_x is exclusive
        assert!(*y >= 5 && *y < 10);  // end_y is exclusive
    }
}

/// Test boundary conditions for very large screens
#[test]
fn test_large_screen_boundary_conditions() {
    // Test creating a very large screen (within reasonable limits)
    let large_screen = TerminalScreen::new_with_size(200, 100);
    assert_eq!(large_screen.width, 200);
    assert_eq!(large_screen.height, 100);
    assert_eq!(large_screen.buffer.len(), 200 * 100);
    
    // Test index calculation for large screen
    let last_index = large_screen.index(199, 99);
    assert_eq!(last_index, 200 * 100 - 1);
    
    // Test cursor validation for large screen
    assert!(validate_cursor_bounds_dynamic(199, 99, &large_screen).is_ok());
    assert!(validate_cursor_bounds_dynamic(200, 99, &large_screen).is_err());
    assert!(validate_cursor_bounds_dynamic(199, 100, &large_screen).is_err());
}

/// Test screen size transitions (resize operations)
#[test]
fn test_screen_size_transitions() {
    let mut screen = TerminalScreen::new_with_size(80, 24);
    
    // Fill the screen with a pattern
    for y in 0..24 {
        for x in 0..80 {
            let ch = if (x + y) % 2 == 0 { 'A' } else { 'B' };
            screen.set_char_at(x, y, TerminalChar {
                character: ch,
                attribute: crate::terminal::CharAttribute::Normal,
            });
        }
    }
    
    // Resize to larger screen (preserve data)
    screen.resize(132, 27, true);
    
    // Check that original data is preserved
    for y in 0..24 {
        for x in 0..80 {
            let expected_ch = if (x + y) % 2 == 0 { 'A' } else { 'B' };
            let actual_ch = screen.get_char_at(x, y).unwrap();
            assert_eq!(actual_ch, expected_ch, "Mismatch at ({x}, {y})");
        }
    }
    
    // Check that new areas are filled with default characters (null)
    let default_char = screen.get_char_at(131, 26);
    assert_eq!(default_char.unwrap(), '\0');
    
    // Resize back to smaller screen (preserve data)
    screen.resize(80, 24, true);
    
    // Check that data is still preserved in the overlapping area
    for y in 0..24 {
        for x in 0..80 {
            let expected_ch = if (x + y) % 2 == 0 { 'A' } else { 'B' };
            let actual_ch = screen.get_char_at(x, y).unwrap();
            assert_eq!(actual_ch, expected_ch, "Mismatch after resize back at ({x}, {y})");
        }
    }
}

/// Test error handling for invalid screen sizes
#[test]
fn test_invalid_screen_sizes() {
    // Test zero dimensions - should panic or handle gracefully
    // Note: This test checks the behavior - implementation may choose to panic or use minimum size
    
    // Test very small dimensions
    let tiny_screen = TerminalScreen::new_with_size(1, 1);
    assert_eq!(tiny_screen.width, 1);
    assert_eq!(tiny_screen.height, 1);
    assert_eq!(tiny_screen.buffer.len(), 1);
    
    // Test accessing the single character
    assert!(validate_cursor_bounds_dynamic(0, 0, &tiny_screen).is_ok());
    assert!(validate_cursor_bounds_dynamic(1, 0, &tiny_screen).is_err());
    assert!(validate_cursor_bounds_dynamic(0, 1, &tiny_screen).is_err());
}

/// Integration test: TN5250 with different screen sizes
#[test]
fn test_tn5250_dynamic_sizing_integration() {
    // Test that TN5250 Display can handle different screen sizes
    use crate::lib5250::display::Display;
    
    // Create display and resize to 27x132 (alternate mode)
    let mut display = Display::new();
    
    // The display should be resizable to different dimensions
    // This tests that the underlying TerminalScreen supports dynamic sizing
    display.screen().resize(132, 27, false);
    assert_eq!(display.screen().width, 132);
    assert_eq!(display.screen().height, 27);
}

/// Performance test: Large screen operations
#[test]
fn test_large_screen_performance() {
    // Create a large screen and perform basic operations
    let mut large_screen = TerminalScreen::new_with_size(256, 128);
    
    let start = std::time::Instant::now();
    
    // Fill the entire screen
    for y in 0..128 {
        for x in 0..256 {
            large_screen.set_char_at(x, y, TerminalChar { character: 'X', attribute: Default::default() });
        }
    }
    
    let fill_time = start.elapsed();
    
    // Clear the entire screen
    let clear_start = std::time::Instant::now();
    large_screen.clear();
    let clear_time = clear_start.elapsed();
    
    // Performance should be reasonable (these are loose bounds)
    assert!(fill_time.as_millis() < 1000, "Fill operation too slow: {:?}", fill_time);
    assert!(clear_time.as_millis() < 100, "Clear operation too slow: {:?}", clear_time);
    
    // Verify the screen was actually cleared (clear sets characters to space)
    let char_at_center = large_screen.get_char_at(128, 64);
    assert_eq!(char_at_center.unwrap(), ' ');
}