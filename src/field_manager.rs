/// Struct for passing field display info to UI
#[derive(Debug, Clone)]
pub struct FieldDisplayInfo {
    pub label: String,
    pub content: String,
    pub is_active: bool,
    pub error_state: Option<FieldError>,
    pub highlighted: bool,
    pub start_row: usize,
    pub start_col: usize,
    pub length: usize,
}

// Field handling for AS/400 terminal forms
// 
// This module provides functionality for detecting, navigating, and managing
// input fields in AS/400 terminal screens.

use crate::terminal::TerminalScreen;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldType {
    /// Regular input field
    Input,
    /// Password field (hidden input)
    Password,
    /// Numeric only field
    Numeric,
    /// Protected/read-only field
    Protected,
    /// Selection field (dropdown/menu)
    Selection,
    /// Automatically send ENTER when field fills
    AutoEnter,
    /// Must be filled before proceeding
    Mandatory,
    /// Visual highlighting when active
    Highlighted,
    /// Skip during navigation
    Bypass,
    /// Multi-segment field
    Continued,
    /// Signed numeric field
    NumericSigned,
    /// Letters, comma, dash, period, space only
    AlphaOnly,
    /// Digits only (stricter than Numeric)
    DigitsOnly,
    /// Auto-convert to uppercase
    UppercaseOnly,
}

#[derive(Debug, Clone, Default)]
pub struct FieldBehavior {
    /// FER - must use Field Exit key to leave field
    pub field_exit_required: bool,
    /// Auto-send ENTER when field is full
    pub auto_enter: bool,
    /// Required field - must be filled
    pub mandatory: bool,
    /// Skip during navigation
    pub bypass: bool,
    /// Right-justify content on field exit
    pub right_adjust: bool,
    /// Fill with zeros vs spaces
    pub zero_fill: bool,
    /// Auto-convert to uppercase
    pub uppercase_convert: bool,
    /// Allow duplicate field operation
    pub dup_enabled: bool,
    /// Custom next field ID for progression
    pub cursor_progression: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldError {
    /// Input validation errors
    CursorProtected,
    NumericOnly,
    AlphaOnly,
    DigitsOnly,
    InvalidCharacter(char),
    InvalidSignPosition,
    
    /// Field operation errors
    FieldExitRequired,
    FieldExitInvalid,
    MandatoryEnter,
    FieldFull,
    NoRoomForInsert,
    
    /// Navigation errors
    NoActiveField,
    FieldNotFound(usize),
    InvalidFieldNavigation,
}

impl FieldError {
    pub fn get_user_message(&self) -> &'static str {
        match self {
            FieldError::CursorProtected => "Cursor is in protected area",
            FieldError::NumericOnly => "Numeric characters only",
            FieldError::AlphaOnly => "Alphabetic characters only",
            FieldError::DigitsOnly => "Digits only",
            FieldError::InvalidCharacter(_) => "Invalid character for this field",
            FieldError::InvalidSignPosition => "Sign must be at beginning or end",
            FieldError::FieldExitRequired => "Use Field Exit key to leave field",
            FieldError::FieldExitInvalid => "Field Exit not allowed here",
            FieldError::MandatoryEnter => "Required field must be filled",
            FieldError::FieldFull => "Field is full",
            FieldError::NoRoomForInsert => "No room to insert character",
            FieldError::NoActiveField => "No field is currently active",
            FieldError::FieldNotFound(_) => "Field not found",
            FieldError::InvalidFieldNavigation => "Invalid field navigation",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    /// Field identifier
    pub id: usize,
    /// Field type
    pub field_type: FieldType,
    /// Start position (row, col) - 1-based
    pub start_row: usize,
    pub start_col: usize,
    /// Field length (number of characters)
    pub length: usize,
    /// Current content
    pub content: String,
    /// Maximum allowed length
    pub max_length: usize,
    /// Whether the field is currently active
    pub active: bool,
    /// Field label/description if detected
    pub label: Option<String>,
    /// Whether field is required
    pub required: bool,
    /// Field behavior settings
    pub behavior: FieldBehavior,
    /// Unique field ID for progression
    pub field_id: usize,
    /// Custom next field ID
    pub next_field_id: Option<usize>,
    /// Custom previous field ID
    pub prev_field_id: Option<usize>,
    /// Group ID for continued fields
    pub continued_group_id: Option<usize>,
    /// Visual highlighting state
    pub highlighted: bool,
    /// Current error state if any
    pub error_state: Option<FieldError>,
    /// Modified Data Tag (MDT)
    pub modified: bool,
    /// Current cursor position in field
    pub cursor_position: usize,
    /// Tab order for navigation (lower numbers first)
    pub tab_order: usize,
}

impl Field {
    pub fn new(id: usize, field_type: FieldType, row: usize, col: usize, length: usize) -> Self {
        Self {
            id,
            field_type,
            start_row: row,
            start_col: col,
            length,
            content: String::new(),
            max_length: length,
            active: false,
            label: None,
            required: false,
            // New enhanced fields
            behavior: FieldBehavior::default(),
            field_id: id, // Use same as id initially
            next_field_id: None,
            prev_field_id: None,
            continued_group_id: None,
            highlighted: false,
            error_state: None,
            modified: false,
            cursor_position: 0,
            tab_order: id, // Default tab order same as id
        }
    }
    
