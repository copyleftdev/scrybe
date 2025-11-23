# 📖 Upload Wiki Pages - 2 Minute Guide

## Step 1: Open the Wiki

Click here: **https://github.com/copyleftdev/scrybe/wiki**

You should see a button that says "Create the first page"

---

## Step 2: Create Home Page

1. Click **"Create the first page"**
2. Keep title as: `Home`
3. **Copy the content below** and paste it into the editor:

---

### Content for Home Page:

```markdown
# 🦉 Welcome to Scrybe Wiki

> **"Welcome, traveler. I am Scrybe. You have just gifted me a fingerprint. My task is to remember it, enrich it, and test its truth."**

---

## 📖 What is Scrybe?

**Scrybe** is a high-fidelity, Rust-powered browser observation system designed to detect and understand automation with **forensic granularity**. More than a passive observer, Scrybe is a vigilant system that watches browsers with contextual memory and scientific rigor.

### Mission
- 🎯 **Understand** bots, not just block them
- 🔬 **Study** behavioral patterns with scientific rigor
- 🧠 **Learn** from every interaction
- 🛡️ **Adapt** to evolving automation techniques

---

## 🚀 Quick Start

### For Users
1. **[Getting Started](Getting-Started)** - Installation and basic setup
2. **[Integration Guide](Integration-Guide)** - Add Scrybe to your website
3. **[Configuration](Configuration)** - Customize Scrybe's behavior

### For Developers
1. **[Development Setup](Development-Setup)** - Clone, build, and run locally
2. **[Architecture Overview](Architecture-Overview)** - System design and components
3. **[Contributing Guide](Contributing)** - How to contribute to Scrybe

### For Administrators
1. **[Deployment Guide](Deployment)** - Production deployment checklist
2. **[Operations Manual](Operations)** - Running Scrybe in production
3. **[Monitoring & Alerts](Monitoring)** - Observability setup

---

## 📚 Documentation Structure

### Core Documentation
- **[Vision](Vision)** - Product vision and philosophy
- **[Architecture](Architecture-Overview)** - System architecture and design
- **[RFCs](RFC-Index)** - Request for Comments (design documents)
- **[API Reference](API-Reference)** - Complete API documentation

### Guides
- **[Integration Guide](Integration-Guide)** - Add Scrybe to your site
- **[SDK Guide](SDK-Guide)** - JavaScript SDK documentation
- **[Deployment Guide](Deployment)** - Production deployment
- **[Security Guide](Security-Guide)** - Security best practices

### Reference
- **[Configuration Reference](Configuration)** - All configuration options
- **[Performance Tuning](Performance-Tuning)** - Optimization guide
- **[Troubleshooting](Troubleshooting)** - Common issues and solutions
- **[FAQ](FAQ)** - Frequently asked questions

### Compliance
- **[Privacy Policy](Privacy-Policy)** - Data collection and usage
- **[GDPR Compliance](GDPR-Compliance)** - EU privacy compliance
- **[Security & Privacy](Security-and-Privacy)** - Security features

---

## 🏗️ Architecture at a Glance

```
┌────────────┐     ┌───────────────┐     ┌──────────────────┐     ┌───────────────┐
│  Browser   │ ──> │  Ingestion    │ ──> │  Enrichment & ML │ ──> │  ClickHouse   │
│  (JS SDK)  │     │  Gateway/API  │     │  Fingerprinting  │     │   Storage     │
└────────────┘     └───────────────┘     └──────────────────┘     └───────────────┘
                             │                      │
                             ▼                      ▼
                   ┌────────────────┐     ┌────────────────┐
                   │ Session Cache  │     │  Analyst UI    │
                   │   (Redis)      │     │  Dashboard     │
                   └────────────────┘     └────────────────┘
