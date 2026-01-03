# Changelog

All notable changes to Racoon Network Operating System will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project foundation
- Rust workspace with 12 crates
- SAI (Switch Abstraction Interface) bindings generation with bindgen
- Dynamic SAI library loading system
- Safe Rust wrappers for SAI APIs (Switch, Port, VLAN, FDB, LAG)
- Common utilities crate (errors, config, logging, types)
- Configuration file support (TOML) with platform-specific configs
- Microservices architecture design (database-centric, SONiC-inspired)
- CI/CD pipeline with GitHub Actions
- Git hooks for automated formatting, linting, and conventional commits
- Comprehensive README and project documentation

### Infrastructure
- Cargo workspace configuration
- Pre-commit hooks for `cargo fmt` and `cargo clippy`
- Commit message validation for Conventional Commits
- Automated changelog management
- Rust edition 2024

## [0.1.0] - 2026-01-02

### Added
- Initial repository setup
- Apache 2.0 license
- Basic .gitignore and .gitattributes
