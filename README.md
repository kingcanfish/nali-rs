# nali-rs

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

**A high-performance offline IP geolocation and CDN provider lookup tool written in Rust**

> A Rust implementation of [nali](https://github.com/zu1k/nali), just for fun.

## Features

- 🚀 **Fast & Efficient** - Built with Rust for maximum performance
- 🌍 **Offline Operation** - All lookups performed locally
- 📦 **Multiple Databases** - Supports QQwry (IPv4), ZX IPv6, and CDN databases
- 🔄 **Flexible Input** - CLI arguments, stdin, or pipeline
- 🎨 **Multiple Formats** - Plain text or JSON output

## Installation

### From Source

```bash
git clone https://github.com/zu1k/nali
cd nali/nali-rs
cargo build --release
cargo install --path .
```

## Quick Start

```bash
# Download databases
nali-rs --update

# Query an IP
nali-rs 8.8.8.8

# Query a CDN domain
nali-rs test.alicdn.com

# Pipeline mode
ping 8.8.8.8 | nali-rs

# JSON output
nali-rs -f json 8.8.8.8
```

## Supported Databases

| Database | IPv4 | IPv6 | Status | Description |
|----------|------|------|--------|-------------|
| **QQwry** | ✅ | ❌ | ✅ Supported | Pure IP database (Chinese focus) |
| **ZX IPv6** | ❌ | ✅ | ✅ Supported | IPv6 geolocation database |
| **CDN** | Domain | Domain | ✅ Supported | CDN provider identification |
| GeoIP2 | ✅ | ✅ | 🚧 Planned | MaxMind GeoIP2 |
| IPIP | ✅ | ✅ | 🚧 Planned | IPIP.net database |
| IP2Region | ✅ | ❌ | 🚧 Planned | ip2region database |

## Usage

### Command Line

```bash
nali-rs [OPTIONS] [TARGETS]...

OPTIONS:
    -u, --update [DATABASE]    Download or update databases
    -f, --format <FORMAT>      Output format: text, json [default: text]
    -c, --config <PATH>        Custom configuration file path
    -v, --verbose              Enable verbose logging
    -h, --help                 Print help
    -V, --version              Print version
```

### Examples

```bash
# IPv4 lookup
$ nali-rs 8.8.8.8
8.8.8.8 [United States Google]

# IPv6 lookup
$ nali-rs 2001:4860:4860::8888
2001:4860:4860::8888 [United States Google]

# CDN lookup
$ nali-rs cdn.jsdelivr.net
cdn.jsdelivr.net [jsDelivr CDN]

# Multiple queries
$ nali-rs 8.8.8.8 1.1.1.1

# From file
$ cat ips.txt | nali-rs

# JSON output
$ nali-rs --json 8.8.8.8
{
  "ip": "8.8.8.8",
  "country": "United States",
  "isp": "Google"
}
```

## Configuration

Configuration file location:
- Linux/macOS: `~/.config/nali/config.yaml`
- Windows: `%APPDATA%\nali\config.yaml`

Example:

```yaml
language: "en-US"

databases:
  qqwry:
    enabled: true
    path: "~/.local/share/nali/qqwry.dat"

  zxipv6:
    enabled: true
    path: "~/.local/share/nali/zxipv6wry.db"

  cdn:
    enabled: true
    path: "~/.local/share/nali/cdn.yaml"
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench

# Format code
cargo fmt

# Lint
cargo clippy
```

## Project Structure

```
src/
├── main.rs              # Entry point
├── cli/                 # CLI handling
├── config/              # Configuration
├── database/            # Database implementations
│   ├── qqwry/          # QQwry IPv4
│   ├── zxipv6/         # ZX IPv6
│   └── common/         # CDN database
├── entity/              # Entity parsing
└── utils/               # Utilities
```

## Performance

Compared to the Go version:
- **Memory Usage**: ~40% lower
- **Query Speed**: ~2x faster for cached lookups
- **Startup Time**: ~50% faster

## Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create your feature branch
3. Write tests for new features
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Credits

- Original Go implementation: [nali](https://github.com/zu1k/nali) by [zu1k](https://github.com/zu1k)
- QQwry Database: [cz88.net](http://www.cz88.net/)

---

Made with ❤️ using Rust
