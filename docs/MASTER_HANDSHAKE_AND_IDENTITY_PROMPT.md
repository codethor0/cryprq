# Master Prompt: Handshake and Identity Implementation

## Purpose

This is a reusable master prompt for AI/dev-assistant tools to guide the implementation of the real CrypRQ v1.0.1 handshake and peer authentication, replacing hardcoded test keys with a secure protocol.

---

## Master Prompt

```
You are an expert Rust network engineer and cryptographic protocol designer helping me evolve the CrypRQ project from a test-mode implementation (static keys) to a full CrypRQ v1.0.1-compliant secure protocol with a real handshake and peer authentication.

Context (current state):
- CrypRQ v1.0.1 spec is defined and aligned with the code record layer.
- We have:
  - 20-byte CrypRQ record headers with Version, MsgType, Flags, Epoch(u8), StreamID(u32), Seq(u64), CiphertextLen(u32).
  - HKDF-based key schedule (salt "cryp-rq v1.0 hs", epoch-scoped labels).
  - TLS 1.3-style XOR nonce construction with static IVs.
  - Epoch as u8, modulo 256, with epoch-scoped keys and rotation.
  - VPN and file transfer going through the CrypRQ record layer.
- CLI (send-file / receive-file) currently uses **hardcoded test keys**.
- The test matrix, master QA prompt, and security notes docs already exist:
  - docs/TEST_MATRIX.md
  - docs/MASTER_QA_PROMPT.md
  - docs/SECURITY_NOTES.md
- There is an updated protocol spec describing:
  - CRYPRQ_CLIENT_HELLO / CRYPRQ_SERVER_HELLO / CRYPRQ_CLIENT_FINISH
  - Hybrid ML-KEM (Kyber768) + X25519 handshake
  - HS key schedule and hs_auth_key for verify_data
  - Peer authentication options (Ed25519, X.509, libp2p-style peer IDs, or PSK)
  - Epoch-scoped traffic keys for application data

Your tasks:

1. **Repo scan and mapping**
   - Identify the following modules and summarize their current responsibilities:
     - Handshake / connection setup code (even if it still uses libp2p Noise).
     - The record layer implementation (core/src/record.rs, node/src/record_layer.rs or similar).
     - The key derivation logic (crypto/src/kdf.rs).
     - The Tunnel struct and how it currently obtains keys.
     - CLI entrypoints for send-file and receive-file.
   - Identify where hardcoded keys are defined and how they flow into Tunnel/DirectionKeys.

2. **Design the new handshake integration plan (implementation-level)**
   - Define exactly:
     - Where to implement CRYPRQ_CLIENT_HELLO, CRYPRQ_SERVER_HELLO, CRYPRQ_CLIENT_FINISH messages (file paths and structs).
     - How to wire ML-KEM (Kyber768) + X25519 to produce:
       - ss_kem
       - ss_x
       - master_secret
       - hs_auth_key
       - initial traffic keys for epoch 0.
     - How the handshake will output:
       - Directional application keys (keys_outbound, keys_inbound)
       - Static IVs for each direction
       - The initial epoch and sequence counters.
   - Choose a concrete authentication mode for now (e.g., Ed25519 static identity keys or PSK) and describe:
     - How identities/keys are stored/configured on disk.
     - How verify_data is constructed and checked using hs_auth_key and the transcript.

3. **Code changes (step-by-step)**
   Propose an ordered implementation plan with specific file edits. For each step, list:
   - New types/structs (e.g., HandshakeState, ClientHello, ServerHello, ClientFinish).
   - New functions (e.g., run_handshake(…)).
   - Required changes to Tunnel:
     - Remove dependency on hardcoded keys.
     - Accept established keys from handshake result.
     - Initialize DirectionKeys and seq counters from that result.
   - Changes to CLI:
     - For receive-file:
       - Listen, perform handshake with the peer, then start raw record layer loop.
     - For send-file:
       - Connect to peer, perform handshake, then start file transfer.

4. **Replace hardcoded keys with real handshake output**
   - Remove or disable the current static key paths.
   - Ensure all keys used by:
     - Record::encrypt / Record::decrypt
     - Tunnel::send_vpn_packet / handle_incoming_record
     - FileTransferManager
     - CLI send-file / receive-file
     come from the handshake result (no test constants).

5. **Update tests + test docs**
   - Propose unit tests for:
     - Handshake message encoding/decoding.
     - Hybrid key derivation from fixed test vectors (ss_kem, ss_x → master_secret → traffic keys).
     - verify_data computation and verification using hs_auth_key.
   - Propose integration tests that:
     - Perform a full handshake between two in-process peers.
     - Then run a small file transfer and verify SHA-256.
   - Suggest updates to:
     - docs/TEST_MATRIX.md to add "Handshake and Peer Auth" section.
     - docs/SECURITY_NOTES.md to reflect that hardcoded keys are gone and describe the new trust model.

6. **Security review checklist**
   - Generate a concise checklist to review after implementing the handshake:
     - No place where keys are logged.
     - RNG usage is always CSPRNG (OS-backed).
     - Identity secrets (Ed25519 / PSK) are stored and loaded securely.
     - verify_data covers the entire handshake transcript.

Output format:
- SECTION 1: Current Implementation Map (files + functions).
- SECTION 2: Handshake Integration Plan (high-level).
- SECTION 3: Concrete Step-by-Step Implementation Plan (low-level).
- SECTION 4: Testing & Docs Updates.
- SECTION 5: Security Checklist.

Be specific and implementation-oriented. Reference exact file paths and function names where possible. Assume the goal is to move from "test-only stack" to a credible v1.0.1 secure protocol suitable for internal VPN/file-transfer use.
```

