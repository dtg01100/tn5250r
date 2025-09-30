#!/bin/bash

echo "=== TN5250R Development Environment Verification ==="
echo ""

echo "🔧 System Information:"
echo "OS: $(cat /etc/os-release | grep PRETTY_NAME | cut -d'"' -f2)"
echo "Architecture: $(uname -m)"
echo ""

echo "🦀 Rust Toolchain:"
rustc --version
cargo --version
echo ""

echo "📦 Essential Libraries Check:"

# Check X11 libraries
echo "✅ X11 Development Libraries:"
pkg-config --exists x11 && echo "  - libx11-dev: OK" || echo "  - libx11-dev: MISSING"
pkg-config --exists xrandr && echo "  - libxrandr-dev: OK" || echo "  - libxrandr-dev: MISSING"
pkg-config --exists xi && echo "  - libxi-dev: OK" || echo "  - libxi-dev: MISSING"
pkg-config --exists xcursor && echo "  - libxcursor-dev: OK" || echo "  - libxcursor-dev: MISSING"

# Check Wayland libraries
echo ""
echo "✅ Wayland Libraries:"
pkg-config --exists wayland-client && echo "  - wayland-client: OK" || echo "  - wayland-client: MISSING"
pkg-config --exists wayland-cursor && echo "  - wayland-cursor: OK" || echo "  - wayland-cursor: MISSING"
pkg-config --exists wayland-egl && echo "  - wayland-egl: OK" || echo "  - wayland-egl: MISSING"

# Check OpenGL/Mesa libraries
echo ""
echo "✅ OpenGL/Mesa Libraries:"
pkg-config --exists gl && echo "  - OpenGL: OK" || echo "  - OpenGL: MISSING"
pkg-config --exists glu && echo "  - GLU: OK" || echo "  - GLU: MISSING"
pkg-config --exists egl && echo "  - EGL: OK" || echo "  - EGL: MISSING"

# Check XKB
echo ""
echo "✅ Keyboard Support:"
pkg-config --exists xkbcommon && echo "  - xkbcommon: OK" || echo "  - xkbcommon: MISSING"
pkg-config --exists xkbcommon-x11 && echo "  - xkbcommon-x11: OK" || echo "  - xkbcommon-x11: MISSING"

# Check font libraries
echo ""
echo "✅ Font Libraries:"
pkg-config --exists fontconfig && echo "  - fontconfig: OK" || echo "  - fontconfig: MISSING"
pkg-config --exists freetype2 && echo "  - freetype2: OK" || echo "  - freetype2: MISSING"

# Check audio
echo ""
echo "✅ Audio Libraries:"
pkg-config --exists alsa && echo "  - ALSA: OK" || echo "  - ALSA: MISSING"

echo ""
echo "🔍 Environment Variables:"
echo "DISPLAY: $DISPLAY"
echo "WAYLAND_DISPLAY: $WAYLAND_DISPLAY"
echo "XDG_RUNTIME_DIR: $XDG_RUNTIME_DIR"
echo "XDG_SESSION_TYPE: $XDG_SESSION_TYPE"
echo "WINIT_UNIX_BACKEND: $WINIT_UNIX_BACKEND"

echo ""
echo "🎯 TN5250R Build Test:"
if cargo check --quiet 2>/dev/null; then
    echo "✅ TN5250R builds successfully"
else
    echo "❌ TN5250R build failed - check dependencies"
fi

echo ""
echo "=== Verification Complete ==="