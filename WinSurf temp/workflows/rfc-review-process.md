# RFC Review Process

**Description**: Complete workflow for reviewing and approving RFCs (Request for Comments) documents

## Workflow Steps

### 1. Identify the RFC to Review

First, determine which RFC needs review:

```bash
# List all RFCs in draft or review status
ls -la docs/rfcs/RFC-*.md
```

Ask the user which RFC number to review, or check the open PR for RFC filename.

### 2. Read the RFC Thoroughly

Read the entire RFC document:

```bash
# Open the RFC
cat docs/rfcs/RFC-XXXX-[topic].md
```

Pay special attention to:
- Summary and motivation
- Technical design
- Security considerations
- Performance implications
- Testing strategy

### 3. Multi-Perspective Review

Analyze the RFC from multiple expert perspectives:

#### A. Architecture Review
- Is the design scalable and maintainable?
- Does it integrate well with existing components?
- Are there simpler alternatives?
- Document concerns in review notes

#### B. Security Review
- Are all security considerations addressed?
- Is input validation comprehensive?
- Are cryptographic operations correct?
- Check for authentication/authorization gaps
- Verify GDPR compliance if applicable

#### C. Performance Review
- What is the performance impact?
- Are there bottlenecks?
- Can this handle 100k req/sec?
- Review latency targets (< 10ms p99)

#### D. Privacy Review
- Is PII collection avoided?
- Is consent properly handled?
- Are retention policies defined?
- Check for data minimization

#### E. Testing Review
- Is the testing strategy comprehensive?
- Are edge cases covered?
- Is test coverage ≥ 90% planned?
- Are benchmarks defined?

#### F. Operations Review
- How will this be deployed?
- What are the rollback procedures?
- Are monitoring/alerting plans clear?
- Is there a disaster recovery plan?

### 4. Create Review Summary

Create a structured review document:

```markdown
# RFC-XXXX Review Summary

**Reviewer**: [Your Name]  
**Date**: [YYYY-MM-DD]  
**Status**: [APPROVED | NEEDS CHANGES | REJECTED]

## Executive Summary

[One paragraph summary of review conclusion]

## Strengths

- [List positive aspects]

## Concerns

### Critical Issues (Must Fix)
- [ ] Issue 1: [Description]
- [ ] Issue 2: [Description]

### Major Issues (Should Fix)
- [ ] Issue 3: [Description]

### Minor Issues (Consider)
- Issue 4: [Description]

## Security Analysis

[Security-specific findings]

## Performance Analysis

[Performance implications and recommendations]

## Recommendations

1. [Recommendation 1]
2. [Recommendation 2]

## Approval Conditions

- [ ] All critical issues addressed
- [ ] Security review sign-off
- [ ] Performance benchmarks added
- [ ] Test coverage plan approved
```

### 5. Document Findings in GitHub

If there's a PR associated with the RFC:

```bash
# Add review comments to PR
gh pr review [PR_NUMBER] --comment -b "
## Review Summary

[Paste your review summary]
"
```

### 6. Update RFC Status

If approved:

```bash
# Update RFC header
# Change Status from 'Review' to 'Accepted'
```

Edit the RFC file to update:
```markdown
**Status**: Accepted  
**Reviewed**: YYYY-MM-DD  
**Reviewers**: [Names]
```

### 7. Create Follow-Up Issues

For any action items identified:

```bash
# Create GitHub issues for implementation tasks
gh issue create --title "Implement RFC-XXXX: [Topic]" --body "
## Implementation Tasks

From RFC-XXXX:

- [ ] Task 1
- [ ] Task 2

**References**:
- RFC: docs/rfcs/RFC-XXXX-[topic].md
- Review: [link to review]
"
```

### 8. Notify Stakeholders

Summarize the review outcome and notify the team:

```
Review complete for RFC-XXXX: [Topic]

Status: [APPROVED/NEEDS CHANGES]

Key Findings:
- [Summary point 1]
- [Summary point 2]

Next Steps:
- [Action item 1]
- [Action item 2]

Full review: [link]
```

## Quality Gates

Before approving an RFC:
- [ ] All sections complete and detailed
- [ ] Security implications documented
- [ ] Performance targets defined
- [ ] Testing strategy comprehensive
- [ ] API design reviewed
- [ ] Implementation plan realistic
- [ ] No critical blockers identified
- [ ] Code examples compile/run
- [ ] Diagrams clear and accurate

## Review Checklist

Use this checklist for every RFC review:

### Completeness
- [ ] Summary clear and concise
- [ ] Motivation well-articulated
- [ ] Technical design detailed
- [ ] API design specified
- [ ] Security section complete
- [ ] Performance considerations addressed
- [ ] Testing strategy defined
- [ ] Implementation plan broken down
- [ ] Success metrics identified

### Technical Soundness
- [ ] Architecture scalable
- [ ] Design patterns appropriate
- [ ] Error handling comprehensive
- [ ] Edge cases considered
- [ ] Dependencies justified

### Security & Privacy
- [ ] Threat model documented
- [ ] Input validation specified
- [ ] Authentication/authorization clear
- [ ] PII collection avoided
- [ ] GDPR compliance verified

### Testability
- [ ] Unit tests planned
- [ ] Integration tests defined
- [ ] Performance tests specified
- [ ] Coverage targets set

### Operations
- [ ] Deployment strategy clear
- [ ] Rollback plan defined
- [ ] Monitoring planned
- [ ] Disaster recovery addressed

## Common Issues to Watch For

### Red Flags
- Vague requirements or goals
- Missing security considerations
- No performance targets
- Unclear API boundaries
- Incomplete error handling
- Missing test strategy
- No rollback plan

### Green Flags
- Clear, specific requirements
- Comprehensive threat model
- Concrete performance benchmarks
- Well-defined APIs
- Detailed test plan
- Operational runbooks

## After Approval

Once RFC is approved:
1. Update RFC status to "Accepted"
2. Create implementation issues
3. Add to project roadmap
4. Schedule implementation phases
5. Assign ownership

## If Changes Needed

If RFC needs revision:
1. Document all required changes
2. Set clear approval criteria
3. Request specific updates
4. Schedule follow-up review
5. Track progress on changes

Remember: The goal is to ensure high-quality, well-thought-out designs before implementation begins. Be thorough but constructive in reviews.
