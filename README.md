# Racoon Network Operating System

A modern, high-performance Network Operating System (NOS) written in Rust, designed for data center switching with vendor-independent SAI (Switch Abstraction Interface) integration.

## Overview

Racoon is a full-featured data center NOS that provides:

- **L2 Switching**: VLAN management, FDB (MAC learning), Link Aggregation (LAG)
- **Vendor-Neutral**: Hardware abstraction through SAI for multi-vendor support
- **Microservices Architecture**: Modular design inspired by SONiC
- **Production-Ready**: Built with Rust for memory safety and performance
- **Extensible**: Plugin-based architecture for future features (L3, QoS, ACLs)

## Architecture

### Microservices Design

Racoon consists of independent services that communicate through a central database:

- **racoon-database**: Valkey-compatible state database with pub/sub
- **racoon-syncd**: SAI synchronization daemon (ASIC_DB → hardware)
- **racoon-orchd**: Orchestration engine (configuration → SAI objects)
- **racoon-portd**: Port state manager
- **racoon-fdbsyncd**: FDB learning synchronization
- **racoon-mgmtd**: Management interface (CLI, REST API)
- **racoon-configd**: Configuration management
- **racoon-eventd**: Event and logging system

### Data Flow

```
User (CLI/API)
  → racoon-mgmtd
  → CONFIG_DB
  → racoon-orchd (VlanOrch/FdbOrch/LagOrch)
  → APPL_DB
  → ASIC_DB
  → racoon-syncd
  → Vendor SAI Library (.so)
  → Hardware ASIC
```

## Features (Phase 1 - Current)

### L2 Switching
- ✅ VLAN creation and management (1-4094)
- ✅ VLAN member management (tagged/untagged ports)
- ✅ Dynamic MAC learning (FDB)
- ✅ Static FDB entries
- ✅ Link Aggregation (LAG / Port Channel)
- ✅ Port configuration (speed, MTU, admin state)

### Management
- ✅ Configuration via TOML files
- ✅ Comprehensive logging with tracing
- ✅ SAI library dynamic loading

## Building

### Prerequisites

```bash
# Install Rust (1.75+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies (Ubuntu/Debian)
sudo apt-get install build-essential pkg-config libclang-dev
```

### Build All Crates

```bash
# Clone repository
git clone https://github.com/johnwillman/Racoon.git
cd Racoon

# Initialize SAI submodule
git submodule update --init --recursive

# Build workspace
cargo build --release --workspace

# Run tests
cargo test --workspace
```

### Build Specific Crate

```bash
# Build just the SAI bindings
cargo build -p racoon-sai

# Build the database
cargo build -p racoon-database
```

## Project Structure

```
racoon/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── racoon-common/            # Shared utilities, types, errors
│   ├── racoon-sai/               # SAI bindings and FFI layer
│   ├── racoon-database/          # State database service
│   ├── racoon-syncd/             # SAI sync daemon
│   ├── racoon-orchd/             # Orchestration daemon
│   └── ...                       # Other daemons
├── sai/SAI/                      # SAI headers (git submodule)
├── config/                       # Configuration files
│   ├── racoon.toml              # Main configuration
│   └── platform/                 # Platform-specific configs
└── docs/                         # Documentation
```

## Configuration

### Main Configuration

Edit [config/racoon.toml](config/racoon.toml):

```toml
[platform]
name = "virtual"
sai_library = "/usr/lib/libsai.so"

[database]
host = "127.0.0.1"
port = 6379

[management]
rest_api_port = 8080
```

### Platform Configuration

Platform-specific settings in [config/platform/](config/platform/):

- `virtual.toml` - Virtual/test platform
- `broadcom_td4.toml` - Broadcom Tomahawk 4
- `mellanox_spectrum.toml` - Mellanox Spectrum (future)

## Development

### Running with Virtual SAI

For development without physical hardware:

```bash
# Install SAI Virtual Switch
# (Instructions vary by platform)

# Set SAI library path
export SAI_LIBRARY_PATH=/usr/lib/libsai_vsai.so

# Run daemons
cargo run --bin racoon-database
cargo run --bin racoon-syncd
cargo run --bin racoon-orchd
```

### Testing

```bash
# Unit tests
cargo test --workspace

# Integration tests
cargo test --test l2_switching_test

# With logging
RUST_LOG=debug cargo test
```

## Roadmap

### Phase 1: L2 Switching (Current)
- ✅ VLAN management
- ✅ FDB handling
- ✅ LAG support
- ✅ Port configuration
- 🚧 CLI and REST API
- 🚧 Integration tests

### Phase 2: L3 Routing
- Static routes
- ARP/NDP
- Virtual Router
- Router Interface
- FRRouting integration (BGP, OSPF)

### Phase 3: Advanced Features
- ACLs (Access Control Lists)
- QoS (Quality of Service)
- Mirroring/SPAN
- Tunneling (VXLAN, GRE)

### Phase 4: High Availability
- Warm boot
- Fast reboot
- Configuration synchronization

### Phase 5: Telemetry
- gNMI/OpenConfig
- Streaming telemetry
- Enhanced monitoring

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

### Development Guidelines

- Follow Rust best practices and idioms
- Add tests for new features
- Update documentation
- Run `cargo fmt` and `cargo clippy` before committing

## License

Apache License 2.0. See [LICENSE](LICENSE) for details.

## Acknowledgments

- [Open Compute Project](https://www.opencomputeproject.org/) for SAI
- [SONiC](https://sonic-net.github.io/SONiC/) for architectural inspiration
- The Rust community for excellent tooling

## Support

- GitHub Issues: https://github.com/johnwillman/Racoon/issues
- Documentation: [docs/](docs/)

---

**Status**: Phase 1 Development (L2 Switching Foundation)

Built with Rust 🦀 for the future of network infrastructure.
