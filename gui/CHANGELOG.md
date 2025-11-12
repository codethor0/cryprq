# Changelog

All notable changes to CrypRQ Desktop will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2025-01-15

### Added
- Structured logging schema (v1) with versioned JSONL format
- Strict secret redaction for logs (bearer tokens, private keys, authorization headers)
- Diagnostics export with session summary and metrics snapshot
- System tray dev hooks for CI testing (`devsnapshot`)
- Fault injection testing hooks (`devsimulateExit`)
- Rotation UX improvements with event-driven countdown synchronization
- Enhanced error handling with user-friendly messages
- Log validation with automatic wrapping of invalid entries

### Changed
- Logging system now uses structured JSONL format instead of plain text
- Diagnostics export includes computed session statistics and timeline
- Tray updates are now unified through `updateTrayFromSession` function
- Rotation countdown resyncs automatically from metrics and events

### Fixed
- Tray state consistency across session start/restart/error flows
- Log redaction now covers nested objects and all string fields
- Rotation countdown jitter handling with periodic resync

### Security
- All secrets are redacted before being written to logs
- Structured log validation prevents malformed entries
- Enhanced redaction patterns for comprehensive secret protection

## [1.0.0] - Initial Release

### Added
- Initial Electron GUI with Dashboard, Peers, Settings, and Logs screens
- System tray integration with quick actions
- Prometheus metrics polling
- Key rotation countdown display
- Peer management (add, remove, connect, disconnect)
- Settings persistence
- Log viewer with filtering
- Cross-platform support (Windows, macOS, Linux)

