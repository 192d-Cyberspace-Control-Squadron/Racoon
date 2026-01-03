# Racoon NOS Development Guide

## Getting Started

### Prerequisites

1. **Rust Toolchain** (1.75+):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update
   ```

2. **Valkey/Redis**:
   ```bash
   # Docker (recommended)
   docker run -d -p 6379:6379 --name racoon-db valkey/valkey:latest

   # Or native Redis
   brew install redis  # macOS
   apt install redis   # Ubuntu/Debian
   ```

3. **Development Tools**:
   ```bash
   # Install clippy and rustfmt
   rustup component add clippy rustfmt

   # Install cargo-watch for auto-rebuild
   cargo install cargo-watch
   ```

4. **SAI Headers** (optional, for development):
   ```bash
   git submodule update --init --recursive
   # SAI headers will be in sai/SAI/inc/
   ```

## Project Structure

```
racoon/
├── crates/
│   ├── racoon-common/          # Common types and utilities
│   ├── racoon-sai/             # SAI bindings and wrappers
│   ├── racoon-db-client/       # Database client
│   ├── racoon-database/        # Database schema
│   ├── racoon-orchd/           # Orchestration daemon
│   │   ├── src/
│   │   │   ├── lib.rs          # Library exports
│   │   │   ├── main.rs         # orchd binary
│   │   │   └── vlan_orch.rs    # VLAN orchestration
│   ├── racoon-syncd/           # Synchronization daemon
│   │   ├── src/
│   │   │   ├── lib.rs          # Library exports
│   │   │   ├── main.rs         # syncd binary
│   │   │   └── vlan_sync.rs    # VLAN synchronization
│   ├── racoon-cli/             # CLI tool
│   ├── racoon-mgmtd/           # Management daemon
│   ├── racoon-configd/         # Configuration daemon
│   ├── racoon-portd/           # Port daemon
│   ├── racoon-eventd/          # Event daemon
│   └── racoon-fdbsyncd/        # FDB sync daemon
├── docs/                        # Documentation
├── examples/                    # Examples and tests
├── sai/                         # SAI submodule
├── Cargo.toml                   # Workspace manifest
└── README.md
```

## Building

### Quick Build

```bash
# Build all crates
cargo build

# Build specific binary
cargo build --bin racoon-orchd
cargo build --bin racoon-syncd

# Release build (optimized)
cargo build --release
```

### Build Flags

```bash
# Enable all features
cargo build --all-features

# Disable default features
cargo build --no-default-features

# Build with specific features
cargo build --features "feature1,feature2"
```

## Running

### Start Daemons

```bash
# Terminal 1: Start database
docker run -p 6379:6379 valkey/valkey:latest

# Terminal 2: Start orchd
cargo run --bin racoon-orchd

# Terminal 3: Start syncd (requires SAI library)
# Note: Will fail without SAI library - this is expected for development
SAI_LIBRARY_PATH=/path/to/libsai.so cargo run --bin racoon-syncd
```

### Environment Variables

```bash
# Database URL
export RACOON_DB_URL="redis://127.0.0.1:6379"

# SAI library path
export SAI_LIBRARY_PATH="/usr/lib/libsai.so"

# Logging level
export RUST_LOG="debug"
export RUST_LOG="racoon_orchd=debug,racoon_syncd=info"
```

### Development Mode

```bash
# Auto-rebuild and run on file changes
cargo watch -x 'run --bin racoon-orchd'

# Run with debug logging
RUST_LOG=debug cargo run --bin racoon-orchd

# Run with backtrace on panic
RUST_BACKTRACE=1 cargo run --bin racoon-orchd
```

## Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p racoon-orchd

# Run specific test
cargo test test_vlan_id_validation

# Run tests with output
cargo test -- --nocapture

# Run tests with logging
RUST_LOG=debug cargo test
```

### Integration Tests

```bash
# End-to-end VLAN creation test
./examples/vlan_create_test.sh

# Manual database testing
redis-cli -n 4 SET "VLAN|Vlan100" '{"vlanid": 100, "description": "Test"}'
redis-cli -n 0 GET "VLAN_TABLE:Vlan100"
redis-cli -n 1 KEYS "ASIC_STATE:*"
```

### Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/
```

## Code Quality

### Pre-commit Hooks

The project has automatic pre-commit hooks that run:
1. `cargo fmt --all --check` - Code formatting
2. `cargo clippy --workspace -- -D warnings` - Linting

These run automatically on every commit.

### Manual Quality Checks

```bash
# Format code
cargo fmt --all

