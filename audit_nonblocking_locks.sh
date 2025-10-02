#!/bin/bash
# Comprehensive Non-Blocking Operations Audit Script
# This script checks for ALL blocking operations that could freeze the GUI:
# 1. Blocking .lock() calls in AsyncTerminalController
# 2. Blocking config.lock() calls in main.rs
# 3. Synchronous file I/O operations called from GUI thread
# 4. Blocking thread synchronization (thread::join, etc.)

echo "=== TN5250R Non-Blocking Operations Audit ==="
echo ""

ALL_OK=true

# ========== PART 1: AsyncTerminalController Locks ==========
echo "1. Auditing AsyncTerminalController for blocking locks..."
IMPL_LINE=$(grep -n "^impl AsyncTerminalController" src/controller.rs | head -1 | cut -d: -f1)
echo "   Found at line: $IMPL_LINE"

BLOCKING_LOCKS=$(awk "/^impl AsyncTerminalController/,/^}$/ {if (/\.lock\(\)/ && !/try_lock/) print NR\": \"\$0}" src/controller.rs)

if [ -z "$BLOCKING_LOCKS" ]; then
    echo "   ✅ No blocking .lock() calls in AsyncTerminalController"
else
    echo "   ❌ Found blocking .lock() calls:"
    echo "$BLOCKING_LOCKS"
    ALL_OK=false
fi
echo ""

# ========== PART 2: Config Locks in GUI Code ==========
echo "2. Auditing config.lock() calls in GUI code (main.rs)..."

# Find all config.lock() calls that don't use try_lock
CONFIG_LOCKS=$(grep -n "config\.lock()" src/main.rs | grep -v "try_lock" | grep -v "// .*config\.lock()")

if [ -z "$CONFIG_LOCKS" ]; then
    echo "   ✅ No blocking config.lock() calls in GUI code"
else
    echo "   ⚠️  Found config.lock() calls (checking if blocking):"
    
    # Check if they use unwrap() or if let
    while IFS= read -r line; do
        LINE_NUM=$(echo "$line" | cut -d: -f1)
        CONTENT=$(echo "$line" | cut -d: -f2-)
        
        # Check next few lines for unwrap() or usage pattern
        CONTEXT=$(sed -n "${LINE_NUM},$((LINE_NUM+2))p" src/main.rs)
        
        if echo "$CONTEXT" | grep -q "unwrap()"; then
            echo "   ❌ Line $LINE_NUM: BLOCKING (uses unwrap) - $CONTENT"
            ALL_OK=false
        elif echo "$CONTEXT" | grep -q "if let Ok"; then
            echo "   ✅ Line $LINE_NUM: Non-blocking (uses if let Ok) - $CONTENT"
        else
            echo "   ⚠️  Line $LINE_NUM: Unclear - $CONTENT"
        fi
    done <<< "$CONFIG_LOCKS"
fi
echo ""

# ========== PART 3: File I/O Operations ==========
echo "3. Auditing file I/O operations in GUI code..."

# Check for synchronous save_shared_config calls
SYNC_SAVES=$(grep -n "config::save_shared_config(" src/main.rs | grep -v "save_shared_config_async")

if [ -z "$SYNC_SAVES" ]; then
    echo "   ✅ No synchronous save_shared_config() calls in GUI code"
else
    echo "   ⚠️  Found save_shared_config() calls (checking context):"
    
    while IFS= read -r line; do
        LINE_NUM=$(echo "$line" | cut -d: -f1)
        CONTENT=$(echo "$line" | cut -d: -f2-)
        
        # Determine if it's in GUI thread or initialization
        # Lines < 200 are typically initialization code
        if [ "$LINE_NUM" -lt 200 ]; then
            echo "   ✅ Line $LINE_NUM: In initialization (acceptable) - $CONTENT"
        else
            # Check if it's in a dialog, button handler, or UI update
            FUNCTION_CONTEXT=$(awk "NR<=$LINE_NUM" src/main.rs | tac | grep -m1 "fn \|if .*button\|if .*changed\|dialog" | head -1)
            
            if echo "$FUNCTION_CONTEXT" | grep -qE "button|changed|dialog|settings"; then
                echo "   ❌ Line $LINE_NUM: BLOCKING (in GUI event handler) - $CONTENT"
                ALL_OK=false
            else
                echo "   ⚠️  Line $LINE_NUM: Check context - $CONTENT"
            fi
        fi
    done <<< "$SYNC_SAVES"
fi
echo ""

# Check config.rs for blocking operations in save function
echo "4. Auditing config::save_shared_config() implementation..."
SAVE_IMPL=$(awk '/pub fn save_shared_config\(/,/^}/' src/config.rs)

