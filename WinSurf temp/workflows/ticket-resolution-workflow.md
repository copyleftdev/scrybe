# Ticket Resolution Workflow

**Description**: Complete workflow for resolving GitHub issues/tickets with mandatory quality gates - ensures acceptance criteria met, Clippy clean, cargo check passes, and all tests pass before closing and merging.

## Pre-Requisites

Before starting this workflow:
- Ticket/issue number must be provided
- Ticket must have clear acceptance criteria
- Development branch must exist

## Workflow Steps

### 1. Load and Understand the Ticket

```bash
# Get the ticket number from user or parameter
TICKET_NUMBER=[ISSUE_NUMBER]

# View full ticket details
gh issue view $TICKET_NUMBER

# Extract key information
gh issue view $TICKET_NUMBER --json title,body,labels,assignees
```

Read and understand:
- **Title**: What is being requested?
- **Description**: Detailed requirements
- **Acceptance Criteria**: What must be true when done?
- **Labels**: Type (bug/feature), priority, component
- **Linked PRs**: Is there already a PR?

### 2. Verify Acceptance Criteria Exists

Check if ticket has clear acceptance criteria:

```markdown
## Required Format in Ticket:

### Acceptance Criteria
- [ ] Criterion 1: [Specific, testable condition]
- [ ] Criterion 2: [Specific, testable condition]
- [ ] Criterion 3: [Specific, testable condition]
```

**If acceptance criteria is missing or unclear:**

```bash
gh issue comment $TICKET_NUMBER --body "
⚠️ **Missing Acceptance Criteria**

This ticket cannot be resolved without clear acceptance criteria.

Please add a checklist in this format:

### Acceptance Criteria
- [ ] Criterion 1: [What must be true]
- [ ] Criterion 2: [What must be verified]
- [ ] Criterion 3: [How to test]

Once added, development can proceed.
"
```

**STOP**: Do not proceed until acceptance criteria is added.

### 3. Find or Verify Development Branch

```bash
# Check if PR exists for this issue
EXISTING_PR=$(gh pr list --search "in:title #$TICKET_NUMBER" --json number --jq '.[0].number')

if [ -n "$EXISTING_PR" ]; then
  echo "Found existing PR #$EXISTING_PR"
  gh pr checkout $EXISTING_PR
else
  echo "No PR found. Need to create branch."
  # Branch naming: issue-123-short-description
  BRANCH_NAME="issue-${TICKET_NUMBER}-$(echo $TITLE | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | cut -c1-30)"
  git checkout -b $BRANCH_NAME
fi

# Verify on correct branch
git branch --show-current
```

### 4. Review Implementation Changes

```bash
# Show what has changed
git diff main...HEAD

# List changed files
git diff --name-only main...HEAD

# Check commit history
git log main..HEAD --oneline
```

Analyze the changes:
- Do they address the ticket requirements?
- Are they focused and minimal?
- Do they introduce unnecessary changes?

### 5. Verify Each Acceptance Criterion

**For each criterion in the ticket, verify it is met:**

#### Example Process:

**Criterion 1**: "API endpoint returns 400 for invalid input"

```bash
# Test the criterion
curl -X POST http://localhost:8080/v1/endpoint \
  -H "Content-Type: application/json" \
  -d '{"invalid": "data"}' \
  -w "\nStatus: %{http_code}\n"

# Expected: Status: 400
```

**Criterion 2**: "Fingerprint generation completes in <5ms"

```bash
# Run benchmark
cargo bench -- fingerprint

# Check output shows <5ms average
```

**Criterion 3**: "Documentation updated with new API"

```bash
# Check if docs were updated
git diff main...HEAD -- 'docs/**/*.md'

# Verify API docs include new endpoint
cat docs/api/endpoints.md | grep -A 10 "new-endpoint"
```

**Document verification results:**

```bash
gh issue comment $TICKET_NUMBER --body "
## Acceptance Criteria Verification

- [x] Criterion 1: Verified - API returns 400 for invalid input
- [x] Criterion 2: Verified - Benchmark shows 2.3ms average (target: <5ms)
- [x] Criterion 3: Verified - Documentation updated in docs/api/endpoints.md

All acceptance criteria met ✅
"
```

**If any criterion is NOT met:**

