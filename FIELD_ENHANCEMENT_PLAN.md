# TN5250R Enhanced Field Management Implementation Plan

## Overview
This document outlines the implementation plan for bringing advanced tn5250j behaviors to the Rust TN5250R implementation. The plan is organized into phases based on complexity, dependencies, and user impact.

## Current State Assessment

### ✅ Already Implemented
- Basic field detection (both protocol-based and heuristic)
- Simple field navigation (Tab/Shift+Tab)
- Basic field types (Input, Password, Numeric, Protected, Selection)
- Terminal character attributes (Normal, Intensified, NonDisplay, Protected, etc.)
- Function key mapping (F1-F24)
- Basic cursor management

### 🔧 Partially Implemented
- Field manager with basic operations
- Character input validation
- Field content management

### ✅ Now Implemented (Sept 2025)
- Advanced field types (Auto-enter, Mandatory, Continued, Highlighted, Bypass, Right-adjust, Zero-fill, Uppercase)
- Field highlighting and visual feedback (highlighted, error state)
- Comprehensive field exit logic (field_exit_required, mandatory)
- Continued field grouping and navigation
- Error handling and visual indicators (FieldError, error_state)
- Auto-enter functionality (auto_enter)
- Field adjustment (right-justify, zero-fill, uppercase)

### 🔧 In Progress / Next Steps
- Expand protocol attribute mapping for edge cases
- Refine continued field grouping logic
- UI integration for error and highlight feedback

## Implementation Phases

---

## Phase 1: Foundation Enhancement (Weeks 1-2)
**Goal**: Enhance existing field infrastructure to support advanced behaviors

### 1.1 Extended Field Attributes
**Files to modify**: `src/field_manager.rs`, `src/protocol_state.rs`

**Add to existing FieldType enum:**
```rust
pub enum FieldType {
    // Existing types
    Input,
    Password, 
    Numeric,
    Protected,
    Selection,
    
    // New enhanced types
    AutoEnter,       // Automatically send ENTER when field fills
    Mandatory,       // Must be filled before proceeding
    Highlighted,     // Visual highlighting when active
    Bypass,          // Skip during navigation
    Continued,       // Multi-segment field
    NumericSigned,   // Signed numeric field
    AlphaOnly,       // Letters, comma, dash, period, space only
    DigitsOnly,      // Digits only (stricter than Numeric)
}
```

**Add field behavior flags:**
```rust
pub struct FieldBehavior {
    pub field_exit_required: bool,    // FER - must use Field Exit key
    pub right_adjust: bool,           // Right-adjust on field exit
    pub zero_fill: bool,              // Fill with zeros vs spaces
    pub uppercase_convert: bool,      // Auto-convert to uppercase
    pub dup_enabled: bool,            // Allow duplicate field operation
    pub cursor_progression: Option<usize>, // Custom next field ID
}
```

**Priority**: HIGH (Foundation for all other features)
**Effort**: 2 days

### 1.2 Enhanced Field Structure
**Extend Field struct with new attributes:**
```rust
pub struct Field {
    // Existing fields...
    
    // New behavior fields
    pub behavior: FieldBehavior,
    pub highlighted: bool,
    pub auto_enter: bool,
    pub mandatory: bool,
    pub field_id: usize,
    pub next_field_id: Option<usize>,
    pub prev_field_id: Option<usize>,
    pub continued_group: Option<usize>, // Group ID for continued fields
}
```

**Priority**: HIGH
**Effort**: 1 day

### 1.3 Error System Foundation
**Create new error handling system:**
```rust
#[derive(Debug, Clone)]
pub enum FieldError {
    CursorProtected,
    NumericOnly,
    AlphaOnly,
    DigitsOnly,
    FieldExitInvalid,
    MandatoryEnter,
    FieldFull,
    InvalidCharacter(char),
}

pub struct ErrorState {
    pub current_error: Option<FieldError>,
    pub error_position: Option<(usize, usize)>,
    pub error_display_time: Option<std::time::Instant>,
}
```

