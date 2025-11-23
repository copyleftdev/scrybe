# Data Processing Agreement (DPA)

**Between**: [Customer Name] ("Data Controller")  
**And**: [Scrybe Provider] ("Data Processor")  
**Effective Date**: [Date]

## 1. Definitions

**Personal Data**: As defined in GDPR Article 4(1)  
**Processing**: As defined in GDPR Article 4(2)  
**Data Subject**: As defined in GDPR Article 4(1)  
**Supervisory Authority**: As defined in GDPR Article 4(21)

## 2. Subject Matter and Duration

**Purpose**: Bot detection and behavioral analysis services  
**Duration**: Term of the service agreement  
**Nature of Processing**: Automated processing of browser fingerprints and behavioral signals

## 3. Data Processed

### 3.1 Categories of Data
- Browser fingerprints (pseudonymized)
- Behavioral patterns (anonymized)
- Network signals (IP hashed)
- **NO PII collected**

### 3.2 Categories of Data Subjects
- Website visitors
- End users

## 4. Processor Obligations (Article 28(3))

### 4.1 Processing Instructions
Process Personal Data only on documented instructions from Controller.

### 4.2 Confidentiality (Article 28(3)(b))
All personnel with access to Personal Data are bound by confidentiality.

### 4.3 Security Measures (Article 32)
Implement appropriate technical and organizational measures:
- TLS 1.3 encryption
- IP address hashing (SHA-256)
- 90-day automatic deletion
- Access controls
- Regular security audits

### 4.4 Sub-Processors
**Current Sub-Processors:**
- [Cloud Provider] - Hosting
- [Database Provider] - Storage

**Notification**: 30 days' notice for new sub-processors

### 4.5 Data Subject Rights (Article 28(3)(e))
Assist Controller in fulfilling data subject requests:
- Right of access
- Right to erasure
- Right to data portability

### 4.6 Security Breach (Article 33)
Notify Controller within 24 hours of breach discovery.

### 4.7 Audits (Article 28(3)(h))
Allow Controller audits with 30 days' notice.

### 4.8 Deletion (Article 28(3)(g))
Delete or return all Personal Data upon termination.

## 5. Controller Obligations

### 5.1 Legal Basis
Controller ensures lawful basis for processing (e.g., consent).

### 5.2 Instructions
Provide clear, documented processing instructions.

### 5.3 Data Accuracy
Ensure data provided is accurate and up-to-date.

## 6. Data Transfers

### 6.1 International Transfers
**EU to US**: Standard Contractual Clauses (SCCs)  
**Adequacy Decision**: [If applicable]

### 6.2 Additional Safeguards
- Data localization where required
- Encryption in transit and at rest

## 7. Security Measures

### 7.1 Technical Measures
- Encryption (TLS 1.3, AES-256)
- Pseudonymization (IP hashing)
- Access controls
- Logging and monitoring

### 7.2 Organizational Measures
- Staff training
- Incident response plan
- Regular security audits
- Penetration testing

## 8. Data Breach Notification

**Timeline**: Within 24 hours to Controller  
**Content**: 
- Nature of breach
- Categories/number of data subjects
- Likely consequences
- Measures taken/proposed

## 9. Sub-Processing

### 9.1 Authorization
Controller authorizes current sub-processors listed above.

### 9.2 New Sub-Processors
30 days' written notice before engaging new sub-processors.

### 9.3 Liability
Processor remains fully liable to Controller for sub-processor performance.

## 10. Data Subject Rights

### 10.1 Assistance
Processor will assist Controller in:
- Responding to access requests (Article 15)
- Rectification (Article 16)
- Erasure (Article 17)
- Data portability (Article 20)

### 10.2 Response Time
Assistance provided within 7 business days of request.

## 11. Audits and Inspections

### 11.1 Controller Audits
- 30 days' written notice
- During business hours
- Reasonable frequency (annually)
- Controller bears costs

### 11.2 Documentation
Processor provides evidence of GDPR compliance.

## 12. Data Return and Deletion

### 12.1 Upon Termination
Within 30 days:
- Delete all Personal Data, or
- Return to Controller (if requested)

### 12.2 Certification
Provide written certification of deletion.

### 12.3 Exceptions
Retention required by law (with notification).

## 13. Liability and Indemnification

### 13.1 Processor Liability
Liable only for damage caused by non-compliance with GDPR obligations.

### 13.2 Indemnification
Processor indemnifies Controller for:
- Supervisory authority fines
- Data subject compensation
- Arising from Processor breach

## 14. Notices

**Controller**:  
[Name]  
[Address]  
[Email]

**Processor**:  
[Scrybe Provider Name]  
[Address]  
security@scrybe.io

## 15. Governing Law

**Jurisdiction**: [EU Member State or UK]  
**GDPR Application**: EU GDPR / UK GDPR

## 16. Amendments

Amendments require written agreement by both parties.

## 17. Severability

Invalid provisions do not affect remaining agreement.

---

## Signatures

**Data Controller**:  
Name: _______________  
Signature: _______________  
Date: _______________

**Data Processor**:  
Name: _______________  
Signature: _______________  
Date: _______________

---

**Annex 1: Standard Contractual Clauses**  
[Attach EU SCCs for international transfers]

**Annex 2: Technical and Organizational Measures**  
[Detailed security measures - see SECURITY.md]

**Annex 3: Sub-Processor List**  
[Current sub-processors with details]

---

**Version**: 1.0.0  
**Last Updated**: January 22, 2025
