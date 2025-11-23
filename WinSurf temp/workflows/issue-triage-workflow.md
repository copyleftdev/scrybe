# GitHub Issue Triage Workflow

**Description**: Process for triaging and categorizing GitHub issues for the Scrybe project

## Workflow Steps

### 1. Review New Issues

```bash
# List all new issues (no labels)
gh issue list --label "needs-triage" --limit 50

# Or list all issues without any labels
gh issue list --json number,title,labels | jq '.[] | select(.labels | length == 0)'
```

### 2. Read Issue Thoroughly

For each issue:

```bash
# View full issue details
gh issue view [ISSUE_NUMBER]
```

Read carefully:
- Issue title and description
- Steps to reproduce (if bug)
- Expected vs actual behavior
- Environment details
- Any attached logs or screenshots

### 3. Categorize by Type

Determine issue type and add appropriate label:

#### Bug Report
```bash
# Mark as bug
gh issue edit [ISSUE_NUMBER] --add-label "bug"

# Additional labels based on severity:
# - Critical: Production down, data loss, security vulnerability
# - High: Major functionality broken, significant user impact
# - Medium: Feature partially broken, workaround available
# - Low: Minor issue, cosmetic problem

gh issue edit [ISSUE_NUMBER] --add-label "severity:critical"
```

#### Feature Request
```bash
# Mark as enhancement
gh issue edit [ISSUE_NUMBER] --add-label "enhancement"

# Add priority:
# - P0: Critical feature, blocks major use case
# - P1: Important feature, significant user value
# - P2: Nice to have, moderate value
# - P3: Low priority, minimal impact

gh issue edit [ISSUE_NUMBER] --add-label "priority:p1"
```

#### Documentation
```bash
# Mark as documentation
gh issue edit [ISSUE_NUMBER] --add-label "documentation"
```

#### Question/Support
```bash
# Mark as question
gh issue edit [ISSUE_NUMBER] --add-label "question"

# If it's actually a support request:
gh issue edit [ISSUE_NUMBER] --add-label "support"
```

### 4. Add Component Labels

Identify which component is affected:

```bash
# Rust backend components
gh issue edit [ISSUE_NUMBER] --add-label "component:ingestion"    # Ingestion gateway
gh issue edit [ISSUE_NUMBER] --add-label "component:fingerprint"  # Fingerprinting engine
gh issue edit [ISSUE_NUMBER] --add-label "component:storage"      # ClickHouse/Redis
gh issue edit [ISSUE_NUMBER] --add-label "component:api"          # REST API

# JavaScript SDK
gh issue edit [ISSUE_NUMBER] --add-label "component:sdk"

# Infrastructure
gh issue edit [ISSUE_NUMBER] --add-label "component:infra"        # K8s, Docker, etc.

# Documentation
gh issue edit [ISSUE_NUMBER] --add-label "component:docs"
```

### 5. Security Issue Handling

If issue involves security:

```bash
# DO NOT add public labels
# Instead, close the issue with message:

gh issue close [ISSUE_NUMBER] --comment "
Thank you for reporting this potential security issue. 

For security vulnerabilities, please report privately to security@scrybe.io 
instead of public GitHub issues.

We take security seriously and will respond within 24 hours.

See our security policy: https://github.com/copyleftdev/scrybe/security/policy
"
```

**Action**: Forward details to security team immediately.

### 6. Reproduce Bug Issues

For bug reports, attempt to reproduce:

#### A. Check Environment
```bash
# Verify reporter's environment matches supported versions
# - Rust version
# - Node.js version (for SDK)
# - OS
# - Dependencies
```

#### B. Try to Reproduce Locally
```bash
# Follow steps from issue
# Document results in issue comment

gh issue comment [ISSUE_NUMBER] --body "
**Reproduction Attempt**

Environment:
- Rust: $(rustc --version)
- OS: $(uname -a)

Result: [Could reproduce / Could not reproduce]

Details: [Explanation]
"
```