```bash
gh issue comment $TICKET_NUMBER --body "
## Acceptance Criteria Verification

- [x] Criterion 1: ✅ Verified
- [ ] Criterion 2: ❌ FAILED - Benchmark shows 12ms (target: <5ms)
- [x] Criterion 3: ✅ Verified

**Cannot proceed**: Criterion 2 must be met before merging.

Required action: Optimize implementation to meet <5ms target.
"
```

**STOP**: Do not proceed until ALL criteria pass.

### 6. Run Cargo Check

```bash
echo "=== Running cargo check ==="

# Check all packages
cargo check --workspace --all-features

# Capture exit code
CHECK_EXIT_CODE=$?

if [ $CHECK_EXIT_CODE -ne 0 ]; then
  echo "❌ cargo check FAILED"
  
  gh issue comment $TICKET_NUMBER --body "
## Quality Gate: cargo check ❌

\`cargo check\` failed with errors.

**Cannot proceed with merge until compilation errors are fixed.**

Please fix all errors and re-run this workflow.
"
  
  exit 1
else
  echo "✅ cargo check PASSED"
fi
```

**STOP**: Do not proceed if cargo check fails.

### 7. Run Clippy (Rust Linter)

```bash
echo "=== Running Clippy ==="

# Run clippy with warnings denied
cargo clippy --all-features --workspace -- -D warnings

# Capture exit code
CLIPPY_EXIT_CODE=$?

if [ $CLIPPY_EXIT_CODE -ne 0 ]; then
  echo "❌ Clippy FAILED"
  
  # Get clippy output details
  CLIPPY_OUTPUT=$(cargo clippy --all-features --workspace -- -D warnings 2>&1)
  
  gh issue comment $TICKET_NUMBER --body "
## Quality Gate: Clippy ❌

Clippy found issues that must be fixed:

\`\`\`
${CLIPPY_OUTPUT:0:2000}
\`\`\`

**Cannot proceed with merge until all Clippy warnings are resolved.**

Run locally:
\`\`\`bash
cargo clippy --all-features --workspace -- -D warnings
\`\`\`

Fix all warnings and re-run this workflow.
"
  
  exit 1
else
  echo "✅ Clippy PASSED"
fi
```

**STOP**: Do not proceed if Clippy has warnings.

### 8. Run All Tests

```bash
echo "=== Running All Tests ==="

# Run all tests with verbose output
cargo test --workspace --all-features -- --nocapture

# Capture exit code
TEST_EXIT_CODE=$?

if [ $TEST_EXIT_CODE -ne 0 ]; then
  echo "❌ Tests FAILED"
  
  # Get test failure details
  TEST_OUTPUT=$(cargo test --workspace --all-features 2>&1 | tail -n 100)
  
  gh issue comment $TICKET_NUMBER --body "
## Quality Gate: Tests ❌

Tests failed. See failures below:

\`\`\`
${TEST_OUTPUT:0:2000}
\`\`\`

**Cannot proceed with merge until all tests pass.**

Run locally:
\`\`\`bash
cargo test --workspace --all-features
\`\`\`

Fix failing tests and re-run this workflow.
"
  
  exit 1
else
  echo "✅ All Tests PASSED"
fi
```

**STOP**: Do not proceed if any tests fail.

### 9. Check Test Coverage

```bash
echo "=== Checking Test Coverage ==="

# Generate coverage report
cargo tarpaulin --workspace --out Json > coverage.json

# Extract coverage percentage
COVERAGE=$(jq '.coverage' coverage.json)

# Remove coverage.json
rm coverage.json

echo "Coverage: ${COVERAGE}%"

# Check if coverage meets 90% threshold
if (( $(echo "$COVERAGE < 90" | bc -l) )); then
  echo "❌ Coverage BELOW THRESHOLD"
  
  gh issue comment $TICKET_NUMBER --body "
## Quality Gate: Coverage ❌

Test coverage is ${COVERAGE}% (threshold: 90%)

**Cannot proceed with merge until coverage meets 90% minimum.**

Add tests to increase coverage and re-run this workflow.
"
  
  exit 1
else
  echo "✅ Coverage PASSED (${COVERAGE}%)"
fi
```

**STOP**: Do not proceed if coverage is below 90%.

### 10. Run JavaScript/TypeScript Tests (If SDK Changed)

