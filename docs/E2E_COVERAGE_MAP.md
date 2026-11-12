# What replaced the legacy Docker E2E placeholders

The Docker harness in `docker/e2e/` starts two containers in the insecure
test mode and reads their logs. Three of its five tests claimed to verify the
hybrid handshake, file-transfer integrity and packet validation, but only
looked for words in the logs; the CI baseline turned them into explicit
`UNRESOLVED` failures because they asserted nothing.

They are retired. Each claim is now asserted by tests that exercise it, listed
below. The machine-readable form is `docker/e2e/coverage_map.json`;
`scripts/tests/test_e2e_gates.py` fails if any listed assertion disappears, if
a retired placeholder comes back, or if the E2E workflow stops running the
two scripts as unconditional steps.

Where each assertion runs in CI: the Rust tests in `cargo test --workspace`
(job `Build and Test`), the scripts in the job `Authenticated VPN E2E` of
`.github/workflows/e2e-tests.yml`.

## 02_mlkem_x25519_hybrid.sh

Claim: both peers complete the ML-KEM-768 + X25519 handshake and agree on the
same keys; a wrong or substituted peer does not.

| Assertion | Where | What it shows |
|---|---|---|
| `pinned_peers_agree_on_transcript_bound_keys` | `node/src/auth_handshake.rs` | Both state machines end with the same master secret and session id |
| `man_in_the_middle_key_share_substitution_is_detected` | same | A substituted ML-KEM or X25519 share ends the handshake |
| `wrong_key_confirmation_fails_and_yields_no_session` | same | No session without matching key confirmation |
| `low_order_key_share_from_an_authentic_peer_is_fatal` | same | A degenerate X25519 share is refused |
| `test_session_secrets_bind_transcript_and_both_shares` | `crypto/src/kdf.rs` | The secret depends on both shares and the transcript |
| "both processes derived the same session id" | `scripts/e2e-authenticated-file-transfer.sh` | Two separate processes agree, over UDP |
| "sender refused a receiver with another identity" | same | Wrong identity gets no tunnel |
| "dialer got no session with another identity" | `scripts/e2e-authenticated-vpn.sh` | Same for the VPN link |

Stronger than the placeholder, which accepted the words "ML-KEM" and "X25519"
in a log.

## 04_file_transfer_integrity.sh

Claim: a file sent between two processes arrives complete, and the receiver
publishes it only if its SHA-256 matches.

| Assertion | Where | What it shows |
|---|---|---|
| "file arrived byte-identical (sha256 ...)" | `scripts/e2e-authenticated-file-transfer.sh` | The received file's SHA-256 equals the sent file's, across two processes |
| "receiver established nothing and wrote nothing" | same | An unpinned sender leaves no file |
| `rejects_and_removes_file_with_hash_mismatch` | `node/src/file_transfer.rs` | A file whose hash differs from the announcement is removed, not published |
| `publishes_only_after_complete_verified_transfer` | same | Nothing appears under the final name before verification |
| `rejects_data_beyond_announced_size` | same | Oversized data is refused |

The end-to-end script cannot send a corrupted file, because the tunnel
authenticates every record; the mismatch case is therefore a receiver test.

## 05_packet_validation.sh

Claim: replayed, forged and modified packets are injected and rejected, and
nothing is delivered twice or altered.

| Assertion | Where | What it shows |
|---|---|---|
| `replayed_and_forged_datagrams_are_rejected_on_the_wire` | `node/src/tests.rs` | Datagrams injected into a live UDP tunnel are rejected |
| `any_modified_or_truncated_record_is_rejected_without_changing_state` | same | Every modification of a record fails authentication |
| `replayed_and_reordered_records_are_handled_per_session` | same | Replay window behaviour |
| `tampered_records_are_rejected_and_do_not_disturb_the_session` | `p2p/src/secure_vpn_tests.rs` | Same for the VPN link, with the test as the transport |
| `replayed_and_reflected_records_are_rejected` | same | Replay and reflection on the VPN link |
| "no packet is delivered twice under replay" | `scripts/e2e-authenticated-vpn.sh` | A relay that duplicates and replays datagrams causes no duplicate delivery |
| "20 ICMP round trips arrive exactly once and unmodified" | same | Delivery is exact under normal conditions |

In the VPN script the relay sits below QUIC, which rejects most bad datagrams
before the record layer sees them. The record layer's own rejection is what
the Rust tests assert.

## What the Docker harness still checks

| Test | Scope |
|---|---|
| `01_legacy_quic_connection.sh` | Two test-mode containers establish a libp2p/QUIC connection. Not the CrypRQ handshake |
| `02_legacy_rotation_timer.sh` | The legacy local rotation timer fires. Not the session keys of the authenticated protocol |

The harness must run exactly these two and reports
`E2E_RESULT passed=2 failed=0 skipped=0` on success. A pass says nothing about
the authenticated protocol.

## Enforcement

A failing script fails its job and the workflow run. Whether a failed run also
blocks a merge depends on the repository's branch protection; the check names
to require are listed in `docs/CI_VPN_GATES.md`.