#### C. Label Based on Reproducibility
```bash
# If reproduced
gh issue edit [ISSUE_NUMBER] --add-label "confirmed"

# If cannot reproduce
gh issue edit [ISSUE_NUMBER] --add-label "cannot-reproduce"

# If need more info
gh issue edit [ISSUE_NUMBER] --add-label "needs-info"
```

### 7. Assess Impact & Priority

Evaluate issue impact:

#### Critical Issues (Immediate Action)
- Production system down
- Data loss or corruption
- Security vulnerability
- Cannot process fingerprints

```bash
gh issue edit [ISSUE_NUMBER] --add-label "severity:critical"
gh issue edit [ISSUE_NUMBER] --add-label "priority:p0"

# Notify team immediately
echo "🚨 Critical issue #[ISSUE_NUMBER]: [TITLE]" | slack
```

#### High Priority
- Major feature broken
- Significant user impact
- Performance degradation

```bash
gh issue edit [ISSUE_NUMBER] --add-label "severity:high"
gh issue edit [ISSUE_NUMBER] --add-label "priority:p1"
```

#### Medium Priority
- Feature partially broken
- Workaround available
- Moderate user impact

```bash
gh issue edit [ISSUE_NUMBER] --add-label "severity:medium"
gh issue edit [ISSUE_NUMBER] --add-label "priority:p2"
```

#### Low Priority
- Minor bug
- Cosmetic issue
- Rare edge case

```bash
gh issue edit [ISSUE_NUMBER] --add-label "severity:low"
gh issue edit [ISSUE_NUMBER] --add-label "priority:p3"
```

### 8. Estimate Complexity

Add effort estimate:

```bash
# Small: 1-2 hours
gh issue edit [ISSUE_NUMBER] --add-label "effort:small"

# Medium: 1-3 days
gh issue edit [ISSUE_NUMBER] --add-label "effort:medium"

# Large: 1+ weeks
gh issue edit [ISSUE_NUMBER] --add-label "effort:large"

# Unknown: Need investigation
gh issue edit [ISSUE_NUMBER] --add-label "effort:unknown"
```

### 9. Assign or Add to Backlog

#### A. Assign Immediately (Critical/High)
```bash
# Assign to appropriate team member
gh issue edit [ISSUE_NUMBER] --add-assignee @username

# Add to current sprint/milestone
gh issue edit [ISSUE_NUMBER] --milestone "Sprint 5"
```

#### B. Add to Backlog (Medium/Low)
```bash
# Add to backlog project
gh issue edit [ISSUE_NUMBER] --add-label "status:backlog"
```

#### C. Mark as Help Wanted (Good First Issues)
```bash
# Good for new contributors
gh issue edit [ISSUE_NUMBER] --add-label "good-first-issue"

# Community can help
gh issue edit [ISSUE_NUMBER] --add-label "help-wanted"
```

### 10. Duplicate Detection

Check for duplicates:

```bash
# Search for similar issues
gh issue list --search "keyword" --state all

# If duplicate found
gh issue close [ISSUE_NUMBER] --comment "
Duplicate of #[ORIGINAL_ISSUE_NUMBER]

Closing in favor of the original issue. Please follow that thread for updates.
"

gh issue edit [ISSUE_NUMBER] --add-label "duplicate"
```

### 11. Won't Fix / Not Planned

For issues that won't be addressed:

```bash
gh issue close [ISSUE_NUMBER] --comment "
Thank you for the suggestion. After review, we've decided not to pursue this because:

[Reason: out of scope / by design / not aligned with roadmap / etc.]

We appreciate your feedback and encourage you to open new issues for other ideas!
"

gh issue edit [ISSUE_NUMBER] --add-label "wontfix"
```

### 12. Request More Information

If issue lacks details:

```bash
gh issue comment [ISSUE_NUMBER] --body "
Thank you for reporting this issue! To help us investigate, could you please provide:

- [ ] Steps to reproduce the issue
- [ ] Expected behavior
- [ ] Actual behavior
- [ ] Version of Scrybe (run \`scrybe-server --version\`)
- [ ] Operating system and version
- [ ] Relevant logs or error messages
- [ ] Screenshots (if applicable)

We'll review once we have these details. Thanks!
"

gh issue edit [ISSUE_NUMBER] --add-label "needs-info"
```

