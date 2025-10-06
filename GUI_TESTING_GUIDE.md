# Quick GUI Testing Guide

## ✅ GUI Access is Working!

You can now test the ANSI input fixes interactively.

## Start the Application

```bash
cargo run --bin tn5250r -- --server as400.example.com --port 23 --user myuser --password myuser
```

Or with debug mode:
```bash
cargo run --bin tn5250r -- --debug --server as400.example.com --port 23 --user myuser --password myuser
```

## What You Should See

1. **Connection Screen** - Shows "Connecting to as400.example.com:23..."
2. **Sign-On Screen** - Should display:
   ```
                          Sign On
   
   System  . . . . . :   S215D18V
   Subsystem . . . . :   QINTER
   Display . . . . . :   <display-name>
   
   User  . . . . . . . . . . . . . .
   Password  . . . . . . . . . . . .
   Program/procedure . . . . . . . .
   Menu  . . . . . . . . . . . . . .
   Current library . . . . . . . . .
   ```

## Testing Input (The New Feature!)

### Test 1: Basic Character Input
1. **Click** in the "User" field (or any visible field)
2. **Type** some characters (e.g., "test")
3. **Expected**: Characters should appear on screen as you type
4. **How it works**: 
   - Each character sent as raw ASCII byte to server
   - Server echoes it back
   - Screen updates immediately

### Test 2: Backspace
1. Type several characters
2. **Press Backspace**
3. **Expected**: Previous character should be erased
4. **How it works**: Sends `\b \b` (backspace-space-backspace sequence)

### Test 3: Delete Key
1. Type some text
2. Move cursor (if possible)
3. **Press Delete**
4. **Expected**: Character at cursor should be removed
5. **How it works**: Sends ANSI delete escape sequence `ESC[3~`

### Test 4: Enter Key
1. Fill in user and password
2. **Press Enter**
3. **Expected**: Form should submit, proceed to next screen
4. **How it works**: Sends carriage return (0x0D)

## What's Fixed

### ✅ Display Bug (Fixed in commit c8e3340)
- **Before**: Garbled text like `'UPPMCsareuesonrrsgurwreoan.rm.td/'`
- **After**: Clear text like `"Sign On System: S215D18V"`
- **Fix**: Corrected coordinate transpose in AnsiProcessor

### ✅ Input Bug (Fixed in commit 83bbd72)
- **Before**: Typing gave "No active field" error
- **After**: Characters sent directly to terminal in ANSI mode
- **Fix**: Added ANSI mode support to type_char(), backspace(), delete()

## Debug Panel (if using --debug)

Press the **🐛 button** to see:
- Connection state and timing
- Terminal content preview (1944 chars)
- Raw data dump (hex + ASCII)
- Current cursor position
- Field information

## Troubleshooting

### If characters don't appear:
1. Check the debug panel - is `use_ansi_mode: true`?
2. Look at terminal output - any error messages?
3. Try clicking in different areas of the screen
4. Check network connection (connection state in debug panel)

### If display is garbled:
1. Verify you have the latest code (commit 83bbd72 or later)
2. Restart the application
3. Check that AnsiProcessor coordinate fix is applied

### If connection fails:
1. Verify server is reachable: `ping as400.example.com`
2. Check port is open: `telnet as400.example.com 23`
3. Verify credentials are correct
4. Look at debug output for connection errors

## Expected Behavior Summary

| Action | What Happens | Technical Details |
|--------|--------------|-------------------|
| Type 'A' | 'A' appears on screen | Sends 0x41, server echoes back |
| Backspace | Previous char erased | Sends 0x08 0x20 0x08 |
| Delete | Char at cursor removed | Sends ESC[3~ (0x1B 0x5B 0x33 0x7E) |
| Enter | Submit/execute | Sends 0x0D (CR) |
| Click field | Cursor moves | Sends cursor position to server |

## Success Indicators

✅ Screen shows clear, readable text (not garbled)  
✅ You can see cursor (green highlight)  
✅ Typing characters makes them appear  
✅ Backspace removes characters  
✅ No "No active field" errors  
✅ Server responds to input  

## Next Features to Test (Future)

- Arrow keys (not yet implemented)
- Tab navigation (limited in ANSI mode)
- Home/End keys (not yet implemented)
- Copy/paste (not yet implemented)

## Report Issues

If you find problems, check:
1. Terminal output for error messages
2. Debug panel (🐛 button) for detailed state
3. Run with `--debug --verbose` for maximum detail
4. Compare with `test_connection` program output

Enjoy testing! 🎉
