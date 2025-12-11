# Release Process

This document describes how to release a new version of the Claude SDK for Rust.

## Pre-Release Checklist

Before creating a release, ensure:

- [ ] All tests pass: `cargo test --all-features`
- [ ] Clippy is clean: `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Code is formatted: `cargo fmt --all -- --check`
- [ ] Documentation builds: `cargo doc --no-deps --all-features`
- [ ] Examples work with current API
- [ ] CHANGELOG.md is updated with user-facing changes
- [ ] README.md reflects current feature status

## Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0): Breaking API changes
- **MINOR** (0.1.0): New features, backwards compatible
- **PATCH** (0.1.1): Bug fixes, backwards compatible

Until 1.0.0, minor versions may include breaking changes.

## Release Steps

### 1. Update Version Number

Update version in `Cargo.toml`:

```toml
[package]
version = "0.2.0"  # Increment appropriately
```

### 2. Update CHANGELOG.md

#### Option A: Automated (Recommended)

Use Claude to generate changelog entries from git commits:

```bash
export ANTHROPIC_API_KEY="your-api-key"
cargo run --bin update-changelog
```

This will:
1. Fetch all commits since the last release tag
2. Ask Claude to analyze and categorize them
3. Generate changelog entries in Keep a Changelog format
4. Save to `CHANGELOG_GENERATED.md`

Review and copy the generated entries to `CHANGELOG.md` under `[Unreleased]`.

#### Option B: Manual

Add a new section for the release manually:

```markdown
## [0.2.0] - 2025-12-11

### Added
- Streaming API support with SSE parsing
- Model registry with Claude 4.5/4.x/3.x models
- Bedrock regional endpoint support (global, us, eu, ap)
- Extended context window support (1M tokens)

### Changed
- Removed Default impl for MessagesRequest (breaking)
- Model constants now include metadata and constraints

### Fixed
- Corrected Bedrock global endpoint IDs
```

### 3. Update System Files

Update `.claude/system/history.txt`:

```
[2025-12-11 10:00] RELEASE: Preparing v0.2.0
[2025-12-11 10:15] RELEASE: Updated Cargo.toml version to 0.2.0
[2025-12-11 10:20] RELEASE: Updated CHANGELOG.md with release notes
```

### 4. Commit Changes

```bash
git add Cargo.toml CHANGELOG.md .claude/system/history.txt
git commit -m "chore: prepare release v0.2.0"
git push origin main
```

### 5. Create Git Tag

```bash
# Create annotated tag
git tag -a v0.2.0 -m "Release version 0.2.0

- Streaming API support
- Model registry with constraints
- Bedrock regional endpoints
- Extended context support"

# Push tag to trigger release workflow
git push origin v0.2.0
```

### 6. GitHub Actions Automation

The `release.yml` workflow will automatically:

1. ✅ Verify tag matches Cargo.toml version
2. ✅ Run full test suite
3. ✅ Check code formatting
4. ✅ Run clippy
5. ✅ Publish to crates.io
6. ✅ Create GitHub Release with notes

**Required secrets:**
- `CARGO_REGISTRY_TOKEN` - Token from crates.io for publishing

### 7. Manual Steps (if automation fails)

If the GitHub Action fails, publish manually:

```bash
# Login to crates.io
cargo login <your-token>

# Publish the crate
cargo publish --dry-run  # Test first
cargo publish            # Actual publish
```

### 8. Post-Release

After the release:

- [ ] Verify crate appears on [crates.io](https://crates.io/crates/claude-sdk)
- [ ] Check that docs are published on [docs.rs](https://docs.rs/claude-sdk)
- [ ] Test installation: `cargo add claude-sdk@0.2.0`
- [ ] Announce on Discord/Twitter if major release

## Hotfix Releases

For critical bugs in production:

1. Create a hotfix branch from the tag: `git checkout -b hotfix/v0.1.1 v0.1.0`
2. Fix the bug
3. Update version to 0.1.1 (patch bump)
4. Create PR, merge, and tag v0.1.1
5. The release workflow handles the rest

## Breaking Changes

When introducing breaking changes (before 1.0.0):

1. Document in CHANGELOG.md under `### Breaking Changes`
2. Provide migration guide in release notes
3. Consider deprecation warnings first if possible
4. Bump minor version (0.1.0 → 0.2.0)

Example breaking changes:
- Removing public APIs
- Changing function signatures
- Renaming types
- Changing default behavior

## Pre-1.0.0 Development

While in 0.x versions:
- Minor versions (0.1, 0.2) may include breaking changes
- Patch versions (0.1.1, 0.1.2) are always backwards compatible
- Document all breaking changes prominently
- Aim for 1.0.0 when API is stable and feature-complete

## Release Checklist Template

```markdown
## Release v0.X.0 Checklist

- [ ] Version bumped in Cargo.toml
- [ ] CHANGELOG.md updated
- [ ] README.md reflects new features
- [ ] All tests pass
- [ ] Clippy clean
- [ ] Code formatted
- [ ] Examples updated
- [ ] System files updated (history.txt, features.json)
- [ ] Changes committed
- [ ] Tag created: v0.X.0
- [ ] Tag pushed
- [ ] GitHub Release created
- [ ] Published to crates.io
- [ ] Verified on crates.io
- [ ] Docs published on docs.rs
```

## Troubleshooting

### "Version already exists on crates.io"
- You cannot republish the same version
- Bump to the next patch version (0.1.1 → 0.1.2)
- Create a new tag and republish

### "CI workflow failed"
- Check the error logs in GitHub Actions
- Fix locally, commit, and push
- May need to delete and recreate the tag:
  ```bash
  git tag -d v0.2.0
  git push origin :refs/tags/v0.2.0
  # Fix issues, commit
  git tag v0.2.0
  git push origin v0.2.0
  ```

### "Docs don't appear on docs.rs"
- docs.rs builds automatically after crates.io publish
- Check build status: https://docs.rs/crate/claude-sdk/latest/builds
- If build fails, check for missing documentation or examples
