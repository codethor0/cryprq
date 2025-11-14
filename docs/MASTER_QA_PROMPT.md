# Master QA Prompt for CrypRQ v1.0.1

## Purpose

This is a reusable prompt for AI/dev-assistant tools to perform systematic QA and protocol alignment checks on the CrypRQ v1.0.1 implementation.

---

## Master Prompt

```
You are an expert QA and cryptographic protocol engineer reviewing the CrypRQ v1.0.1 implementation.

Goal:
Validate that the implementation matches the CrypRQ v1.0.1 protocol spec and that the new record-layer-based data path (VPN + file transfer + CLI) behaves correctly.

Context:
- CrypRQ uses a 20-byte record header (Version, MsgType, Flags, Epoch[u8], StreamID[u32], Seq[u64], CiphertextLen[u32]).
- Keys are derived with HKDF using salt "cryp-rq v1.0 hs" and epoch-scoped labels.
- Nonces use TLS 1.3-style XOR of a static IV with the sequence number.
- Epoch is u8 and rotates modulo 256.
- File transfer and VPN now go through the CrypRQ record layer (no direct libp2p request-response).

Tasks:

1. Scan the repository for:
   - Record header definition and encode/decode logic.
   - HKDF-based key schedule implementation (handshake keys, traffic keys, epoch-scoped keys).
   - Nonce construction (make_nonce or equivalent).
   - FileTransferManager and Tunnel APIs for FILE_META/FILE_CHUNK/FILE_ACK and VPN_PACKET.
   - CLI commands for send-file and receive-file.

2. Verify protocol alignment:
   - Confirm header field sizes and layout match the spec.
   - Confirm Epoch is u8 and is included in labels for epoch-scoped keys.
   - Confirm nonces are constructed via XOR of static IV and sequence number, not simple counters.
   - Confirm sequence numbers are per-direction and monotonic.

3. Generate a concrete manual test plan:
   - Commands to run receive-file and send-file for:
     - Tiny file (1 chunk)
     - Multi-chunk medium file
     - Large file (forces at least one epoch rotation)
   - Commands to compute and compare SHA-256 hashes.
   - Steps to test concurrent transfers on different Stream IDs.
   - Steps to test aggressive epoch rotation and epoch wrap-around.

4. Generate a VPN test plan:
   - How to start the VPN stack.
   - How to verify VPN_PACKET messages are flowing via the record layer.
   - How to test ping and HTTP traffic over the tunnel.

5. Produce a short checklist of "release blockers vs future work":
   - MUST fix before calling CrypRQ v1.0.1 "production-ready".
   - SHOULD fix soon (e.g., replacing hardcoded test keys with real identity/handshake).
   - NICE TO HAVE (observability, metrics, docs polish).

When you answer:
- Be specific: reference concrete file paths and functions.
- Provide ready-to-run CLI commands.
- Separate findings into: PROTOCOL ALIGNMENT, FUNCTIONAL TESTS, SECURITY WARNINGS, and TODOs.
```

---

## Usage Instructions

1. **Copy the Master Prompt** above into your AI/dev-assistant tool.

2. **Run the analysis** - The tool will:
 - Scan the codebase for protocol implementation
 - Verify alignment with the v1.0.1 spec
 - Generate test plans
 - Identify security issues

3. **Review the output** organized into:
 - **PROTOCOL ALIGNMENT:** Code matches spec
 - **FUNCTIONAL TESTS:** Test commands and expected results
 - **SECURITY WARNINGS:** Issues that must be fixed
 - **TODOs:** Future improvements

4. **Execute tests** using the generated test plan.

5. **Iterate** - Re-run the prompt after fixes to verify changes.

---

## Expected Output Format

The QA tool should produce:

### PROTOCOL ALIGNMENT

**Record Header:**
- File: `core/src/record.rs`
- Status: / 
- Details: [specific findings]

**HKDF Key Schedule:**
- File: `crypto/src/kdf.rs`
- Status: / 
- Details: [specific findings]

**Nonce Construction:**
- File: `node/src/crypto_utils.rs`
- Status: / 
- Details: [specific findings]

### FUNCTIONAL TESTS

**Test 1: Tiny File**
```bash
# Commands...
# Expected: ...
```

**Test 2: Medium File**
```bash
# Commands...
# Expected: ...
```

### SECURITY WARNINGS

1. **Hardcoded Keys:** [location] - MUST FIX
2. **Missing Handshake:** [location] - MUST FIX

### TODOs

1. [Future improvement]
2. [Future improvement]

---

## Customization

You can customize the prompt by:

- **Adding specific file paths** if your structure differs
- **Including version numbers** for dependencies
- **Specifying test environments** (Docker, local, etc.)
- **Adding domain-specific checks** (compliance, performance, etc.)

---

## Example Customization

For a focused check on just file transfer:

```
[Include Master Prompt]

Focus Areas:
- File transfer correctness only
- Skip VPN tests
- Skip web UI tests
```

For a security audit:

```
[Include Master Prompt]

Focus Areas:
- Security-critical code paths only
- Key management
- Authentication/authorization
- Input validation
```

