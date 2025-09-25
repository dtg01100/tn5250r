# TN5250R Field Enhancement Technical Implementation Guide

## Architecture Overview

The enhanced field management system will be built as an extension to the existing field infrastructure, maintaining backward compatibility while adding sophisticated tn5250j-inspired behaviors.

## Core Architecture Changes

### 1. Enhanced Field Data Structure

```rust
// src/field_manager.rs - Enhanced Field Structure
#[derive(Debug, Clone)]
pub struct EnhancedField {
    // Core field properties
    pub id: usize,
    pub start_row: usize,
    pub start_col: usize,
    pub length: usize,
    pub max_length: usize,
    
    // Content management
    pub content: String,
    pub cursor_position: usize,
    pub modified: bool,
    
    // Field type and behavior
    pub field_type: FieldType,
    pub behavior: FieldBehavior,
    pub attributes: FieldAttributes,
    
    // Navigation and relationships
    pub field_id: usize,
    pub next_field_id: Option<usize>,
    pub prev_field_id: Option<usize>,
    pub continued_group_id: Option<usize>,
    
    // State management
    pub active: bool,
    pub highlighted: bool,
    pub error_state: Option<FieldError>,
    
    // Visual properties
    pub label: Option<String>,
    pub help_text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FieldBehavior {
    pub field_exit_required: bool,        // Must use Field Exit to leave
    pub auto_enter: bool,                 // Auto-send ENTER when full
    pub mandatory: bool,                  // Required field
    pub bypass: bool,                     // Skip during navigation
    pub right_adjust: bool,               // Right-justify on exit
    pub zero_fill: bool,                  // Fill with zeros vs spaces
    pub uppercase_convert: bool,          // Auto-convert to uppercase
    pub dup_enabled: bool,               // Allow DUP field operation
    pub cursor_progression: Option<usize>, // Custom next field
}

#[derive(Debug, Clone)]
pub struct FieldAttributes {
    pub intensified: bool,
    pub non_display: bool,
    pub protected: bool,
    pub numeric: bool,
    pub signed_numeric: bool,
    pub highlighting: FieldHighlighting,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldHighlighting {
    None,
    Entry,      // Highlight on entry
    Active,     // Highlight while active
    Error,      // Error highlighting
    Mandatory,  // Mandatory field highlighting
}
```

### 2. Enhanced Field Types

```rust
// src/field_manager.rs - Extended Field Types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldType {
    // Basic types (existing)
    Input,
    Password,
    Protected,
    Selection,
    
    // Enhanced numeric types
    Numeric,              // Basic numeric (digits, +, -, ., ,, space)
    NumericSigned,        // Signed numeric (+ or - at end)
    DigitsOnly,          // Digits only (stricter than numeric)
    
    // Enhanced text types
    AlphaOnly,           // Letters, comma, dash, period, space
    AlphaShift,          // Mixed case allowed
    UppercaseOnly,       // Auto-convert to uppercase
    
    // Special field types
    AutoEnter,           // Auto-advance when full
    Bypass,              // Skip during navigation
    Continued,           // Multi-segment field
    Highlighted,         // Visual highlighting
    Mandatory,           // Required input
    
    // Advanced types
    RightToLeft,         // RTL text support
    DateField,           // Date input with validation
    TimeField,           // Time input with validation
}
```

### 3. Field Manager Enhancement