```bash
# Check if SDK files changed
SDK_CHANGED=$(git diff --name-only main...HEAD | grep -c "^sdk/" || echo "0")

if [ "$SDK_CHANGED" -gt 0 ]; then
  echo "=== SDK Changes Detected - Running JS/TS Tests ==="
  
  cd sdk/
  
  # Install dependencies
  npm ci
  
  # Run type check
  echo "Running type check..."
  npm run type-check
  TYPE_CHECK_EXIT=$?
  
  # Run linting
  echo "Running ESLint..."
  npm run lint
  LINT_EXIT=$?
  
  # Run tests
  echo "Running tests..."
  npm test
  TEST_EXIT=$?
  
  # Run build
  echo "Building SDK..."
  npm run build
  BUILD_EXIT=$?
  
  cd ..
  
  # Check all passed
  if [ $TYPE_CHECK_EXIT -ne 0 ] || [ $LINT_EXIT -ne 0 ] || [ $TEST_EXIT -ne 0 ] || [ $BUILD_EXIT -ne 0 ]; then
    echo "❌ SDK Quality Checks FAILED"
    
    gh issue comment $TICKET_NUMBER --body "
## Quality Gate: SDK Checks ❌

SDK quality checks failed:
- Type Check: $([ $TYPE_CHECK_EXIT -eq 0 ] && echo '✅' || echo '❌')
- ESLint: $([ $LINT_EXIT -eq 0 ] && echo '✅' || echo '❌')
- Tests: $([ $TEST_EXIT -eq 0 ] && echo '✅' || echo '❌')
- Build: $([ $BUILD_EXIT -eq 0 ] && echo '✅' || echo '❌')

**Cannot proceed with merge until all SDK checks pass.**

Run locally in sdk/:
\`\`\`bash
npm run type-check
npm run lint
npm test
npm run build
\`\`\`
"
    
    exit 1
  else
    echo "✅ SDK Quality Checks PASSED"
  fi
fi
```

**STOP**: Do not proceed if SDK checks fail.

### 11. Run Security Audit

```bash
echo "=== Running Security Audit ==="

# Audit Rust dependencies
echo "Auditing Rust dependencies..."
cargo audit
RUST_AUDIT_EXIT=$?

# Audit JS dependencies (if SDK exists)
if [ -d "sdk" ]; then
  echo "Auditing JavaScript dependencies..."
  cd sdk/
  npm audit --audit-level=high
  JS_AUDIT_EXIT=$?
  cd ..
else
  JS_AUDIT_EXIT=0
fi

if [ $RUST_AUDIT_EXIT -ne 0 ] || [ $JS_AUDIT_EXIT -ne 0 ]; then
  echo "❌ Security Audit FAILED"
  
  gh issue comment $TICKET_NUMBER --body "
## Quality Gate: Security Audit ❌

Security vulnerabilities found:
- Rust Dependencies: $([ $RUST_AUDIT_EXIT -eq 0 ] && echo '✅' || echo '❌ VULNERABILITIES FOUND')
- JS Dependencies: $([ $JS_AUDIT_EXIT -eq 0 ] && echo '✅' || echo '❌ VULNERABILITIES FOUND')

**Cannot proceed with merge until all vulnerabilities are resolved.**

Run locally:
\`\`\`bash
cargo audit
cd sdk && npm audit
\`\`\`
"
  
  exit 1
else
  echo "✅ Security Audit PASSED"
fi
```

**STOP**: Do not proceed if security vulnerabilities found.

### 12. Verify Documentation Updates

```bash
echo "=== Checking Documentation Updates ==="

# Check if code changes require doc updates
CODE_CHANGED=$(git diff --name-only main...HEAD | grep -E '\.(rs|ts|js)$' | wc -l)
DOCS_CHANGED=$(git diff --name-only main...HEAD | grep -E '\.md$|^docs/' | wc -l)

if [ "$CODE_CHANGED" -gt 0 ] && [ "$DOCS_CHANGED" -eq 0 ]; then
  echo "⚠️  Code changed but no documentation updates detected"
  
  # Ask for confirmation
  gh issue comment $TICKET_NUMBER --body "
## Documentation Check ⚠️

Code changes detected but no documentation updates found.

**Question**: Does this change require documentation updates?
- API changes → Update API docs
- New features → Update user guide
- Behavior changes → Update relevant docs

If documentation is not needed, please comment why.
If documentation is needed, please update docs before merging.
"
  
  # Don't fail, but warn
  echo "Warning logged - manual verification needed"
else
  echo "✅ Documentation check OK"
fi
```

