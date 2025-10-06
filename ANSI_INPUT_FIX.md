# ANSI Mode Input Fix

## Problem
When connected to an AS/400 system that responds with ANSI/VT100 data instead of native 5250 protocol, the terminal emulator could not accept keyboard input. Characters typed were rejected with "No active field" errors.

## Root Cause
The `type_char()`, `backspace()`, and `delete()` methods in `TerminalController` were designed for 5250 mode, which uses structured fields with metadata. They required an active field from the `field_manager`.

In ANSI/VT100 mode:
- There are no structured fields
- Input should be sent directly to the terminal as raw bytes
- The server echoes characters back
- No local field management is needed

## Solution Implemented

### 1. Modified `type_char()` in `src/controller.rs`
```rust
pub fn type_char(&mut self, ch: char) -> Result<(), String> {
    // In ANSI mode, send characters directly to the terminal (server will echo)
    if self.use_ansi_mode {
        // Send the character as raw ASCII byte
        let byte = ch as u8;
        self.send_input(&[byte])?;
        return Ok(());
    }
    
    // 5250 mode: Use field-based input (existing code)
    // ...
}
```

### 2. Modified `backspace()` in `src/controller.rs`
```rust
pub fn backspace(&mut self) -> Result<(), String> {
    // In ANSI mode, send backspace directly
    if self.use_ansi_mode {
        // Send backspace (0x08) followed by space and another backspace
        // This is the standard way to backspace in terminals: \b \b
        self.send_input(&[0x08, 0x20, 0x08])?;
        return Ok(());
    }
    
    // 5250 mode: Use field-based backspace (existing code)
    // ...
}
```

### 3. Modified `delete()` in `src/controller.rs`
```rust
pub fn delete(&mut self) -> Result<(), String> {
    // In ANSI mode, send delete escape sequence (ESC[3~)
    if self.use_ansi_mode {
        self.send_input(&[0x1B, 0x5B, 0x33, 0x7E])?; // ESC [ 3 ~
        return Ok(());
    }
    
    // 5250 mode: Use field-based delete (existing code)
    // ...
}
```

## How It Works

### ANSI Mode Input Flow
1. User types a character in the GUI
2. `egui::Event::Text` event captured in `main.rs` line ~1270
3. Event handler calls `controller.type_char(ch)`
4. Character sent as raw ASCII byte to AS/400 system
5. AS/400 echoes character back with cursor position update
6. AnsiProcessor processes the echo and updates screen buffer
7. GUI displays the updated screen with the character visible

### Special Keys
- **Backspace**: Sends `\b \b` (0x08, 0x20, 0x08) - standard terminal backspace
- **Delete**: Sends `ESC[3~` - ANSI delete sequence
- **Enter**: Already handled correctly by `FunctionKey::Enter` sending 0x0D

## Testing

To test the fix:
```bash
cargo run --bin tn5250r -- --server as400.example.com --port 23 --user myuser --password myuser
```

Once the Sign-On screen appears:
1. Click in a field (e.g., "User" or "Password" field)
2. Type characters - they should appear on screen
3. Backspace should delete characters
4. Enter should submit the form

## Compatibility

The fix maintains full backward compatibility:
- **5250 mode**: Still uses structured field management (unchanged)
- **ANSI mode**: Now properly handles raw character input
- **Mode detection**: Automatic based on incoming data (ESC [ sequences)

## Related Files
- `src/controller.rs` - Input handling methods modified
- `src/main.rs` - Keyboard event capture (unchanged)
- `src/ansi_processor.rs` - ANSI escape sequence processing (unchanged)
- `src/keyboard.rs` - Function key mappings (unchanged)