```rust
// src/field_manager.rs - Enhanced Field Manager
#[derive(Debug)]
pub struct EnhancedFieldManager {
    fields: std::collections::HashMap<usize, EnhancedField>,
    field_order: Vec<usize>,
    continued_groups: std::collections::HashMap<usize, Vec<usize>>,
    current_field_id: Option<usize>,
    error_state: ErrorState,
    auto_enter_pending: bool,
    next_field_id: usize,
}

impl EnhancedFieldManager {
    pub fn new() -> Self {
        Self {
            fields: std::collections::HashMap::new(),
            field_order: Vec::new(),
            continued_groups: std::collections::HashMap::new(),
            current_field_id: None,
            error_state: ErrorState::new(),
            auto_enter_pending: false,
            next_field_id: 1,
        }
    }
    
    // Core field operations
    pub fn add_field(&mut self, field: EnhancedField) -> usize {
        let id = self.next_field_id;
        self.next_field_id += 1;
        
        let mut field = field;
        field.id = id;
        
        // Handle continued fields
        if let Some(group_id) = field.continued_group_id {
            self.continued_groups
                .entry(group_id)
                .or_insert_with(Vec::new)
                .push(id);
        }
        
        self.field_order.push(id);
        self.fields.insert(id, field);
        id
    }
    
    // Navigation with enhanced logic
    pub fn navigate_to_next_field(&mut self) -> Result<Option<usize>, FieldError> {
        let current_id = match self.current_field_id {
            Some(id) => id,
            None => return Ok(self.get_first_input_field()),
        };
        
        // Handle field exit validation
        self.validate_field_exit(current_id)?;
        
        // Get next field using progression logic
        let next_id = self.get_next_field_with_progression(current_id);
        
        // Handle continued fields
        if let Some(next_id) = next_id {
            if self.is_continued_field(next_id) {
                return self.navigate_continued_field(next_id);
            }
            
            // Skip bypass fields
            if self.is_bypass_field(next_id) {
                self.current_field_id = Some(next_id);
                return self.navigate_to_next_field(); // Recursive skip
            }
        }
        
        self.set_current_field(next_id)?;
        Ok(next_id)
    }
    
    // Field exit validation
    pub fn validate_field_exit(&mut self, field_id: usize) -> Result<(), FieldError> {
        let field = self.get_field(field_id)?;
        
        // Check mandatory fields
        if field.behavior.mandatory && field.content.trim().is_empty() {
            return Err(FieldError::MandatoryEnter);
        }
        
        // Check field exit required
        if field.behavior.field_exit_required && !self.explicit_field_exit {
            return Err(FieldError::FieldExitRequired);
        }
        
        // Apply field adjustments
        if field.behavior.right_adjust {
            self.apply_field_adjustment(field_id)?;
        }
        
        // Mark as modified
        if let Some(field) = self.fields.get_mut(&field_id) {
            field.modified = true;
        }
        
        Ok(())
    }
    
    // Auto-enter handling
    pub fn check_auto_enter(&mut self, field_id: usize) -> Result<bool, FieldError> {
        let field = self.get_field(field_id)?;
        
        if field.behavior.auto_enter && field.content.len() >= field.max_length {
            self.auto_enter_pending = true;
            return Ok(true);
        }
        
        Ok(false)
    }
}
```

### 4. Enhanced Input Processing

```rust
// src/field_manager.rs - Input Processing
impl EnhancedFieldManager {
    pub fn process_character_input(&mut self, ch: char) -> Result<InputResult, FieldError> {
        let field_id = self.current_field_id
            .ok_or(FieldError::NoActiveField)?;
            
        let field = self.get_field_mut(field_id)?;
        
        // Validate character against field type
        self.validate_character_input(field, ch)?;
        
        // Apply uppercase conversion if needed
        let ch = if field.behavior.uppercase_convert {
            ch.to_ascii_uppercase()
        } else {
            ch
        };
        
        // Insert character
        field.content.insert(field.cursor_position, ch);
        field.cursor_position += 1;
        
        // Check for auto-enter
        if self.check_auto_enter(field_id)? {
            return Ok(InputResult::AutoEnter);
        }
        
        // Check if field is full
        if field.cursor_position >= field.max_length {
            if field.behavior.field_exit_required {
                return Ok(InputResult::FieldExitRequired);
            } else {
                return Ok(InputResult::FieldFull);
            }
        }
        
        Ok(InputResult::Continue)
    }
    
    fn validate_character_input(&self, field: &EnhancedField, ch: char) -> Result<(), FieldError> {
        match field.field_type {
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
                if !ch.is_ascii_digit() && !"+- ".contains(ch) {
                    return Err(FieldError::NumericOnly);
                }
                // Additional validation for sign position
                if (ch == '+' || ch == '-') && field.cursor_position != field.max_length - 1 {
                    return Err(FieldError::InvalidSignPosition);
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
}

#[derive(Debug, PartialEq)]
pub enum InputResult {
    Continue,
    FieldFull,
    FieldExitRequired,
    AutoEnter,
    NavigateNext,
}
```

