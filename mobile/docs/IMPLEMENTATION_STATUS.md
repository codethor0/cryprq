# CrypRQ Mobile Implementation Status

##  Completed Milestones

### M1 - Bootstrap React Native App Structure 
- [x] TypeScript configuration
- [x] React Navigation setup (bottom tabs + stack)
- [x] Zustand store with MMKV persistence
- [x] Theme system (light/dark/system)
- [x] Directory structure
- [x] Package.json with all dependencies

### M2 - Backend Integration 
- [x] Backend service with Prometheus metrics parsing
- [x] Profile support (LOCAL/LAN/REMOTE)
- [x] Metrics polling (2s interval)
- [x] Connection state management
- [x] Peer management (add/remove/connect/disconnect)
- [x] Reachability testing

### M3 - Background Refresh + Notifications 
- [x] Background fetch service (15min minimum)
- [x] AppState-aware polling (fast foreground, paused background)
- [x] Local notifications (Notifee)
- [x] Notification settings (connect/disconnect, rotations)
- [x] Rotation notifications

### M4 - Testing Strategy 
- [x] Jest + React Native Testing Library setup
- [x] Unit tests for store and components
- [x] Detox E2E test configuration
- [x] Test IDs added to key components
- [x] E2E test scenarios (Dashboard, Peers)

### M5 - Dockerized Fake Backend + CI 
- [x] Docker Compose configuration
- [x] Reuses existing fake-cryprq backend
- [x] GitHub Actions workflow for Android
- [x] GitHub Actions workflow for iOS
- [x] Android emulator runner integration

### M6 - Mobile UX Parity 
- [x] Dashboard screen (status, metrics, throughput chart)
- [x] Peers screen (list, add, connect/disconnect, test)
- [x] Settings screen (profile, endpoint, rotation, notifications)
- [x] Logs modal (filter, search, level filter)
- [x] Shared components (StatusPill, Card, Button)
- [x] Theme-aware styling

### M7 - Release Builds & Fastlane 
- [x] Fastlane configuration (Android + iOS)
- [x] Build lanes (debug, release, beta)
- [x] CI workflows for tagged releases
- [x] Artifact uploads

### M8 - On-Device Core Plan 
- [x] Implementation plan document
- [x] Android JNI + VpnService approach
- [x] iOS Network Extension approach
- [x] Risk assessment
- [x] Milestone breakdown

##  Next Steps

### Immediate (Before First Build)

1. **Initialize React Native Project**
   ```bash
   cd mobile
   npx react-native init CrypRQ --template react-native-template-typescript
   # Then merge our src/ structure
   ```

2. **Install Dependencies**
   ```bash
   npm install
   cd ios && pod install && cd ..
   ```

3. **Configure Native Projects**
   - Android: Update `android/app/build.gradle` with package name `io.cryprq.mobile`
   - iOS: Update bundle identifier to `io.cryprq.mobile`

4. **Fix TypeScript Errors**
   - Add proper type definitions
   - Fix Card component children prop (make optional)
   - Add proper types for EventEmitter

### Testing

1. **Run Unit Tests**
   ```bash
   npm test
   ```

2. **Run E2E Tests**
   ```bash
   # Start fake backend
   docker compose -f docker-compose.yml up -d fake-cryprq
   
   # Android
   npm run e2e:android
   
   # iOS
   npm run e2e:ios
   ```

### Known Issues

1. **Card Component**: `children` prop should be optional
2. **EventEmitter Types**: Need to properly type EventEmitter3
3. **SettingsScreen**: Picker import needs verification
4. **Background Service**: AppState listener API may need updates for React Native 0.73

##  File Structure

```
mobile/
 src/
    app/
       App.tsx              # Root component with navigation
    screens/
       DashboardScreen.tsx  # Main dashboard
       PeersScreen.tsx      # Peer management
       SettingsScreen.tsx   # Settings
       LogsModal.tsx        # Logs viewer
    components/
       StatusPill.tsx       # Status indicator
       Card.tsx             # Card container
       Button.tsx           # Button component
    store/
       appStore.ts          # Zustand store
    services/
       backend.ts           # Backend integration
       notifications.ts     # Notifications
       background.ts       # Background tasks
    utils/
       validation.ts        # Validators
    theme/
       index.ts             # Theme definitions
    types/
        index.ts             # Main types
        errors.ts            # Error types
 e2e/
    dashboard.e2e.ts         # Dashboard E2E tests
    peers.e2e.ts            # Peers E2E tests
 tests/
    setup.ts                # Test setup
    unit/                   # Unit tests
 fastlane/                   # Fastlane configs
 docker-compose.yml          # Fake backend
 package.json                # Dependencies
```

##  Quick Start

1. **Clone and Setup**
   ```bash
   cd mobile
   npm install
   cd ios && pod install && cd ..
   ```

2. **Start Fake Backend**
   ```bash
   docker compose -f docker-compose.yml up -d fake-cryprq
   ```

3. **Run App**
   ```bash
   # Android
   npm run android
   
   # iOS
   npm run ios
   ```

4. **Configure Profile**
   - Open Settings tab
   - Select "Local" profile (default)
   - Or "LAN" with endpoint `http://localhost:9464`

5. **Connect**
   - Go to Dashboard
   - Tap "Connect"
   - Watch metrics update!

##  Notes

- The app is in "controller mode" - it connects to external CrypRQ nodes
- For on-device core, see `docs/ON_DEVICE_CORE.md`
- All store state persists via MMKV
- Background refresh runs every 15 minutes minimum
- Notifications require user permission

