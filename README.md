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

### Configuration File Location

Configuration files are searched in the following priority order:
1. Directory specified by `NALI_CONFIG_HOME` environment variable
2. Directory specified by `NALI_HOME` environment variable
3. Linux/macOS: `~/.config/nali-rs/config.yaml`
4. Linux/macOS (XDG): `$XDG_CONFIG_HOME/nali-rs/config.yaml`
5. Windows: `%APPDATA%\nali-rs\config.yaml`

### Database File Location

Database files are searched in the following priority order:
1. Paths specified in `database_paths` configuration
2. Directory specified by `NALI_DB_HOME` environment variable
3. Directory specified by `NALI_HOME` environment variable
4. Linux/macOS: `~/.local/share/nali-rs/`
5. Linux/macOS (XDG): `$XDG_DATA_HOME/nali-rs/`
6. Windows: `%APPDATA%\nali-rs\`

### Configuration Format

```yaml
# Database configuration
database:
  # Selected databases
  ipv4_database: "qqwry"
  ipv6_database: "zxipv6wry"
  cdn_database: "cdn"
  
  # Language for output
  language: "zh-CN"
  
  # Custom database file paths (overrides default locations)
  database_paths:
    qqwry: "/custom/path/qqwry.dat"
    zxipv6wry: "/custom/path/zxipv6wry.db"
  
  # Database definitions with download information
  databases:
    - name: "qqwry"
      name_alias: ["chunzhen"]
      format: "qqwry"
      file: "qqwry.dat"
      languages: ["zh-CN"]
      types: ["IPv4"]
      download_urls:
        - "https://github.com/metowolf/qqwry.dat/releases/latest/download/qqwry.dat"
    
    - name: "zxipv6wry"
      name_alias: ["zxipv6"]
      format: "ipdb"
      file: "zxipv6wry.db"
      languages: ["zh-CN"]
      types: ["IPv6"]
      download_urls:
        - "https://ip.zxinc.org/ip.7z"
    
    - name: "cdn"
      format: "yaml"
      file: "cdn.yml"
      languages: ["zh-CN"]
      types: ["CDN"]
      download_urls:
        - "https://cdn.jsdelivr.net/gh/4ft35t/cdn/src/cdn.yml"

# Output configuration
output:
  enable_colors: true
  json: false
  use_gbk: false

# Global configuration
global:
  verbose: false
```

### Environment Variables

The following environment variables can override configuration:

- `NALI_CONFIG_HOME`: Custom configuration file directory
- `NALI_DB_HOME`: Custom database file directory
- `NALI_HOME`: Custom configuration and database directory
- `NALI_DB_IP4`: Override IPv4 database name
- `NALI_DB_IP6`: Override IPv6 database name
- `NALI_DB_CDN`: Override CDN database name
- `NALI_LANG`: Override output language

### Auto-Generation

On first run, if no configuration file exists, nali-rs will automatically generate a default configuration file containing information for all supported databases.

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

not tested

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
