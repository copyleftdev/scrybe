# Git Hooks for Scrybe

Pre-commit hooks to ensure code quality before commits reach the repository.

## 🚀 Installation

```bash
# From project root
./.githooks/install.sh
```

This will install the pre-commit hook that runs automatically before every commit.

## 🔍 What Gets Checked

The pre-commit hook runs the same checks as CI/CD:

1. **Format Check** (`cargo fmt --check`)
   - Ensures code is properly formatted
   - Fast check, no modifications

2. **Clippy** (`cargo clippy -- -D warnings`)
   - Lints for common mistakes and anti-patterns
   - Zero warnings policy enforced

3. **Tests** (`cargo test --workspace`)
   - Runs all unit and integration tests
   - Ensures no broken functionality

4. **Security Audit** (`cargo audit`)
   - Checks dependencies for known vulnerabilities
   - Requires `cargo-audit` (auto-installs if missing)

5. **Build Check** (`cargo build --workspace`)
   - Verifies the project compiles
   - Catches compilation errors

## ⚡ Fast Mode

For quick commits when you're iterating:

```bash
# Install fast mode (skips tests and audit)
cp .githooks/pre-commit-fast .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

Fast mode only runs:
- Format check
- Clippy
- Build check

**Note**: CI will still run full checks, so use this responsibly.

## 🔧 Usage

### Normal Commit
```bash
git commit -m "feat: add new feature"
# Hooks run automatically
```

### Bypass Hooks (Not Recommended)
```bash
git commit --no-verify -m "wip: work in progress"
```

**Warning**: Only use `--no-verify` for:
- Work-in-progress commits on feature branches
- Emergency hotfixes (with team approval)
- Local experimentation

CI will catch issues anyway, but bypassing hooks slows down the feedback loop.

## 🛠️ Troubleshooting

### Hook Fails on Format
```bash
# Fix formatting automatically
cargo fmt --all

# Then commit again
git commit
```

### Hook Fails on Clippy
```bash
# See warnings
cargo clippy --workspace -- -D warnings

# Fix issues, then commit
```

### Hook Fails on Tests
```bash
# Run tests to see failures
cargo test --workspace

# Fix tests, then commit
```

### Hook Fails on Security Audit
```bash
# See vulnerability details
cargo audit

# Update dependencies or add exceptions
# Then commit
```

### Hook Takes Too Long
```bash
# Use fast mode for development
cp .githooks/pre-commit-fast .git/hooks/pre-commit

# Or bypass once
git commit --no-verify
```

## 📝 Customization

### Skip Specific Checks

Edit `.git/hooks/pre-commit` and comment out sections you don't need:

```bash
# # 3. Tests (commented out)
# echo "3️⃣  Running tests..."
# if cargo test --workspace --all-features; then
#     echo -e "${GREEN}✓ All tests passed${NC}"
# fi
```

### Add Custom Checks

Add new checks before the final success message:

```bash
# 6. Custom Check
echo "6️⃣  Running custom check..."
if ./scripts/custom-check.sh; then
    echo -e "${GREEN}✓ Custom check passed${NC}"
else
    echo -e "${RED}✗ Custom check failed${NC}"
    exit 1
fi
```

## 🔄 Updating Hooks

When hooks are updated in the repository:

```bash
# Re-run installer
./.githooks/install.sh
```

## 🗑️ Uninstalling

```bash
# Remove pre-commit hook
rm .git/hooks/pre-commit

# Restore git defaults
git config --unset core.hooksPath
```

## 🎯 Best Practices

1. **Run hooks before pushing** - Even if you bypass locally, CI will catch issues
2. **Keep hooks fast** - Long-running hooks slow development
3. **Fix issues immediately** - Don't accumulate technical debt
4. **Use fast mode during development** - Switch to full mode before PR
5. **Never commit broken code** - Even with `--no-verify`

## 📊 Hook Performance

Typical run times on a modern laptop:

| Check | Time | Can Skip? |
|-------|------|-----------|
| Format | < 1s | ❌ No |
| Clippy | 5-10s | ⚠️ Not recommended |
| Tests | 10-30s | ✅ Yes (fast mode) |
| Audit | 2-5s | ✅ Yes (fast mode) |
| Build | 5-15s | ❌ No |

**Total (full)**: ~30-60 seconds  
**Total (fast)**: ~10-20 seconds

## 🔗 Integration with CI/CD

The hooks mirror `.github/workflows/ci.yml`:

- ✅ Same checks run locally and in CI
- ✅ Catch issues before pushing
- ✅ Faster feedback loop
- ✅ Reduced CI failures

## 🆘 Support

If hooks are causing issues:

1. Check you're on latest main: `git pull origin main`
2. Reinstall hooks: `./.githooks/install.sh`
3. Try fast mode: `cp .githooks/pre-commit-fast .git/hooks/pre-commit`
4. Report issues in GitHub

---

**Pro Tip**: Set up a git alias for quick bypass:

```bash
git config alias.commit-quick 'commit --no-verify'

# Use it
git commit-quick -m "wip: quick commit"
```
