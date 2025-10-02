#!/bin/bash
# AI Agent Integration Script for Iterative Lock Auditing
# This script provides structured output that an AI agent can parse to systematically fix blocking locks

echo "=== AI Agent Lock Audit Report ==="
echo "Format: JSON-like structured output for AI parsing"
echo ""

echo "{"
echo '  "audit_date": "'$(date -Iseconds)'",'
echo '  "project": "TN5250R",'
echo '  "file": "src/controller.rs",'

# Find AsyncTerminalController implementation  
IMPL_LINE=$(grep -n "^impl AsyncTerminalController" src/controller.rs | head -1 | cut -d: -f1)
echo '  "async_controller_impl_line": '$IMPL_LINE','

# Find all GUI-called methods
echo '  "gui_called_methods": ['
GUI_METHODS=$(grep -o "self\.controller\.[a-z_]*(" src/main.rs | sed 's/self\.controller\.//' | sed 's/(//' | sort -u)
FIRST=true
for method in $GUI_METHODS; do
    if [ "$FIRST" = true ]; then
        echo -n '    "'$method'"'
        FIRST=false
    else
        echo ','
        echo -n '    "'$method'"'
    fi
done
echo ""
echo '  ],'

# Find all blocking .lock() calls in AsyncTerminalController
echo '  "blocking_locks": ['
BLOCKING_LOCKS=$(awk '/^impl AsyncTerminalController/,/^impl [^A]/ {if (/\.lock\(\)/ && !/try_lock/) print NR":"$0}' src/controller.rs)
FIRST=true
while IFS= read -r line; do
    if [ ! -z "$line" ]; then
        LINE_NUM=$(echo "$line" | cut -d: -f1)
        CODE=$(echo "$line" | cut -d: -f2-)
        
        if [ "$FIRST" = true ]; then
            FIRST=false
        else
            echo ','
        fi
        
        echo -n '    {"line": '$LINE_NUM', "code": "'"$(echo "$CODE" | sed 's/"/\\"/g' | tr -d '\n')"'"}'
    fi
done <<< "$BLOCKING_LOCKS"
echo ""
echo '  ],'

# Analyze each GUI method for blocking locks
echo '  "gui_method_analysis": ['
FIRST=true
for method in $GUI_METHODS; do
    # Find the method in AsyncTerminalController
    METHOD_START=$(grep -n "pub fn $method\(" src/controller.rs | grep -A1 "impl AsyncTerminalController" | tail -1 | cut -d: -f1)
    
    if [ ! -z "$METHOD_START" ]; then
        # Check if method uses try_lock or lock
        METHOD_IMPL=$(sed -n "${METHOD_START},$((METHOD_START+30))p" src/controller.rs)
        
        USES_TRY_LOCK=false
        USES_LOCK=false
        
        if echo "$METHOD_IMPL" | grep -q "try_lock"; then
            USES_TRY_LOCK=true
        fi
        
        if echo "$METHOD_IMPL" | grep -q "\.lock()" && ! echo "$METHOD_IMPL" | grep -q "try_lock"; then
            USES_LOCK=true
        fi
        
        if [ "$FIRST" = true ]; then
            FIRST=false
        else
            echo ','
        fi
        
        echo -n '    {"method": "'$method'", "line": '$METHOD_START', "uses_try_lock": '$USES_TRY_LOCK', "uses_blocking_lock": '$USES_LOCK'}'
    fi
done
echo ""
echo '  ],'

# Summary
NUM_GUI_METHODS=$(echo "$GUI_METHODS" | wc -w)
NUM_BLOCKING=$(echo "$BLOCKING_LOCKS" | grep -c .)
NUM_SAFE=$(grep -c "try_lock" src/controller.rs)

echo '  "summary": {'
echo '    "total_gui_methods": '$NUM_GUI_METHODS','
echo '    "blocking_locks_found": '$NUM_BLOCKING','
echo '    "try_locks_used": '$NUM_SAFE','

# Determine if all GUI methods are safe
ALL_SAFE=true
for method in $GUI_METHODS; do
    METHOD_START=$(grep -n "pub fn $method\(" src/controller.rs | tail -1 | cut -d: -f1)
    if [ ! -z "$METHOD_START" ]; then
        METHOD_IMPL=$(sed -n "${METHOD_START},$((METHOD_START+20))p" src/controller.rs)
        if echo "$METHOD_IMPL" | grep -q "\.lock()" && ! echo "$METHOD_IMPL" | grep -q "try_lock"; then
            ALL_SAFE=false
            break
        fi
    fi
done

echo '    "all_gui_methods_safe": '$ALL_SAFE
echo '  },'

echo '  "recommendations": ['
if [ "$ALL_SAFE" = true ]; then
    echo '    "All GUI-called methods use non-blocking locks. No action needed."'
else
    echo '    "Some GUI methods still use blocking locks. Run fix_blocking_locks.sh to auto-fix.",'
    echo '    "Review disconnect() method - may need manual attention.",'
    echo '    "Test GUI responsiveness after fixes."'
fi
echo '  ]'

echo "}"

# Exit code: 0 if all safe, 1 if issues found
if [ "$ALL_SAFE" = true ]; then
    exit 0
else
    exit 1
fi
