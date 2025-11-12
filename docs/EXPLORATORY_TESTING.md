# Exploratory Testing Guide

This document describes the exploratory testing process for CrypRQ to verify technology functionality and identify edge cases.

## Overview

Exploratory testing involves ad-hoc testing to discover how the application behaves under various conditions, edge cases, and unexpected inputs.

## Test Categories

### 1. Basic Functionality

**Purpose**: Verify core application functionality.

**Tests**:
- Application starts and shows help
- Command-line arguments parsed correctly
- Configuration loading works

**Script**: `scripts/exploratory-testing.sh`

### 2. Cryptographic Operations

**Purpose**: Verify cryptographic algorithms function correctly.

**Tests**:
- Hybrid handshake (ML-KEM + X25519)
- PPK derivation and expiration
- Key rotation mechanism
- Zero-knowledge proofs

**Script**: `scripts/crypto-validation.sh`

### 3. Edge Cases

**Purpose**: Test application behavior with invalid or unexpected inputs.

**Tests**:
- Invalid command-line arguments
- Empty configuration
- Missing dependencies
- Network failures
- Resource exhaustion

**Script**: `scripts/exploratory-testing.sh`

### 4. Error Handling

**Purpose**: Verify graceful error handling.

**Tests**:
- Panic handlers present
- Error messages informative
- Recovery mechanisms work
- Logging captures errors

**Script**: `scripts/exploratory-testing.sh`

### 5. Resource Usage

**Purpose**: Monitor resource consumption.

**Tests**:
- Binary size acceptable
- Memory usage reasonable
- CPU usage efficient
- Network bandwidth appropriate

**Script**: `scripts/performance-benchmark.sh`

### 6. Network Functionality

**Purpose**: Verify network operations.

**Tests**:
- Listener starts correctly
- Connections established
- Data transmission works
- Network errors handled

**Script**: `scripts/end-to-end-tests.sh`

## Running Exploratory Tests

### Quick Test

```bash
bash scripts/exploratory-testing.sh
```

### Comprehensive Test Suite

```bash
## Run all exploratory tests
bash scripts/exploratory-testing.sh
bash scripts/performance-benchmark.sh
bash scripts/crypto-validation.sh
bash scripts/end-to-end-tests.sh
```

### Docker Environment

```bash
## Run in Docker
docker exec cryprq-test-runner bash -c "cd /workspace && bash scripts/exploratory-testing.sh"
```

## Test Scenarios

### Scenario 1: Normal Operation

1. Start listener
2. Connect dialer
3. Verify handshake
4. Verify key rotation
5. Verify data transmission

### Scenario 2: Error Recovery

1. Start listener
2. Kill listener process
3. Restart listener
4. Verify recovery
5. Verify state consistency

### Scenario 3: Resource Limits

1. Start with limited memory
2. Start with limited CPU
3. Start with limited network
4. Verify graceful degradation

### Scenario 4: Concurrent Operations

1. Multiple listeners
2. Multiple dialers
3. Concurrent connections
4. Verify no race conditions

## Edge Cases to Test

1. **Invalid Inputs**
   - Empty strings
   - Null values
   - Extremely long strings
   - Special characters
   - Unicode characters

2. **Network Conditions**
   - Slow connections
   - Packet loss
   - High latency
   - Connection timeouts

3. **Resource Constraints**
   - Low memory
   - Limited CPU
   - Disk space issues
   - File descriptor limits

4. **Concurrency**
   - Multiple threads
   - Race conditions
   - Deadlocks
   - Resource contention

## Test Results

Test results are logged to:
- `exploratory-test-*.log` - Test execution logs
- `performance-benchmark-*.log` - Performance metrics

## Best Practices

1. **Document Findings**: Record all issues and observations
2. **Reproduce Issues**: Ensure issues can be reproduced
3. **Prioritize**: Focus on critical functionality first
4. **Automate**: Convert manual tests to automated tests
5. **Iterate**: Continuously improve test coverage

## Continuous Improvement

- Add new test scenarios based on findings
- Update test scripts with new edge cases
- Share findings with development team
- Document patterns and anti-patterns

## Summary

Exploratory testing helps ensure:
-  Application functions correctly
-  Edge cases handled gracefully
-  Performance meets requirements
-  Security features work as intended
-  Error handling is robust

For questions or issues, see:
- `docs/TESTING.md` - General testing guide
- `docs/PERFORMANCE.md` - Performance optimization
- `CONTRIBUTING.md` - Contribution guidelines

