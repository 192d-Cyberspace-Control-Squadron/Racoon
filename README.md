# Racoon Network Operating System

A modern, high-performance Network Operating System (NOS) written in Rust, designed for data center switching with vendor-independent SAI (Switch Abstraction Interface) integration.

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-0.5.0-green.svg)](Cargo.toml)

## Overview

Racoon NOS is a full-featured data center network operating system that provides:

- **L2 Switching**: Complete VLAN orchestration and hardware synchronization
- **Vendor-Neutral**: Hardware abstraction through SAI for multi-vendor support
- **Database-Centric**: SONiC-inspired architecture with Valkey/Redis state management
- **Production-Ready**: Built with Rust for memory safety, concurrency, and performance
- **Extensible**: Microservices design for future features (L3, QoS, ACLs)

## Quick Start

```bash
# 1. Start the database
docker run -d -p 6379:6379 valkey/valkey:latest

# 2. Build Racoon NOS
cargo build --release

# 3. Run the orchestration daemon
cargo run --release --bin racoon-orchd

# 4. Run the synchronization daemon (in another terminal)
# Note: Requires SAI library for hardware programming
cargo run --release --bin racoon-syncd

# 5. Test VLAN creation
./examples/vlan_create_test.sh
```

See [examples/README.md](examples/README.md) for detailed testing instructions.

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

## Features

### ✅ Implemented (v0.5.0)

**Core Infrastructure**:

- Database-centric state management with Valkey/Redis
- Pub/sub event-driven architecture
- Complete VLAN orchestration (CONFIG_DB → APPL_DB)
- Hardware synchronization via SAI (APPL_DB → ASIC)
- Type-safe VLAN ID validation (1-4094)
- Concurrent state tracking with DashMap
- Dynamic SAI library loading

**Daemons**:

- `racoon-orchd`: Orchestration daemon with VlanOrch
- `racoon-syncd`: SAI synchronization daemon with VlanSync
- Database client with connection pooling
- Structured logging with tracing

**SAI Integration**:

- Auto-generated SAI bindings (bindgen)
- Type-safe SAI API wrappers
- VLAN create/delete operations
- VLAN member management (tagged/untagged)
- FDB, LAG, Bridge, Port API foundations

**Testing & Documentation**:

- End-to-end VLAN creation test script
- Comprehensive architecture documentation
- API reference guide
- Development workflow guide

### 🚧 In Progress

- CLI interface
- Port orchestration and synchronization
- FDB synchronization
- LAG orchestration

### 📋 Planned

**Phase 2 - L3 Routing**:

- Static routes
- ARP/NDP
- Virtual Router
- Router Interface
- FRR integration (BGP, OSPF)

**Phase 3 - Advanced Features**:

- ACLs (Access Control Lists)
- QoS (Quality of Service)
- Mirroring/SPAN
- Tunneling (VXLAN, GRE)

**Phase 4 - High Availability**:

- Warm boot
- Fast reboot
- State reconciliation
- Configuration synchronization

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

## Documentation

Comprehensive documentation is available in the `docs/` directory:

- **[Architecture Guide](docs/architecture.md)**: System architecture, data flow, and design patterns
- **[Development Guide](docs/development.md)**: Building, testing, debugging, and contributing
- **[API Reference](docs/api-reference.md)**: Complete API documentation for all modules
- **[Examples](examples/README.md)**: End-to-end testing and usage examples

## Support

- **Issues**: [GitHub Issues](https://github.com/johnwillman/Racoon/issues)
- **Discussions**: [GitHub Discussions](https://github.com/johnwillman/Racoon/discussions)
- **Documentation**: [docs/](docs/)

---

**Status**: v0.5.0 - L2 VLAN Orchestration Complete

Built with Rust 🦀 for the future of network infrastructure.
