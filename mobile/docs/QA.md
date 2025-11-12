# Mobile QA Checklist

This document provides a reproducible QA checklist for CrypRQ Mobile before release.

## Prerequisites

- Android Emulator (API 33, Pixel 6 profile) or iOS Simulator (iPhone 15, iOS 17)
- Docker Compose (for fake backend)
- Fake backend running: `docker compose -f mobile/docker-compose.yml up -d fake-cryprq`

## Test Scenarios

### 1. First-Run Consent Flow

**Steps:**
1. Fresh install (clear app data)
2. Launch app
3. Verify EULA + Privacy consent screen appears
4. Toggle telemetry OFF
5. Accept EULA and Privacy Policy
6. Tap "Accept and Continue"

**Expected:**
- App proceeds to Dashboard
- Telemetry remains OFF in Settings
- No network calls to telemetry endpoints

**Screenshots:**
- `qa-first-run-consent.png`

---

### 2. Connect → Rotate → Disconnect Flow

**Steps:**
1. Navigate to Dashboard
2. Verify status shows "Disconnected"
3. Tap "Connect"
4. Wait for connection (≤2s)
5. Verify status shows "Connected"
6. Verify peer ID displayed
7. Verify rotation countdown visible and decrementing
8. Wait for rotation (or use Developer screen to simulate)
9. Verify status shows "Rotating" then "Connected"
10. Verify toast notification appears (if enabled)
11. Tap "Disconnect"
12. Verify status shows "Disconnected"

**Expected:**
- All status transitions smooth
- Rotation countdown accurate
- Toast appears only once per rotation
- Tray icon (if applicable) updates correctly

**Screenshots:**
- `qa-connect-flow.png`
- `qa-rotation-flow.png`
- `qa-disconnect-flow.png`

---

### 3. Endpoint Profiles (LOCAL/LAN/REMOTE)

**Steps:**
1. Navigate to Settings
2. Select "LOCAL" profile
3. Verify endpoint shows `http://127.0.0.1:9464`
4. Switch to "LAN" profile
5. Enter endpoint: `http://192.168.1.100:9464`
6. Save settings
7. Switch to "REMOTE" profile
8. Enter HTTP endpoint: `http://example.com`
9. Save settings

**Expected:**
- LOCAL profile works with fake backend
- LAN profile accepts custom endpoint
- REMOTE profile rejects HTTP (shows error: "REMOTE profile requires HTTPS")
- REMOTE profile accepts HTTPS: `https://example.com`

**Screenshots:**
- `qa-endpoint-local.png`
- `qa-endpoint-lan.png`
- `qa-endpoint-remote-error.png`

---

### 4. Invalid Endpoint Handling

**Steps:**
1. Navigate to Settings
2. Select "LAN" profile
3. Enter invalid endpoint: `not-a-url`
4. Save settings
5. Navigate to Dashboard
6. Tap "Connect"

**Expected:**
- Validation error shown for invalid URL
- Connection fails gracefully
- Error message displayed (no crash)

**Screenshots:**
- `qa-invalid-endpoint.png`

---

### 5. Peer Management

**Steps:**
1. Navigate to Peers screen
2. Tap "Add Peer"
3. Enter invalid multiaddr: `invalid-multiaddr`
4. Enter peer ID: `QmTest123...`
5. Verify "Add" button disabled
6. Enter valid multiaddr: `/ip4/127.0.0.1/udp/9999/quic-v1/p2p/QmTest1234567890123456789012345678901234567890`
7. Enter valid peer ID: `QmTest1234567890123456789012345678901234567890`
8. Tap "Add"
9. Verify peer appears in list
10. Tap "Test" on peer
11. Wait for reachability test (≤3s)

**Expected:**
- Invalid multiaddr rejected
- Valid peer added successfully
- Reachability test completes
- Latency displayed

