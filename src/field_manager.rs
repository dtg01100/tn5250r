//! Field handling for AS/400 terminal forms
//! 
//! This module provides functionality for detecting, navigating, and managing
//! input fields in AS/400 terminal screens.

use crate::terminal::TerminalScreen;

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
        }
    }
    
    /// Check if a position is within this field
    pub fn contains_position(&self, row: usize, col: usize) -> bool {
        row == self.start_row && 
        col >= self.start_col && 
        col < self.start_col + self.length
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
    pub fn insert_char(&mut self, ch: char, offset: usize) -> bool {
        // Check field type restrictions
        match self.field_type {
            FieldType::Numeric => {
                if !ch.is_ascii_digit() && ch != '.' && ch != '-' && ch != '+' {
                    return false; // Invalid character for numeric field
                }
            }
            FieldType::Protected => {
                return false; // Cannot edit protected fields
            }
            _ => {}
        }
        
        // Check length limits
        if self.content.len() >= self.max_length {
            return false;
        }
        
        // Insert character at the specified offset
        if offset <= self.content.len() {
            self.content.insert(offset, ch);
            true
        } else {
            false
        }
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
}

impl FieldManager {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            active_field: None,
            next_field_id: 1,
            cursor_row: 1,
            cursor_col: 1,
        }
    }
    
    /// Detect fields on the terminal screen
    pub fn detect_fields(&mut self, screen: &TerminalScreen) {
        // Clear existing fields
        self.fields.clear();
        self.active_field = None;
        self.next_field_id = 1;
        
        // Debug: Print the first few lines of screen content
        let screen_text = screen.to_string();
        let lines: Vec<&str> = screen_text.lines().collect();
        
        // Print debug info for the first few lines
        eprintln!("=== FIELD DETECTION DEBUG ===");
        for (i, line) in lines.iter().take(5).enumerate() {
            eprintln!("Line {}: '{}'", i + 1, line);
        }
        eprintln!("=============================");
        
        for (row_idx, line) in lines.iter().enumerate() {
            let row = row_idx + 1; // Convert to 1-based
            
            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }
            
            // Only detect fields that have labels followed by colon and input patterns
            if line.contains(": ") && (line.contains("_") || line.contains(".")) {
                self.detect_labeled_input_fields(line, row);
            }
        }
        
        // Debug: Report detected fields
        eprintln!("Detected {} fields:", self.fields.len());
        for field in &self.fields {
            eprintln!("  Field {} at ({}, {}): {:?} - '{}'", 
                field.id, field.start_row, field.start_col, field.field_type, 
                field.label.as_ref().unwrap_or(&"<no label>".to_string()));
        }
        
        // Activate first field if any fields were found
        if !self.fields.is_empty() {
            self.fields[0].active = true;
            self.active_field = Some(0);
            
            // Set cursor to first field
            let field = &self.fields[0];
            self.set_cursor_position(field.start_row, field.start_col);
        }
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
        let re_pattern = r"([A-Za-z][A-Za-z0-9\s]*):[\s_]+";
        
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
    
    /// Navigate to next field
    pub fn next_field(&mut self) {
        if let Some(current) = self.active_field {
            self.fields[current].active = false;
            let next = (current + 1) % self.fields.len();
            self.active_field = Some(next);
            self.fields[next].active = true;
            
            // Update cursor position to the beginning of the new field
            let field = &self.fields[next];
            self.set_cursor_position(field.start_row, field.start_col);
        }
    }
    
    /// Navigate to previous field
    pub fn previous_field(&mut self) {
        if let Some(current) = self.active_field {
            self.fields[current].active = false;
            let prev = if current == 0 { self.fields.len() - 1 } else { current - 1 };
            self.active_field = Some(prev);
            self.fields[prev].active = true;
            
            // Update cursor position to the beginning of the new field
            let field = &self.fields[prev];
            self.set_cursor_position(field.start_row, field.start_col);
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
    
    /// Get all fields
    pub fn get_fields(&self) -> &[Field] {
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
                    
                    for (i, &ch) in chars.iter().enumerate().skip(field_start) {
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
    
    /// Get the number of detected fields
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
    
    /// Get the current active field index (for testing)
    pub fn get_active_field_index(&self) -> Option<usize> {
        self.active_field
    }
}