    /// Check if a position is within this field
    pub fn contains_position(&self, row: usize, col: usize) -> bool {
        row == self.start_row && 
        col >= self.start_col && 
        col < self.start_col + self.length
    }

    /// Set advanced attributes based on protocol attribute byte or heuristics
    pub fn set_enhanced_attributes(&mut self, attribute: u8) {
        // Example mapping, real mapping should use protocol docs
        self.behavior.auto_enter = attribute & 0x80 != 0;
        self.behavior.mandatory = attribute & 0x40 != 0;
        self.behavior.bypass = attribute & 0x08 != 0;
        self.behavior.right_adjust = attribute & 0x04 != 0;
        self.behavior.zero_fill = attribute & 0x02 != 0;
        self.behavior.uppercase_convert = attribute & 0x01 != 0;
        self.highlighted = attribute & 0x20 != 0;
        // Continued field grouping (example: attribute & 0x10)
        if attribute & 0x10 != 0 {
            self.continued_group_id = Some(self.id); // simplistic, real impl should group
        }
    }
    
    /// Get cursor position within the field (0-based offset)
    pub fn get_cursor_offset(&self, row: usize, col: usize) -> Option<usize> {
        if self.contains_position(row, col) {
            Some(col - self.start_col)
        } else {
            None
        }
    }
    
    /// Insert character at current cursor position
    /// SECURITY: Enhanced with comprehensive input sanitization and bounds checking
    pub fn insert_char(&mut self, ch: char, offset: usize) -> Result<bool, FieldError> {
        // Clear any previous errors
        self.clear_error();

        // CRITICAL FIX: Enhanced character validation with multiple security checks
        if !self.is_character_safe(ch) {
            let error = FieldError::InvalidCharacter(ch);
            self.set_error(error.clone());
            return Err(error);
        }

        // Validate character input based on field type
        if let Err(error) = self.validate_character(ch) {
            self.set_error(error.clone());
            return Err(error);
        }

        // CRITICAL FIX: Enhanced offset validation with better bounds checking
        if offset > self.max_length {
            eprintln!("SECURITY: Invalid offset {} exceeds field max length {}", offset, self.max_length);
            let error = FieldError::NoRoomForInsert;
            self.set_error(error.clone());
            return Err(error);
        }

        // Check length limits with safety margin
        if self.content.len() >= self.max_length {
            let error = FieldError::FieldFull;
            self.set_error(error.clone());
            return Err(error);
        }

        // Check if there's room to insert (enhanced validation)
        if offset > self.content.len() {
            let error = FieldError::NoRoomForInsert;
            self.set_error(error.clone());
            return Err(error);
        }

        // CRITICAL FIX: Additional validation for edge cases
        if self.content.len() + 1 > self.max_length {
            let error = FieldError::FieldFull;
            self.set_error(error.clone());
            return Err(error);
        }

        // SECURITY: Sanitize character before insertion
        let sanitized_ch = self.sanitize_character(ch);

        // Insert character at the specified offset
        self.content.insert(offset, sanitized_ch);
        self.modified = true;

        // Apply transformations if needed (with bounds checking)
        if self.field_type == FieldType::UppercaseOnly || self.behavior.uppercase_convert {
            if offset < self.content.len() {
                if let Some(last_char) = self.content.chars().nth(offset) {
                    let upper_char = last_char.to_uppercase().collect::<String>();
                    if upper_char.len() == 1 && upper_char != last_char.to_string() {
                        self.content.remove(offset);
                        self.content.insert_str(offset, &upper_char);
                    }
                }
            }
        }

        Ok(true)
    }
    
    /// Delete character at offset
    pub fn delete_char(&mut self, offset: usize) -> bool {
        if self.field_type == FieldType::Protected {
            return false;
        }
        
        if offset < self.content.len() {
            self.content.remove(offset);
            true
        } else {
            false
        }
    }
    
    /// Backspace at offset
    pub fn backspace(&mut self, offset: usize) -> bool {
        if self.field_type == FieldType::Protected {
            return false;
        }
        
        if offset > 0 && offset <= self.content.len() {
            self.content.remove(offset - 1);
            true
        } else {
            false
        }
    }
    
    /// Clear field content
    pub fn clear(&mut self) {
        if self.field_type != FieldType::Protected {
            self.content.clear();
        }
    }
    
