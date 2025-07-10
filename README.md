# wazuh-cli-rs

A modern, fast, and user-friendly command-line interface for managing Wazuh SIEM platform, written in Rust.

## Features

- **Agent Management**: List, add, remove, restart, and upgrade Wazuh agents
- **Service Control**: Start, stop, restart, and check status of Wazuh services  
- **Configuration Management**: View and modify CLI configuration settings
- **Interactive Mode**: Built-in shell with command completion and hints
- **Multiple Output Formats**: Table (default) and JSON output support
- **Secure Authentication**: JWT-based authentication with automatic token refresh
- **TLS Support**: Custom CA certificates and client certificate authentication
- **Progress Indicators**: Visual feedback for long-running operations
- **Cross-Platform**: Works on Linux, macOS, and Windows

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/your-org/wazuh-cli-rs.git
cd wazuh-cli-rs

# Build and install
cargo build --release
cargo install --path .
```

### Pre-built Binaries

Download the latest release for your platform from the [releases page](https://github.com/your-org/wazuh-cli-rs/releases).

## Configuration

On first run, initialize the configuration:

```bash
wazuh-cli config init
```

This creates a configuration file at `~/.wazuh-cli/config.toml`. Edit it to set your Wazuh API credentials:

```toml
[api]
host = "your-wazuh-server.com"
port = 55000
protocol = "https"

[auth]
username = "your-username"
password = "your-password"
```

## Usage

### Agent Management

```bash
# List all agents
wazuh-cli agent list

# List agents with specific status
wazuh-cli agent list --status active

# Get details for a specific agent
wazuh-cli agent get 001

# Add a new agent
wazuh-cli agent add --name "web-server-01" --ip "192.168.1.100"

# Remove an agent
wazuh-cli agent remove 001

# Restart an agent
wazuh-cli agent restart 001

# Restart all agents
wazuh-cli agent restart all

# Upgrade an agent
wazuh-cli agent upgrade 001 --version 4.8.0
```

### Service Control

```bash
# Check service status
wazuh-cli control status

# Get manager information
wazuh-cli control info

# Restart all services
wazuh-cli control restart
```

### Configuration Management

```bash
# Show current configuration
wazuh-cli config show

# Get a specific configuration value
wazuh-cli config get api.host

# Set a configuration value
wazuh-cli config set api.host new-server.com

# Edit configuration in your default editor
wazuh-cli config edit
```

### Interactive Mode

Start an interactive shell session:

```bash
wazuh-cli interactive
# or simply
wazuh-cli
```

### Output Formats

```bash
# Default table output
wazuh-cli agent list

# JSON output
wazuh-cli agent list --json

# JSON output (short flag)
wazuh-cli agent list -j
```

### Verbosity Levels

```bash
# Normal output
wazuh-cli agent list

# Verbose output
wazuh-cli -v agent list

# Very verbose output
wazuh-cli -vv agent list

# Debug output
wazuh-cli -vvv agent list
```

## Advanced Usage

### Custom Configuration File

```bash
wazuh-cli --config /path/to/custom/config.toml agent list
```

### TLS Configuration

For self-signed certificates or custom CA:

```toml
[tls]
verify = true
ca_cert = "/path/to/ca.crt"
client_cert = "/path/to/client.crt"
client_key = "/path/to/client.key"
```

### Environment Variables

- `WAZUH_CLI_CONFIG`: Path to configuration file
- `RUST_LOG`: Set logging level (e.g., `debug`, `info`, `warn`, `error`)

## Development

### Building from Source

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- agent list
```

### Project Structure

```
wazuh-cli-rs/
├── src/
│   ├── main.rs           # Entry point
│   ├── cli.rs            # CLI argument parsing
│   ├── client.rs         # Wazuh API client
│   ├── config.rs         # Configuration management
│   ├── error.rs          # Error types
│   ├── models.rs         # Data models
│   ├── output.rs         # Output formatting
│   ├── interactive.rs    # Interactive mode
│   ├── utils.rs          # Utility functions
│   └── commands/         # Command implementations
│       ├── agent.rs      # Agent commands
│       ├── control.rs    # Control commands
│       └── config.rs     # Config commands
├── tests/                # Integration tests
├── Cargo.toml           # Dependencies
└── README.md            # This file
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the GPL-2.0 License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [clap](https://github.com/clap-rs/clap) for CLI parsing
- Uses [tokio](https://tokio.rs/) for async runtime
- HTTP client powered by [reqwest](https://github.com/seanmonstar/reqwest)
- Inspired by the original Wazuh CLI proof of concept

## Security

For security concerns, please email security@example.com instead of using the issue tracker.

## Support

- Documentation: [https://docs.example.com/wazuh-cli](https://docs.example.com/wazuh-cli)
- Issues: [GitHub Issues](https://github.com/your-org/wazuh-cli-rs/issues)
- Discussions: [GitHub Discussions](https://github.com/your-org/wazuh-cli-rs/discussions)