# Check formatting without modifying
cargo fmt --all --check

# Run clippy
cargo clippy --workspace

# Run clippy with all warnings as errors
cargo clippy --workspace -- -D warnings

# Check for security vulnerabilities
cargo audit
```

### Clippy Configuration

Allowed lints in generated SAI bindings:
```rust
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(improper_ctypes)]
#![allow(unpredictable_function_pointer_comparisons)]
#![allow(clippy::all)]
```

## Debugging

### Logging

```rust
use tracing::{info, debug, warn, error};

// Log levels
debug!("Detailed debugging info");
info!("General information");
warn!("Warning condition");
error!("Error occurred: {}", e);

// Structured logging
info!(vlan_id = %vlan_id, oid = ?oid, "Created VLAN");
```

### GDB/LLDB Debugging

```bash
# Build with debug symbols
cargo build

# Run with debugger
rust-gdb target/debug/racoon-orchd
# or
rust-lldb target/debug/racoon-orchd

# Set breakpoints
(gdb) break racoon_orchd::vlan_orch::process_vlan_config
(gdb) run

# Backtrace on panic
RUST_BACKTRACE=full cargo run
```

### Database Inspection

```bash
# Connect to database
redis-cli

# Select database
SELECT 4  # CONFIG_DB
SELECT 0  # APPL_DB
SELECT 1  # ASIC_DB

# List all keys
KEYS *

# Get specific key
GET "VLAN|Vlan100"

# Monitor all commands
MONITOR

# Subscribe to channel
SUBSCRIBE VLAN_TABLE

# Delete key
DEL "VLAN|Vlan100"

# Flush database
FLUSHDB  # Current DB only
FLUSHALL # All databases
```

## Adding New Features

### Adding a New Orchestration Agent

1. **Create module** in `racoon-orchd/src/`:
   ```rust
   // port_orch.rs
   pub struct PortOrch {
       db_client: Arc<DbClient>,
       ports: DashMap<String, PortEntry>,
   }
   ```

2. **Implement orchestration logic**:
   ```rust
   impl PortOrch {
       pub async fn process_port_config(&self, port: &str) -> Result<()> {
           // Read from CONFIG_DB
           // Validate
           // Write to APPL_DB
           // Publish notification
       }
   }
   ```

3. **Add subscriber**:
   ```rust
   #[async_trait]
   impl DbSubscriber for PortOrchSubscriber {
       async fn on_message(&self, channel: String, message: String) {
           // Handle notifications
       }
   }
   ```

4. **Export from lib.rs**:
   ```rust
   pub mod port_orch;
   pub use port_orch::{PortOrch, PortOrchSubscriber};
   ```

### Adding a New SAI API

1. **Add header** to `build.rs`:
   ```rust
   .header(format!("{}/sainewfeature.h", sai_include_path))
   ```

2. **Create wrapper module** in `racoon-sai/src/`:
   ```rust
   // newfeature.rs
   pub struct NewFeatureApi {
       api_table: *const sai_newfeature_api_t,
   }

   impl NewFeatureApi {
       pub fn create_object(&self, ...) -> Result<SaiOid> {
           // SAI API call
       }
   }
   ```

3. **Add to adapter**:
   ```rust
   // In SaiAdapter
   newfeature_api: *const sai_newfeature_api_t,

   pub fn get_newfeature_api(&self) -> &sai_newfeature_api_t {
       unsafe { &*self.newfeature_api }
   }
   ```

4. **Export from lib.rs**:
   ```rust
   pub mod newfeature;
   pub use newfeature::NewFeatureApi;
   ```

## Commit Guidelines

### Conventional Commits

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types**:
- `feat`: New feature (bumps minor version)
- `fix`: Bug fix (bumps patch version)
- `docs`: Documentation changes
- `chore`: Maintenance tasks
- `refactor`: Code refactoring
- `test`: Test additions/changes
- `perf`: Performance improvements

**Examples**:
```bash
git commit -m "feat(vlan): add VLAN member management"
git commit -m "fix(sai): handle null pointer in VLAN API"
git commit -m "docs: update architecture documentation"
git commit -m "chore: bump version to 0.5.0"
```

### Commit Process

1. **Stage changes**:
   ```bash
   git add <files>
   ```

2. **Commit** (pre-commit hooks run automatically):
   ```bash
   git commit -m "feat(orchd): add port orchestration"
   ```

3. **Version bump** (automatic):
   ```bash
   # Version is auto-bumped based on commit type
   git commit -m "chore: bump version to X.Y.Z"
   ```

4. **Push**:
   ```bash
   git push origin main
   ```

## Code Style

### Rust Idioms

```rust
// Use ? operator for error propagation
let value = function_that_might_fail()?;

