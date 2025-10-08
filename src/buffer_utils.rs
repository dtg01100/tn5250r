//! Terminal buffer iteration utilities for TN5250R
//!
//! This module provides efficient iterators and utilities for working with
//! terminal screen buffers, eliminating duplicated buffer iteration patterns.

use crate::terminal::{TERMINAL_WIDTH, TERMINAL_HEIGHT, TerminalChar};

/// Iterator over terminal screen positions
/// 
/// Provides efficient iteration over (x, y, buffer_index) tuples for
/// the entire terminal screen or a specified region.
#[allow(dead_code)]
pub struct TerminalPositionIterator {
    current_x: usize,
    current_y: usize,
    start_x: usize,
    start_y: usize,
    end_x: usize,
    end_y: usize,
}

impl TerminalPositionIterator {
    /// Create iterator for the entire terminal screen
    pub fn full_screen() -> Self {
        Self {
            current_x: 0,
            current_y: 0,
            start_x: 0,
            start_y: 0,
            end_x: TERMINAL_WIDTH,
            end_y: TERMINAL_HEIGHT,
        }
    }
    
    /// Create iterator for a specific region
    /// 
    /// # Arguments
    /// * `start_x` - Starting column (0-based)
    /// * `start_y` - Starting row (0-based)
    /// * `end_x` - Ending column (exclusive)
    /// * `end_y` - Ending row (exclusive)
    pub fn region(start_x: usize, start_y: usize, end_x: usize, end_y: usize) -> Self {
        let bounded_end_x = end_x.min(TERMINAL_WIDTH);
        let bounded_end_y = end_y.min(TERMINAL_HEIGHT);
        let bounded_start_x = start_x.min(bounded_end_x);
        let bounded_start_y = start_y.min(bounded_end_y);
        
        Self {
            current_x: bounded_start_x,
            current_y: bounded_start_y,
            start_x: bounded_start_x,
            start_y: bounded_start_y,
            end_x: bounded_end_x,
            end_y: bounded_end_y,
        }
    }
    
    /// Create iterator for a single row
    pub fn row(y: usize) -> Self {
        if y >= TERMINAL_HEIGHT {
            // Return empty iterator for invalid row
            Self::region(0, 0, 0, 0)
        } else {
            Self::region(0, y, TERMINAL_WIDTH, y + 1)
        }
    }
    
    /// Create iterator for a single column
    pub fn column(x: usize) -> Self {
        if x >= TERMINAL_WIDTH {
            // Return empty iterator for invalid column
            Self::region(0, 0, 0, 0)
        } else {
            Self::region(x, 0, x + 1, TERMINAL_HEIGHT)
        }
    }
}

impl Iterator for TerminalPositionIterator {
    type Item = (usize, usize, usize);
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_y >= self.end_y {
            return None;
        }
        
        let x = self.current_x;
        let y = self.current_y;
        let buffer_index = crate::terminal::TerminalScreen::buffer_index(x, y);
        
        // Advance to next position
        self.current_x += 1;
        if self.current_x >= self.end_x {
            self.current_x = self.start_x;
            self.current_y += 1;
        }
        
        Some((x, y, buffer_index))
    }
}

/// Terminal buffer utilities for common operations
pub struct TerminalBufferUtils;

impl TerminalBufferUtils {
    /// Clear a buffer region by setting all characters to default
    /// 
    /// # Arguments
    /// * `buffer` - Mutable reference to the terminal buffer
    /// * `start_x` - Starting column (0-based)
    /// * `start_y` - Starting row (0-based)
    /// * `end_x` - Ending column (exclusive)
    /// * `end_y` - Ending row (exclusive)
    pub fn clear_region(
        buffer: &mut [TerminalChar],
        start_x: usize,
        start_y: usize,
        end_x: usize,
        end_y: usize
    ) {
        for (_, _, index) in TerminalPositionIterator::region(start_x, start_y, end_x, end_y) {
            if index < buffer.len() {
                buffer[index] = TerminalChar::default();
            }
        }
    }
    
