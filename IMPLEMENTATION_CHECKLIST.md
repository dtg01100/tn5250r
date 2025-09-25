# TN5250R Enhancement Implementation Checklist

## Phase 1: Foundation Enhancement (Immediate Tasks)

### Week 1: Core Infrastructure

#### Day 1-2: Extended Field Types & Attributes
- [ ] **Task 1.1**: Extend `FieldType` enum in `src/field_manager.rs`
  ```rust
  // Add new enum variants: AutoEnter, Mandatory, Highlighted, Bypass, 
  // Continued, NumericSigned, AlphaOnly, DigitsOnly
  ```
  - **Files**: `src/field_manager.rs` (lines 8-17)
  - **Testing**: Create unit tests for new field types
  - **Estimate**: 4 hours

- [ ] **Task 1.2**: Create `FieldBehavior` struct
  ```rust
  // New struct with behavior flags: field_exit_required, auto_enter, 
  // mandatory, bypass, right_adjust, etc.
  ```
  - **Files**: `src/field_manager.rs` (new struct after FieldType)
  - **Testing**: Test behavior flag combinations
  - **Estimate**: 3 hours

- [ ] **Task 1.3**: Create `FieldAttributes` struct  
  ```rust
  // Visual and functional attributes: intensified, non_display, 
  // protected, highlighting type
  ```
  - **Files**: `src/field_manager.rs` (new struct)
  - **Testing**: Test attribute inheritance
  - **Estimate**: 2 hours

#### Day 3: Enhanced Field Structure
- [ ] **Task 1.4**: Extend `Field` struct with new properties
  ```rust
  // Add: behavior, attributes, field_id, next_field_id, prev_field_id,
  // continued_group_id, highlighted, error_state
  ```
  - **Files**: `src/field_manager.rs` (lines 20-45)
  - **Testing**: Test field creation with new properties
  - **Estimate**: 4 hours

- [ ] **Task 1.5**: Update field constructor and methods
  ```rust
  // Modify Field::new() and add new methods for behavior management
  ```
  - **Files**: `src/field_manager.rs` (Field impl block)
  - **Testing**: Test field operations with new structure
  - **Estimate**: 3 hours

#### Day 4-5: Error System Foundation
- [ ] **Task 1.6**: Create `FieldError` enum
  ```rust
  // Comprehensive error types: CursorProtected, NumericOnly, 
  // FieldExitRequired, MandatoryEnter, etc.
  ```
  - **Files**: `src/field_manager.rs` (new enum before Field struct)
  - **Testing**: Test error type coverage
  - **Estimate**: 3 hours

- [ ] **Task 1.7**: Create `ErrorState` struct
  ```rust
  // Error management: current_error, error_position, 
  // error_display_time, auto_clear_timeout
  ```
  - **Files**: `src/field_manager.rs` (new struct)
  - **Testing**: Test error state transitions
  - **Estimate**: 3 hours

- [ ] **Task 1.8**: Integrate error handling into FieldManager
  ```rust
  // Add error_state field to FieldManager and update methods
  ```
  - **Files**: `src/field_manager.rs` (FieldManager struct and impl)
  - **Testing**: Test error propagation
  - **Estimate**: 4 hours

### Week 2: Core Field Operations

#### Day 6-7: Enhanced Field Validation
- [ ] **Task 2.1**: Implement comprehensive input validation
  ```rust
  // Add validate_character_input() method with support for all field types
  ```
  - **Files**: `src/field_manager.rs` (new method in Field impl)
  - **Testing**: Test validation for each field type
  - **Estimate**: 6 hours

- [ ] **Task 2.2**: Add field-specific insertion logic
  ```rust
  // Enhance insert_char() with field behavior support
  ```
  - **Files**: `src/field_manager.rs` (modify existing insert_char method)
  - **Testing**: Test insertion with various field behaviors
  - **Estimate**: 4 hours

#### Day 8-9: Field Manager Enhancement  
- [ ] **Task 2.3**: Add continued field support to FieldManager
  ```rust
  // Add continued_groups HashMap and related methods
  ```
  - **Files**: `src/field_manager.rs` (FieldManager struct)
  - **Testing**: Test continued field grouping
  - **Estimate**: 5 hours

- [ ] **Task 2.4**: Implement enhanced field navigation
  ```rust
  // Add navigate_to_next_field() with progression logic
  ```
  - **Files**: `src/field_manager.rs` (new method in FieldManager impl)
  - **Testing**: Test navigation with bypass and continued fields
  - **Estimate**: 6 hours

#### Day 10: Integration & Testing
- [ ] **Task 2.5**: Update controller integration
  ```rust
  // Modify TerminalController to use enhanced field manager
  ```
  - **Files**: `src/controller.rs` (add enhanced_field_manager field)
  - **Testing**: Integration testing with existing functionality
  - **Estimate**: 4 hours

- [ ] **Task 2.6**: Create comprehensive test suite
  ```rust
  // Unit tests for all new functionality
  ```
  - **Files**: `tests/enhanced_fields.rs` (new test file)
  - **Testing**: Achieve >90% test coverage for new code
  - **Estimate**: 6 hours

## Implementation Commands & Code Snippets

