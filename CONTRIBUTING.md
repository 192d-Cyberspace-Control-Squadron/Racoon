# Contributing to Racoon NOS

Thank you for your interest in contributing to Racoon Network Operating System! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- Rust 1.75+ (install from https://rustup.rs/)
- Git
- libclang (for bindgen)

```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libclang-dev

# macOS
xcode-select --install
```

### Clone and Build

```bash
git clone https://github.com/johnwillman/Racoon.git
cd Racoon
git submodule update --init --recursive
cargo build --workspace
```

## Commit Guidelines

### Conventional Commits

We use [Conventional Commits](https://www.conventionalcommits.org/) for clear and automated versioning.

**Format:** `<type>[optional scope]: <description>`

**Types:**
- `feat:` New feature (bumps minor version: 0.1.0 → 0.2.0)
- `fix:` Bug fix (bumps patch version: 0.1.0 → 0.1.1)
- `docs:` Documentation changes
- `style:` Code style/formatting
- `refactor:` Code refactoring
- `perf:` Performance improvements
- `test:` Adding or updating tests
- `build:` Build system or dependency changes
- `ci:` CI/CD changes
- `chore:` Other changes

**Examples:**
```bash
feat(sai): add VLAN tagging mode support
fix(orchd): resolve race condition in dependency graph
docs: update installation instructions
style: run cargo fmt on all crates
refactor(database): simplify pub/sub implementation
perf(syncd): optimize SAI object creation
test(vlan): add integration tests for VLAN member operations
```

### Breaking Changes

For breaking changes, add `!` after the type or add `BREAKING CHANGE:` in the commit body:

```bash
feat(sai)!: change VlanApi signature for better error handling

BREAKING CHANGE: VlanApi::create_vlan now returns Result<VlanOid> instead of VlanOid
```

## Coding Standards

### Formatting and Linting

Code is automatically checked on commit via Git hooks:

```bash
# Format code
cargo fmt --all

# Check lints
cargo clippy --workspace --all-targets -- -D warnings

# Run tests
cargo test --workspace
```

### Code Style

- Follow Rust standard naming conventions
- Use meaningful variable and function names
- Add documentation comments (`///`) for public APIs
- Keep functions focused and concise
- Prefer explicit error handling over panics

**Example:**
```rust
/// Creates a new VLAN with the specified ID.
///
/// # Arguments
/// * `switch_id` - The SAI switch object ID
/// * `vlan_id` - VLAN identifier (1-4094)
///
/// # Returns
/// * `Ok(SaiOid)` - The created VLAN object ID
/// * `Err(RacoonError)` - If VLAN creation fails
///
/// # Example
/// ```rust
/// let vlan_id = VlanId::new(100).unwrap();
/// let vlan_oid = vlan_api.create_vlan(switch_id, vlan_id)?;
/// ```
pub fn create_vlan(&self, switch_id: SaiOid, vlan_id: VlanId) -> Result<SaiOid> {
    // Implementation
}
```

## Pull Request Process

1. **Fork** the repository
2. **Create a branch** with a descriptive name:
   ```bash
   git checkout -b feat/vlan-qos-support
   ```

3. **Make your changes** following the coding standards

4. **Test your changes**:
   ```bash
   cargo test --workspace
   cargo clippy --workspace
   ```

5. **Commit using conventional commits**:
   ```bash
   git commit -m "feat(vlan): add QoS priority tagging support"
   ```

6. **Push to your fork**:
   ```bash
   git push origin feat/vlan-qos-support
   ```

7. **Create a Pull Request** with:
   - Clear title following conventional commits
   - Description of changes
   - Link to related issues
   - Test results

## Versioning

Versions are automatically bumped based on commit types:

- `feat:` commits bump the **minor** version (0.1.0 → 0.2.0)
- `fix:` commits bump the **patch** version (0.1.0 → 0.1.1)
- Breaking changes (`!` or `BREAKING CHANGE:`) bump the **major** version (0.1.0 → 1.0.0)

The version bump happens automatically via the post-commit Git hook.

## Project Structure

```
racoon/
├── crates/              # All Rust crates
│   ├── racoon-common/   # Shared utilities
│   ├── racoon-sai/      # SAI bindings
│   ├── racoon-database/ # State database
│   ├── racoon-syncd/    # SAI sync daemon
│   ├── racoon-orchd/    # Orchestration
│   └── ...
├── sai/SAI/            # SAI headers (submodule)
├── config/             # Configuration files
├── scripts/            # Utility scripts
├── tests/              # Integration tests
└── docs/               # Documentation
```

## Areas for Contribution

### High Priority
- [ ] Complete SAI bindings constant mapping
- [ ] Implement database crate (sled-based KV store)
- [ ] Build syncd daemon (ASIC_DB → SAI)
- [ ] Create orchestrators (VlanOrch, FdbOrch, PortOrch)
- [ ] Add CLI and REST API

### Testing
- [ ] Unit tests for all crates
- [ ] Integration tests for L2 switching
- [ ] Virtual SAI adapter setup
- [ ] CI/CD improvements

### Documentation
- [ ] API reference documentation
- [ ] Architecture diagrams
- [ ] Configuration guide
- [ ] Deployment guide

## Getting Help

- **GitHub Issues**: https://github.com/johnwillman/Racoon/issues
- **Discussions**: https://github.com/johnwillman/Racoon/discussions
- **Documentation**: [docs/](docs/)

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Collaborate in good faith

## License

By contributing to Racoon, you agree that your contributions will be licensed under the Apache License 2.0.

---

Thank you for contributing to Racoon NOS! 🦀🚀
