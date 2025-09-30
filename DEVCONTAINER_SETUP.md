# TN5250R Development Container Setup

This document describes the complete development container configuration for TN5250R, including all necessary libraries and dependencies for GUI development.

## Container Configuration

### Base Image
- **Image**: `mcr.microsoft.com/devcontainers/rust:1-1-bullseye`
- **OS**: Debian GNU/Linux 11 (bullseye)
- **Architecture**: x86_64

### Installed System Libraries

#### GUI Framework Support
- **X11 Libraries**: `libx11-dev`, `libxrandr-dev`, `libxinerama-dev`, `libxcursor-dev`, `libxi-dev`, `libxext-dev`
- **Wayland Libraries**: `libwayland-dev`, `libwayland-client0`, `libwayland-cursor0`, `libwayland-egl1`
- **OpenGL/Mesa**: `libgl1-mesa-dev`, `libgl1-mesa-glx`, `mesa-utils`, `libglu1-mesa-dev`

#### Input and Keyboard Support
- **XKB Libraries**: `libxkbcommon-dev`, `libxkbcommon-x11-0`, `xkb-data`
- **Additional Input**: `libxtst6`, `libxss-dev`

#### Font and Text Rendering
- **Font Libraries**: `libfontconfig1-dev`, `libfreetype6-dev`, `libxft-dev`
- **GTK Support**: `libgtk-3-dev`

#### Audio Support
- **ALSA**: `libasound2-dev`

#### Additional GUI Support
- **Compositing**: `libxcomposite-dev`, `libxdamage-dev`, `libxfixes-dev`
- **Security**: `libnss3-dev`

#### Development Tools
- **Build Tools**: `build-essential`, `cmake`, `pkg-config`
- **Utilities**: `git`, `curl`, `wget`, `unzip`
- **Testing**: `xvfb` (X Virtual Framebuffer)

### Socket Mounts
- **X11 Socket**: `/tmp/.X11-unix` → `/tmp/.X11-unix`
- **Wayland Socket**: `$XDG_RUNTIME_DIR` → `/tmp/runtime-vscode`
- **Cargo Cache**: Persistent volume for Rust dependencies

### Environment Variables
- `DISPLAY`: Forwarded from host for X11 support
- `WAYLAND_DISPLAY`: Forwarded from host for Wayland support
- `XDG_RUNTIME_DIR`: Set to `/tmp/runtime-vscode`
- `XDG_SESSION_TYPE`: Forwarded from host
- `WINIT_UNIX_BACKEND`: Automatically set based on session type
- `XKB_DEFAULT_LAYOUT`: Set to "us" keyboard layout
- `XKB_DEFAULT_MODEL`: Set to "pc105" keyboard model
- `RUST_LOG`: Set to "info" for debugging
- `LIBGL_ALWAYS_SOFTWARE`: Set to "1" for software rendering fallback

### Container Runtime Options
- `--network=host`: Host network access
- `--ipc=host`: Inter-process communication with host
- `--device=/dev/dri`: Direct rendering interface access (GPU)
- `--security-opt=seccomp=unconfined`: Relaxed security for GUI apps

### VS Code Extensions
- **rust-lang.rust-analyzer**: Rust language server
- **vadimcn.vscode-lldb**: LLDB debugger
- **serayuzgur.crates**: Cargo.toml dependency management
- **tamasfe.even-better-toml**: Enhanced TOML support
- **usernamehw.errorlens**: Inline error display
- **ms-vscode.hexeditor**: Binary file editing
- **ms-vscode.cmake-tools**: CMake support

### VS Code Settings
- Rust-analyzer configured to use Clippy for linting
- All Cargo features enabled for analysis

## Usage

### Running the GUI Application
```bash
# Automatic backend detection
cargo run --release --bin tn5250r

# Force X11 backend
WINIT_UNIX_BACKEND=x11 cargo run --release --bin tn5250r

# Force Wayland backend
WINIT_UNIX_BACKEND=wayland cargo run --release --bin tn5250r
```

### Environment Verification
Run the included verification script:
```bash
./verify_libs.sh
```

### Development Workflow
1. The container automatically detects your host's display protocol (X11 or Wayland)
2. GUI applications will render on your host desktop
3. All necessary development libraries are pre-installed
4. Cargo cache is persistent across container rebuilds

## Troubleshooting

### Common Issues
- **XKB Errors**: The container includes proper XKB configuration and fallbacks
- **EGL Warnings**: Software rendering fallback is enabled for containers without GPU access
- **Font Issues**: Comprehensive font library support is included

### Verification Commands
```bash
# Check display connection
echo $DISPLAY

# Test X11 connectivity
xvfb-run -a echo "X11 available"

# Check OpenGL
glxinfo | head -10

# Verify Wayland
echo $WAYLAND_DISPLAY
```

## Architecture

The container supports both X11 and Wayland display protocols, automatically adapting to your host system. The egui/eframe GUI framework in TN5250R can utilize either backend seamlessly.

### Key Components
- **eframe**: Cross-platform GUI framework
- **winit**: Window management and input handling
- **wgpu/OpenGL**: Graphics rendering backend
- **System libraries**: Complete support for modern Linux GUI development

This configuration provides a complete, production-ready development environment for TN5250R GUI development with full cross-platform compatibility.