---

## Usage Instructions

1. **Copy the Master Prompt** above into your AI/dev-assistant tool when ready to implement the handshake.

2. **Review the output** - The tool will provide:
 - Current codebase mapping
 - Handshake integration design
 - Step-by-step implementation plan
 - Test updates
 - Security checklist

3. **Execute the plan** - Follow the step-by-step implementation plan systematically.

4. **Run tests** - Use the updated test matrix to validate the handshake.

5. **Security review** - Use the security checklist before considering production-ready.

---

## Expected Output Format

### SECTION 1: Current Implementation Map

**Handshake Code:**
- File: `[path]`
- Current: [description]
- Hardcoded keys: [location]

**Record Layer:**
- File: `[path]`
- Current: [description]

**Key Derivation:**
- File: `[path]`
- Current: [description]

**Tunnel:**
- File: `[path]`
- Current: [description]
- Key source: [hardcoded/test keys location]

**CLI:**
- File: `[path]`
- Current: [description]
- Key source: [hardcoded keys location]

---

### SECTION 2: Handshake Integration Plan

**Message Types:**
- CRYPRQ_CLIENT_HELLO: [file path, struct name]
- CRYPRQ_SERVER_HELLO: [file path, struct name]
- CRYPRQ_CLIENT_FINISH: [file path, struct name]

**Key Exchange:**
- ML-KEM: [library/implementation]
- X25519: [library/implementation]
- Hybrid derivation: [function name, file path]

**Authentication:**
- Method: [Ed25519/PSK/etc.]
- Identity storage: [file path, format]
- verify_data: [computation, verification]

**Output:**
- DirectionKeys: [how derived]
- Static IVs: [how derived]
- Initial epoch: [value]
- Sequence counters: [initialization]

---

### SECTION 3: Concrete Step-by-Step Implementation Plan

**Step 1: [Title]**
- Files to create: [list]
- Files to modify: [list]
- New types: [list]
- New functions: [list]
- Changes to existing: [list]

**Step 2: [Title]**
- [Same format]

**Step N: [Title]**
- [Same format]

---

### SECTION 4: Testing & Docs Updates

**Unit Tests:**
- [Test name]: [file path, description]

**Integration Tests:**
- [Test name]: [file path, description]

**Doc Updates:**
- TEST_MATRIX.md: [changes]
- SECURITY_NOTES.md: [changes]

---

### SECTION 5: Security Checklist

- [ ] No keys logged
- [ ] CSPRNG usage verified
- [ ] Identity secrets stored securely
- [ ] verify_data covers full transcript
- [ ] [Additional items]

---

## Customization

You can customize the prompt by:

- **Specifying authentication method:** "Use Ed25519 static identity keys" or "Use PSK from config file"
- **Adding constraints:** "Must be compatible with existing libp2p integration" or "Must support key rotation"
- **Focusing on specific areas:** "Focus on handshake messages first, defer authentication to later phase"

---

## Related Documents

- Protocol Specification: `cryp-rq-protocol-v1.md`
- Test Matrix: `docs/TEST_MATRIX.md`
- Security Notes: `docs/SECURITY_NOTES.md`
- Master QA Prompt: `docs/MASTER_QA_PROMPT.md`

