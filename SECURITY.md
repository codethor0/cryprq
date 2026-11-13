# Security policy

## Status

CrypRQ is a research prototype. It has not been independently audited and is
not intended to protect sensitive traffic. What is implemented, what is tested
and what is not established is recorded in
[Security Notes](docs/SECURITY_NOTES.md).

Only the current `main` branch is maintained. Older tags and the documents
marked as historical describe an earlier web preview and receive no fixes.

## Reporting a vulnerability

Report a suspected vulnerability privately to the maintainer,
[codethor0](https://github.com/codethor0), at the contact address given in the
header of the source files (for example `cli/src/main.rs`). Do not open a
public issue, pull request or discussion that contains exploitable details
before the report has been reviewed and a fix or a disclosure has been
coordinated.

A useful report contains:

- the commit and the component concerned (handshake, record layer, VPN link,
  file transfer, browser portal, build or CI);
- what an attacker needs (network position, a pinned identity, local access)
  and what they gain;
- steps or a minimal test that reproduces the problem;
- whether the finding has been shared with anyone else.

This is an independent project maintained in spare time. Reports are read and
answered as time allows; no response or fix time is promised, and there is no
bug bounty.

## Scope

In scope: the Rust crates, the CLI, the browser portal and its adapter, and the
build and CI configuration in this repository.

Not a vulnerability report:

- limitations that the documentation already states, such as the absence of
  routes, DNS handling and leak protection in the VPN link, classical
  (Ed25519) authentication, or the missing delivery acknowledgement in file
  transfer;
- the legacy libp2p connection mode and the test mode, which are documented as
  unauthenticated;
- findings in third-party dependencies with no effect on CrypRQ. Report those
  to the dependency's maintainers.

Questions and weaknesses without exploitable detail are welcome as ordinary
issues.
