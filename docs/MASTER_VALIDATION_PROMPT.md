# Master Validation Prompt for CrypRQ v1.0.1

## Purpose

This is a comprehensive, reusable prompt for AI/dev-assistant tools to systematically validate the CrypRQ v1.0.1 implementation end-to-end, run tests, check logs against the spec, update validation documentation, and produce a release/no-release verdict.

---

## Master Prompt

```
You are an expert QA engineer and cryptographic protocol auditor helping me validate the CrypRQ v1.0.1 implementation end-to-end.

## Context

The CrypRQ stack currently includes:

- Protocol spec: CrypRQ v1.0.1
  - 20-byte record header (Version, MsgType, Flags, Epoch(u8), StreamID(u32), Seq(u64), CiphertextLen(u32))
  - HKDF-based key schedule with:
    - salt_hs = "cryp-rq v1.0 hs"
    - Epoch-scoped labels for keys
  - TLS 1.3-style XOR nonce construction with per-direction static IVs
  - Epoch as u8 (mod 256), rotation semantics defined
  - Message types: DATA, FILE_META, FILE_CHUNK, FILE_ACK, VPN_PACKET, CONTROL

- Implementation:
  - Record layer in core/node:
    - Encrypt/decrypt with header as AAD
    - DirectionKeys, sequence counters, epoch tracking
  - VPN/TUN path:
    - TUN packets wrapped in MSG_TYPE_VPN_PACKET records
  - File transfer:
    - FileTransferManager using FILE_META / FILE_CHUNK / FILE_ACK via record layer
    - CLI: `cryprq send-file` / `cryprq receive-file` wired through Tunnel + record layer
  - Keys are currently **test-mode** (hardcoded), as documented in SECURITY_NOTES.md.

- Documentation:
  - docs/TEST_MATRIX.md — test plan
  - docs/MASTER_QA_PROMPT.md — general QA helper
  - docs/MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md — next phase (real handshake + identity)
  - docs/VALIDATION_RUN.md — run log + results tracker
  - docs/SECURITY_NOTES.md — current security posture, limitations, and migration path

The test file is:
- Path: `/tmp/testfile.bin`
- SHA-256: `6e2e53bc5d2a187becf1d734d7cea4488042784f188cf4615054d2f2a39db7ec`
- Content: `"Test file for CrypRQ v1.0.1 validation"`

The quick sanity test is:
- Receiver:
  `cryprq receive-file --listen /ip4/0.0.0.0/udp/20440/quic-v1 --output-dir /tmp/receive`
- Sender:
  `cryprq send-file --peer /ip4/127.0.0.1/udp/20440/quic-v1 --file /tmp/testfile.bin`
- Verification:
  `sha256sum /tmp/testfile.bin /tmp/receive/testfile.bin` must match the SHA-256 above.

## Your Goals

1. **Validate correctness** of the CrypRQ v1.0.1 implementation against:
   - The protocol specification (record header, epochs, nonces, key schedule).
   - The behavior described in docs/TEST_MATRIX.md.

2. **Populate and update** docs/VALIDATION_RUN.md with:
   - Each test that was run
   - Parameters
   - Observed results
   - PASS/FAIL status
   - Pointers to relevant logs

3. **Produce a structured QA report**:
   - List which parts of the stack are validated (OK for internal use).
   - List any deviations from the spec or flaky behaviors.
   - Call out any blockers that must be fixed before "web-only" internal deployment.

4. **Confirm security assumptions**:
   - That current limitations (hardcoded keys, no peer auth) match SECURITY_NOTES.md.
   - That there are NO surprises (e.g., keys in logs, nonces reused, broken epoch behavior).

## Tasks

### SECTION 1 — Repo and Docs Scan

1. Locate and skim:
   - Protocol spec document(s) for CrypRQ v1.0.1.
   - `core/src/record.rs` and `node/src/record_layer.rs` (or equivalent).
   - `crypto/src/kdf.rs` (HKDF and key schedule).
   - `node/src/lib.rs` and `node/src/file_transfer.rs`.
   - `docs/TEST_MATRIX.md`
   - `docs/VALIDATION_RUN.md`
   - `docs/SECURITY_NOTES.md`

2. Summarize:
   - How records are constructed (header layout, AAD).
   - How nonces are built (exact XOR construction).
   - How sequence numbers and epochs are stored and reset.
   - How file transfer and VPN traffic are routed through the record layer.

Output as:  
**SECTION 1: Implementation Map**

### SECTION 2 — Minimal Sanity Test (Happy Path)

1. Run the quick sanity test:
   - Receiver: `cryprq receive-file --listen /ip4/0.0.0.0/udp/20440/quic-v1 --output-dir /tmp/receive`
   - Sender: `cryprq send-file --peer /ip4/127.0.0.1/udp/20440/quic-v1 --file /tmp/testfile.bin`

2. Verify:
   - SHA-256 matches `6e2e53bc5d2a187becf1d734d7cea4488042784f188cf4615054d2f2a39db7ec`

3. Collect logs for both sender and receiver.

4. In the logs, confirm:
   - FILE_META → FILE_CHUNK messages flow as expected.
   - Epoch and sequence numbers increment sanely.
   - No AEAD decryption failures.
   - Nonces are not reused for the same key (at least according to debug output if available).

Update `docs/VALIDATION_RUN.md` with:
   - Test name: Minimal sanity test
   - Date/time
   - Commands used
   - Result (PASS/FAIL)
   - Log file paths

Output as:  
**SECTION 2: Minimal Sanity Test Result**

### SECTION 3 — Test Matrix Execution

Using docs/TEST_MATRIX.md:

1. Execute each test class at least once:
   - Tiny file transfer (single chunk)
   - Medium file (multi-chunk) transfer
   - Large file that spans at least one epoch rotation (wait long enough or tweak epoch interval if needed)
   - Concurrent transfers (two or more files at once)
   - Interrupted transfer (kill sender mid-transfer, then observe receiver behavior)
   - VPN/TUN tests:
     - Bring up VPN mode
     - Send ping or simple HTTP over VPN
   - Web UI tests:
     - Start web stack (`docker compose -f docker-compose.web.yml up` or equivalent)
     - Drive a file transfer from the web UI
     - Confirm the record layer is being used under the hood.

2. For each test case:
   - Note expected result from TEST_MATRIX.md.
   - Run the test.
   - Capture:
     - Outcome (PASS/FAIL)
     - Any error messages
     - Relevant log snippets.

3. Update docs/VALIDATION_RUN.md:
   - Use the run template already present.
   - Fill in each test row with:
     - Status (PASS/FAIL)
     - Notes (e.g., "1s jitter but acceptable", or "nonce debug missing for VPN")
     - Log location(s).

Output as:  
**SECTION 3: Test Matrix Summary**  
Use a small table: Test Name / Status / Notes.

### SECTION 4 — Protocol Alignment Checks

Using the protocol spec and the implementation:

1. Verify that **record headers on the wire** match the v1.0.1 spec:
   - 20 bytes exact structure.
   - Correct MessageType values for:
     - DATA
     - FILE_META
     - FILE_CHUNK
     - FILE_ACK
     - VPN_PACKET
     - CONTROL
   - Epoch encoded as u8 and matches what the code believes internally.
   - Ciphertext length field matches actual encrypted payload length.

2. Using logs or a small custom capture:
   - Extract a few example records (hex dumps).
   - Annotate each field and confirm:
     - Version == 0x01
     - MessageType matches expectations
     - Stream IDs are correct (e.g., VPN=1, file streams >=2)
     - Sequence numbers increase monotonically per direction.

3. Confirm nonce construction:
   - From code: IV ⊕ (sequence_number encoded as big-endian) → 96-bit AEAD nonce.
   - From logs: sanity-check that changing the sequence changes nonce bits.

4. Confirm HKDF key schedule usage:
   - salt_hs and labels match spec.
   - Epoch-scoped labels are actually used when deriving keys for each epoch.
   - Sequence counters reset on epoch change.

Output as:  
**SECTION 4: Protocol Alignment Findings**  
- List items as "MATCHES SPEC" or "DEVIATION" with file + line references when possible.

### SECTION 5 — Security Notes Cross-Check

1. Compare current behavior and code with docs/SECURITY_NOTES.md:
   - Confirm that:
     - Hardcoded test keys are indeed still in use.
     - No actual peer auth or handshake is implemented yet (unless that has changed).
     - SECURITY_NOTES.md correctly describes what is and isn't safe to use in production.

2. Scan logs and code for:
   - Any accidental logging of:
     - Keys
     - Nonces
     - Raw plaintext application data

3. If any mismatch is found (e.g., docs say "we do X" but the code does Y), record it.

Output as:  
**SECTION 5: Security Posture Validation**  
- "As-documented and consistent" or list mismatches.

### SECTION 6 — Final Verdict and Next Steps

Based on all tests and checks:

1. Provide a concise verdict:
   - "Suitable for internal testing only" or
   - "Suitable for internal web-only VPN/file-transfer deployment under the constraints in SECURITY_NOTES.md"

2. List:
   - MUST-FIX items before any broader deployment.
   - SHOULD-FIX items (non-blockers, but worth doing).
   - NICE-TO-HAVE items (UX/observability/perf).

3. Point explicitly to the next-phase prompt:
   - docs/MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md
   - and recommend using it to remove hardcoded keys and add real peer authentication.

Output as:  
**SECTION 6: Verdict & Recommended Next Steps**

## Output Format (Important)

Return your findings structured as:

- SECTION 1: Implementation Map  
- SECTION 2: Minimal Sanity Test Result  
- SECTION 3: Test Matrix Summary  
- SECTION 4: Protocol Alignment Findings  
- SECTION 5: Security Posture Validation  
- SECTION 6: Verdict & Recommended Next Steps

Be concrete, reference file paths and function names where possible, and clearly separate "PASS", "FAIL", and "DEVIATION FROM SPEC".
```

