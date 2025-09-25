#!/bin/bash
# Test the field system CLI application
echo "Building field test..."
cargo build --bin field_test

if [ $? -eq 0 ]; then
    echo "Build successful. Running field test..."
    echo ""
    echo "This will connect to pub400.com and test field navigation."
    echo "You should see:"
    echo "1. Connection to pub400.com"
    echo "2. Field detection"
    echo "3. Tab navigation between fields"
    echo "4. Text input in fields"
    echo "5. Backspace functionality"
    echo ""
    read -p "Press Enter to continue with the test..."
    
    cargo run --bin field_test 2>&1 | tee field_test_output.log
    
    echo ""
    echo "Test completed. Output saved to field_test_output.log"
    echo "Check the log to see if fields were detected and navigation worked."
else
    echo "Build failed. Please check the compilation errors."
fi