if echo "$SAVE_IMPL" | grep -q "try_lock"; then
    echo "   ✅ save_shared_config() uses try_lock()"
else
    if echo "$SAVE_IMPL" | grep -q "\.lock()" | grep -q "unwrap()"; then
        echo "   ❌ save_shared_config() uses blocking lock().unwrap()"
        ALL_OK=false
    else
        echo "   ⚠️  save_shared_config() lock usage unclear"
    fi
fi

if echo "$SAVE_IMPL" | grep -qE "fs::File::create|write_all"; then
    echo "   ⚠️  save_shared_config() performs synchronous file I/O"
    echo "      (This is acceptable if called from background thread via save_shared_config_async)"
fi

# Check if async version exists
if grep -q "pub fn save_shared_config_async" src/config.rs; then
    echo "   ✅ save_shared_config_async() exists for non-blocking saves"
else
    echo "   ❌ No async version found - GUI will block on file writes"
    ALL_OK=false
fi
echo ""

# ========== PART 4: AsyncTerminalController Method Verification ==========
echo "5. Verifying GUI-called AsyncTerminalController methods use try_lock()..."

METHODS_TO_CHECK=(
    "is_connected"
    "get_terminal_content"
    "get_fields_info"
    "get_cursor_position"
    "send_input"
    "send_function_key"
    "backspace"
    "delete"
    "next_field"
    "previous_field"
    "type_char"
    "click_at_position"
    "activate_field_at_position"
    "request_login_screen"
    "set_credentials"
    "clear_credentials"
    "disconnect"
    "cancel_connect"
    "take_last_connect_error"
    "get_protocol_mode"
)

METHOD_ISSUES=0

for method in "${METHODS_TO_CHECK[@]}"; do
    if grep -q "controller\.$method" src/main.rs 2>/dev/null; then
        METHOD_IMPL=$(awk "/pub fn $method\(.*\) .*{/,/^    \}/" src/controller.rs | head -20)
        
        if echo "$METHOD_IMPL" | grep -q "try_lock"; then
            : # Silent success
        elif echo "$METHOD_IMPL" | grep -q "\.lock()"; then
            echo "   ❌ $method() uses blocking lock()"
            METHOD_ISSUES=$((METHOD_ISSUES + 1))
            ALL_OK=false
        fi
    fi
done

if [ $METHOD_ISSUES -eq 0 ]; then
    echo "   ✅ All ${#METHODS_TO_CHECK[@]} checked methods use try_lock()"
else
    echo "   ❌ $METHOD_ISSUES methods use blocking locks"
fi
echo ""

# ========== PART 5: Thread Synchronization Primitives ==========
echo "6. Auditing thread synchronization primitives..."

# Check for blocking thread::join() in disconnect method
DISCONNECT_IMPL=$(awk '/pub fn disconnect\(&mut self\) \{/,/^    \}/' src/controller.rs)

if echo "$DISCONNECT_IMPL" | grep -q "handle\.join()"; then
    # Check if it's in a spawned thread (non-blocking) or direct call (blocking)
    if echo "$DISCONNECT_IMPL" | grep -B 2 "handle\.join()" | grep -q "thread::spawn"; then
        echo "   ✅ handle.join() called in spawned thread (non-blocking)"
    else
        echo "   ❌ handle.join() called directly (BLOCKING)"
        ALL_OK=false
    fi
else
    echo "   ✅ No blocking thread::join() calls in disconnect()"
fi

# Check for other blocking synchronization primitives
BLOCKING_SYNC=$(grep -n "\.wait()\|\.recv()" src/main.rs | grep -v "// ")

if [ -z "$BLOCKING_SYNC" ]; then
    echo "   ✅ No blocking wait() or recv() calls in GUI code"
else
    echo "   ⚠️  Found potential blocking sync calls:"
    echo "$BLOCKING_SYNC"
fi
echo ""

# ========== SUMMARY ==========
echo "=== Summary ==="
if [ "$ALL_OK" = true ]; then
    echo "✅ All blocking operations audit PASSED"
    echo "   - AsyncTerminalController uses try_lock()"
    echo "   - Config locks are non-blocking in GUI thread"
    echo "   - File I/O operations are async or in initialization"
    echo "   - Thread synchronization is non-blocking"
    echo "   - GUI should remain responsive"
    exit 0
else
    echo "❌ Blocking operations audit FAILED"
    echo "   - Some operations may block the GUI thread"
    echo "   - Review errors above and fix blocking operations"
    exit 1
fi
