#!/bin/bash

# Launch script for TN5250R in devcontainer with proper GUI environment
# This script ensures the application runs with X11 and software rendering

set -e

echo "🖥️  TN5250R DevContainer GUI Launch"
echo "===================================="

# Force X11 backend
export WINIT_UNIX_BACKEND=x11

# Unset Wayland to prevent backend confusion
unset WAYLAND_DISPLAY

# Force software rendering
export LIBGL_ALWAYS_SOFTWARE=1
export GALLIUM_DRIVER=softpipe

# Mesa debugging (optional)
export MESA_DEBUG=1

# Ensure X11 display is set
export DISPLAY=:1

echo "Environment:"
echo "  DISPLAY=$DISPLAY"
echo "  WINIT_UNIX_BACKEND=$WINIT_UNIX_BACKEND"
echo "  LIBGL_ALWAYS_SOFTWARE=$LIBGL_ALWAYS_SOFTWARE"
echo "  GALLIUM_DRIVER=$GALLIUM_DRIVER"
echo ""

# Check if X server is running
if ! xdpyinfo >/dev/null 2>&1; then
    echo "❌ X11 server not accessible on DISPLAY=$DISPLAY"
    echo "   Make sure the desktop-lite feature is running properly"
    exit 1
fi

echo "✅ X11 server is accessible"
echo ""

# Build and run the application
echo "🔨 Building TN5250R..."
cargo build --bin tn5250r

if [ $? -eq 0 ]; then
    echo "✅ Build successful"
    echo "🚀 Launching TN5250R GUI..."
    echo ""
    
    # Launch with explicit binary specification
    cargo run --bin tn5250r "$@"
else
    echo "❌ Build failed"
    exit 1
fi