**Priority**: HIGH
**Effort**: 1 day

---

## Phase 2: Visual Feedback & Highlighting (Week 3)
**Goal**: Implement field highlighting and visual feedback systems

### 2.1 Field Highlighting System
**Files to modify**: `src/main.rs`, `src/terminal.rs`

**Add highlighting states to CharAttribute:**
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CharAttribute {
    // Existing attributes...
    
    // New highlighting attributes
    FieldHighlighted,     // Active field highlighting
    FieldError,           // Error state highlighting
    FieldMandatory,       // Mandatory field indication
}
```

**Implement highlighting logic:**
```rust
impl FieldManager {
    pub fn set_field_highlighted(&mut self, field_id: usize, highlight: bool) {
        // Implementation for highlighting fields
    }
    
    pub fn highlight_current_field(&mut self, terminal: &mut TerminalScreen) {
        // Automatically highlight active field
    }
}
```

**Priority**: HIGH (Major UX improvement)
**Effort**: 3 days

### 2.2 Error Visual Indicators
**Implement error display in GUI:**
```rust
// In main.rs GUI code
pub fn display_field_error(&mut self, ui: &mut egui::Ui) {
    if let Some(error) = &self.error_state.current_error {
        ui.colored_label(egui::Color32::RED, format!("Error: {:?}", error));
    }
}
```

**Priority**: MEDIUM
**Effort**: 1 day

---

## Phase 3: Advanced Navigation (Week 4)
**Goal**: Implement advanced cursor navigation and field operations

### 3.1 Enhanced Navigation Keys
**Files to modify**: `src/keyboard.rs`, `src/main.rs`

**Add new function keys:**
```rust
pub enum FunctionKey {
    // Existing keys...
    
    // New navigation keys
    FieldExit,        // Field Exit key
    FieldPlus,        // Field Plus (fill and advance)
    FieldMinus,       // Field Minus (fill with minus)
    BeginOfField,     // BOF - jump to field start
    EndOfField,       // EOF - jump to field end
    EraseEOF,         // Erase from cursor to end of field
    EraseField,       // Erase entire field
    DupField,         // Fill field with DUP character
}
```

**Priority**: HIGH
**Effort**: 2 days

### 3.2 Smart Field Navigation
**Implement continued field navigation:**
```rust
impl FieldManager {
    pub fn next_field_smart(&mut self) -> Result<(), String> {
        // Handle continued fields as single unit
        // Skip bypass fields
        // Use cursor progression if defined
    }
    
    pub fn handle_continued_fields(&mut self, current_field: usize) -> Vec<usize> {
        // Return all field IDs in continued group
    }
}
```

**Priority**: MEDIUM
**Effort**: 3 days

---

## Phase 4: Field Exit & Validation (Week 5)
**Goal**: Implement comprehensive field exit logic and validation

### 4.1 Field Exit Logic
**Files to modify**: `src/field_manager.rs`

```rust
impl FieldManager {
    pub fn field_exit(&mut self, field_id: usize) -> Result<bool, FieldError> {
        let field = self.get_field_mut(field_id)?;
        
        // Check mandatory enter
        if field.mandatory && field.content.trim().is_empty() {
            return Err(FieldError::MandatoryEnter);
        }
        
        // Apply field adjustments
        if field.behavior.right_adjust {
            self.right_adjust_field(field_id)?;
        }
        
        // Set MDT (Modified Data Tag)
        field.modified = true;
        
        Ok(true)
    }
    
