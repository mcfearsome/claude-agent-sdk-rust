# Contributing to Claude SDK for Rust

Thank you for your interest in contributing! This document provides guidelines for development.

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Git
- An Anthropic API key (for integration tests)

### Initial Setup

1. Clone the repository:
```bash
git clone https://github.com/mcfearsome/claude-agent-sdk-rust.git
cd claude-agent-sdk-rust
```

2. Install git hooks:
```bash
./scripts/install-hooks.sh
```

3. Verify everything works:
```bash
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

## Development Workflow

### Making Changes

1. Create a feature branch:
```bash
git checkout -b feature/your-feature-name
```

2. Make your changes following our code guidelines (see below)

3. Run the pre-commit checks:
```bash
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all
```

4. Commit with a descriptive message:
```bash
git commit -m "feat: add support for X"
```

The pre-commit hook will automatically run checks before the commit succeeds.

### Commit Message Format

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `test:` - Test additions or changes
- `refactor:` - Code refactoring
- `chore:` - Maintenance tasks

Examples:
```
feat: add streaming API support
fix: correct Bedrock global endpoint IDs
docs: update README with streaming example
test: add tests for model validation
refactor: extract error handling helper
chore: update dependencies
```

## Code Guidelines

### Project Rules

See [.claude/system/rules.txt](.claude/system/rules.txt) for comprehensive project rules.

Key principles:

1. **No `any` type** - Always use proper types
2. **No `unwrap()` in library code** - Propagate errors properly
3. **Type-safe throughout** - Leverage Rust's type system
4. **Match official API exactly** - Field names, types, behavior must match Claude API

### Code Style

- Use `cargo fmt` - All code must be formatted
- Pass `cargo clippy -- -D warnings` - Zero warnings tolerated
- Document public APIs - All public items need doc comments with examples
- Add tests - New features need unit tests, complex features need integration tests

### Error Handling

- Use `thiserror` for library errors
- Use `anyhow` for examples/binaries
- Never use `unwrap()` or `expect()` in library code
- Always propagate errors with `?` operator
- Provide context in error messages

### Testing

```bash
# Run unit tests
cargo test

# Run with all features
cargo test --all-features

# Run integration tests (requires API key)
ANTHROPIC_API_KEY=sk-ant-... cargo test -- --ignored

# Run a specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Documentation

```bash
# Build documentation
cargo doc --no-deps --all-features

# Build and open in browser
cargo doc --no-deps --all-features --open

# Check for broken links
cargo doc --no-deps --all-features 2>&1 | grep warning
```

All public APIs must have:
- Summary description
- `# Example` section with working code
- Parameter documentation for complex functions
- Links to related types/functions

## Feature Development

### Tracking Features

We track features in `.claude/system/features.json`. Each feature has:
- `id`: Unique identifier (SDK001, SDK002, etc.)
- `name`: Short descriptive name
- `description`: What the feature does
- `status`: new / in_progress / completed
- `priority`: 1 (critical) to 5 (nice-to-have)
- `dependencies`: Other features this depends on
- `notes`: Implementation notes

### Implementing a New Feature

1. Check `.claude/system/features.json` for the next priority feature
2. Update its status to `in_progress`
3. Implement the feature following project rules
4. Add tests (unit and/or integration)
5. Update documentation
6. Mark feature as `completed` with notes
7. Update `.claude/system/history.txt` with progress

## Pull Request Process

1. Ensure all tests pass
2. Ensure clippy is clean
3. Ensure code is formatted
4. Update documentation if needed
5. Add entry to CHANGELOG.md under `[Unreleased]`
6. Create PR with:
   - Clear title and description
   - Reference any related issues
   - Explain what changed and why
   - Include examples if adding new APIs

### PR Checklist

- [ ] Tests pass (`cargo test --all-features`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Code formatted (`cargo fmt --check`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Examples work (if applicable)
- [ ] Integration tests pass (if applicable)

## Integration Tests

Integration tests require a real API key and are gated by the `#[ignore]` attribute.

```rust
#[tokio::test]
#[ignore] // Only run when explicitly requested
async fn test_actual_api_call() {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY required for integration tests");

    let client = ClaudeClient::anthropic(api_key);
    // ... test actual API behavior
}
```

Run with:
```bash
ANTHROPIC_API_KEY=sk-ant-... cargo test -- --ignored
```

## Monitoring Upstream Changes

We track the official TypeScript and Python SDKs for API changes:

- TypeScript SDK: https://github.com/anthropics/anthropic-sdk-typescript
- Python SDK: https://github.com/anthropics/anthropic-sdk-python
- API Changelog: https://docs.anthropic.com/en/api/changelog

The GitHub Actions workflow `upstream-check.yml` automatically checks for new models weekly.

## Architecture Decisions

### Why Pin<Box<dyn Stream>> for streaming?

We return `Pin<Box<dyn Stream<Item = Result<StreamEvent>>>>` rather than `impl Stream` because:
- Allows returning different stream implementations internally
- Easier to refactor without breaking API
- Works better with async trait methods (future compatibility)

### Why separate anthropic_id and bedrock_id?

Different platforms use different model ID formats:
- Anthropic API: `claude-sonnet-4-5-20250929`
- AWS Bedrock: `anthropic.claude-sonnet-4-5-20250929-v1:0`
- With prefixes: `global.anthropic.claude-sonnet-4-5-20250929-v1:0`

Storing both allows platform-specific optimizations while maintaining a unified API.

### Why no Default impl for MessagesRequest?

Hardcoding a model in `Default::default()` hides a critical choice from users. Requiring explicit `MessagesRequest::new(model, max_tokens, messages)` makes the model selection intentional and visible.

## Questions?

- Check [.claude/system/](claude/system/) for project tracking
- Read [RELEASE.md](RELEASE.md) for release process
- Open an issue for questions or bugs
- Join discussions for design decisions

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