### 13. Generate Quality Gate Report

```bash
# Generate comprehensive report
gh issue comment $TICKET_NUMBER --body "
# Quality Gate Report ✅

## Acceptance Criteria
All acceptance criteria verified and met ✅

## Code Quality Checks
- [x] **cargo check**: PASSED ✅
- [x] **Clippy**: PASSED (no warnings) ✅
- [x] **Tests**: PASSED (all tests) ✅
- [x] **Coverage**: ${COVERAGE}% (≥90%) ✅
- [x] **Security Audit**: PASSED (no vulnerabilities) ✅

## SDK Checks (if applicable)
$([ "$SDK_CHANGED" -gt 0 ] && echo "- [x] **Type Check**: PASSED ✅
- [x] **ESLint**: PASSED ✅
- [x] **Tests**: PASSED ✅
- [x] **Build**: PASSED ✅" || echo "- No SDK changes")

## Documentation
- Documentation review completed

---

**Status**: All quality gates passed ✅  
**Ready for**: Code review and merge

Next step: Request code review from team member.
"
```

### 14. Create or Update Pull Request

```bash
# Check if PR exists
EXISTING_PR=$(gh pr list --search "in:title #$TICKET_NUMBER" --json number --jq '.[0].number')

if [ -n "$EXISTING_PR" ]; then
  echo "Updating existing PR #$EXISTING_PR"
  
  # Add quality gate report to PR
  gh pr comment $EXISTING_PR --body "
✅ **Quality Gates: ALL PASSED**

This PR is ready for review. All quality checks have passed:
- Acceptance criteria verified
- cargo check passed
- Clippy clean (no warnings)
- All tests passed
- Coverage ≥90%
- Security audit clean

See full report in issue #$TICKET_NUMBER
"

else
  echo "Creating new pull request"
  
  # Push branch
  git push origin HEAD
  
  # Create PR
  gh pr create \
    --title "$(gh issue view $TICKET_NUMBER --json title -q .title)" \
    --body "
Resolves #$TICKET_NUMBER

## Changes
$(git log main..HEAD --oneline)

## Quality Gates
All quality gates passed:
- [x] Acceptance criteria verified
- [x] cargo check passed
- [x] Clippy clean
- [x] All tests passed
- [x] Coverage ≥90%
- [x] Security audit clean

## Testing
Comprehensive testing completed. See quality gate report in #$TICKET_NUMBER

## Ready for Review
This PR is ready for code review and merge.
" \
    --assignee "@me"
  
  # Link PR to issue
  PR_NUMBER=$(gh pr list --head $(git branch --show-current) --json number -q '.[0].number')
  gh issue comment $TICKET_NUMBER --body "Pull Request created: #$PR_NUMBER"
fi
```

### 15. Request Code Review

```bash
# Get PR number
PR_NUMBER=$(gh pr list --search "in:title #$TICKET_NUMBER" --json number --jq '.[0].number')

if [ -n "$PR_NUMBER" ]; then
  echo "Requesting code review for PR #$PR_NUMBER"
  
  # Add reviewers (customize based on team structure)
  gh pr edit $PR_NUMBER --add-reviewer REVIEWER_USERNAME
  
  # Add label
  gh pr edit $PR_NUMBER --add-label "ready-for-review"
  
  echo "✅ Code review requested"
else
  echo "⚠️  Could not find PR to request review"
fi
```

### 16. Final Status Update

```bash
# Update issue status
gh issue comment $TICKET_NUMBER --body "
# Ticket Resolution Status

## ✅ Development Complete

All quality gates passed and PR is ready for review.

### What Was Done
- [x] All acceptance criteria verified and met
- [x] Implementation completed
- [x] All quality checks passed
- [x] Pull request created and ready for review

### Quality Metrics
- **cargo check**: ✅ PASSED
- **Clippy**: ✅ PASSED (no warnings)
- **Tests**: ✅ PASSED (all tests)
- **Coverage**: ${COVERAGE}% ✅
- **Security**: ✅ No vulnerabilities

### Next Steps
1. ✅ Code review by team member
2. ⏳ Approval
3. ⏳ Merge to main
4. ⏳ Close ticket

**Status**: Waiting for code review approval
"

# Add label to issue
gh issue edit $TICKET_NUMBER --add-label "ready-for-review"

echo ""
echo "================================================================"
echo "✅ TICKET RESOLUTION WORKFLOW COMPLETE"
echo "================================================================"
echo ""
echo "Ticket: #$TICKET_NUMBER"
echo "PR: #$PR_NUMBER"
echo "Status: Ready for code review"
echo ""
echo "All quality gates passed. Waiting for approval to merge."
echo ""
```