### 13. Add to Project Board

Organize in project board:

```bash
# Add to appropriate column
# - Triage (newly labeled issues)
# - Backlog (accepted, not scheduled)
# - To Do (scheduled for work)
# - In Progress (being worked on)
# - Review (in PR)
# - Done (closed)

gh issue edit [ISSUE_NUMBER] --add-project "Scrybe Development"
```

### 14. Link to Related Issues/PRs

Create connections:

```bash
gh issue comment [ISSUE_NUMBER] --body "
Related issues:
- #123 (similar bug)
- #456 (depends on this)

Blocked by: #789
Blocks: #012
"
```

### 15. Document Triage Decision

Add triage summary comment:

```bash
gh issue comment [ISSUE_NUMBER] --body "
**Triage Summary**

Type: Bug | Enhancement | Documentation | Question
Severity: Critical | High | Medium | Low
Priority: P0 | P1 | P2 | P3
Component: [Component name]
Effort: Small | Medium | Large | Unknown

**Decision**: [Assigned to sprint / Added to backlog / Needs more info / Won't fix]

**Next Steps**: [What happens next]
"
```

## Triage Matrix

| Type | Severity | Priority | Action |
|------|----------|----------|--------|
| Bug | Critical | P0 | Immediate fix, hotfix release |
| Bug | High | P1 | Fix in current sprint |
| Bug | Medium | P2 | Add to next sprint |
| Bug | Low | P3 | Add to backlog |
| Enhancement | - | P0 | Critical feature, prioritize |
| Enhancement | - | P1 | Add to roadmap |
| Enhancement | - | P2/P3 | Backlog |
| Question | - | - | Answer promptly, close |
| Documentation | - | P1 | Fix quickly |

## Special Cases

### RFC Required
For major changes:
```bash
gh issue edit [ISSUE_NUMBER] --add-label "needs-rfc"
gh issue comment [ISSUE_NUMBER] --body "
This change is significant and requires an RFC (Request for Comments).

Please create an RFC document following our template:
https://github.com/copyleftdev/scrybe/blob/main/docs/rfcs/RFC-TEMPLATE.md

Once the RFC is approved, we can proceed with implementation.
"
```

### Performance Issue
```bash
gh issue edit [ISSUE_NUMBER] --add-label "performance"
gh issue comment [ISSUE_NUMBER] --body "
Thank you for the performance report. To investigate:

1. Please provide benchmark results showing the issue
2. Include profiling data if available
3. Specify expected vs actual performance

We'll investigate and may create a benchmark test to track this.
"
```

### Accessibility Issue
```bash
gh issue edit [ISSUE_NUMBER] --add-label "accessibility"
# Prioritize accessibility issues - P1 by default
gh issue edit [ISSUE_NUMBER] --add-label "priority:p1"
```

## Triage Schedule

- **Daily**: Review critical and high-severity issues
- **Twice weekly**: Full triage of new issues
- **Weekly**: Review and re-prioritize backlog
- **Monthly**: Close stale issues (no activity in 60 days)

## Stale Issue Management

```bash
# Find stale issues (no activity in 60 days)
gh issue list --label "needs-info" --search "updated:<$(date -d '60 days ago' +%Y-%m-%d)"

# Add stale label
gh issue edit [ISSUE_NUMBER] --add-label "stale"

# Close after 90 days
gh issue close [ISSUE_NUMBER] --comment "
This issue has been inactive for 90 days and is being closed.

If this is still relevant, please reopen with updated information.
"
```

## Triage Metrics

Track these metrics weekly:
- Issues opened vs closed
- Average time to first response
- Average time to triage
- Backlog size by priority
- Issues by component

## Quality Checklist

After triaging each issue:
- [ ] Type label added
- [ ] Severity/Priority assigned
- [ ] Component identified
- [ ] Effort estimated
- [ ] Assigned or added to backlog
- [ ] Related issues linked
- [ ] Triage summary posted
- [ ] Removed "needs-triage" label

Remember: Good triage ensures the right work happens at the right time!