// Use if let for Option/Result matching
if let Some(value) = optional {
    // use value
}

// Use pattern matching
match result {
    Ok(value) => // handle success,
    Err(e) => // handle error,
}

// Prefer iterators over loops
items.iter()
    .filter(|item| item.is_valid())
    .map(|item| item.process())
    .collect()
```

### Naming Conventions

```rust
// Types: PascalCase
struct VlanOrch { }
enum DatabaseType { }

// Functions/methods: snake_case
fn process_vlan_config() { }

// Constants: SCREAMING_SNAKE_CASE
const MAX_VLAN_ID: u16 = 4094;

// Modules: snake_case
mod vlan_orch;
```

### Documentation

```rust
/// Create a new VLAN in hardware
///
/// # Arguments
///
/// * `switch_id` - SAI switch object ID
/// * `vlan_id` - VLAN ID (1-4094)
///
/// # Returns
///
/// Returns the SAI object ID (OID) for the created VLAN
///
/// # Errors
///
/// Returns `RacoonError::InvalidVlanId` if VLAN ID is out of range
/// Returns `RacoonError::SaiError` if SAI call fails
///
/// # Example
///
/// ```rust
/// let vlan_oid = vlan_api.create_vlan(switch_id, vlan_id)?;
/// ```
pub fn create_vlan(&self, switch_id: SaiOid, vlan_id: VlanId) -> Result<SaiOid> {
    // Implementation
}
```

## Performance Profiling

### CPU Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin racoon-orchd

# View flamegraph.svg in browser
```

### Memory Profiling

```bash
# Install heaptrack
# macOS: brew install heaptrack
# Linux: apt install heaptrack

# Run with heaptrack
heaptrack target/debug/racoon-orchd

# Analyze results
heaptrack --analyze heaptrack.racoon-orchd.*.gz
```

### Benchmarking

```bash
# Add benchmark to benches/
# Run benchmarks
cargo bench

# Compare benchmarks
cargo bench --bench vlan_bench > before.txt
# Make changes
cargo bench --bench vlan_bench > after.txt
# Compare
cargo benchcmp before.txt after.txt
```

## Troubleshooting

### Common Issues

**Issue**: Compilation fails with SAI binding errors
```bash
# Solution: Clean and rebuild bindings
cargo clean
cargo build
```

**Issue**: Database connection refused
```bash
# Solution: Ensure Redis/Valkey is running
docker ps | grep valkey
redis-cli ping
```

**Issue**: Pre-commit hook fails
```bash
# Solution: Fix issues locally
cargo fmt --all
cargo clippy --workspace --fix
git add .
git commit
```

**Issue**: SAI library not found
```bash
# Solution: Set library path or disable SAI
export SAI_LIBRARY_PATH=/path/to/libsai.so
# Or accept that syncd won't start (orchd works fine)
```

### Getting Help

1. Check existing issues: https://github.com/anthropics/racoon-nos/issues
2. Read the documentation in `docs/`
3. Look at examples in `examples/`
4. Review test cases in `tests/`

## Release Process

### Version Bumping

Automatic via pre-commit hook based on commit type:
- `feat:` → Minor version bump (0.X.0)
- `fix:` → Patch version bump (0.0.X)
- `chore:`, `docs:`, etc. → No version bump

### Creating a Release

```bash
# Tag the release
git tag -a v0.5.0 -m "Release v0.5.0"
git push origin v0.5.0

# Build release binaries
cargo build --release

# Package binaries
tar -czf racoon-nos-v0.5.0-x86_64-linux.tar.gz \
    -C target/release \
    racoon-orchd racoon-syncd

# Create GitHub release with binaries
```

## Contributing

See the main README for contribution guidelines. Key points:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Ensure pre-commit hooks pass
6. Submit a pull request

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [SAI Specification](https://github.com/opencomputeproject/SAI)
- [SONiC Architecture](https://github.com/sonic-net/SONiC/wiki/Architecture)
- [Redis Commands](https://redis.io/commands/)