    /// Set field content
    /// SECURITY: Enhanced with comprehensive input sanitization
    pub fn set_content(&mut self, content: String) {
        if self.field_type != FieldType::Protected {
            // SECURITY: Validate and sanitize content before setting
            let sanitized_content = self.sanitize_field_content(&content);
            self.content = sanitized_content.chars().take(self.max_length).collect();
        }
    }
    
    /// Get display content (with password masking)
    pub fn get_display_content(&self) -> String {
        match self.field_type {
            FieldType::Password => "*".repeat(self.content.len()),
            _ => self.content.clone()
        }
    }
    
    /// Validate field content
    pub fn validate(&self) -> Result<(), String> {
        if self.required && self.content.trim().is_empty() {
            return Err("Field is required".to_string());
        }
        
        match self.field_type {
            FieldType::Numeric => {
                if !self.content.is_empty() && self.content.parse::<f64>().is_err() {
                    return Err("Invalid numeric value".to_string());
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Set field behavior
    pub fn set_behavior(&mut self, behavior: FieldBehavior) {
        self.behavior = behavior;
    }
    
    /// Set field error
    pub fn set_error(&mut self, error: FieldError) {
        self.error_state = Some(error);
    }
    
    /// Clear field error
    pub fn clear_error(&mut self) {
        self.error_state = None;
    }
    
    /// Check if field is part of a continued group
    pub fn is_continued(&self) -> bool {
        self.continued_group_id.is_some()
    }
    
    /// Validate character input based on field type
    pub fn validate_character(&self, ch: char) -> Result<(), FieldError> {
        match self.field_type {
            FieldType::DigitsOnly => {
                if !ch.is_ascii_digit() {
                    return Err(FieldError::DigitsOnly);
                }
            },
            FieldType::Numeric => {
                if !ch.is_ascii_digit() && !"+-., ".contains(ch) {
                    return Err(FieldError::NumericOnly);
                }
            },
            FieldType::NumericSigned => {
                if !ch.is_ascii_digit() && !"+-".contains(ch) {
                    return Err(FieldError::NumericOnly);
                }
            },
            FieldType::AlphaOnly => {
                if !ch.is_alphabetic() && !",.- ".contains(ch) {
                    return Err(FieldError::AlphaOnly);
                }
            },
            FieldType::Protected | FieldType::Bypass => {
                return Err(FieldError::CursorProtected);
            },
            _ => {} // Allow all characters for other types
        }
        Ok(())
    }
    
    /// Check if field should auto-enter when full
    pub fn should_auto_enter(&self) -> bool {
        self.field_type == FieldType::AutoEnter || self.behavior.auto_enter
    }
    
    /// Check if field is mandatory
    pub fn is_mandatory(&self) -> bool {
        self.field_type == FieldType::Mandatory || self.behavior.mandatory || self.required
    }
    
    /// Check if field should be bypassed during navigation
    pub fn should_bypass(&self) -> bool {
        self.field_type == FieldType::Bypass || self.behavior.bypass
    }
    
    /// Apply field-specific text transformations
    pub fn apply_transformations(&mut self) {
        if self.field_type == FieldType::UppercaseOnly || self.behavior.uppercase_convert {
            self.content = self.content.to_uppercase();
        }

        if self.behavior.right_adjust {
            self.content = format!("{:>width$}", self.content, width = self.max_length);
        }

        if self.behavior.zero_fill && self.field_type == FieldType::Numeric {
            if let Ok(_) = self.content.parse::<i32>() {
                self.content = format!("{:0width$}", self.content, width = self.max_length);
            }
        }
    }

    /// SECURITY: Check if character is safe for input
    fn is_character_safe(&self, ch: char) -> bool {
        // Reject control characters except common safe ones
        if ch.is_control() {
            return matches!(ch, '\n' | '\r' | '\t');
        }

        // Reject characters that could be used for injection attacks
        let dangerous_chars = ['<', '>', '"', '\'', '&', '|', ';', '$', '`', '\0'];
        if dangerous_chars.contains(&ch) {
            return false;
        }

        // Reject very high Unicode characters that might be used for attacks
        if ch as u32 > 0x10FFFF {
            return false;
        }

        true
    }

    /// SECURITY: Sanitize character for safe input
    fn sanitize_character(&self, ch: char) -> char {
        // Convert potentially dangerous characters to safe alternatives
        match ch {
            '\0' => ' ',  // Null byte to space
            '\u{FFFD}' => '?', // Replacement character to question mark
            '\u{FFFE}' | '\u{FFFF}' => '?', // BOM characters to question mark
            c if c.is_control() && !matches!(c, '\n' | '\r' | '\t') => '?',
            c => c, // Keep safe characters as-is
        }
    }

    /// SECURITY: Sanitize field content to prevent injection attacks
    fn sanitize_field_content(&self, content: &str) -> String {
        content.chars()
            .map(|ch| self.sanitize_character(ch))
            .filter(|&ch| self.is_character_safe(ch))
            .collect::<String>()
            .chars()
            .take(self.max_length) // Enforce length limit
            .collect()
    }
}

#[derive(Debug)]
pub struct FieldManager {
    /// List of detected fields
    fields: Vec<Field>,
    /// Currently active field index
    active_field: Option<usize>,
    /// Field counter for IDs
    next_field_id: usize,
    /// Current cursor row (1-based)
    cursor_row: usize,
    /// Current cursor column (1-based)
    cursor_col: usize,
    /// Groups of continued fields (group_id -> Vec<field_indices>)
    continued_groups: HashMap<usize, Vec<usize>>,
    /// Current error state
    error_state: Option<FieldError>,
}

impl FieldManager {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            active_field: None,
            next_field_id: 1,
            cursor_row: 1,
            cursor_col: 1,
            continued_groups: HashMap::new(),
            error_state: None,
        }
    }
    
    /// Detect fields on the terminal screen
    pub fn detect_fields(&mut self, screen: &TerminalScreen) {
        // Use lib5250 field detection
        let _ = crate::lib5250::field::detect_fields_from_screen(screen);
    }
    

    

    

    

    

    



    
    /// Navigate to next field with enhanced logic
    pub fn next_field(&mut self) -> Result<(), FieldError> {
        self.navigate_to_next_field()
    }
    
    /// Navigate to previous field with enhanced logic
    pub fn previous_field(&mut self) -> Result<(), FieldError> {
        self.navigate_to_previous_field()
    }
    
    /// Enhanced field navigation with progression logic
    pub fn navigate_to_next_field(&mut self) -> Result<(), FieldError> {
        if self.fields.is_empty() {
            return Err(FieldError::NoActiveField);
        }
        
        let current_idx = self.active_field.unwrap_or(0);
        let current_field = &self.fields[current_idx];
        
        // Check for custom cursor progression
        if let Some(next_id) = current_field.next_field_id {
            if let Some(next_idx) = self.find_field_by_id(next_id) {
                return self.activate_field_by_index(next_idx);
            }
        }
        
        // Check for continued field logic
        if let Some(group_id) = current_field.continued_group_id {
            if let Some(next_in_group) = self.find_next_in_continued_group(group_id, current_idx) {
                return self.activate_field_by_index(next_in_group);
            }
        }
        
        // Standard field progression with bypass logic
        let mut next_idx = current_idx;
        let start_idx = next_idx;
        
        loop {
            next_idx = (next_idx + 1) % self.fields.len();
            
            // Avoid infinite loop
            if next_idx == start_idx {
                return Err(FieldError::InvalidFieldNavigation);
            }
            
            let candidate_field = &self.fields[next_idx];
            
            // Skip bypass fields
            if candidate_field.should_bypass() {
                continue;
            }
            
            // Found a valid field
            return self.activate_field_by_index(next_idx);
        }
    }
    
    /// Enhanced previous field navigation
    pub fn navigate_to_previous_field(&mut self) -> Result<(), FieldError> {
        if self.fields.is_empty() {
            return Err(FieldError::NoActiveField);
        }
        
        let current_idx = self.active_field.unwrap_or(0);
        let current_field = &self.fields[current_idx];
        
        // Check for custom cursor progression
        if let Some(prev_id) = current_field.prev_field_id {
            if let Some(prev_idx) = self.find_field_by_id(prev_id) {
                return self.activate_field_by_index(prev_idx);
            }
        }
        
        // Check for continued field logic
        if let Some(group_id) = current_field.continued_group_id {
            if let Some(prev_in_group) = self.find_prev_in_continued_group(group_id, current_idx) {
                return self.activate_field_by_index(prev_in_group);
            }
        }
        
        // Standard field progression with bypass logic
        let mut prev_idx = current_idx;
        let start_idx = prev_idx;
        
        loop {
            prev_idx = if prev_idx == 0 { self.fields.len() - 1 } else { prev_idx - 1 };
            
            // Avoid infinite loop
            if prev_idx == start_idx {
                return Err(FieldError::InvalidFieldNavigation);
            }
            
            let candidate_field = &self.fields[prev_idx];
            
            // Skip bypass fields
            if candidate_field.should_bypass() {
                continue;
            }
            
            // Found a valid field
            return self.activate_field_by_index(prev_idx);
        }
    }
    
    /// Get currently active field
    pub fn get_active_field(&self) -> Option<&Field> {
        self.active_field.map(|idx| &self.fields[idx])
    }
    
    /// Get mutable reference to active field
    pub fn get_active_field_mut(&mut self) -> Option<&mut Field> {
        if let Some(idx) = self.active_field {
            Some(&mut self.fields[idx])
        } else {
            None
        }
    }
    
    /// Get field at position
    pub fn get_field_at_position(&self, row: usize, col: usize) -> Option<&Field> {
        self.fields.iter().find(|field| field.contains_position(row, col))
    }
    
    /// Get all fields as slice
    pub fn get_fields_slice(&self) -> &[Field] {
        &self.fields
    }
    
    /// Set active field by position
    pub fn set_active_field_at_position(&mut self, row: usize, col: usize) -> bool {
        // CRITICAL FIX: Enhanced position validation with bounds checking
        // Prevent crashes from invalid cursor positions

        // Validate input coordinates
        if row == 0 || col == 0 {
            eprintln!("SECURITY: Invalid position ({}, {}) - zero coordinate", row, col);
            return false;
        }

        // Validate coordinates are within reasonable terminal bounds
        if row > 100 || col > 200 {
            eprintln!("SECURITY: Position ({}, {}) exceeds reasonable bounds", row, col);
            return false;
        }

        for (idx, field) in self.fields.iter_mut().enumerate() {
            field.active = false;
            if field.contains_position(row, col) {
                field.active = true;
                self.active_field = Some(idx);
                return true;
            }
        }
        false
    }
    
    /// Validate all fields
    pub fn validate_all(&self) -> Vec<(usize, String)> {
        let mut errors = Vec::new();
        for field in &self.fields {
            if let Err(error) = field.validate() {
                errors.push((field.id, error));
            }
        }
        errors
    }
    
    /// Get field values as a map
    pub fn get_field_values(&self) -> std::collections::HashMap<String, String> {
        let mut values = std::collections::HashMap::new();
        for field in &self.fields {
            let key = field.label.clone().unwrap_or_else(|| format!("field_{}", field.id));
            values.insert(key, field.content.clone());
        }
        values
    }
    
    /// Clear all fields
    pub fn clear_all_fields(&mut self) {
        for field in &mut self.fields {
            field.clear();
        }
    }
    
    /// Type a character in the current active field with enhanced features
    /// SECURITY: Enhanced with comprehensive input sanitization
    pub fn type_char(&mut self, ch: char) -> Result<bool, String> {
        if let Some(field_idx) = self.active_field {
            // CRITICAL FIX: Enhanced field index validation with bounds checking
            if field_idx >= self.fields.len() {
                eprintln!("SECURITY: Invalid field index: {}", field_idx);
                return Err("Invalid field index".to_string());
            }

            let field = &mut self.fields[field_idx];
            if field.field_type != FieldType::Protected {
                // CRITICAL FIX: Enhanced character validation with multiple checks
                if !field.is_character_safe(ch) {
                    eprintln!("SECURITY: Dangerous character rejected: {}", ch as u32);
                    return Err("Invalid character".to_string());
                }

                // Validate character input based on field type
                if let Err(error) = field.validate_character(ch) {
                    return Err(error.get_user_message().to_string());
                }

                // CRITICAL FIX: Enhanced length validation with safety checks
                if field.content.len() >= field.max_length {
                    return Err("Field is full".to_string());
                }

                // Additional safety check for content length
                if field.content.len() + 1 > field.max_length {
                    return Err("Field would exceed maximum length".to_string());
                }

                let sanitized_ch = field.sanitize_character(ch);
                field.content.push(sanitized_ch);
                field.modified = true;

                // Auto-advance to next field if field is now full and auto-enter is enabled
                let field_full = field.content.len() >= field.max_length;
                if field_full && field.should_auto_enter() {
                    // Try to advance to next field
                    let _ = self.tab_to_next_field();
                }

                Ok(field_full)
            } else {
                Err("Cannot type in protected field".to_string())
            }
        } else {
            Err("No field selected".to_string())
        }
    }
    
    /// Backspace in the current active field
    pub fn backspace(&mut self) -> Result<(), String> {
        if let Some(field_idx) = self.active_field {
            if field_idx < self.fields.len() {
                let field = &mut self.fields[field_idx];
                if field.field_type != FieldType::Protected {
                    if !field.content.is_empty() {
                        field.content.pop();
                    }
                    Ok(())
                } else {
                    Err("Cannot edit protected field".to_string())
                }
            } else {
                Err("Invalid field index".to_string())
            }
        } else {
            Err("No field selected".to_string())
        }
    }
    
    /// Delete in the current active field (acts like backspace for simplicity)
    pub fn delete(&mut self) -> Result<(), String> {
        self.backspace()
    }
    
    /// Update cursor position
    pub fn set_cursor_position(&mut self, row: usize, col: usize) {
        // CRITICAL FIX: Enhanced cursor position validation with bounds checking
        // Prevent invalid cursor positions that could cause crashes

        // Validate row and column are reasonable (terminal dimensions)
        if row == 0 || col == 0 {
            eprintln!("SECURITY: Invalid cursor position ({}, {}) - zero coordinate", row, col);
            return;
        }

        // Additional validation: check against reasonable terminal bounds
        if row > 100 || col > 200 { // Reasonable terminal size limits
            eprintln!("SECURITY: Cursor position ({}, {}) exceeds reasonable bounds", row, col);
            return;
        }

        self.cursor_row = row;
        self.cursor_col = col;
    }
    
    /// Get current cursor position (1-based)
    pub fn get_cursor_position(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }
    
    /// Set active field based on cursor position and move cursor there
    pub fn activate_field_at_cursor(&mut self) -> bool {
        // CRITICAL FIX: Enhanced cursor position validation before field lookup
        // Prevent crashes from invalid cursor positions

        // Validate cursor position is reasonable
        if self.cursor_row == 0 || self.cursor_col == 0 {
            eprintln!("SECURITY: Invalid cursor position ({}, {})", self.cursor_row, self.cursor_col);
            return false;
        }

        // Validate cursor position is within reasonable bounds
        if self.cursor_row > 100 || self.cursor_col > 200 {
            eprintln!("SECURITY: Cursor position ({}, {}) exceeds reasonable bounds", self.cursor_row, self.cursor_col);
            return false;
        }

        if let Some(field_idx) = self.fields.iter().position(|f|
            f.contains_position(self.cursor_row, self.cursor_col)) {

            // CRITICAL FIX: Validate field index before accessing
            if field_idx >= self.fields.len() {
                eprintln!("SECURITY: Field index {} out of bounds", field_idx);
                return false;
            }

            // Deactivate current field
            if let Some(current_idx) = self.active_field {
                if current_idx < self.fields.len() {
                    self.fields[current_idx].active = false;
                }
            }

            // Activate new field
            self.fields[field_idx].active = true;
            self.active_field = Some(field_idx);

            true
        } else {
            false
        }
    }
    
    /// Set active field by clicking at a position
    pub fn click_at_position(&mut self, row: usize, col: usize) -> bool {
        self.set_cursor_position(row, col);
        self.activate_field_at_cursor()
    }
    
    /// Update the terminal screen with current field content and cursor position
    pub fn update_terminal_display(&self, terminal: &mut crate::terminal::TerminalScreen) {
        // Update screen with field contents
        for field in &self.fields {
            let row = field.start_row.saturating_sub(1); // Convert to 0-based
            let start_col = field.start_col.saturating_sub(1); // Convert to 0-based
            
            // Clear the field area first
            for i in 0..field.length.min(crate::terminal::TERMINAL_WIDTH - start_col) {
                if start_col + i < crate::terminal::TERMINAL_WIDTH && row < crate::terminal::TERMINAL_HEIGHT {
                    let index = crate::terminal::TerminalScreen::buffer_index(start_col + i, row);
                    terminal.buffer[index].character = '_';
                    terminal.buffer[index].attribute = match field.field_type {
                        FieldType::Protected => crate::terminal::CharAttribute::Protected,
                        FieldType::Password => crate::terminal::CharAttribute::NonDisplay,
                        FieldType::Numeric => crate::terminal::CharAttribute::Numeric,
                        _ => crate::terminal::CharAttribute::Normal,
                    };
                }
            }
            
            // Write field content
            for (i, ch) in field.content.chars().enumerate() {
                if i >= field.length {
                    break;
                }
                if start_col + i < crate::terminal::TERMINAL_WIDTH && row < crate::terminal::TERMINAL_HEIGHT {
                    let index = crate::terminal::TerminalScreen::buffer_index(start_col + i, row);
                    terminal.buffer[index].character = ch;
                }
            }
        }
        
        // Update cursor position in terminal
        terminal.cursor_x = self.cursor_col.saturating_sub(1); // Convert to 0-based
        terminal.cursor_y = self.cursor_row.saturating_sub(1); // Convert to 0-based
        
        terminal.dirty = true;
    }
    

    

    

    

    

    
    /// Find field by field ID
    fn find_field_by_id(&self, field_id: usize) -> Option<usize> {
        self.fields.iter().position(|field| field.field_id == field_id)
    }
    
    /// Activate field by index
    fn activate_field_by_index(&mut self, index: usize) -> Result<(), FieldError> {
        if index >= self.fields.len() {
            return Err(FieldError::FieldNotFound(index));
        }
        
        // Deactivate current field
        if let Some(current) = self.active_field {
            self.fields[current].active = false;
        }
        
        // Activate new field
        self.fields[index].active = true;
        self.active_field = Some(index);
        
        // Update cursor position
        let field = &self.fields[index];
        self.set_cursor_position(field.start_row, field.start_col);
        
        Ok(())
    }
    
    /// Find next field in continued group
    fn find_next_in_continued_group(&self, group_id: usize, current_idx: usize) -> Option<usize> {
        if let Some(group_fields) = self.continued_groups.get(&group_id) {
            if let Some(pos) = group_fields.iter().position(|&idx| idx == current_idx) {
                let next_pos = (pos + 1) % group_fields.len();
                return Some(group_fields[next_pos]);
            }
        }
        None
    }
    
    /// Find previous field in continued group
    fn find_prev_in_continued_group(&self, group_id: usize, current_idx: usize) -> Option<usize> {
        if let Some(group_fields) = self.continued_groups.get(&group_id) {
            if let Some(pos) = group_fields.iter().position(|&idx| idx == current_idx) {
                let prev_pos = if pos == 0 { group_fields.len() - 1 } else { pos - 1 };
                return Some(group_fields[prev_pos]);
            }
        }
        None
    }
    
    /// Add field to continued group
    pub fn add_field_to_continued_group(&mut self, field_idx: usize, group_id: usize) {
        self.continued_groups.entry(group_id).or_default().push(field_idx);
        if field_idx < self.fields.len() {
            self.fields[field_idx].continued_group_id = Some(group_id);
        }
    }
    
    /// Remove field from continued group
    pub fn remove_field_from_continued_group(&mut self, field_idx: usize, group_id: usize) {
        if let Some(group_fields) = self.continued_groups.get_mut(&group_id) {
            group_fields.retain(|&idx| idx != field_idx);
            if group_fields.is_empty() {
                self.continued_groups.remove(&group_id);
            }
        }
        if field_idx < self.fields.len() {
            self.fields[field_idx].continued_group_id = None;
        }
    }
    
    /// Set error state
    pub fn set_error(&mut self, error: FieldError) {
        self.error_state = Some(error);
    }
    
    /// Clear error state
    pub fn clear_error(&mut self) {
        self.error_state = None;
    }
    
    /// Get current error state
    pub fn get_error(&self) -> Option<&FieldError> {
        self.error_state.as_ref()
    }

    // Test helper methods (pub only for testing) 
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    pub fn get_fields(&self) -> &Vec<Field> {
        &self.fields
    }

    /// Get detailed field info for UI rendering
    pub fn get_fields_display_info(&self) -> Vec<FieldDisplayInfo> {
        self.fields.iter().map(|field| {
            FieldDisplayInfo {
                label: field.label.clone().unwrap_or_else(|| format!("Field {}", field.id)),
                content: field.get_display_content(),
                is_active: field.active,
                error_state: field.error_state.clone(),
                highlighted: field.highlighted,
                start_row: field.start_row,
                start_col: field.start_col,
                length: field.length,
            }
        }).collect()
    }

    /// Add field for testing purposes
    pub fn add_field_for_test(&mut self, field: Field) {
        let field_id = field.field_id;
        self.fields.push(field);
        if field_id >= self.next_field_id {
            self.next_field_id = field_id + 1;
        }
    }

    /// Navigate to next field by tab order
    pub fn tab_to_next_field(&mut self) -> Result<(), FieldError> {
        if self.fields.is_empty() {
            return Err(FieldError::NoActiveField);
        }
        
        let current_tab_order = self.active_field
            .map(|idx| self.fields[idx].tab_order)
            .unwrap_or(0);
            
        // Find next field by tab order, skipping bypass fields
        let mut candidates: Vec<_> = self.fields.iter()
            .enumerate()
            .filter(|(_, field)| !field.should_bypass() && field.tab_order > current_tab_order)
            .collect();
        
        candidates.sort_by_key(|(_, field)| field.tab_order);
        
        if let Some((idx, _)) = candidates.first() {
            return self.activate_field_by_index(*idx);
        }
        
        // Wrap around to first field
        let mut candidates: Vec<_> = self.fields.iter()
            .enumerate()
            .filter(|(_, field)| !field.should_bypass())
            .collect();
        
        candidates.sort_by_key(|(_, field)| field.tab_order);
        
        if let Some((idx, _)) = candidates.first() {
            return self.activate_field_by_index(*idx);
        }
        
        Err(FieldError::InvalidFieldNavigation)
    }
    
    /// Navigate to previous field by tab order
    pub fn tab_to_previous_field(&mut self) -> Result<(), FieldError> {
        if self.fields.is_empty() {
            return Err(FieldError::NoActiveField);
        }
        
        let current_tab_order = self.active_field
            .map(|idx| self.fields[idx].tab_order)
            .unwrap_or(usize::MAX);
            
        // Find previous field by tab order, skipping bypass fields
        let mut candidates: Vec<_> = self.fields.iter()
            .enumerate()
            .filter(|(_, field)| !field.should_bypass() && field.tab_order < current_tab_order)
            .collect();
        
        candidates.sort_by_key(|(_, field)| std::cmp::Reverse(field.tab_order));
        
        if let Some((idx, _)) = candidates.first() {
            return self.activate_field_by_index(*idx);
        }
        
        // Wrap around to last field
        let mut candidates: Vec<_> = self.fields.iter()
            .enumerate()
            .filter(|(_, field)| !field.should_bypass())
            .collect();
        
        candidates.sort_by_key(|(_, field)| std::cmp::Reverse(field.tab_order));
        
        if let Some((idx, _)) = candidates.first() {
            return self.activate_field_by_index(*idx);
        }
        
        Err(FieldError::InvalidFieldNavigation)
    }

    /// Test helper: Get continued groups
    pub fn get_continued_groups(&self) -> &HashMap<usize, Vec<usize>> {
        &self.continued_groups
    }

    /// Test helper: Get error state
    pub fn get_error_state(&self) -> Option<&FieldError> {
        self.error_state.as_ref()
    }

    /// Validate field exit (called when leaving a field)
    pub fn validate_field_exit(&self, field_idx: usize) -> Result<(), FieldError> {
        if field_idx >= self.fields.len() {
            return Err(FieldError::FieldNotFound(field_idx));
        }
        
        let field = &self.fields[field_idx];
        
        // Check mandatory fields
        if field.is_mandatory() && field.content.trim().is_empty() {
            return Err(FieldError::MandatoryEnter);
        }
        
        // Check field exit required
        if field.behavior.field_exit_required {
            return Err(FieldError::FieldExitRequired);
        }
        
        // Validate field content based on type
        match field.field_type {
            FieldType::Numeric => {
                if !field.content.is_empty() && field.content.parse::<f64>().is_err() {
                    return Err(FieldError::NumericOnly);
                }
            }
            FieldType::DigitsOnly => {
                if !field.content.chars().all(|c| c.is_ascii_digit()) {
                    return Err(FieldError::DigitsOnly);
                }
            }
            FieldType::AlphaOnly => {
                if !field.content.chars().all(|c| c.is_alphabetic() || ",.- ".contains(c)) {
                    return Err(FieldError::AlphaOnly);
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Attempt to exit current field with validation
    pub fn exit_current_field(&mut self) -> Result<(), FieldError> {
        if let Some(field_idx) = self.active_field {
            self.validate_field_exit(field_idx)?;
            // Mark field as modified if it has content
            if !self.fields[field_idx].content.is_empty() {
                self.fields[field_idx].modified = true;
            }
            Ok(())
        } else {
            Err(FieldError::NoActiveField)
        }
    }

    /// Get all modified fields (MDT - Modified Data Tag)
    pub fn get_modified_fields(&self) -> Vec<&Field> {
        self.fields.iter().filter(|field| field.modified).collect()
    }
    
    /// Clear MDT flags for all fields
    pub fn clear_modified_flags(&mut self) {
        for field in &mut self.fields {
            field.modified = false;
        }
    }
    
    /// Check if any fields have been modified
    pub fn has_modified_fields(&self) -> bool {
        self.fields.iter().any(|field| field.modified)
    }

    /// Get the index of the currently active field
    pub fn get_active_field_index(&self) -> Option<usize> {
        self.active_field
    }

    /// Test helper: Set active field index directly
    pub fn set_active_field_for_test(&mut self, index: Option<usize>) {
        self.active_field = index;
    }

    /// COMPREHENSIVE VALIDATION: Validate field manager consistency
    /// This method ensures all field manager data structures are consistent
    pub fn validate_field_manager_consistency(&self) -> Result<(), String> {
        // Validate active field index
        if let Some(active_idx) = self.active_field {
            if active_idx >= self.fields.len() {
                return Err(format!("Active field index {} out of bounds (fields: {})",
                                 active_idx, self.fields.len()));
            }

            // Validate active field is actually marked as active
            if !self.fields[active_idx].active {
                return Err(format!("Active field index {} is not marked as active", active_idx));
            }
        }

        // Validate all field positions and bounds
        for (idx, field) in self.fields.iter().enumerate() {
            // Validate field coordinates
            if field.start_row == 0 || field.start_col == 0 {
                return Err(format!("Field {} has invalid coordinates ({}, {})",
                                 idx, field.start_row, field.start_col));
            }

            if field.start_row > 100 || field.start_col > 200 {
                return Err(format!("Field {} coordinates exceed reasonable bounds", idx));
            }

            // Validate field length
            if field.length == 0 {
                return Err(format!("Field {} has zero length", idx));
            }

            if field.length > 1000 {
                return Err(format!("Field {} length {} is unreasonably large", idx, field.length));
            }

            // Validate field content length doesn't exceed max_length
            if field.content.len() > field.max_length {
                return Err(format!("Field {} content length {} exceeds max_length {}",
                                 idx, field.content.len(), field.max_length));
            }

            // Validate cursor position within field
            if field.active && field.cursor_position >= field.length {
                return Err(format!("Field {} cursor position {} out of bounds (length: {})",
                                 idx, field.cursor_position, field.length));
            }
        }

        // Validate continued groups
        for (group_id, field_indices) in &self.continued_groups {
            if field_indices.is_empty() {
                return Err(format!("Empty continued group {}", group_id));
            }

            for &field_idx in field_indices {
                if field_idx >= self.fields.len() {
                    return Err(format!("Continued group {} references invalid field index {}",
                                     group_id, field_idx));
                }
            }
        }

        Ok(())
    }
}

impl Default for FieldManager {
    fn default() -> Self { Self::new() }
}
