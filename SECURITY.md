# Forgeyard Security Policy

The Forgeyard project team takes security vulnerabilities very seriously. We appreciate your efforts to responsibly disclose security flaws.

---

## 1. Supported Versions

Security updates and patches are provided for the following release branches:

| Version Branch | Supported for Security Updates |
| :--- | :--- |
| `main` (Latest Development) | ✅ Yes |
| `v1.x` (Current Stable Release) | ✅ Yes |
| `< v1.0` (Pre-release) | ❌ Upgrade to latest release |

---

## 2. Reporting a Vulnerability

**Please do NOT report security vulnerabilities through public GitHub issues, discussions, or social media.**

If you believe you have discovered a security vulnerability in Forgeyard (e.g. sandbox escape, unauthenticated remote code execution, cryptographic flaw in CAS indexing, privilege escalation, or unauthorized data exposure):

1. **Email Details**: Send a detailed report to `security@forgeyard.dev` (or directly to project maintainers).
2. **Report Information**: Include:
   * Description of the vulnerability and affected components/crates.
   * Step-by-step proof-of-concept (PoC) or reproduction steps.
   * Impact assessment and potential mitigation options.
3. **Response SLA**: Maintainers will acknowledge receipt of your vulnerability report within **48 hours** and provide periodic updates on patch progress.

---

## 3. Responsible Disclosure Policy

* Maintainers will work with you to analyze, fix, and verify the vulnerability.
* Once a fix is created and verified, maintainers will issue a patched release and publish a Security Advisory (GHSA / CVE).
* We ask that you keep the vulnerability confidential until a patch has been publicly released.