### 5. Visual Enhancement System

```rust
// src/terminal.rs - Enhanced Visual Attributes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CharAttribute {
    // Existing attributes
    Normal,
    Intensified,
    NonDisplay,
    Protected,
    Numeric,
    
    // Enhanced field attributes
    FieldHighlighted,     // Active field highlighting
    FieldError,           // Error state
    FieldMandatory,       // Mandatory field indicator
    FieldActive,          // Currently active field
    FieldModified,        // Modified field indicator
    
    // Special display attributes
    CursorPosition,       // Current cursor position
    FieldBoundary,        // Field start/end markers
}

// src/main.rs - Enhanced GUI Rendering
impl TN5250RApp {
    fn render_enhanced_field(&self, ui: &mut egui::Ui, field: &EnhancedField, 
                           screen_content: &str) {
        let start_pos = field.start_row * TERMINAL_WIDTH + field.start_col;
        let field_content = &screen_content[start_pos..start_pos + field.length];
        
        // Choose colors based on field state
        let (bg_color, text_color, border_color) = match field.get_visual_state() {
            FieldVisualState::Normal => (egui::Color32::WHITE, egui::Color32::BLACK, None),
            FieldVisualState::Active => (egui::Color32::LIGHT_BLUE, egui::Color32::BLACK, 
                                       Some(egui::Color32::BLUE)),
            FieldVisualState::Highlighted => (egui::Color32::YELLOW, egui::Color32::BLACK,
                                            Some(egui::Color32::GOLD)),
            FieldVisualState::Error => (egui::Color32::LIGHT_RED, egui::Color32::DARK_RED,
                                      Some(egui::Color32::RED)),
            FieldVisualState::Mandatory => (egui::Color32::LIGHT_GREEN, egui::Color32::BLACK,
                                          Some(egui::Color32::GREEN)),
        };
        
        // Render field with enhanced visual feedback
        let response = ui.add(
            egui::TextEdit::singleline(&mut field.content.clone())
                .desired_width(field.length as f32 * 8.0) // Approximate char width
                .background_color(bg_color)
                .text_color(text_color)
        );
        
        // Add border for special states
        if let Some(border_color) = border_color {
            ui.painter().rect_stroke(
                response.rect, 
                egui::Rounding::same(2.0), 
                egui::Stroke::new(2.0, border_color)
            );
        }
        
        // Show field indicators
        self.render_field_indicators(ui, field, &response);
    }
    
    fn render_field_indicators(&self, ui: &mut egui::Ui, field: &EnhancedField, 
                             response: &egui::Response) {
        // Mandatory field indicator
        if field.behavior.mandatory {
            ui.painter().text(
                response.rect.right_top() + egui::vec2(2.0, 0.0),
                egui::Align2::LEFT_TOP,
                "*",
                egui::FontId::proportional(12.0),
                egui::Color32::RED,
            );
        }
        
        // Auto-enter indicator
        if field.behavior.auto_enter {
            ui.painter().text(
                response.rect.right_top() + egui::vec2(2.0, 12.0),
                egui::Align2::LEFT_TOP,
                "↵",
                egui::FontId::proportional(10.0),
                egui::Color32::GREEN,
            );
        }
        
        // Field type indicator
        if matches!(field.field_type, FieldType::Password) {
            // Show password field indicator
        }
    }
}

#[derive(Debug, PartialEq)]
enum FieldVisualState {
    Normal,
    Active,
    Highlighted,
    Error,
    Mandatory,
}

impl EnhancedField {
    fn get_visual_state(&self) -> FieldVisualState {
        if self.error_state.is_some() {
            FieldVisualState::Error
        } else if self.active && self.highlighted {
            FieldVisualState::Highlighted
        } else if self.active {
            FieldVisualState::Active
        } else if self.behavior.mandatory && self.content.trim().is_empty() {
            FieldVisualState::Mandatory
        } else {
            FieldVisualState::Normal
        }
    }
}
```

