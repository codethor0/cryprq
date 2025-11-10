# FAQ

**Is CrypRQ production ready?**  
No. Packet forwarding and tunnel management remain under development.

**Why use both ML-KEM and X25519?**  
ML-KEM provides PQ resilience; X25519 maintains interoperability and defense-in-depth.

**Does CrypRQ manage peers automatically?**  
No. Peer IDs must be exchanged securely by operators.

**Which ports must be open?**  
UDP/TCP 9999 by default. Adjust multiaddrs to change ports.

**Where are keys stored?**  
In-memory only; rotated every `CRYPRQ_ROTATE_SECS` seconds.

**Can I shorten rotation?**  
Yes. Set `CRYPRQ_ROTATE_SECS=<seconds>` before launching.

**Is mDNS discovery available?**  
The P2P crate includes mDNS, but hardened deployments should rely on manual dialing.

**How do I audit dependencies?**  
Run `cargo audit` and `cargo deny check ...` (also enforced in CI).

**What platforms are supported?**  
Linux and macOS are tested. Windows is unverified.

**How do I report vulnerabilities?**  
Email `codethor@gmail.com` (PGP in SECURITY.md).

---

**Checklist**
- [ ] Understood project readiness.
- [ ] Configured rotation to match policy.
- [ ] Planned secure peer distribution.

