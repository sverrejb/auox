# Auox (Aurum Oxydatum) 💰

A terminal-based banking application (currently) for SpareBank1, written in Rust.

## Features

- 💳 **Account Management** - View all your accounts, including credit cards
- 📊 **Transaction History** - Browse your transaction history with color-coded amounts
- 💸 **Transfers** - Move money between accounts.
- 🎨 **Modern TUI** - Okay looking terminal interface with animations and effects
- ⚡ **Fast & Lightweight** - Built with Rust for speed and efficiency (or because I really like Rust 🤷‍♂️)

## Prerequisites

- Rust (install from [rustup.rs](https://rustup.rs))
- SpareBank 1 banking account
- SpareBank 1 API credentials (client ID and secret)

## Installation

```bash
# Clone the repository
git clone https://github.com/sverrejb/auox.git
cd auox

# Build the project
cargo build --release

# Run the application
cargo run --release
```

## Configuration

On first run, Auox will create a config file at:
- **macOS/Linux**: `~/.config/auox/config.toml`
- **Windows**: `%APPDATA%\auox\config.toml`

Edit the config file and add your SpareBank 1 API credentials:

```toml
client_id = "your-client-id"
client_secret = "your-client-secret"
```

## Usage

### First Launch

1. Run `cargo run`.
2. Your browser will open for OAuth authentication
3. Log in to SpareBank 1 and authorize the application
4. The app will save your tokens and start automatically

### Built with:

- **ratatui** - Terminal UI framework
- **crossterm** - Terminal manipulation
- **tachyonfx** - Visual effects and animations
- **reqwest** - HTTP client
- **serde** - Serialization framework
- **tui-input** - Text input widgets


## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Disclaimer

This is an unofficial application and is not affiliated with or endorsed by SpareBank1. Use at your own risk.
