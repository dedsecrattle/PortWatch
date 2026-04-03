# PortWatch

[![Crates.io](https://img.shields.io/crates/v/portwatch.svg)](https://crates.io/crates/portwatch)
[![License](https://img.shields.io/crates/l/portwatch.svg)](https://github.com/dedsecrattle/PortWatch#license)
[![Downloads](https://img.shields.io/crates/d/portwatch.svg)](https://crates.io/crates/portwatch)

A cross-platform TUI (Terminal User Interface) for monitoring network ports and managing processes. Built with Rust for performance and portability.

<img width="1512" height="831" alt="image" src="https://github.com/user-attachments/assets/97bf7fdb-6964-44fe-97fb-376dc1cb81be" />


<img width="1505" height="830" alt="image" src="https://github.com/user-attachments/assets/71505f48-eee0-4083-9bcc-37cfe7201adc" />


## Features

- 🔍 **Real-time Port Monitoring** - View all listening and established connections
- 🖥️ **Process Details** - Inspect memory, CPU, command line, and environment variables
- ⚡ **Process Control** - Gracefully terminate or force kill processes
- 🔎 **Smart Filtering** - Filter by port number, process name, protocol, or state
- 🎨 **Rich TUI** - Clean interface with color-coded states and intuitive navigation
- 🌍 **Cross-Platform** - Works on Linux, macOS, and Windows

## Installation

### From crates.io (Recommended)

```bash
cargo install portwatch
```

### From Source

```bash
git clone https://github.com/dedsecrattle/PortWatch
cd PortWatch
cargo install --path .
```

### Build from Repository

```bash
git clone https://github.com/dedsecrattle/PortWatch
cd PortWatch
cargo build --release
./target/release/portwatch
```

## Usage

### Basic Usage

```bash
# Start PortWatch with default settings
portwatch

# Set custom refresh interval (in milliseconds)
portwatch --refresh-interval 5000

# Start with a filter applied
portwatch --filter 3000
```

### Keyboard Shortcuts

#### Navigation
- `↑` - Move up
- `↓` - Move down
- `Enter` - View detailed process information

#### Actions
- `r` - Refresh port list
- `/` - Start filter mode (type to filter)
- `Esc` - Clear filter
- `k` - Graceful stop (SIGTERM on Unix, graceful termination on Windows)
- `K` - Force kill (SIGKILL on Unix, force termination on Windows)

#### Other
- `?` - Toggle help screen
- `q` / `Ctrl+C` - Quit

### Filtering

Press `/` to enter filter mode, then type:
- Port number: `3000`
- Process name: `node`
- Protocol: `tcp` or `udp`
- State: `listen`, `established`, etc.

Press `Enter` to apply or `Esc` to cancel.

## Platform-Specific Details

### Linux
- Uses `/proc` filesystem for native port-to-PID mapping
- Parses `/proc/net/tcp`, `/proc/net/udp` for connection data
- Requires read access to `/proc` (usually available to all users)

### macOS
- Uses `lsof -i` for port scanning
- Requires `lsof` to be available (standard on macOS)

### Windows
- Uses `netstat -ano` for port scanning
- Requires `netstat` to be available (standard on Windows)

## Architecture

PortWatch uses a modular architecture with platform-specific backends:

```
src/
├── main.rs           # Entry point and event loop
├── app.rs            # Application state and reducer
├── events.rs         # Input handling and actions
├── models.rs         # Core data structures
├── backends/         # Platform-specific implementations
│   ├── mod.rs        # Backend traits
│   ├── linux.rs      # Linux /proc implementation
│   ├── macos.rs      # macOS lsof implementation
│   └── windows.rs    # Windows netstat implementation
└── ui/               # TUI components
    ├── mod.rs
    ├── theme.rs
    ├── layout.rs
    ├── ports_table.rs
    ├── details.rs
    └── footer.rs
```

## Requirements

- Rust 1.70 or later
- Platform-specific tools:
  - Linux: Access to `/proc` filesystem
  - macOS: `lsof` command
  - Windows: `netstat` command

## Permissions

Some operations may require elevated permissions:
- **Viewing ports**: Usually works without special permissions
- **Killing processes**: May require root/admin for processes owned by other users

## Future Enhancements

- Stack sampling on Linux (perf/eBPF)
- Process tree view
- Export to JSON/CSV
- Configuration file support
- Container awareness
- Language-specific detection (Node.js, Python, Java)
- Network activity sparklines

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
