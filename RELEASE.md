# Release Guide for AuthKit v0.1.0

This document outlines the steps to prepare and publish AuthKit v0.1.0 to crates.io.

---

## Pre-Release Checklist

### ✅ Code Quality

- [x] All tests passing (68/68 tests)
- [x] Zero compiler warnings
- [x] Code formatted with `rustfmt`
- [x] No clippy warnings
- [x] Documentation complete

### ✅ Documentation

- [x] README.md complete with examples
- [x] CHANGELOG.md created and up-to-date
- [x] API documentation (doc comments)
- [x] Examples directory with working examples
- [x] AGENTS.md for contributors
- [x] LICENSE files (MIT and Apache-2.0)

### ✅ Package Metadata

- [x] Cargo.toml properly configured
- [x] Version set to 0.1.0
- [x] Authors, description, keywords defined
- [x] License specified (MIT OR Apache-2.0)
- [x] Repository, homepage, documentation URLs
- [x] Exclude list for unnecessary files

### ✅ Features

- [x] Core authentication (register, login, verify, logout)
- [x] Email verification (send, verify, resend)
- [x] SQLite support
- [x] PostgreSQL support
- [x] Argon2 password hashing
- [x] bcrypt password hashing support
- [x] Comprehensive test coverage

---

## Verification Steps

Run these commands to verify everything is ready:

### 1. Clean Build

```bash
cargo clean
cargo build --all-features
```

### 2. Run All Tests

```bash
cargo test --all-features
```

Expected: **68 tests passing**

### 3. Check Documentation

```bash
cargo doc --all-features --no-deps --open
```

Verify all public APIs are documented.

### 4. Run Clippy

```bash
cargo clippy --all-features -- -D warnings
```

Expected: **No warnings**

### 5. Check Formatting

```bash
cargo fmt -- --check
```

Expected: **All files formatted**

### 6. Run Examples

```bash
cargo run --example email_verification --features sqlite
```

Expected: **Example runs successfully**

### 7. Verify Package Contents

```bash
cargo package --list
```

Review the list to ensure:
- All necessary files are included
- No sensitive files are included
- `internal/` directory is excluded
- `.github/` directory is excluded

### 8. Build Package

```bash
cargo package --allow-dirty
```

This creates a `.crate` file in `target/package/`.

### 9. Test the Package

```bash
cargo package --allow-dirty
cd target/package
tar -xzf authkit-0.1.0.crate
cd authkit-0.1.0
cargo test --all-features
```

Expected: **All tests pass from the packaged version**

---

## Publishing Steps

### Prerequisites

1. **Crates.io Account**
   - Create account at https://crates.io
   - Generate API token from account settings

2. **Login to Crates.io**
   ```bash
   cargo login <your-api-token>
   ```

### Publish (Dry Run)

First, do a dry run to ensure everything is correct:

```bash
cargo publish --dry-run --allow-dirty
```

Review the output carefully.

### Publish (For Real)

If the dry run succeeds:

```bash
cargo publish
```

⚠️ **Warning:** Publishing to crates.io is **permanent**. You cannot delete a published version.

---

## Post-Release Steps

### 1. Create Git Tag

```bash
git tag -a v0.1.0 -m "Release v0.1.0 - Initial release with email verification"
git push origin v0.1.0
```

### 2. Create GitHub Release

Go to https://github.com/Akshay2642005/authkit/releases/new

- **Tag:** v0.1.0
- **Title:** AuthKit v0.1.0 - Initial Release
- **Description:** Copy from CHANGELOG.md

Include:
- Summary of features
- Installation instructions
- Link to documentation
- Link to examples

### 3. Update Documentation

If using docs.rs:
- Verify documentation built successfully at https://docs.rs/authkit
- Check that all features are documented

### 4. Announce

Consider announcing on:
- Reddit: /r/rust
- Twitter/X: #rustlang
- This Week in Rust
- Rust Users Forum

### 5. Update Repository

After release:

1. Update version in Cargo.toml to next development version:
   ```toml
   version = "0.2.0-dev"
   ```

2. Add `[Unreleased]` section to CHANGELOG.md:
   ```markdown
   ## [Unreleased]
   
   ### Added
   - TBD
   
   ### Changed
   - TBD
   ```

3. Commit and push:
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: prepare for v0.2.0 development"
   git push
   ```

---

## Troubleshooting

### Package Too Large

If you get "package too large" error:
- Check `cargo package --list` for unexpected files
- Add more entries to `exclude` in Cargo.toml
- Consider using `.cargo_vcs_info.json`

### Missing Documentation

If docs.rs build fails:
- Ensure all dependencies are properly specified
- Check that feature flags work correctly
- Review docs.rs build log

### Test Failures

If tests fail in packaged version:
- Check file paths in tests
- Ensure test fixtures are included
- Verify feature flags are correct

### Publishing Errors

Common errors:
- **"crate name already exists"** - Name taken, choose different name
- **"token invalid"** - Run `cargo login` again
- **"insufficient permissions"** - Ensure you're logged in correctly
- **"failed to verify package"** - Run tests with `--all-features`

---

## Version Numbering

AuthKit follows [Semantic Versioning](https://semver.org/):

- **MAJOR** (x.0.0): Breaking changes
- **MINOR** (0.x.0): New features (backward compatible)
- **PATCH** (0.0.x): Bug fixes (backward compatible)

For v0.x.x releases:
- Breaking changes are allowed in minor versions
- Once v1.0.0 is released, strict semver applies

---

## Release Checklist Summary

Before publishing:

- [ ] All tests pass: `cargo test --all-features`
- [ ] No warnings: `cargo clippy --all-features -- -D warnings`
- [ ] Formatted: `cargo fmt -- --check`
- [ ] Examples work: `cargo run --example email_verification`
- [ ] Package builds: `cargo package --allow-dirty`
- [ ] Documentation complete: `cargo doc --all-features`
- [ ] CHANGELOG.md updated
- [ ] Cargo.toml metadata correct
- [ ] License files present
- [ ] Git working directory clean

After publishing:

- [ ] Create git tag
- [ ] Create GitHub release
- [ ] Verify on crates.io
- [ ] Verify on docs.rs
- [ ] Update version for next development cycle
- [ ] Announce release

---

## Quick Command Reference

```bash
# Build and test
cargo clean && cargo build --all-features
cargo test --all-features
cargo clippy --all-features -- -D warnings
cargo fmt -- --check

# Documentation
cargo doc --all-features --no-deps --open

# Package
cargo package --list
cargo package --allow-dirty
cargo publish --dry-run --allow-dirty

# Publish
cargo login <token>
cargo publish

# Git
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

---

## Support

For issues or questions:
- GitHub Issues: https://github.com/Akshay2642005/authkit/issues
- Email: akshay2642005@gmail.com

---

**Ready to release!** 🚀