# DemonHide

[![CI](https://github.com/homeos-linux/demonhide/workflows/CI/badge.svg)](https://github.com/homeos-linux/demonhide/actions/workflows/ci.yml)
[![Code Quality](https://github.com/homeos-linux/demonhide/workflows/Code%20Quality/badge.svg)](https://github.com/homeos-linux/demonhide/actions/workflows/quality.yml)
[![Release](https://github.com/homeos-linux/demonhide/workflows/Release/badge.svg)](https://github.com/homeos-linux/demonhide/actions/workflows/release.yml)

A lightweight daemon for automatically managing pointer constraints on Wayland compositors for XWayland fullscreen applications with hidden cursors.

## Overview

DemonHide monitors XWayland applications and automatically locks the mouse pointer when fullscreen applications with hidden cursors are detected, preventing cursor movement outside the application window. This is particularly useful for:

- Fullscreen applications running through XWayland
- Media players and video applications
- Preventing cursor "escaping" during fullscreen use
- Improving user experience on multi-monitor setups

## Features

- 🖥️ **Automatic fullscreen detection** - Detects XWayland fullscreen applications
- 👁️ **Cursor visibility detection** - Monitors cursor state using X11/XFixes
- 🔒 **Wayland pointer locking** - Uses modern Wayland pointer constraints protocol
- 🚀 **Pure Rust implementation** - Fast, safe, and reliable
- ⚡ **Real-time monitoring** - Responsive detection and locking
- 🛡️ **Resource efficient** - Minimal system impact

## Requirements

- **Wayland compositor** with pointer constraints support (most modern compositors)
- **XWayland** for X11 application compatibility
- **Rust** 1.82+ for building
- **System packages**:
  - `wayland-client` library
  - `wayland-protocols`
  - `glib2`
  - `libX11` and `libXfixes` (for cursor detection)

### Supported Compositors

- Sway
- GNOME Shell (Mutter)
- KDE Plasma (KWin)
- wlroots-based compositors
- Most compositors supporting `zwp_pointer_constraints_v1`

## Installation

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd demonhide

# Build release version
cargo build --release

# Install (optional)
sudo cp target/release/demonhide /usr/local/bin/
```

### Dependencies (Fedora/RHEL)

```bash
sudo dnf install wayland-devel wayland-protocols-devel glib2-devel libX11-devel libXfixes-devel
```

### Dependencies (Ubuntu/Debian)

```bash
sudo apt install libwayland-dev wayland-protocols libglib2.0-dev libx11-dev libxfixes-dev
```

### Dependencies (Arch Linux)

```bash
sudo pacman -S wayland wayland-protocols glib2 libx11 libxfixes
```

## Usage

### Manual Execution

```bash
# Run in foreground
./target/release/demonhide

# Run in background
./target/release/demonhide &
```

### Systemd User Service

Create `~/.config/systemd/user/demonhide.service`:

```ini
[Unit]
Description=DemonHide Pointer Lock Daemon
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/local/bin/demonhide
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
```

Enable and start:

```bash
systemctl --user daemon-reload
systemctl --user enable demonhide.service
systemctl --user start demonhide.service
```

### Desktop Autostart

Create `~/.config/autostart/demonhide.desktop`:

```ini
[Desktop Entry]
Type=Application
Name=DemonHide
Exec=/usr/local/bin/demonhide
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
```

## Application Detection

DemonHide automatically detects when to lock the pointer by monitoring:

### Detection Criteria
- **XWayland session**: Both `WAYLAND_DISPLAY` and `DISPLAY` environment variables present
- **Fullscreen applications**: Applications covering the entire screen dimensions
- **Hidden cursor**: Applications that have hidden or minimized their cursor (≤1x1 pixels)

### Technical Details
- Uses X11 `XGetInputFocus` to find the currently focused window
- Checks window attributes with `XGetWindowAttributes` for fullscreen detection
- Uses XFixes extension (`XFixesGetCursorImage`) for cursor visibility detection

## Configuration

Currently, DemonHide works with sensible defaults. Future versions may include:

- Custom application patterns
- Configurable detection sensitivity
- Per-application settings
- GUI configuration tool

## Troubleshooting

### Pointer Lock Not Working

1. **Check compositor support**:
   ```bash
   # Verify your compositor supports pointer constraints
   wayland-info | grep zwp_pointer_constraints
   ```

2. **Check running games**:
   ```bash
   # Monitor detected processes (debug build)
   cargo run
   ```

2. **Check XWayland availability**:
   ```bash
   echo $DISPLAY  # Should show XWayland display (e.g., :0)
   ```

3. **Verify Wayland session**:
   ```bash
   echo $XDG_SESSION_TYPE  # Should output "wayland"
   ```

### Permission Issues

- Ensure the binary has execute permissions
- For system-wide installation, verify PATH includes `/usr/local/bin`

### High CPU Usage

- Normal CPU usage should be minimal (~0.1%)
- High usage may indicate X11 connection issues
- Check for excessive debug output in non-release builds

### XFixes Extension Issues

```bash
# Check if XFixes extension is available
xdpyinfo | grep XFIXES
```

## Development

### Building for Development

```bash
# Debug build with additional logging
cargo build

# Run with debug output
cargo run

# Run tests
cargo test
```

### Architecture

- **Process Detection**: Uses `sysinfo` crate for efficient process monitoring
- **Wayland Integration**: Direct protocol implementation via `wayland-client`
- **Event Loop**: GLib main loop for event handling
- **State Management**: Debounced state machine for reliable game detection

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Acknowledgments

- Wayland developers for the pointer constraints protocol
- Rust Wayland ecosystem maintainers
- Gaming on Linux community

## Changelog

### v0.1.0
- Initial release
- Pure Rust implementation
- Automatic game detection
- Wayland pointer constraints support
- Systemd service support
