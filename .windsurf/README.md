# Scrybe - Windsurf Rules & Workflows

This directory contains comprehensive rules and workflows for developing the Scrybe browser fingerprinting system using Windsurf Cascade.

> **Note**: This is temporarily named "WinSurf temp" instead of `.windsurf` to avoid conflicts with system restrictions. When ready for production use, rename this directory to `.windsurf`.

## 📁 Directory Structure

```
WinSurf temp/
├── rules/                              # Coding standards and guidelines
│   ├── tigerstyle-rust-coding.md     # Rust coding standards (TigerStyle)
│   ├── security-privacy-requirements.md # Security & privacy rules
│   ├── testing-quality-standards.md   # Testing requirements
│   ├── javascript-typescript-sdk.md   # JS/TS SDK standards
│   └── rfc-documentation-standards.md # Documentation guidelines
│
└── workflows/                          # Repeatable process workflows
    ├── rfc-review-process.md          # Review RFCs
    ├── code-review-workflow.md        # Review pull requests
    ├── security-audit-workflow.md     # Security audits
    ├── testing-ci-workflow.md         # Testing & CI checks
    ├── deployment-workflow.md         # Production deployment
    ├── issue-triage-workflow.md       # GitHub issue management
    ├── feature-implementation-workflow.md # End-to-end feature development
    └── ticket-resolution-workflow.md  # Complete ticket with quality gates
```

## 🎯 Quick Start

### Using Rules

Rules are automatically applied by Windsurf Cascade based on their activation mode:

- **Always On**: Applied to all relevant files automatically
- **Glob Pattern**: Applied when file matches pattern (e.g., `*.rs`, `*.ts`)
- **Manual**: Activated with `@mention` in Cascade

To reference a rule manually:
```
@tigerstyle-rust-coding Please review this Rust code for compliance
```

### Using Workflows

Workflows are invoked in Cascade using slash commands:

```
/rfc-review-process          # Review an RFC document
/code-review-workflow        # Review a pull request
/security-audit-workflow     # Run security audit
/testing-ci-workflow         # Run full test suite
/deployment-workflow         # Deploy to production
/issue-triage-workflow       # Triage GitHub issues
/feature-implementation-workflow  # Implement new feature
/ticket-resolution-workflow  # Resolve ticket with quality gates
```

## 📋 Rules Overview

### 1. TigerStyle Rust Coding Standards
**File**: `rules/tigerstyle-rust-coding.md`  
**Activation**: Always On (for `*.rs`, `Cargo.toml`)

Core principles:
- **Safety First**: No `unwrap()` or `panic!()` in production
- **Error Handling**: Always use `Result` with context
- **Type Safety**: Leverage type system for correctness
- **Testing**: Minimum 90% coverage
- **Documentation**: All public APIs documented

### 2. Security & Privacy Requirements
**File**: `rules/security-privacy-requirements.md`  
**Activation**: Always On

Critical requirements:
- Zero trust input validation
- HMAC-SHA256 authentication
- No PII collection
- GDPR compliance
- Constant-time cryptographic comparisons
- Rate limiting on all endpoints

### 3. Testing & Quality Standards
**File**: `rules/testing-quality-standards.md`  
**Activation**: Always On

Requirements:
- 90% minimum test coverage
- Unit, integration, and property-based tests
- Benchmarks for performance-critical code
- No clippy warnings
- Formatted code (rustfmt)

### 4. JavaScript/TypeScript SDK Standards
**File**: `rules/javascript-typescript-sdk.md`  
**Activation**: Glob (`*.ts`, `*.js`, `*.tsx`, `*.jsx`)

Standards:
- TypeScript strict mode
- Bounded collections (DoS prevention)
- GDPR consent management
- HMAC authentication
- Graceful degradation

### 5. RFC & Documentation Standards
**File**: `rules/rfc-documentation-standards.md`  
**Activation**: Glob (`docs/**/*.md`, `*.md`)

Guidelines:
- RFC structure and template
- API documentation format
- Clear, concise writing style
- Working code examples
- Mermaid diagrams for architecture

## 🔄 Workflows Overview

### 1. RFC Review Process
**Command**: `/rfc-review-process`

Reviews design documents from multiple perspectives (architecture, security, performance, privacy, testing, operations).

**Use when**: Reviewing RFC documents before implementation approval.

### 2. Code Review Workflow
**Command**: `/code-review-workflow`

Comprehensive code review covering safety, security, performance, testing, and documentation.

**Use when**: Reviewing pull requests.

### 3. Security Audit Workflow
**Command**: `/security-audit-workflow`

Full security audit including dependency scanning, code analysis, penetration testing scenarios, and compliance checks.

**Use when**: Quarterly audits, pre-release, or after security incidents.