    /// Fill a buffer region with a specific character
    /// 
    /// # Arguments
    /// * `buffer` - Mutable reference to the terminal buffer
    /// * `start_x` - Starting column (0-based)
    /// * `start_y` - Starting row (0-based)
    /// * `end_x` - Ending column (exclusive)
    /// * `end_y` - Ending row (exclusive)
    /// * `fill_char` - The character to fill with
    pub fn fill_region(
        buffer: &mut [TerminalChar],
        start_x: usize,
        start_y: usize,
        end_x: usize,
        end_y: usize,
        fill_char: TerminalChar
    ) {
        for (_, _, index) in TerminalPositionIterator::region(start_x, start_y, end_x, end_y) {
            if index < buffer.len() {
                buffer[index] = fill_char;
            }
        }
    }
    
    /// Copy data from one buffer to another within a region
    /// 
    /// # Arguments
    /// * `src_buffer` - Source buffer to copy from
    /// * `dst_buffer` - Destination buffer to copy to
    /// * `start_x` - Starting column (0-based)
    /// * `start_y` - Starting row (0-based)
    /// * `end_x` - Ending column (exclusive)
    /// * `end_y` - Ending row (exclusive)
    pub fn copy_region(
        src_buffer: &[TerminalChar],
        dst_buffer: &mut [TerminalChar],
        start_x: usize,
        start_y: usize,
        end_x: usize,
        end_y: usize
    ) {
        for (_, _, index) in TerminalPositionIterator::region(start_x, start_y, end_x, end_y) {
            if index < src_buffer.len() && index < dst_buffer.len() {
                dst_buffer[index] = src_buffer[index];
            }
        }
    }
    
    /// Save buffer region to a 2D array
    /// 
    /// # Arguments
    /// * `buffer` - Source buffer to save from
    /// * `start_x` - Starting column (0-based)
    /// * `start_y` - Starting row (0-based)
    /// * `end_x` - Ending column (exclusive)
    /// * `end_y` - Ending row (exclusive)
    /// 
    /// # Returns
    /// A 2D array containing the saved region
    pub fn save_region_to_array(
        buffer: &[TerminalChar],
        start_x: usize,
        start_y: usize,
        end_x: usize,
        end_y: usize
    ) -> Vec<Vec<TerminalChar>> {
        let width = end_x.saturating_sub(start_x).min(TERMINAL_WIDTH);
        let height = end_y.saturating_sub(start_y).min(TERMINAL_HEIGHT);
        let mut result = vec![vec![TerminalChar::default(); width]; height];
        
        for (x, y, index) in TerminalPositionIterator::region(start_x, start_y, end_x, end_y) {
            if index < buffer.len() {
                let rel_x = x - start_x;
                let rel_y = y - start_y;
                if rel_y < result.len() && rel_x < result[rel_y].len() {
                    result[rel_y][rel_x] = buffer[index];
                }
            }
        }
        
        result
    }
    
    /// Restore buffer region from a 2D array
    /// 
    /// # Arguments
    /// * `buffer` - Destination buffer to restore to
    /// * `saved_data` - 2D array containing the saved data
    /// * `start_x` - Starting column (0-based)
    /// * `start_y` - Starting row (0-based)
    pub fn restore_region_from_array(
        buffer: &mut [TerminalChar],
        saved_data: &[Vec<TerminalChar>],
        start_x: usize,
        start_y: usize
    ) {
        for (row_idx, row) in saved_data.iter().enumerate() {
            let y = start_y + row_idx;
            if y >= TERMINAL_HEIGHT {
                break;
            }
            
            for (col_idx, &ch) in row.iter().enumerate() {
                let x = start_x + col_idx;
                if x >= TERMINAL_WIDTH {
                    break;
                }
                
                let index = crate::terminal::TerminalScreen::buffer_index(x, y);
                if index < buffer.len() {
                    buffer[index] = ch;
                }
            }
        }
    }
    
    /// Count non-default characters in a region
    /// 
    /// # Arguments
    /// * `buffer` - Buffer to analyze
    /// * `start_x` - Starting column (0-based)
    /// * `start_y` - Starting row (0-based)
    /// * `end_x` - Ending column (exclusive)
    /// * `end_y` - Ending row (exclusive)
    /// 
    /// # Returns
    /// Count of non-default characters in the region
    pub fn count_non_default_chars(
        buffer: &[TerminalChar],
        start_x: usize,
        start_y: usize,
        end_x: usize,
        end_y: usize
    ) -> usize {
        let default_char = TerminalChar::default();
        let mut count = 0;
        
        for (_, _, index) in TerminalPositionIterator::region(start_x, start_y, end_x, end_y) {
            if index < buffer.len() && buffer[index] != default_char {
                count += 1;
            }
        }
        
        count
    }
}

