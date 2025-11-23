# Data Breach Notification Procedure

**Document Owner**: Security Team  
**Last Updated**: January 22, 2025  
**Review Frequency**: Annually

## 1. Overview

This procedure outlines steps to follow when a data breach is discovered, ensuring compliance with GDPR Article 33 (notification to supervisory authority) and Article 34 (notification to data subjects).

## 2. Timeline Requirements

| Action | Timeline | Authority |
|--------|----------|-----------|
| Internal notification | Immediate | GDPR implied |
| Controller notification | 24 hours | DPA requirement |
| Supervisory authority | 72 hours | GDPR Article 33 |
| Data subjects | Without undue delay | GDPR Article 34 |

## 3. Breach Discovery

### 3.1 Detection Methods
- Security monitoring alerts
- Audit log anomalies
- Third-party notification
- Internal report
- External researcher

### 3.2 Initial Response (Hour 0)
1. **Confirm breach** - Verify it's a genuine incident
2. **Contain immediately** - Stop ongoing breach
3. **Notify security team** - security@scrybe.io
4. **Document everything** - Start incident log

## 4. Breach Assessment (Hours 0-4)

### 4.1 Scope Assessment
- [ ] What data was accessed/disclosed?
- [ ] How many data subjects affected?
- [ ] What categories of personal data?
- [ ] What is the root cause?
- [ ] Is the breach ongoing?

### 4.2 Risk Assessment
**Low Risk**: Hashed fingerprints only (no PII)  
**Medium Risk**: Multiple sessions, potential correlation  
**High Risk**: Plain text IPs, identifiable patterns  
**Critical Risk**: PII exposed (should never happen)

### 4.3 Regulatory Trigger
**Notify if:**
- Risk to rights and freedoms of data subjects
- Potential for discrimination, identity theft, financial loss
- Any unauthorized access to personal data

**No notification if:**
- Data encrypted and keys secure
- Measures immediately eliminate risk
- Risk unlikely to materialize

## 5. Containment (Hours 0-8)

### 5.1 Immediate Actions
- [ ] Isolate affected systems
- [ ] Revoke compromised credentials
- [ ] Block malicious IP addresses
- [ ] Disable compromised API keys
- [ ] Take forensic snapshots

### 5.2 Evidence Preservation
- [ ] Preserve logs (do not modify)
- [ ] Screenshot alerts
- [ ] Document timeline
- [ ] Record all actions taken

## 6. Controller Notification (Within 24 Hours)

**Notify all affected Controllers** (customers using Scrybe)

### 6.1 Notification Content
```
Subject: DATA BREACH NOTIFICATION - Immediate Action Required

Dear [Controller Name],

We are writing to inform you of a data breach affecting your account.

INCIDENT DETAILS:
- Date/Time Discovered: [timestamp]
- Nature of Breach: [description]
- Data Categories Affected: [list]
- Number of Data Subjects: [estimate]

PERSONAL DATA INVOLVED:
- Browser fingerprints: [Yes/No]
- Behavioral data: [Yes/No]
- IP addresses (hashed): [Yes/No]
- PII: [Should be NO]

ROOT CAUSE:
[Technical explanation]

CONTAINMENT MEASURES TAKEN:
[List immediate actions]

RISK ASSESSMENT:
[Low/Medium/High/Critical]

YOUR OBLIGATIONS:
As Data Controller, you must assess whether notification to your supervisory authority and data subjects is required under GDPR Articles 33 and 34.

ASSISTANCE PROVIDED:
We will provide:
- Detailed incident report
- Forensic analysis
- List of affected data subjects
- Technical assistance

CONTACT:
Security Team: security@scrybe.io
Phone: [Emergency number]

We apologize for this incident and are taking all necessary steps to prevent recurrence.

Regards,
[Name], Chief Security Officer
```

## 7. Supervisory Authority Notification (Within 72 Hours)

**Submit to relevant Data Protection Authority**

