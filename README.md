# warp-tui

A terminal user interface (TUI) for managing Cloudflare WARP VPN connections.

Built with Rust and [Ratatui], `warp-tui` provides a real-time, interactive terminal interface for monitoring and controlling your WARP connection status without relying on GUI applications.

## Features

- **Real-time Status Monitoring**: Live updates of WARP connection state with color-coded indicators
- **Interactive Controls**: Connect, disconnect, and refresh WARP status directly from the terminal
- **Minimal Resource Usage**: Lightweight application with < 10MB memory footprint
- **Cross-platform Support**: Works on Linux, macOS, and Windows
- **Auto-refresh**: Configurable periodic status updates (default: 1000ms)

## Prerequisites

- [Cloudflare WARP](https://developers.cloudflare.com/warp-client/) must be installed
- `warp-cli` must be available in your system PATH

## Installation

### Install Script (Linux, macOS, Windows)

Download and run the pre-built binary for your platform (x86_64 and arm64) directly from the latest [GitHub release](https://github.com/mertssmnoglu/warp-tui/releases/latest):

```bash
curl -fsSL https://raw.githubusercontent.com/mertssmnoglu/warp-tui/main/scripts/install.sh | sh
```

This installs `warp-tui` into `$HOME/.local/bin` by default. Set `INSTALL_DIR` to change the install location, or `VERSION` to install a specific release (e.g. `VERSION=v1.2.3`):

```bash
curl -fsSL https://raw.githubusercontent.com/mertssmnoglu/warp-tui/main/scripts/install.sh | INSTALL_DIR=/usr/local/bin sh
```

Each GitHub Release also attaches a standalone binary and a `.zip` archive (binary + README + LICENSE) for every OS/architecture, so you can download either format directly from the [releases page](https://github.com/mertssmnoglu/warp-tui/releases/latest).

### From Source

```bash
git clone https://github.com/mertssmnoglu/warp-tui.git
cd warp-tui
cargo install --path .
```

### Using Cargo

```bash
cargo install warp-tui
```

## Usage

Simply run the application:

```bash
warp-tui
```

## Create a Desktop entry

You can create a Linux desktop entry for `warp-tui` using the provided script:

```shell
chmod +x scripts/generate-warp-tui-desktop.sh
./scripts/generate-warp-tui-desktop.sh
```

### Controls

- **C** - Connect to WARP
- **D** - Disconnect from WARP  
- **R** - Refresh status manually
- **Q/Esc/Ctrl+C** - Quit application

## Development

### Building

```bash
cargo build
```

### Running

```bash
cargo run
```

### Testing

```bash
cargo test
```

## Technical Stack

- **Language**: Rust (Edition 2024)
- **TUI Framework**: Ratatui 0.29.0
- **Terminal Handling**: Crossterm 0.28.1
- **Async Runtime**: Tokio 1.0
- **Error Handling**: color-eyre, thiserror

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Copyright (c) Mert Şişmanoğlu <me@mertsismanoglu.com>

This project is licensed under the GNU Affero General Public License v3.0 ([LICENSE] or <https://www.gnu.org/licenses/agpl-3.0.html>).

[Ratatui]: https://ratatui.rs
[LICENSE]: ./LICENSE