/// Extension trait for TerminalScreen to add iterator methods
pub trait TerminalScreenIterExt {
    /// Iterate over all positions in the terminal
    fn iter_positions(&self) -> TerminalPositionIterator {
        TerminalPositionIterator::full_screen()
    }
    
    /// Iterate over positions in a specific region
    fn iter_region(&self, start_x: usize, start_y: usize, end_x: usize, end_y: usize) -> TerminalPositionIterator {
        TerminalPositionIterator::region(start_x, start_y, end_x, end_y)
    }
    
    /// Iterate over positions in a specific row
    fn iter_row(&self, y: usize) -> TerminalPositionIterator {
        TerminalPositionIterator::row(y)
    }
    
    /// Iterate over positions in a specific column
    fn iter_column(&self, x: usize) -> TerminalPositionIterator {
        TerminalPositionIterator::column(x)
    }
}

// Implement the trait for TerminalScreen
impl TerminalScreenIterExt for crate::terminal::TerminalScreen {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_screen_iterator() {
        let positions: Vec<_> = TerminalPositionIterator::full_screen().collect();
        assert_eq!(positions.len(), TERMINAL_WIDTH * TERMINAL_HEIGHT);
        
        // Check first position
        assert_eq!(positions[0], (0, 0, 0));
        
        // Check last position
        let last_x = TERMINAL_WIDTH - 1;
        let last_y = TERMINAL_HEIGHT - 1;
        let last_index = crate::terminal::TerminalScreen::buffer_index(last_x, last_y);
        assert_eq!(positions.last(), Some(&(last_x, last_y, last_index)));
    }
    
    #[test]
    fn test_region_iterator() {
        let positions: Vec<_> = TerminalPositionIterator::region(1, 1, 3, 3).collect();
        assert_eq!(positions.len(), 4); // 2x2 region
        
        assert_eq!(positions[0].0, 1); // x
        assert_eq!(positions[0].1, 1); // y
        assert_eq!(positions[1], (2, 1, crate::terminal::TerminalScreen::buffer_index(2, 1)));
        assert_eq!(positions[2], (1, 2, crate::terminal::TerminalScreen::buffer_index(1, 2)));
        assert_eq!(positions[3], (2, 2, crate::terminal::TerminalScreen::buffer_index(2, 2)));
    }
    
    #[test]
    fn test_row_iterator() {
        let positions: Vec<_> = TerminalPositionIterator::row(5).collect();
        assert_eq!(positions.len(), TERMINAL_WIDTH);
        
        // All positions should be in row 5
        for (_, y, _) in positions {
            assert_eq!(y, 5);
        }
    }
    
    #[test]
    fn test_column_iterator() {
        let positions: Vec<_> = TerminalPositionIterator::column(10).collect();
        assert_eq!(positions.len(), TERMINAL_HEIGHT);
        
        // All positions should be in column 10
        for (x, _, _) in positions {
            assert_eq!(x, 10);
        }
    }
    
    #[test]
    fn test_invalid_region_bounds() {
        // Out of bounds region should be clamped
        let positions: Vec<_> = TerminalPositionIterator::region(
            TERMINAL_WIDTH - 1, 
            TERMINAL_HEIGHT - 1, 
            TERMINAL_WIDTH + 10, 
            TERMINAL_HEIGHT + 10
        ).collect();
        assert_eq!(positions.len(), 1); // Only one valid position
    }
    
    #[test]
    fn test_invalid_row_column() {
        // Invalid row should return empty iterator
        let positions: Vec<_> = TerminalPositionIterator::row(TERMINAL_HEIGHT + 1).collect();
        assert_eq!(positions.len(), 0);
        
        // Invalid column should return empty iterator
        let positions: Vec<_> = TerminalPositionIterator::column(TERMINAL_WIDTH + 1).collect();
        assert_eq!(positions.len(), 0);
    }
}