### 4. Testing & CI Workflow
**Command**: `/testing-ci-workflow`

Complete testing suite: unit, integration, E2E, performance, and quality checks.

**Use when**: Before every commit, in PR checks, and pre-deployment.

### 5. Deployment Workflow
**Command**: `/deployment-workflow`

Safe production deployment with blue-green or canary strategies, verification, and rollback procedures.

**Use when**: Deploying to production.

### 6. Issue Triage Workflow
**Command**: `/issue-triage-workflow`

Systematic GitHub issue categorization, prioritization, and assignment.

**Use when**: Managing GitHub issues.

### 7. Feature Implementation Workflow
**Command**: `/feature-implementation-workflow`

End-to-end feature development from design through deployment.

**Use when**: Implementing new features.

### 8. Ticket Resolution Workflow
**Command**: `/ticket-resolution-workflow`

Complete ticket resolution with mandatory quality gates: verifies acceptance criteria, runs cargo check, Clippy, all tests, coverage checks, and security audits before allowing merge.

**Use when**: Resolving GitHub issues/tickets - ensures all quality standards met before closing.

## 🎨 Scrybe-Specific Context

### Project Overview
Scrybe is a high-fidelity, Rust-powered browser fingerprinting and anti-bot detection system with:
- Multi-layer fingerprinting (TLS, HTTP, browser, behavioral)
- Real-time anomaly detection
- Privacy-first design (GDPR compliant)
- High performance (100k req/sec target)

### Tech Stack
- **Backend**: Rust (TigerStyle compliant)
- **SDK**: TypeScript with bounded collections
- **Storage**: ClickHouse (analytics) + Redis (cache)
- **Security**: HMAC-SHA256, TLS 1.3, nonce validation
- **Deployment**: Kubernetes with blue-green strategy

### Performance Targets
- Ingestion: 100k sessions/sec
- Latency: <10ms p99
- Fingerprint generation: <5ms
- Redis lookup: <1ms
- Test coverage: ≥90%

### Security Requirements
- No PII collection
- IP hashing (SHA-256 + salt)
- GDPR Article 6(1)(a) compliance
- 90-day data retention
- Right to erasure support

## 🚀 Common Use Cases

### Starting Development
```
/feature-implementation-workflow
```
Follow the workflow to implement a new feature from design through deployment.

### Before Committing
```
/testing-ci-workflow
```
Run all quality checks to ensure code is ready.

### Reviewing Code
```
/code-review-workflow
```
Systematic review of pull request.

### Pre-Release Security Check
```
/security-audit-workflow
```
Comprehensive security audit before release.

### Deploying to Production
```
/deployment-workflow
```
Safe deployment with verification and rollback capability.

## 📚 Additional Resources

### Scrybe Documentation
- [README.md](../README.md) - Project overview
- [docs/rfcs/](../docs/rfcs/) - RFC documents
- [docs/vision.md](../docs/vision.md) - Product vision

### External References
- [TigerStyle](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md) - Rust coding philosophy
- [Windsurf Documentation](https://docs.windsurf.com) - Official Windsurf docs
- [OWASP Top 10](https://owasp.org/www-project-top-ten/) - Security best practices

## 🔧 Customization

### Adding New Rules

1. Create new rule file in `rules/`
2. Specify activation mode:
   - `**Activation**: Always On`
   - `**Activation**: Glob pattern \`*.ext\``
   - `**Activation**: Manual`
3. Document standards clearly with examples
4. Include checklist for verification

### Adding New Workflows

1. Create new workflow file in `workflows/`
2. Start with description and use case
3. Break into numbered steps
4. Include command examples
5. Add quality gates and checklists
6. Document expected outcomes

## 🎯 Best Practices

### For Rules
- Keep rules specific and actionable
- Provide code examples (good ✅ vs bad ❌)
- Include rationale for each rule
- Link to authoritative sources
- Update rules as project evolves

### For Workflows
- Make steps sequential and clear
- Include command examples
- Document expected outputs
- Provide troubleshooting guidance
- Include quality gates

## 🤝 Contributing

When updating rules or workflows:
1. Test with actual use cases
2. Get team review
3. Update this README if adding new files
4. Document changes in commit message
5. Keep examples up to date

## 📞 Support

For questions about:
- **Rules**: Review the specific rule file or ask in team chat
- **Workflows**: Follow the workflow step-by-step, escalate if blocked
- **Windsurf**: Check [official documentation](https://docs.windsurf.com)

## 🔄 Migration to Production

When ready to use with Windsurf:

```bash
# Rename directory
mv "WinSurf temp" .windsurf

# Verify Windsurf detection
# Rules and workflows should now appear in Windsurf Customizations panel
```

---

**Built for Scrybe 🦉 | Powered by Windsurf Cascade 🌊 | Following TigerStyle 🐯**