### 6. Enhanced Key Processing

```rust
// src/keyboard.rs - Enhanced Key Handling
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnhancedFunctionKey {
    // Existing function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24,
    Enter,
    
    // Enhanced field operations
    FieldExit,        // Explicit field exit
    FieldPlus,        // Field plus operation
    FieldMinus,       // Field minus operation
    BeginOfField,     // Jump to field start
    EndOfField,       // Jump to field end
    EraseEOF,         // Erase to end of field
    EraseField,       // Erase entire field
    DupField,         // Duplicate field operation
    
    // Enhanced navigation
    NextWord,         // Next word navigation
    PrevWord,         // Previous word navigation
    Home,             // Home position
    Clear,            // Clear screen
    Insert,           // Toggle insert mode
    Delete,           // Delete character
    
    // System operations
    Reset,            // Reset error state
    SysReq,           // System request
    Attn,             // Attention key
}

// src/main.rs - Enhanced Key Processing in GUI
impl TN5250RApp {
    fn handle_enhanced_key_input(&mut self, ctx: &egui::Context) -> Result<(), String> {
        let input = ctx.input(|i| i.clone());
        
        // Handle field-specific keys
        for event in &input.events {
            match event {
                egui::Event::Key { key, pressed: true, modifiers, .. } => {
                    match key {
                        egui::Key::F1 => self.handle_function_key(EnhancedFunctionKey::F1)?,
                        egui::Key::F2 => self.handle_function_key(EnhancedFunctionKey::F2)?,
                        // ... other function keys
                        
                        egui::Key::Tab => {
                            if modifiers.shift {
                                self.handle_function_key(EnhancedFunctionKey::PrevField)?;
                            } else {
                                self.handle_function_key(EnhancedFunctionKey::NextField)?;
                            }
                        },
                        
                        egui::Key::Enter => {
                            if modifiers.ctrl {
                                self.handle_function_key(EnhancedFunctionKey::FieldExit)?;
                            } else {
                                self.handle_function_key(EnhancedFunctionKey::Enter)?;
                            }
                        },
                        
                        egui::Key::Home => {
                            if modifiers.ctrl {
                                self.handle_function_key(EnhancedFunctionKey::BeginOfField)?;
                            } else {
                                self.handle_function_key(EnhancedFunctionKey::Home)?;
                            }
                        },
                        
                        egui::Key::End => {
                            self.handle_function_key(EnhancedFunctionKey::EndOfField)?;
                        },
                        
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        
        Ok(())
    }
    
    fn handle_function_key(&mut self, key: EnhancedFunctionKey) -> Result<(), String> {
        match key {
            EnhancedFunctionKey::FieldExit => {
                self.controller.field_exit()?;
            },
            EnhancedFunctionKey::FieldPlus => {
                self.controller.field_plus_operation()?;
            },
            EnhancedFunctionKey::FieldMinus => {
                self.controller.field_minus_operation()?;
            },
            EnhancedFunctionKey::BeginOfField => {
                self.controller.goto_beginning_of_field()?;
            },
            EnhancedFunctionKey::EndOfField => {
                self.controller.goto_end_of_field()?;
            },
            EnhancedFunctionKey::EraseEOF => {
                self.controller.erase_to_end_of_field()?;
            },
            EnhancedFunctionKey::EraseField => {
                self.controller.erase_field()?;
            },
            EnhancedFunctionKey::DupField => {
                self.controller.duplicate_field()?;
            },
            _ => {
                // Handle other keys...
            }
        }
        
        Ok(())
    }
}
```

### 7. Error Handling System

