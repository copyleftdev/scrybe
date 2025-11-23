# Contributing to Scrybe

Thank you for your interest in contributing to Scrybe! This document provides guidelines and instructions for contributing.

## Code of Conduct

This project adheres to a Code of Conduct. By participating, you are expected to uphold this code.

## Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git

### Getting Started

1. Fork and clone the repository:
   ```bash
   git clone https://github.com/yourusername/scrybe.git
   cd scrybe
   ```

2. Build the project:
   ```bash
   cargo build --workspace
   ```

3. Run tests:
   ```bash
   cargo test --workspace
   ```

## TigerStyle Compliance

Scrybe follows [TigerStyle](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md) coding standards:

### Critical Rules

1. **No `unwrap()` or `panic!()` in production code**
   - Use `Result` and `Option` with proper error handling
   - Use `.expect()` only in tests with detailed error messages

2. **Explicit error handling**
   - Use `map_err` for error conversion (no `From` implementations)
   - Provide detailed error context

3. **Type safety**
   - Leverage the type system to prevent invalid states
   - Use newtype patterns for domain-specific types

4. **Documentation**
   - All public APIs must have rustdoc comments
   - Include examples in documentation
   - Document errors and panics

5. **Testing**
   - Minimum 90% test coverage required
   - Write unit tests in same file with `#[cfg(test)]`
   - Integration tests in `tests/` directory

## Pull Request Process

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**
   - Follow TigerStyle guidelines
   - Write tests for new functionality
   - Update documentation

3. **Run quality checks**
   ```bash
   # Format code
   cargo fmt --all

   # Check for warnings
   cargo clippy --workspace -- -D warnings

   # Run tests
   cargo test --workspace

   # Check documentation
   cargo doc --workspace --no-deps
   ```

4. **Commit with conventional commits**
   ```bash
   git commit -m "feat(core): add fingerprint validation"
   git commit -m "fix(gateway): handle shutdown gracefully"
   git commit -m "docs(readme): update installation instructions"
   ```

   Prefixes:
   - `feat`: New feature
   - `fix`: Bug fix
   - `docs`: Documentation changes
   - `test`: Test additions/changes
   - `refactor`: Code refactoring
   - `perf`: Performance improvements
   - `chore`: Build/tooling changes

5. **Push and create PR**
   ```bash
   git push origin feature/your-feature-name
   ```

   Then create a pull request on GitHub.

## PR Checklist

Before submitting a PR, ensure:

- [ ] Code follows TigerStyle guidelines
- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace -- -D warnings`
- [ ] Code is formatted: `cargo fmt --all`
- [ ] Documentation is updated
- [ ] Test coverage is maintained (>90%)
- [ ] Commit messages follow conventional commits
- [ ] PR description clearly explains changes

## Issue Guidelines

When creating issues:

1. **Search existing issues** to avoid duplicates
2. **Use appropriate labels** (bug, enhancement, documentation, etc.)
3. **Provide context**:
   - For bugs: steps to reproduce, expected vs actual behavior
   - For features: use case, proposed solution
4. **Link to relevant RFCs** if applicable

## RFC Process

For major features or architectural changes:

1. Create an RFC in `docs/rfcs/`
2. Follow the RFC template
3. Open a PR for the RFC
4. Gather feedback from reviewers
5. Once approved, implement in separate PR(s)

## Questions?

- **Technical questions**: Open a GitHub discussion
- **Bug reports**: Open a GitHub issue
- **Security issues**: Email security@scrybe.io (create this email)

Thank you for contributing to Scrybe! 🦉