    fn right_adjust_field(&mut self, field_id: usize) -> Result<(), FieldError> {
        // Implement right justification with zero/space fill
    }
}
```

**Priority**: HIGH
**Effort**: 3 days

### 4.2 Advanced Validation
**Implement comprehensive input validation:**
```rust
impl Field {
    pub fn validate_input(&self, ch: char) -> Result<(), FieldError> {
        match self.field_type {
            FieldType::NumericSigned => {
                if !ch.is_ascii_digit() && ch != '+' && ch != '-' {
                    return Err(FieldError::NumericOnly);
                }
            },
            FieldType::AlphaOnly => {
                if !ch.is_alphabetic() && !",.- ".contains(ch) {
                    return Err(FieldError::AlphaOnly);
                }
            },
            FieldType::DigitsOnly => {
                if !ch.is_ascii_digit() {
                    return Err(FieldError::DigitsOnly);
                }
            },
            // ... other validations
        }
        Ok(())
    }
}
```

**Priority**: MEDIUM
**Effort**: 2 days

---

## Phase 5: Auto-Enter & Field Progression (Week 6)
**Goal**: Implement auto-enter functionality and custom field progression

### 5.1 Auto-Enter Implementation
```rust
impl FieldManager {
    pub fn check_auto_enter(&mut self, field_id: usize) -> bool {
        let field = &self.fields[field_id];
        
        if field.auto_enter && field.content.len() >= field.max_length {
            // Trigger auto-enter
            return true;
        }
        false
    }
    
    pub fn handle_auto_enter(&mut self) -> Result<(), String> {
        // Send ENTER key automatically
        // Navigate to next logical field
    }
}
```

**Priority**: HIGH (Major UX improvement)
**Effort**: 2 days

### 5.2 Custom Cursor Progression
```rust
impl FieldManager {
    pub fn get_next_field_by_progression(&self, current_field: usize) -> Option<usize> {
        if let Some(field) = self.fields.get(&current_field) {
            if let Some(next_id) = field.next_field_id {
                return Some(next_id);
            }
        }
        
        // Fall back to sequential navigation
        self.get_next_field_sequential(current_field)
    }
}
```

**Priority**: MEDIUM
**Effort**: 2 days

---

## Phase 6: Advanced Features (Week 7-8)
**Goal**: Implement remaining advanced features

### 6.1 Field Operations
**Implement special field operations:**
- Field Plus/Minus operations
- DUP field functionality
- Erase operations
- Insert mode handling

### 6.2 Cursor Enhancements
**Visual cursor improvements:**
- Cursor size/style options
- Blinking control
- Cross-hair rulers
- Insert mode indication

### 6.3 Enhanced Error Handling
**Comprehensive error system:**
- Error message display
- Error recovery mechanisms
- Field-specific error states

---

## Testing Strategy

### Unit Tests
- Field validation logic
- Navigation algorithms
- Auto-enter functionality
- Error handling

### Integration Tests
- Full field lifecycle
- Multi-field operations
- Real AS/400 screen scenarios

### Manual Testing
- AS/400 system integration
- User experience validation
- Performance testing

---

## Risk Mitigation

### High Risk Items
1. **Breaking existing functionality**: Use feature flags during development
2. **Performance impact**: Profile field detection and navigation code
3. **Compatibility issues**: Test with multiple AS/400 systems

### Contingency Plans
1. **Rollback capability**: Keep existing field system as fallback
2. **Incremental deployment**: Enable features gradually
3. **User feedback integration**: Beta testing with real users

---

## Success Metrics

### Functional Metrics
- All tn5250j field behaviors implemented
- Zero regression in existing functionality
- Performance within 10% of current implementation

### User Experience Metrics
- Reduced data entry errors
- Faster form completion
- Intuitive navigation experience

---

## Resource Requirements

### Development Time: 8 weeks (1 developer)
### Testing Time: 2 weeks
### Documentation Time: 1 week

**Total Project Duration: 11 weeks**

### Dependencies
- No external library changes required
- Existing codebase modifications only
- Backward compatibility maintained

---

## Implementation Notes

### Code Organization
- Keep existing modules intact
- Add new functionality as extensions
- Use composition over inheritance
- Maintain clear separation of concerns

### Performance Considerations
- Lazy evaluation where possible
- Minimal memory allocation in hot paths
- Efficient field lookup algorithms
- Cache field relationships

### Maintainability
- Comprehensive documentation
- Clear API boundaries
- Extensive test coverage
- Consistent coding standards