---

## Quick Start Commands

If you want to run the minimal sanity test immediately:

```bash
# Build
cargo build --release -p cryprq

# Terminal 1 - Receiver
cryprq receive-file \
  --listen /ip4/0.0.0.0/udp/20440/quic-v1 \
  --output-dir /tmp/receive

# Terminal 2 - Sender
cryprq send-file \
  --peer /ip4/127.0.0.1/udp/20440/quic-v1 \
  --file /tmp/testfile.bin

# Verify
sha256sum /tmp/testfile.bin /tmp/receive/testfile.bin
# Expected: Both should show 6e2e53bc5d2a187becf1d734d7cea4488042784f188cf4615054d2f2a39db7ec
```

After that first PASS, you've got your anchor, and you can let the master validation prompt drive the more exhaustive matrix + log review.

---

## Usage Instructions

1. **Copy the Master Prompt** above into your AI/dev-assistant tool that has repository access.

2. **The tool will:**
   - Scan the codebase and documentation
   - Run the test matrix systematically
   - Check protocol alignment
   - Validate security posture
   - Update VALIDATION_RUN.md
   - Produce a structured QA report

3. **Review the output** organized into 6 sections:
   - Implementation Map
   - Minimal Sanity Test Result
   - Test Matrix Summary
   - Protocol Alignment Findings
   - Security Posture Validation
   - Verdict & Recommended Next Steps