### 7.1 Responsible DPA
- **EU users**: Their national DPA
- **UK users**: ICO (Information Commissioner's Office)
- **Multi-jurisdiction**: Lead supervisory authority

### 7.2 Notification Form
Most DPAs have online forms. Include:

1. **Nature of breach**
   - Description of incident
   - Timeline
   - How discovered

2. **Categories and approximate numbers**
   - Types of personal data
   - Number of data subjects affected
   - Number of personal data records

3. **Contact point**
   - DPO or contact person
   - Email and phone

4. **Likely consequences**
   - Risk assessment
   - Potential harm to data subjects

5. **Measures taken/proposed**
   - Containment actions
   - Mitigation measures
   - Prevention measures

## 8. Data Subject Notification (If High Risk)

### 8.1 Criteria for Notification
**Must notify if:**
- High risk to rights and freedoms
- Potential for identity theft
- Financial loss possible
- Discrimination risk

**Example notification:**

```
Subject: Important Security Notice

Dear User,

We are writing to inform you about a security incident that may affect you.

WHAT HAPPENED:
[Brief, clear explanation]

WHAT INFORMATION WAS INVOLVED:
[List specific data types]

WHAT WE ARE DOING:
[Containment and prevention measures]

WHAT YOU SHOULD DO:
[Specific actions, if any]

HOW TO CONTACT US:
For questions: privacy@scrybe.io
For support: support@scrybe.io

We sincerely apologize and are committed to protecting your information.

Regards,
[Name], Privacy Officer
```

## 9. Investigation (Days 1-7)

### 9.1 Root Cause Analysis
- [ ] How did breach occur?
- [ ] What vulnerabilities exploited?
- [ ] What controls failed?
- [ ] Was it preventable?

### 9.2 Forensic Analysis
- [ ] Engage external forensics (if needed)
- [ ] Analyze logs and system dumps
- [ ] Identify attack vectors
- [ ] Document findings

## 10. Remediation (Days 1-30)

### 10.1 Immediate Fixes
- [ ] Patch vulnerabilities
- [ ] Strengthen access controls
- [ ] Update security rules
- [ ] Rotate credentials

### 10.2 Long-term Improvements
- [ ] Architecture changes
- [ ] Process improvements
- [ ] Training updates
- [ ] Technology upgrades

## 11. Documentation

### 11.1 Breach Register (Article 33(5))
Maintain internal record:
- Date/time of breach
- Facts of breach
- Effects
- Remedial action

### 11.2 Incident Report
Complete report including:
- Executive summary
- Detailed timeline
- Technical analysis
- Lessons learned
- Action items

## 12. Post-Incident Review (Day 30)

### 12.1 Lessons Learned Session
- What went well?
- What could be improved?
- What should we do differently?
- What new controls needed?

### 12.2 Update Procedures
- Revise this document
- Update security controls
- Enhance monitoring
- Improve training

## 13. Templates

### 13.1 Breach Assessment Form
```
BREACH ID: [YYYY-MM-DD-NNN]
DISCOVERED: [Date/Time]
DISCOVERED BY: [Name/System]
SEVERITY: [Low/Medium/High/Critical]

AFFECTED SYSTEMS:
- [ ] Web Gateway
- [ ] Database
- [ ] Cache
- [ ] SDK

DATA CATEGORIES:
- [ ] Fingerprints (hashed)
- [ ] Behavioral data
- [ ] IP addresses (hashed)
- [ ] Session IDs
- [ ] Other: _______

ESTIMATED AFFECTED DATA SUBJECTS: _______

ROOT CAUSE (preliminary): _______

CONTAINMENT STATUS:
- [ ] Ongoing
- [ ] Contained
- [ ] Resolved

NOTIFICATIONS REQUIRED:
- [ ] Controllers (within 24h)
- [ ] DPA (within 72h)
- [ ] Data Subjects (if high risk)
```

## 14. Contact Information

**Internal**:
- Security Team: security@scrybe.io
- Privacy Officer: privacy@scrybe.io
- DPO: dpo@scrybe.io
- Legal: legal@scrybe.io

**External**:
- Forensics Provider: [Contact]
- Legal Counsel: [Contact]
- PR/Communications: [Contact]

**Supervisory Authorities**:
- UK ICO: casework@ico.org.uk / +44 303 123 1113
- Irish DPC: info@dataprotection.ie / +353 761 104 800
- German BfDI: poststelle@bfdi.bund.de / +49 228 997799-0

## 15. Checklist

**Immediate (Hour 0-4):**
- [ ] Confirm breach
- [ ] Contain breach
- [ ] Notify security team
- [ ] Start documentation
- [ ] Assess scope

**Short-term (Hour 4-24):**
- [ ] Complete risk assessment
- [ ] Notify Controllers
- [ ] Preserve evidence
- [ ] Begin investigation

**Medium-term (Day 1-3):**
- [ ] Notify DPA (if required)
- [ ] Notify data subjects (if required)
- [ ] Continue investigation
- [ ] Implement fixes

**Long-term (Day 3-30):**
- [ ] Complete investigation
- [ ] Update breach register
- [ ] Post-incident review
- [ ] Update procedures

---

**Version**: 1.0.0  
**Last Updated**: January 22, 2025  
**Next Review**: January 22, 2026
