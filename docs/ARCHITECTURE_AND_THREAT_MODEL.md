# CrypRQ Architecture and Threat Model

## Overview

CrypRQ is a QUIC/libp2p-based peer-to-peer tunnel that uses ML-KEM (Kyber768) + X25519 hybrid encryption to provide secure connectivity with forward-looking post-quantum protection.

**Key Characteristics:**
- **Transport:** QUIC over UDP with libp2p networking
- **Encryption:** ML-KEM (Kyber768) + X25519 hybrid key establishment
- **Modes:** P2P tunnels, file transfer via Web Tester, and future system-wide VPN mode (requires TUN + OS integration)
- **Target Use Case:** Secure peer-to-peer connectivity with post-quantum cryptographic protection

## High-Level Architecture

### Components

#### Core Node/Binary (cryprq CLI)

**Listener Mode:**
- Binds to `/ip4/0.0.0.0/udp/<port>/quic-v1`
- Waits for incoming connections from dialers
- Performs ML-KEM + X25519 hybrid key establishment on connection
- Supports key rotation at configurable intervals (default: 300 seconds)

**Dialer Mode:**
- Connects to `/ip4/.../udp/<port>/quic-v1` (peer address)
- Initiates QUIC handshake and hybrid key establishment
- Establishes encrypted tunnel for data transfer

**Key Features:**
- QUIC/libp2p transport layer
- Hybrid key establishment: ML-KEM (Kyber768) + X25519
- Key rotation mechanism for forward secrecy
- Optional VPN mode using TUN interfaces (system-level, OS-dependent, requires admin/root privileges)

#### Web Tester Front-End

**Single-Page Application:**
- Served at http://localhost:8787
- React + TypeScript (Vite build)
- Spawns cryprq processes (listener/dialer) with appropriate arguments

**Features:**
- **Guided Tests:** Pre-configured test scenarios (Local Loopback P2P, Two-Tab Web Test, VPN Demo)
- **Connection Health:** Real-time connection state, handshake direction/time, ping latency, epoch, rotation interval
- **Advanced Metrics:** Aggregated session statistics (connection attempts, ping stats, key rotations, file transfers, errors)
- **File Transfer UX:** State machine with progress, throughput, and completion status
- **Connection Profiles:** Save/load named profiles (mode, port, peer address, VPN toggle) to localStorage
- **CLI Snippet Generator:** Generate and copy equivalent CLI commands
- **Bug Report Draft Generator:** Pre-filled bug report template with session state
- **Debug Console:** Real-time log stream with filters (All, Errors, Rotation, Peer, Status) and log export

#### Test Harness and Docker E2E

**E2E Test Suite:**
- Orchestrated via `docker/e2e/docker-compose.e2e.yml`
- Validates:
  - QUIC handshake establishment
  - ML-KEM + X25519 hybrid encryption
  - Key rotation mechanism
  - File transfer integrity
  - Packet validation

**Test Execution:**
- Runs in isolated Docker containers
- Listener and dialer peers communicate over QUIC
- Test runner validates expected events and behaviors

### Data Flows

**Connection Establishment:**
1. Dialer initiates QUIC connection to Listener
2. Both peers perform ML-KEM + X25519 hybrid key establishment
3. Encrypted tunnel is established with authenticated session
4. Key rotation occurs at configurable intervals (default: 300 seconds)

**File Transfer:**
1. File transfer is layered on top of the encrypted tunnel
2. File metadata and chunks are sent over the QUIC connection
3. Integrity is verified at the receiving end
4. Transfer state is tracked in the Web Tester UI

**Key Rotation:**
1. At each rotation interval, new ML-KEM + X25519 keys are generated
2. Keys are exchanged over the existing encrypted session
3. Subsequent data uses the new keys
4. Rotation events are logged and visible in Connection Health and Advanced Metrics

## Threat Model

### Assets

**Confidentiality and Integrity of Data:**
- Data transmitted over the tunnel should be protected from eavesdropping and tampering
- File transfers should maintain integrity (no corruption or tampering)

**Correctness of Hybrid Key Establishment:**
- ML-KEM + X25519 key exchange should be performed correctly
- No downgrade to weaker cryptographic primitives

**Session Authenticity:**
- Connections should be authenticated to prevent man-in-the-middle attacks
- Key rotation should maintain forward secrecy

### Assumptions

**Endpoint Security:**
- Endpoints are not fully compromised (attacker does not own both endpoints)
- Operating system and kernel networking stack are reasonably secure
- Application code is not maliciously modified