### 17. Post-Review Merge (After Approval)

**Once code review is approved, complete the merge:**

```bash
# Verify PR is approved
PR_NUMBER=$(gh pr list --search "in:title #$TICKET_NUMBER" --json number --jq '.[0].number')
APPROVED=$(gh pr view $PR_NUMBER --json reviewDecision -q .reviewDecision)

if [ "$APPROVED" = "APPROVED" ]; then
  echo "✅ PR approved, proceeding with merge"
  
  # Merge PR
  gh pr merge $PR_NUMBER --squash --delete-branch
  
  # Close ticket
  gh issue close $TICKET_NUMBER --comment "
# ✅ Ticket Resolved and Merged

This ticket has been successfully resolved:
- All acceptance criteria met
- All quality gates passed
- Code reviewed and approved
- Merged to main branch

**Merged PR**: #$PR_NUMBER

Thank you for your contribution!
"
  
  echo ""
  echo "================================================================"
  echo "✅ TICKET FULLY RESOLVED"
  echo "================================================================"
  echo ""
  echo "Ticket #$TICKET_NUMBER closed"
  echo "PR #$PR_NUMBER merged to main"
  echo ""
  
else
  echo "⚠️  PR not yet approved. Current status: $APPROVED"
  echo "Waiting for code review approval before merge."
fi
```

## Quality Gate Checklist

Before this workflow can complete, ALL must pass:

### Acceptance Criteria
- [ ] All acceptance criteria defined in ticket
- [ ] Each criterion independently verified
- [ ] All criteria met and documented

### Code Quality
- [ ] `cargo check` passes (no compilation errors)
- [ ] `cargo clippy` clean (no warnings with `-D warnings`)
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Test coverage ≥ 90%

### Security
- [ ] `cargo audit` clean (no vulnerabilities)
- [ ] `npm audit` clean (if SDK changed)

### SDK Quality (if applicable)
- [ ] TypeScript type check passes
- [ ] ESLint passes with no warnings
- [ ] All JavaScript tests pass
- [ ] SDK builds successfully

### Documentation
- [ ] Documentation updated if needed
- [ ] API docs current
- [ ] Examples working

### Review Process
- [ ] Pull request created
- [ ] Code review requested
- [ ] Code review approved
- [ ] Ready to merge

## Failure Handling

If ANY quality gate fails:
1. Workflow STOPS immediately
2. Failure is documented in ticket comment
3. Specific commands provided to reproduce locally
4. Clear action items specified
5. Re-run workflow after fixes

## Example Usage in Cascade

```
/ticket-resolution-workflow

Cascade: "What is the ticket/issue number?"
User: "123"

Cascade: [Runs entire workflow, checking each gate]

Output: Either
- ✅ All gates passed, ready for review
- ❌ Gate X failed, specific fix needed
```

## Integration with CI/CD

This workflow can be:
- Run manually via Cascade
- Triggered automatically on PR creation
- Enforced as required check before merge

## Success Criteria

Workflow succeeds when:
- ✅ All acceptance criteria verified
- ✅ cargo check passes
- ✅ Clippy has zero warnings
- ✅ All tests pass
- ✅ Coverage ≥ 90%
- ✅ No security vulnerabilities
- ✅ PR created and approved
- ✅ Ticket closed and merged

## Common Issues

### "Acceptance criteria not found"
**Solution**: Add clear acceptance criteria to ticket first

### "Clippy warnings won't fix"
**Solution**: Run `cargo clippy --fix` or fix manually

### "Coverage below 90%"
**Solution**: Add tests for uncovered code paths

### "Tests failing intermittently"
**Solution**: Fix flaky tests - ensure determinism

Remember: **No exceptions**. All quality gates must pass before merge. This ensures code quality, security, and maintainability of the Scrybe codebase.