```rust
// src/field_manager.rs - Comprehensive Error System
#[derive(Debug, Clone, PartialEq)]
pub enum FieldError {
    // Input validation errors
    CursorProtected,
    NumericOnly,
    AlphaOnly,
    DigitsOnly,
    InvalidCharacter(char),
    InvalidSignPosition,
    
    // Field operation errors
    FieldExitRequired,
    FieldExitInvalid,
    MandatoryEnter,
    FieldFull,
    NoRoomForInsert,
    
    // Navigation errors
    NoActiveField,
    FieldNotFound(usize),
    InvalidFieldNavigation,
    
    // System errors
    DuplicateFieldId,
    InvalidFieldDefinition,
    ContinuedFieldError,
}

impl FieldError {
    pub fn get_user_message(&self) -> &'static str {
        match self {
            FieldError::CursorProtected => "Cursor is in protected area",
            FieldError::NumericOnly => "Numeric characters only",
            FieldError::AlphaOnly => "Alphabetic characters only", 
            FieldError::DigitsOnly => "Digits only",
            FieldError::FieldExitRequired => "Use Field Exit key to leave field",
            FieldError::MandatoryEnter => "Required field must be filled",
            FieldError::FieldFull => "Field is full",
            FieldError::NoRoomForInsert => "No room to insert character",
            _ => "Field operation error",
        }
    }
    
    pub fn get_error_code(&self) -> u16 {
        match self {
            FieldError::CursorProtected => 0x0001,
            FieldError::NumericOnly => 0x0002,
            FieldError::AlphaOnly => 0x0003,
            FieldError::FieldExitRequired => 0x0004,
            FieldError::MandatoryEnter => 0x0005,
            _ => 0x0000,
        }
    }
}

#[derive(Debug)]
pub struct ErrorState {
    pub current_error: Option<FieldError>,
    pub error_field_id: Option<usize>,
    pub error_timestamp: Option<std::time::Instant>,
    pub error_count: usize,
    pub auto_clear_timeout: std::time::Duration,
}

impl ErrorState {
    pub fn new() -> Self {
        Self {
            current_error: None,
            error_field_id: None,
            error_timestamp: None,
            error_count: 0,
            auto_clear_timeout: std::time::Duration::from_secs(3),
        }
    }
    
    pub fn set_error(&mut self, error: FieldError, field_id: Option<usize>) {
        self.current_error = Some(error);
        self.error_field_id = field_id;
        self.error_timestamp = Some(std::time::Instant::now());
        self.error_count += 1;
    }
    
    pub fn clear_error(&mut self) {
        self.current_error = None;
        self.error_field_id = None;
        self.error_timestamp = None;
    }
    
    pub fn should_auto_clear(&self) -> bool {
        if let Some(timestamp) = self.error_timestamp {
            timestamp.elapsed() > self.auto_clear_timeout
        } else {
            false
        }
    }
}
```

### 8. Integration Points

```rust
// src/controller.rs - Enhanced Controller Integration
impl TerminalController {
    pub fn initialize_enhanced_fields(&mut self) -> Result<(), String> {
        // Upgrade existing field manager to enhanced version
        let enhanced_manager = EnhancedFieldManager::from_basic(
            std::mem::take(&mut self.field_manager)
        );
        
        self.enhanced_field_manager = Some(enhanced_manager);
        Ok(())
    }
    
    // Enhanced field operations
    pub fn field_exit(&mut self) -> Result<(), String> {
        if let Some(ref mut manager) = self.enhanced_field_manager {
            manager.explicit_field_exit = true;
            manager.navigate_to_next_field()
                .map_err(|e| e.get_user_message().to_string())?;
            manager.explicit_field_exit = false;
        }
        Ok(())
    }
    
    pub fn field_plus_operation(&mut self) -> Result<(), String> {
        if let Some(ref mut manager) = self.enhanced_field_manager {
            let field_id = manager.current_field_id
                .ok_or("No active field")?;
            manager.fill_field_and_advance(field_id, ' ')?;
        }
        Ok(())
    }
    
    pub fn field_minus_operation(&mut self) -> Result<(), String> {
        if let Some(ref mut manager) = self.enhanced_field_manager {
            let field_id = manager.current_field_id
                .ok_or("No active field")?;
            manager.fill_field_and_advance(field_id, '-')?;
        }
        Ok(())
    }
}
```

This technical implementation guide provides the detailed code structure needed to implement all the tn5250j behaviors in the Rust TN5250R system. The modular approach ensures that each feature can be implemented incrementally while maintaining system stability.