**Network Environment:**
- Basic network connectivity exists between endpoints
- Firewall rules allow UDP traffic on the selected port (for P2P mode)

**System-Wide VPN Mode:**
- Assumes correct TUN interface setup and routing
- Requires admin/root privileges
- OS-level integration (e.g., macOS Network Extension, Linux TUN) is properly configured

### Non-Goals

**This project does not currently provide:**

1. **Anonymity or Traffic Obfuscation:**
   - CrypRQ does not hide the fact that two endpoints are communicating
   - Traffic patterns and metadata are not obfuscated
   - No Tor-like routing or traffic mixing

2. **Resistance to Global Passive Adversaries:**
   - CrypRQ does not protect against powerful global passive adversaries correlating metadata
   - Timing analysis and traffic pattern analysis are not mitigated

3. **OS-Level Kill-Switch and Leak Protection:**
   - Until VPN mode is fully implemented and tested with OS integration, CrypRQ does not provide:
     - DNS leak protection
     - IPv6 leak protection
     - WebRTC leak protection
     - Kill-switch behavior (automatic disconnection if tunnel fails)

4. **Formal Security Proofs:**
   - While CrypRQ uses well-studied cryptographic primitives (ML-KEM, X25519, QUIC), formal proofs of the complete system are out of scope for this document

### Threats and Mitigations

#### On-Path Passive Attackers

**Threat:** Attacker can observe traffic between endpoints but cannot modify it.

**Mitigation:**
- QUIC + ML-KEM + X25519 encryption protects data confidentiality
- Hybrid encryption provides post-quantum protection (ML-KEM) and classical security (X25519)
- Key rotation provides forward secrecy

**Limitations:**
- Metadata (source/destination IPs, port numbers, packet sizes, timing) may be observable
- Traffic patterns may be analyzed

#### On-Path Active Attackers

**Threat:** Attacker can intercept, modify, or inject traffic between endpoints.

**Mitigation:**
- libp2p + QUIC security properties provide authenticated sessions
- Replay protection prevents replay attacks
- Key establishment is authenticated to prevent man-in-the-middle attacks

**Limitations:**
- Active attackers may be able to disrupt connectivity (denial of service)
- Initial connection establishment may be vulnerable if endpoints are not properly authenticated (depends on libp2p configuration)

#### Future Quantum Adversaries

**Threat:** Adversaries with access to quantum computers that can break classical cryptography (e.g., X25519).

**Mitigation:**
- ML-KEM (Kyber768) hybrid encryption provides post-quantum protection
- Hybrid approach maintains classical security (X25519) while adding post-quantum security (ML-KEM)

**Limitations:**
- Protection depends on correct implementation and appropriate parameter choices
- ML-KEM is a relatively new standard; long-term security properties are still being evaluated
- Full post-quantum migration may require additional protocol changes

#### Endpoint Compromise

**Threat:** Attacker gains control of one or both endpoints.

**Assumption:** This threat is out of scope. If an endpoint is compromised, all security guarantees are lost.

**Mitigation:** None (by design). CrypRQ assumes endpoints are trusted.

#### Implementation Vulnerabilities

**Threat:** Bugs in implementation that could lead to security vulnerabilities.

**Mitigation:**
- Use of well-tested libraries (libp2p, QUIC, ML-KEM implementations)
- E2E test suite validates core functionality
- Code review and testing practices

**Limitations:**
- No formal verification of implementation
- Security audits are recommended for production use

## Future Work

### OS-Level VPN Validation

**Planned Enhancements:**
- DNS/IPv6/WebRTC leak tests
- Kill-switch behavior validation
- macOS Network Extension integration and test harness
- Linux TUN routing validation

**Status:** These features are planned but not yet fully implemented or tested. See `docs/VPN_TEST_PLAN.md` for current testing procedures.

### Security Audits

**Recommendations:**
- External security audit of cryptographic implementation
- Formal verification of key establishment protocol
- Penetration testing of Web Tester and CLI interfaces

**Status:** Future work; not yet scheduled.

## Additional Resources

- `docs/VPN_TEST_PLAN.md`: VPN and P2P tunnel test procedures
- `docs/FILE_TRANSFER_TEST_PLAN.md`: File transfer testing procedures
- `docs/CI_E2E_INVESTIGATION_RUNBOOK.md`: How to investigate CI/E2E failures
- `docs/TROUBLESHOOTING_SUPPORT_BUNDLE.md`: Troubleshooting guide
- `docs/WEB_TESTER_GUIDE.md`: Web Tester usage guide
- `docs/QUICKSTART_CLI.md`: CLI quickstart guide

