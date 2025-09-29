# TN5250R DevContainer GUI Setup

## Overview

The TN5250R devcontainer has been configured to support GUI applications using:
- **Desktop Environment**: Fluxbox via `desktop-lite` feature
- **Display**: VNC server on port 5901 with noVNC web interface on port 6080
- **Graphics Backend**: Software rendering with OpenGL/Mesa
- **Window System**: X11 (Wayland disabled for compatibility)

## Quick Start

### Option 1: Using the Launch Script (Recommended)

```bash
./launch-gui-devcontainer.sh
```

This script automatically:
- Sets up the proper environment variables
- Forces X11 backend usage  
- Enables software rendering
- Builds and runs the TN5250R GUI application

### Option 2: Manual Launch

```bash
# Set environment for GUI compatibility
export WINIT_UNIX_BACKEND=x11
export LIBGL_ALWAYS_SOFTWARE=1
export GALLIUM_DRIVER=softpipe
unset WAYLAND_DISPLAY

# Run the application
cargo run --bin tn5250r
```

## Accessing the GUI

### VNC Desktop
1. **Web Browser**: Open http://localhost:6080 in your browser
2. **VNC Client**: Connect to `localhost:5901` (password: `vscode`)

### Port Forwarding
The following ports are automatically forwarded:
- **6080**: noVNC web interface (recommended)
- **5901**: VNC server (for desktop VNC clients)

## Configuration Details

### DevContainer Features
- **desktop-lite**: Provides Fluxbox desktop environment with VNC access
- **rust**: Full Rust toolchain with common utilities

### Environment Variables
```bash
DISPLAY=:1                    # X11 display target
WINIT_UNIX_BACKEND=x11       # Force X11 backend (disable Wayland)
LIBGL_ALWAYS_SOFTWARE=1      # Enable software OpenGL rendering
GALLIUM_DRIVER=softpipe      # Use software Mesa driver
```

### Graphics Dependencies
- `libgl1-mesa-dev`: OpenGL development libraries
- `libgl1-mesa-glx`: OpenGL runtime libraries
- `mesa-utils`: Mesa utilities for debugging
- X11 development packages for window management

### eFrame Configuration
The application uses only the `glow` (OpenGL) backend with software rendering:
```toml
eframe = { version = "0.27", default-features = false, features = ["accesskit", "default_fonts", "glow"] }
```

## Troubleshooting

### Application Won't Start
1. **Check X11 server**: `xdpyinfo` should work without errors
2. **Verify environment**: Ensure `WAYLAND_DISPLAY` is unset
3. **Check VNC**: Process list should show `tigervnc` and `fluxbox`

### Graphics Issues
- Software rendering is used by default (no GPU acceleration needed)
- Mesa debugging can be enabled with `export MESA_DEBUG=1`
- OpenGL info: `glxinfo` (if available)

### VNC Connection Issues
- **Web interface**: Ensure port 6080 is accessible
- **VNC client**: Use password `vscode` for port 5901
- **Container status**: Check that desktop-lite feature initialized properly

## Performance Notes

- Software rendering provides good compatibility but limited performance
- Suitable for terminal emulation and basic GUI applications
- No hardware acceleration available in devcontainer environment
- Frame rates optimized for functionality over smoothness

## Development Workflow

1. **Code changes**: Edit source files as normal
2. **Testing**: Use `./launch-gui-devcontainer.sh` to test GUI
3. **Debugging**: Terminal output visible alongside GUI
4. **Distribution**: GUI works identically outside devcontainer with proper graphics drivers