### Step 1: Extend FieldType enum
```bash
# Edit src/field_manager.rs
# Find the FieldType enum (around line 8) and add new variants
```

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldType {
    // Existing types
    Input,
    Password,
    Numeric,
    Protected,
    Selection,
    
    // Enhanced types (ADD THESE)
    AutoEnter,       // Automatically send ENTER when field fills
    Mandatory,       // Must be filled before proceeding  
    Highlighted,     // Visual highlighting when active
    Bypass,          // Skip during navigation
    Continued,       // Multi-segment field
    NumericSigned,   // Signed numeric field
    AlphaOnly,       // Letters, comma, dash, period, space only
    DigitsOnly,      // Digits only (stricter than Numeric)
    UppercaseOnly,   // Auto-convert to uppercase
}
```

### Step 2: Add FieldBehavior struct
```rust
// Add this after the FieldType enum
#[derive(Debug, Clone)]
pub struct FieldBehavior {
    pub field_exit_required: bool,    // FER - must use Field Exit key
    pub auto_enter: bool,             // Auto-send ENTER when full
    pub mandatory: bool,              // Required field
    pub bypass: bool,                 // Skip during navigation
    pub right_adjust: bool,           // Right-justify on exit
    pub zero_fill: bool,              // Fill with zeros vs spaces
    pub uppercase_convert: bool,      // Auto-convert to uppercase
    pub dup_enabled: bool,           // Allow duplicate field operation
    pub cursor_progression: Option<usize>, // Custom next field ID
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
```

### Step 3: Add FieldError enum
```rust
// Add this before the Field struct
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
}
```

### Step 4: Extend Field struct
```rust
// Find the Field struct (around line 20) and add new fields
#[derive(Debug, Clone)]
pub struct Field {
    // Existing fields
    pub id: usize,
    pub field_type: FieldType,
    pub start_row: usize,
    pub start_col: usize,
    pub length: usize,
    pub content: String,
    pub max_length: usize,
    pub active: bool,
    pub label: Option<String>,
    pub required: bool,
    
    // NEW FIELDS - ADD THESE
    pub behavior: FieldBehavior,
    pub field_id: usize,              // Unique field ID for progression
    pub next_field_id: Option<usize>, // Custom next field
    pub prev_field_id: Option<usize>, // Custom previous field  
    pub continued_group_id: Option<usize>, // Group ID for continued fields
    pub highlighted: bool,            // Visual highlighting state
    pub error_state: Option<FieldError>, // Current error if any
    pub modified: bool,               // Modified Data Tag (MDT)
    pub cursor_position: usize,       // Current cursor position in field
}
```

### Step 5: Update Field constructor
```rust
// Modify the Field::new method to include new fields
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
            // NEW FIELDS
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
    
    // ADD NEW METHODS
    pub fn set_behavior(&mut self, behavior: FieldBehavior) {
        self.behavior = behavior;
    }
    
    pub fn set_error(&mut self, error: FieldError) {
        self.error_state = Some(error);
    }
    
    pub fn clear_error(&mut self) {
        self.error_state = None;
    }
    
    pub fn is_continued(&self) -> bool {
        self.continued_group_id.is_some()
    }
    
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
```

## Testing Strategy for Phase 1

### Unit Test Creation
```bash
# Create test file
touch tests/enhanced_fields.rs
```

```rust
// tests/enhanced_fields.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::field_manager::*;
    
    #[test]
    fn test_field_type_extensions() {
        // Test new field types
        assert_ne!(FieldType::AutoEnter, FieldType::Input);
        assert_ne!(FieldType::Mandatory, FieldType::Protected);
    }
    
    #[test]
    fn test_field_behavior_defaults() {
        let behavior = FieldBehavior::default();
        assert!(!behavior.auto_enter);
        assert!(!behavior.mandatory);
        assert!(!behavior.field_exit_required);
    }
    
    #[test]
    fn test_character_validation() {
        let mut field = Field::new(1, FieldType::DigitsOnly, 1, 1, 10);
        
        assert!(field.validate_character('5').is_ok());
        assert!(field.validate_character('a').is_err());
        assert_eq!(
            field.validate_character('a'),
            Err(FieldError::DigitsOnly)
        );
    }
    
    #[test]
    fn test_field_error_messages() {
        assert_eq!(
            FieldError::NumericOnly.get_user_message(),
            "Numeric characters only"
        );
    }
}
```

## Build & Test Commands

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run specific test module  
cargo test enhanced_fields

# Run with output
cargo test -- --nocapture

# Check for compilation errors
cargo check

# Format code
cargo fmt

# Run clippy for linting
cargo clippy
```

## Success Criteria for Phase 1

- [ ] All new field types compile without errors
- [ ] Field behavior system is functional
- [ ] Error handling system provides clear messages
- [ ] Character validation works for all field types  
- [ ] Tests achieve >85% code coverage for new functionality
- [ ] No regression in existing field functionality
- [ ] Memory usage remains stable
- [ ] Performance impact <5% for field operations

## Next Phase Preview

Phase 2 will focus on:
1. Visual feedback implementation (field highlighting)
2. GUI integration of error messages
3. Enhanced cursor visual indicators
4. Field state visual representation

The foundation built in Phase 1 enables all subsequent enhancements while maintaining system stability and backward compatibility.