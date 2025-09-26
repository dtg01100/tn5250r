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

#[derive(Debug, Clone)]
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

impl Default for FieldBehavior {
    fn default() -> Self {
        Self {
            field_exit_required: false,
            auto_enter: false,
            mandatory: false,
            bypass: false,
            right_adjust: false,
            zero_fill: false,
            uppercase_convert: false,
            dup_enabled: false,
            cursor_progression: None,
        }
    }
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
    pub fn insert_char(&mut self, ch: char, offset: usize) -> Result<bool, FieldError> {
        // Clear any previous errors
        self.clear_error();
        
        // Validate character input
        if let Err(error) = self.validate_character(ch) {
            self.set_error(error.clone());
            return Err(error);
        }
        
        // Check length limits
        if self.content.len() >= self.max_length {
            let error = FieldError::FieldFull;
            self.set_error(error.clone());
            return Err(error);
        }
        
        // Check if there's room to insert
        if offset > self.content.len() {
            let error = FieldError::NoRoomForInsert;
            self.set_error(error.clone());
            return Err(error);
        }
        
        // Insert character at the specified offset
        self.content.insert(offset, ch);
        self.modified = true;
        
        // Apply transformations if needed
        if self.field_type == FieldType::UppercaseOnly || self.behavior.uppercase_convert {
            if let Some(last_char) = self.content.chars().nth(offset) {
                let upper_char = last_char.to_uppercase().collect::<String>();
                if upper_char.len() == 1 && upper_char != last_char.to_string() {
                    self.content.remove(offset);
                    self.content.insert_str(offset, &upper_char);
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
    pub fn set_content(&mut self, content: String) {
        if self.field_type != FieldType::Protected {
            self.content = content.chars().take(self.max_length).collect();
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
    
    /// Detect fields marked with underscores
    fn detect_underscore_fields(&mut self, line: &str, row: usize) {
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            if chars[i] == '_' {
                let start_col = i + 1; // 1-based
                let mut length = 0;
                
                // Count consecutive underscores
                while i < chars.len() && chars[i] == '_' {
                    length += 1;
                    i += 1;
                }
                
                // Only create fields for reasonable lengths
                if length >= 2 {
                    let field_type = self.determine_field_type(line, start_col);
                    let mut field = Field::new(self.next_field_id, field_type, row, start_col, length);
                    field.label = self.extract_field_label(line, start_col);
                    
                    self.fields.push(field);
                    self.next_field_id += 1;
                }
            } else {
                i += 1;
            }
        }
    }
    
    /// Detect fields with colon patterns (Label: _____)
    fn detect_colon_fields(&mut self, line: &str, row: usize) {
        // Look for "word:" followed by spaces or underscores
        let _re_pattern = r"([A-Za-z][A-Za-z0-9\s]*):[\s_]+";
        
        // Simple pattern matching for colon fields
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            if chars[i] == ':' && i > 0 {
                // Look for spaces or underscores after colon
                let mut j = i + 1;
                while j < chars.len() && (chars[j] == ' ' || chars[j] == '_') {
                    j += 1;
                }
                
                let field_length = j - i - 1;
                if field_length >= 2 {
                    let start_col = i + 2; // After colon and first space
                    let field_type = self.determine_field_type(line, start_col);
                    let mut field = Field::new(self.next_field_id, field_type, row, start_col, field_length);
                    
                    // Extract label before colon
                    let mut label_start = i;
                    while label_start > 0 && chars[label_start - 1].is_alphanumeric() {
                        label_start -= 1;
                    }
                    if label_start < i {
                        field.label = Some(chars[label_start..i].iter().collect::<String>().trim().to_string());
                    }
                    
                    self.fields.push(field);
                    self.next_field_id += 1;
                }
            }
            i += 1;
        }
    }
    
    /// Detect AS/400 specific field patterns
    fn detect_as400_patterns(&mut self, line: &str, row: usize) {
        let lower_line = line.to_lowercase();
        
        // Common AS/400 field patterns
        if lower_line.contains("user") && (lower_line.contains("name") || lower_line.contains("id")) {
            // Look for nearby input area
            if let Some(col) = self.find_input_area(line, "user") {
                let mut field = Field::new(self.next_field_id, FieldType::Input, row, col, 10);
                field.label = Some("User Name".to_string());
                field.required = true;
                self.fields.push(field);
                self.next_field_id += 1;
            }
        }
        
        if lower_line.contains("password") {
            if let Some(col) = self.find_input_area(line, "password") {
                let mut field = Field::new(self.next_field_id, FieldType::Password, row, col, 20);
                field.label = Some("Password".to_string());
                field.required = true;
                self.fields.push(field);
                self.next_field_id += 1;
            }
        }
        
        if lower_line.contains("program") || lower_line.contains("procedure") {
            if let Some(col) = self.find_input_area(line, "program") {
                let field = Field::new(self.next_field_id, FieldType::Input, row, col, 10);
                self.fields.push(field);
                self.next_field_id += 1;
            }
        }
        
        if lower_line.contains("menu") {
            if let Some(col) = self.find_input_area(line, "menu") {
                let field = Field::new(self.next_field_id, FieldType::Input, row, col, 10);
                self.fields.push(field);
                self.next_field_id += 1;
            }
        }
        
        if lower_line.contains("library") {
            if let Some(col) = self.find_input_area(line, "library") {
                let field = Field::new(self.next_field_id, FieldType::Input, row, col, 10);
                self.fields.push(field);
                self.next_field_id += 1;
            }
        }
    }
    
    /// Find input area near a keyword
    fn find_input_area(&self, line: &str, keyword: &str) -> Option<usize> {
        if let Some(keyword_pos) = line.to_lowercase().find(keyword) {
            // Look for colon after keyword
            let remaining = &line[keyword_pos..];
            if let Some(colon_pos) = remaining.find(':') {
                let after_colon = keyword_pos + colon_pos + 1;
                
                // Skip spaces and find input area
                let chars: Vec<char> = line.chars().collect();
                let mut i = after_colon;
                while i < chars.len() && chars[i] == ' ' {
                    i += 1;
                }
                
                if i < chars.len() {
                    return Some(i + 1); // 1-based
                }
            }
        }
        None
    }
    
    /// Determine field type based on context
    fn determine_field_type(&self, line: &str, _col: usize) -> FieldType {
        let lower_line = line.to_lowercase();
        
        if lower_line.contains("password") {
            FieldType::Password
        } else if lower_line.contains("number") || lower_line.contains("amount") || lower_line.contains("qty") {
            FieldType::Numeric
        } else {
            FieldType::Input
        }
    }
    
    /// Extract field label from line
    fn extract_field_label(&self, line: &str, col: usize) -> Option<String> {
        // Look for text before the field
        let chars: Vec<char> = line.chars().collect();
        let mut label_end = col.saturating_sub(2); // Before field start
        
        // Skip backwards over spaces and underscores
        while label_end > 0 && (chars[label_end] == ' ' || chars[label_end] == '_' || chars[label_end] == ':') {
            label_end -= 1;
        }
        
        if label_end == 0 {
            return None;
        }
        
        // Find start of label (word boundary)
        let mut label_start = label_end;
        while label_start > 0 && chars[label_start - 1].is_alphanumeric() {
            label_start -= 1;
        }
        
        if label_start <= label_end {
            let label: String = chars[label_start..=label_end].iter().collect();
            let trimmed = label.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
        
        None
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
    
    /// Type a character in the current active field
    pub fn type_char(&mut self, ch: char) -> Result<(), String> {
        if let Some(field_idx) = self.active_field {
            if field_idx < self.fields.len() {
                let field = &mut self.fields[field_idx];
                if field.field_type != FieldType::Protected {
                    // Add character if within field length limit
                    if field.content.len() < field.max_length {
                        field.content.push(ch);
                        Ok(())
                    } else {
                        Err("Field is full".to_string())
                    }
                } else {
                    Err("Cannot type in protected field".to_string())
                }
            } else {
                Err("Invalid field index".to_string())
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
        self.cursor_row = row;
        self.cursor_col = col;
    }
    
    /// Get current cursor position (1-based)
    pub fn get_cursor_position(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }
    
    /// Set active field based on cursor position and move cursor there
    pub fn activate_field_at_cursor(&mut self) -> bool {
        if let Some(field_idx) = self.fields.iter().position(|f| 
            f.contains_position(self.cursor_row, self.cursor_col)) {
            
            // Deactivate current field
            if let Some(current_idx) = self.active_field {
                self.fields[current_idx].active = false;
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
                    terminal.buffer[row][start_col + i].character = '_';
                    terminal.buffer[row][start_col + i].attribute = match field.field_type {
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
                    terminal.buffer[row][start_col + i].character = ch;
                }
            }
        }
        
        // Update cursor position in terminal
        terminal.cursor_x = self.cursor_col.saturating_sub(1); // Convert to 0-based
        terminal.cursor_y = self.cursor_row.saturating_sub(1); // Convert to 0-based
        
        terminal.dirty = true;
    }
    
    /// Detect common AS/400 login screen fields with improved patterns
    fn detect_common_as400_fields(&mut self, line: &str, row: usize) {
        let lower_line = line.to_lowercase();
        
        // Look for patterns like "User ID" followed by dots or spaces
        if lower_line.contains("user") && (lower_line.contains("id") || lower_line.contains("name")) {
            // Find the position after "User ID" or similar
            if let Some(user_pos) = lower_line.find("user") {
                // Look for dots, underscores, or spaces after the label
                let search_start = user_pos + 4; // After "user"
                if search_start < line.len() {
                    let remaining = &line[search_start..];
                    if let Some(field_start) = self.find_input_sequence(remaining) {
                        let field_col = search_start + field_start + 1; // Convert to 1-based
                        if field_col <= 80 {
                            let mut field = Field::new(self.next_field_id, FieldType::Input, row, field_col, 10);
                            field.label = Some("User ID".to_string());
                            field.required = true;
                            self.fields.push(field);
                            self.next_field_id += 1;
                            eprintln!("Detected User ID field at ({}, {})", row, field_col);
                        }
                    }
                }
            }
        }
        
        // Look for password fields
        if lower_line.contains("password") || lower_line.contains("sign on") {
            if let Some(pass_pos) = lower_line.find("password").or_else(|| lower_line.find("sign on")) {
                let search_start = pass_pos + 8; // After "password"
                if search_start < line.len() {
                    let remaining = &line[search_start..];
                    if let Some(field_start) = self.find_input_sequence(remaining) {
                        let field_col = search_start + field_start + 1; // Convert to 1-based
                        if field_col <= 80 {
                            let mut field = Field::new(self.next_field_id, FieldType::Password, row, field_col, 10);
                            field.label = Some("Password".to_string());
                            field.required = true;
                            self.fields.push(field);
                            self.next_field_id += 1;
                            eprintln!("Detected Password field at ({}, {})", row, field_col);
                        }
                    }
                }
            }
        }
        
        // Look for system name or library fields
        if lower_line.contains("system") || lower_line.contains("library") {
            if let Some(sys_pos) = lower_line.find("system").or_else(|| lower_line.find("library")) {
                let search_start = sys_pos + 6; // After "system"/"library"
                if search_start < line.len() {
                    let remaining = &line[search_start..];
                    if let Some(field_start) = self.find_input_sequence(remaining) {
                        let field_col = search_start + field_start + 1; // Convert to 1-based
                        if field_col <= 80 {
                            let mut field = Field::new(self.next_field_id, FieldType::Input, row, field_col, 10);
                            field.label = Some("System".to_string());
                            self.fields.push(field);
                            self.next_field_id += 1;
                            eprintln!("Detected System field at ({}, {})", row, field_col);
                        }
                    }
                }
            }
        }
        
        // Look for any sequence of dots or underscores that might be input fields
        self.detect_generic_input_sequences(line, row);
    }
    
    /// Find input sequences (dots, underscores, spaces in patterns that suggest input fields)
    fn find_input_sequence(&self, text: &str) -> Option<usize> {
        let chars: Vec<char> = text.chars().collect();
        
        for (i, &ch) in chars.iter().enumerate() {
            if ch == '.' || ch == '_' || ch == ' ' {
                // Count consecutive similar characters
                let mut count = 1;
                let mut j = i + 1;
                while j < chars.len() && (chars[j] == ch || 
                    (ch == ' ' && chars[j] == '.') || 
                    (ch == '.' && chars[j] == ' ') ||
                    (ch == ' ' && chars[j] == '_') ||
                    (ch == '_' && chars[j] == ' ')) {
                    count += 1;
                    j += 1;
                }
                
                // If we found a sequence of 3 or more, it's likely an input field
                if count >= 3 {
                    return Some(i);
                }
            }
        }
        
        None
    }
    
    /// Detect labeled input fields (most reliable method)
    fn detect_labeled_input_fields(&mut self, line: &str, row: usize) {
        // Look for pattern: "Label . . . : ________" or "Label : ________"
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 2 {
            let label_part = parts[0].trim();
            let field_part = parts[1];
            
            // Skip if label part is empty or too long
            if label_part.is_empty() || label_part.len() > 30 {
                return;
            }
            
            // Look for underscores or dots that indicate input field
            if let Some(field_start) = self.find_input_sequence(field_part) {
                let field_col = parts[0].len() + 1 + field_start + 1; // Position after colon + spaces
                
                if field_col <= 80 {
                    // Determine field type based on label
                    let field_type = if label_part.to_lowercase().contains("password") {
                        FieldType::Password
                    } else {
                        FieldType::Input
                    };
                    
                    // Count the length of the input field
                    let mut length = 0;
                    let chars: Vec<char> = field_part.chars().collect();
                    let mut start_found = false;
                    
                    for (_i, &ch) in chars.iter().enumerate().skip(field_start) {
                        if ch == '_' || ch == '.' {
                            if !start_found {
                                start_found = true;
                            }
                            length += 1;
                        } else if start_found && ch != ' ' {
                            break;
                        }
                    }
                    
                    if length >= 3 {
                        let label_text = label_part.trim_end_matches('.').trim().to_string();
                        let mut field = Field::new(self.next_field_id, field_type.clone(), row, field_col, length);
                        field.label = Some(label_text.clone());
                        field.required = true;
                        self.fields.push(field);
                        self.next_field_id += 1;
                        
                        eprintln!("Detected {} field at ({}, {}) length {} - '{}'", 
                                 match field_type {
                                     FieldType::Password => "Password",
                                     _ => "Input",
                                 }, 
                                 row, field_col, length, label_text);
                    }
                }
            }
        }
    }
    
    /// Detect generic input sequences on a line
    fn detect_generic_input_sequences(&mut self, line: &str, row: usize) {
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            let ch = chars[i];
            
            if ch == '.' || ch == '_' {
                // Count consecutive dots or underscores
                let start_pos = i;
                let mut count = 0;
                
                while i < chars.len() && (chars[i] == '.' || chars[i] == '_' || chars[i] == ' ') {
                    if chars[i] == '.' || chars[i] == '_' {
                        count += 1;
                    }
                    i += 1;
                }
                
                // If we found a significant sequence, create a field
                if count >= 5 {
                    let field_col = start_pos + 1; // Convert to 1-based
                    if field_col <= 80 {
                        let field = Field::new(self.next_field_id, FieldType::Input, row, field_col, count);
                        self.fields.push(field);
                        self.next_field_id += 1;
                        eprintln!("Detected generic input field at ({}, {}) length {}", row, field_col, count);
                    }
                }
            } else {
                i += 1;
            }
        }
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
        self.continued_groups.entry(group_id).or_insert_with(Vec::new).push(field_idx);
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
}