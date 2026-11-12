#!/bin/bash
# Copyright (c) 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Test: Key Rotation
# Observes local P2P key-generation rotation, not negotiated tunnel key agreement

set -euo pipefail

LISTENER_HOST="${LISTENER_CONTAINER:-cryprq-e2e-listener}"
ROTATION_INTERVAL="${CRYPRQ_ROTATE_SECS:-2}"

echo "Test: legacy local rotation timer (not the session keys of the authenticated protocol)"
echo "Verifying key rotation mechanism..."

if ! command -v docker >/dev/null 2>&1; then
    echo "[FAIL] UNRESOLVED: Docker CLI is required to observe rotation."
    exit 2
fi

# The first two local key-generation events may occur immediately at startup.
# Require a third distinct epoch with the isolated test interval in a bounded wait.
for attempt in $(seq 1 15); do
    LOGS=$(docker logs "$LISTENER_HOST" 2>&1)
    EPOCH_COUNT=$(sed -n "s/.*event=key_rotation status=success epoch=\\([0-9][0-9]*\\).*interval_secs=${ROTATION_INTERVAL}$/\\1/p" <<< "$LOGS" | sort -u | wc -l | tr -d ' ')
    if [ "$EPOCH_COUNT" -ge 3 ]; then
        echo "[PASS] Three distinct local P2P key epochs observed at the isolated ${ROTATION_INTERVAL}s interval."
        exit 0
    fi
    sleep 1
done

echo "[FAIL] UNRESOLVED: periodic local key rotation was not observed; startup alone is insufficient."
exit 1