```

**Components**:
- **[Browser SDK](SDK-Guide)** - Client-side JavaScript agent
- **[Ingestion Gateway](Ingestion-Gateway)** - Rust HTTP server
- **[Enrichment Pipeline](Enrichment-Pipeline)** - Fingerprinting and ML
- **[ClickHouse Storage](Storage)** - Long-term analytics database
- **[Redis Cache](Session-Cache)** - Fast session lookups

---

## 🎯 Key Features

### 🔬 Multi-Layer Fingerprinting
- **Network Layer**: TLS JA3/JA4, TCP fingerprints
- **HTTP Layer**: Header ordering, version negotiation
- **Browser Layer**: Canvas, WebGL, audio fingerprints
- **Behavioral**: Mouse, scroll, keyboard patterns

### 🧠 Machine Learning
- **Anomaly Detection**: Percentile-based thresholds
- **Similarity Clustering**: MinHash fingerprint matching
- **Bot Probability Scoring**: Multi-signal ML model
- **Adaptive Learning**: Thresholds update from real data

### 🛡️ Security & Privacy
- **Zero PII**: No personal data collection
- **GDPR Compliant**: Explicit consent for EU visitors
- **IP Hashing**: SHA-256 salted (not masked)
- **Replay Protection**: Nonce-based authentication

### ⚡ Performance
- **100k req/sec** - Ingestion throughput
- **<10ms latency** - Request processing (p99)
- **<5ms enrichment** - Fingerprint generation
- **<1ms cache** - Redis session lookups

---

## 📊 Current Status

**Version**: v0.2.0 (RFC Phase)  
**Status**: 🎯 **Design Complete** - Ready for Implementation

### ✅ Completed
- Complete RFC suite (7 documents)
- Multi-disciplinary review
- All critical blockers addressed
- Security hardening designed
- GDPR compliance planned
- Production readiness checklist

### 🔨 Next Steps
- **Phase 1** (Weeks 1-2): Core infrastructure
- **Phase 2** (Weeks 3-4): Security features
- **Phase 3** (Weeks 5-6): SDK & enrichment
- **Phase 4** (Weeks 7-8): Storage & reliability
- **Phase 5** (Weeks 9-10): Testing & hardening

---

## 🤝 Community

### Get Involved
- **[GitHub Issues](https://github.com/copyleftdev/scrybe/issues)** - Report bugs or request features
- **[Discussions](https://github.com/copyleftdev/scrybe/discussions)** - Ask questions and share ideas
- **[Contributing Guide](Contributing)** - Learn how to contribute

### Support
- **[Troubleshooting](Troubleshooting)** - Common issues and solutions
- **[FAQ](FAQ)** - Frequently asked questions
- **GitHub Issues** - Technical support

---

## 🦉 Philosophy

> "The best defense is not to be invisible, but to be **understood**."

Scrybe doesn't just detect bots—it studies them. Every fingerprint, every behavioral anomaly, every timing quirk becomes part of a living knowledge base. The system learns, adapts, and evolves.

Like its namesake suggests, Scrybe is both:
- **Scribe** (recorder of truth)
- **Scrying** (diviner of hidden meaning)

It sees not just what browsers do, but what they **are**.

---

## 📖 Quick Links

### Getting Started
- [Installation](Getting-Started#installation)
- [Quick Start Guide](Getting-Started#quick-start)
- [Your First Integration](Integration-Guide#basic-integration)

### Documentation
- [Architecture Overview](Architecture-Overview)
- [API Reference](API-Reference)
- [Configuration Reference](Configuration)

### Development
- [Development Setup](Development-Setup)
- [Building from Source](Development-Setup#building)
- [Running Tests](Development-Setup#testing)

### Operations
- [Deployment Checklist](Deployment#checklist)
- [Monitoring Setup](Monitoring)
- [Backup & Recovery](Operations#backup-and-recovery)

---

## 🏷️ Version Information

- **Current Version**: v0.2.0 (RFC Phase)
- **Next Release**: v0.3.0 (Alpha - Phase 1 complete)
- **Target GA**: v1.0.0 (10 weeks)

---

**Built with Rust 🦀 | Powered by Curiosity 🦉 | Guided by TigerStyle 🐯**

[View on GitHub](https://github.com/copyleftdev/scrybe) | [Report Issue](https://github.com/copyleftdev/scrybe/issues/new) | [Start Contributing](Contributing)
```

---

4. Click **"Save Page"** at the bottom

---

## Step 3: Add More Pages

Now you can use the terminal to add the rest! After creating the Home page, the wiki git repo becomes available:

```bash
# Clone the wiki repo
git clone https://github.com/copyleftdev/scrybe.wiki.git

# Copy prepared pages
cp /tmp/wiki-getting-started.md scrybe.wiki/Getting-Started.md
cp /tmp/wiki-rfc-index.md scrybe.wiki/RFC-Index.md

# Commit and push
cd scrybe.wiki
git add .
git commit -m "docs: Add Getting Started and RFC Index pages"
git push origin master
```

---

## 🎉 Done!

Your wiki will now have:
- ✅ **Home** - Main landing page
- ✅ **Getting Started** - 5-minute setup guide
- ✅ **RFC Index** - Complete RFC documentation

Visit: **https://github.com/copyleftdev/scrybe/wiki**

---

## 📁 All Prepared Content

The complete wiki content is in:
- `/tmp/wiki-home.md` (for Home page - content above)
- `/tmp/wiki-getting-started.md` (for Getting Started)
- `/tmp/wiki-rfc-index.md` (for RFC Index)

You can view them with:
```bash
cat /tmp/wiki-home.md
cat /tmp/wiki-getting-started.md
cat /tmp/wiki-rfc-index.md
```