**Screenshots:**
- `qa-peer-add-invalid.png`
- `qa-peer-add-valid.png`
- `qa-peer-reachability.png`

---

### 6. Background Fetch & Notifications

**Steps:**
1. Enable notifications in Settings
2. Connect to peer
3. Put app in background
4. Wait for rotation (or simulate via Developer screen)
5. Verify notification appears: "Keys rotated at HH:MM:SS"

**Expected:**
- Notification appears even when app in background
- Notification text accurate
- No duplicate notifications

**Screenshots:**
- `qa-background-notification.png`

---

### 7. Logs Modal

**Steps:**
1. Navigate to Dashboard
2. Tap "View Logs"
3. Verify last 200 lines displayed
4. Enter search query: "error"
5. Verify only error logs shown
6. Enter search query: "rotation"
7. Verify rotation-related logs shown
8. Clear search
9. Verify all logs shown

**Expected:**
- Logs modal opens
- Search filters correctly
- No PII visible in logs
- Logs scroll smoothly

**Screenshots:**
- `qa-logs-modal.png`
- `qa-logs-search-error.png`
- `qa-logs-search-rotation.png`

---

### 8. Developer Screen

**Steps:**
1. Navigate to Settings
2. Tap version text 5 times
3. Verify Developer screen opens
4. Tap "Simulate Rotation"
5. Verify rotation simulated
6. Tap "Switch to LAN"
7. Verify profile switched

**Expected:**
- Developer screen accessible
- Simulation works
- Quick endpoint switch works

**Screenshots:**
- `qa-developer-screen.png`

---

### 9. Accessibility (A11Y)

**Steps:**
1. Enable screen reader (TalkBack on Android, VoiceOver on iOS)
2. Navigate through Dashboard
3. Verify all buttons have labels
4. Verify focus order logical
5. Navigate to Settings
6. Verify all controls accessible
7. Check color contrast in light/dark themes

**Expected:**
- All actions reachable via screen reader
- Focus order makes sense
- Sufficient contrast (WCAG AA)

**Screenshots:**
- `qa-a11y-light.png`
- `qa-a11y-dark.png`

---

### 10. Security Checks

**Steps:**
1. Enable verbose logging (Developer screen)
2. Connect to peer
3. Export logs
4. Search logs for: "bearer", "privKey", "authorization"
5. Verify no secrets found

**Expected:**
- No secrets in logs
- All sensitive data redacted

**Command:**
```bash
grep -i "bearer\|privKey\|authorization" logs.txt
## Should return no matches
```

---

## Test Results Template

```
Date: YYYY-MM-DD
Tester: [Name]
Platform: Android/iOS
Version: [Version]
Build: [Build Number]

[ ] Test 1: First-Run Consent
[ ] Test 2: Connect → Rotate → Disconnect
[ ] Test 3: Endpoint Profiles
[ ] Test 4: Invalid Endpoint Handling
[ ] Test 5: Peer Management
[ ] Test 6: Background Fetch & Notifications
[ ] Test 7: Logs Modal
[ ] Test 8: Developer Screen
[ ] Test 9: Accessibility
[ ] Test 10: Security Checks

Overall: PASS / FAIL
Notes: [Any issues found]
```

---

## Emulator Profiles

### Android
- **AVD Name:** Pixel_6_API_33
- **API Level:** 33
- **Architecture:** x86_64
- **Profile:** pixel_6

### iOS
- **Device:** iPhone 15
- **iOS Version:** 17.0
- **Simulator:** Xcode Simulator

---

## Commands

```bash
## Start fake backend
docker compose -f mobile/docker-compose.yml up -d fake-cryprq

## Run Android build
cd mobile && npm run android:build

## Run iOS build
cd mobile && npm run ios:build

## Run Detox E2E (Android)
cd mobile && npx detox test -c android.emu.debug

## Run Detox E2E (iOS)
cd mobile && npx detox test -c ios.sim.debug
```