4. **Use the verdict** to determine:
   - Whether the stack is ready for internal testing/deployment
   - What must be fixed before production
   - What the next phase should focus on

---

## Expected Output Format

### SECTION 1: Implementation Map
- Record construction: [file paths, function names]
- Nonce construction: [file paths, function names]
- Sequence/epoch management: [file paths, function names]
- Routing: [file paths, function names]

### SECTION 2: Minimal Sanity Test Result
- Status: PASS / FAIL
- SHA-256 Match: ✅ / ❌
- Logs: [file paths]
- Observations: [key findings]

### SECTION 3: Test Matrix Summary
| Test Name | Status | Notes |
|-----------|--------|-------|
| Tiny file | PASS/FAIL | ... |
| Medium file | PASS/FAIL | ... |
| Large file | PASS/FAIL | ... |
| Concurrent | PASS/FAIL | ... |
| Interrupted | PASS/FAIL | ... |
| VPN/TUN | PASS/FAIL | ... |
| Web UI | PASS/FAIL | ... |

### SECTION 4: Protocol Alignment Findings
- Record header: MATCHES SPEC / DEVIATION [details]
- Message types: MATCHES SPEC / DEVIATION [details]
- Nonce construction: MATCHES SPEC / DEVIATION [details]
- HKDF keys: MATCHES SPEC / DEVIATION [details]

### SECTION 5: Security Posture Validation
- Hardcoded keys: As documented / Mismatch
- Peer auth: As documented / Mismatch
- Key logging: None found / Found [locations]
- Nonce reuse: None found / Found [locations]

### SECTION 6: Verdict & Recommended Next Steps
- Verdict: [Suitable for... / Not suitable for...]
- MUST-FIX: [list]
- SHOULD-FIX: [list]
- NICE-TO-HAVE: [list]
- Next Phase: [recommendation]

---

## Related Documents

- Test Matrix: `docs/TEST_MATRIX.md`
- Validation Run Tracker: `docs/VALIDATION_RUN.md`
- Security Notes: `docs/SECURITY_NOTES.md`
- Master QA Prompt: `docs/MASTER_QA_PROMPT.md`
- Handshake Prompt: `docs/MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md`
- Protocol Spec: `cryp-rq-protocol-v1.md`

