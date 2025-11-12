# CrypRQ 1.1.0 Release Announcement

**Template for Slack/Email (60-second paste)**

---

## Subject: CrypRQ 1.1.0 is live 🚀

CrypRQ 1.1.0 is now available with new features, improved stability, and enhanced security.

### What's new

**Desktop**:
- 📊 **Live Charts**: Real-time throughput and latency charts with EMA smoothing, unit toggle (bytes/KB/MB), and auto-scaling
- 🔔 **Latency Alerts**: Visual warnings when latency exceeds 250ms
- 🛡️ **Kill-Switch**: Automatic disconnect on app quit (default ON)
- 📦 **Diagnostics Export**: One-tap diagnostics export with automatic secret redaction
- 🐛 **Report Issue**: Streamlined issue reporting with support token and diagnostics

**Mobile (iOS & Android)**:
- 📱 **Controller Apps**: Full controller mode for iOS and Android
- 📦 **Report Issue**: Generate redacted diagnostics ZIP and share via native share sheet
- 🔒 **Privacy Controls**: First-run consent, telemetry opt-in, encrypted storage

**Stability & Security**:
- ✅ Redaction-by-default (no secrets in logs)
- ✅ Signed builds (macOS/Windows)
- ✅ CI guardrails (SBOM, npm audit, license checks)
- ✅ Rate-limited error toasts (prevents UI spam)

### How to update

**Desktop**:
- Download from [GitHub Release](https://github.com/[org]/cryprq/releases/tag/v1.1.0)
- Or use auto-update if enabled

**Mobile**:
- **iOS**: Install via TestFlight (link: [TestFlight URL])
- **Android**: Install via Play Store Internal Testing (link: [Play Store URL])

### Need help?

If you encounter any issues:
1. **Desktop**: Use Help → Report Issue (exports diagnostics + copies support token)
2. **Mobile**: Use Settings → Report Issue (generates redacted ZIP)
3. Attach the diagnostics ZIP when contacting support

### What's next

- Feature flags for instant rollback
- Telemetry v0 (opt-in health metrics)
- Prometheus→JSON bridge for health dashboard

---

**Full Release Notes**: [GitHub Release URL]  
**Documentation**: [Docs URL]  
**Support**: [Support Email/URL]

---

**Short version (for quick posts)**:

🚀 CrypRQ 1.1.0 is live! New: live charts, diagnostics export, kill-switch, mobile controller apps. Download: [GitHub Release]. Need help? Use Help → Report Issue (desktop) or Settings → Report Issue (mobile).

