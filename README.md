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

Copyright (c) Mert Şişmanoğlu <mertssmnoglu@gmail.com>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[Ratatui]: https://ratatui.rs
[LICENSE]: ./LICENSE
