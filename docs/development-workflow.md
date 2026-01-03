# Development Workflow

## Overview

Racoon uses automated tooling to ensure code quality and consistent versioning through Git hooks and Conventional Commits.

## Version Management

### Automatic Version Bumping

Versions are **automatically bumped** based on commit type:

| Commit Type | Version Bump | Example |
|-------------|-------------|---------|
| `feat:` | **Minor** (0.1.0 → 0.2.0) | New features |
| `fix:` | **Patch** (0.1.0 → 0.1.1) | Bug fixes |
| `feat!:` or `BREAKING CHANGE:` | **Major** (0.1.0 → 1.0.0) | Breaking changes |
| Other types | No bump | Documentation, style, refactor, etc. |

### How It Works

1. **Make your changes** and commit using conventional commits
2. **Post-commit hook** automatically runs and:
   - Detects commit type (`feat`, `fix`, etc.)
   - Bumps version in `Cargo.toml`
   - Updates `CHANGELOG.md` with new version
   - Updates `Cargo.lock`
   - Stages the changes
3. **Create a follow-up commit** to finalize the version bump:
   ```bash
   git commit -m "chore: bump version to X.Y.Z"
   ```

### Manual Version Bump

If needed, you can manually run the version bump script:

```bash
./scripts/bump-version.sh
```

## Git Hooks

### Pre-Commit Hook

**Location:** `.git/hooks/pre-commit`

Runs before each commit:
1. ✅ **Formatting check**: `cargo fmt --all -- --check`
2. ✅ **Linting**: `cargo clippy --workspace --all-targets -- -D warnings`

If either fails, the commit is blocked. Fix issues and try again.

**Bypass (use sparingly):**
```bash
git commit --no-verify -m "your message"
```

### Commit-Msg Hook

**Location:** `.git/hooks/commit-msg`

Validates commit message format:
- Must follow [Conventional Commits](https://www.conventionalcommits.org/) specification
- Blocks non-compliant messages
- Allows merge and revert commits

### Post-Commit Hook

**Location:** `.git/hooks/post-commit`

Automatically runs version bumping after `feat:` or `fix:` commits.

## Conventional Commits

### Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types

- **feat**: New feature (bumps minor)
- **fix**: Bug fix (bumps patch)
- **docs**: Documentation only
- **style**: Code style (formatting, etc.)
- **refactor**: Code refactoring
- **perf**: Performance improvement
- **test**: Adding tests
- **build**: Build system changes
- **ci**: CI/CD changes
- **chore**: Other changes

### Examples

```bash
# Feature (bumps 0.1.0 → 0.2.0)
feat(sai): add support for VLAN priority tagging

# Bug fix (bumps 0.1.0 → 0.1.1)
fix(orchd): resolve race condition in VLAN orchestrator

# Breaking change (bumps 0.1.0 → 1.0.0)
feat(database)!: redesign pub/sub API

BREAKING CHANGE: Database.subscribe() now returns Stream instead of Receiver
```

## Daily Workflow

### 1. Start Feature/Fix

```bash
# Create branch
git checkout -b feat/my-feature

# Make changes
# ...

# Check what changed
git status
git diff
```

### 2. Format and Lint

```bash
# Format code (automatic on commit, but good to run manually)
cargo fmt --all

# Check for issues
cargo clippy --workspace

# Run tests
cargo test --workspace
```

### 3. Commit

```bash
# Add files
git add .

# Commit (hooks run automatically)
git commit -m "feat(component): add new functionality"

# If it's a feat/fix, version is bumped
# Check staged changes
git status

# Commit the version bump
git commit -m "chore: bump version to X.Y.Z"
```

### 4. Push and PR

```bash
# Push branch
git push origin feat/my-feature

# Create PR on GitHub
```

## CHANGELOG Management

### Format

The [CHANGELOG.md](../CHANGELOG.md) follows [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
## [Unreleased]

### Added
- New features

### Changed
- Changes in existing functionality

### Deprecated
- Soon-to-be removed features

### Removed
- Removed features

### Fixed
- Bug fixes

### Security
- Security fixes

## [X.Y.Z] - YYYY-MM-DD
...
```

### Updating

1. **During development**: Add changes to `[Unreleased]` section
2. **On version bump**: Automated script moves unreleased changes to new version section
3. **Manual updates**: Edit CHANGELOG.md directly if needed

## Troubleshooting

### Pre-commit Hook Fails

**Problem:** Formatting or clippy errors

**Solution:**
```bash
# Fix formatting
cargo fmt --all

# Fix clippy issues
cargo clippy --workspace --all-targets -- -D warnings
# Address the warnings shown

# Try commit again
git commit -m "your message"
```

### Commit Message Rejected

**Problem:** Message doesn't follow conventional commits

**Solution:**
```bash
# Edit commit message
git commit --amend

# Or use correct format from the start
git commit -m "feat(scope): description"
```

### Version Not Bumping

**Problem:** Post-commit hook didn't run

**Solution:**
```bash
# Manually run version bump
./scripts/bump-version.sh

# Commit the changes
git commit -m "chore: bump version"
```

### Bypassing Hooks (Emergency)

```bash
# Skip all hooks (use only when necessary!)
git commit --no-verify -m "your message"
```

## Best Practices

1. **Commit Often**: Small, focused commits are easier to review
2. **Clear Messages**: Write descriptive commit messages
3. **Test First**: Run tests before committing
4. **Review Changes**: Use `git diff` to review before committing
5. **Keep CHANGELOG Updated**: Add to `[Unreleased]` as you work
6. **Version Bumps**: Let automation handle it, don't manually edit versions
7. **Breaking Changes**: Clearly document in commit body

## CI/CD Integration

GitHub Actions automatically runs on every push:

```yaml
- Checkout code with submodules
- Setup Rust toolchain
- Check formatting (cargo fmt)
- Run clippy lints
- Build workspace
- Run tests
```

See [.github/workflows/ci.yml](../.github/workflows/ci.yml) for details.

---

Happy coding! 🦀🚀
