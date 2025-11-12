# CrypRQ Mobile

React Native mobile app for controlling and monitoring CrypRQ nodes.

## Features

- **Controller Mode**: Connect to CrypRQ nodes via LOCAL, LAN, or REMOTE profiles
- **Real-time Metrics**: Monitor connection status, latency, throughput, and key rotation
- **Peer Management**: Add, remove, connect/disconnect peers
- **Logs Viewer**: View and filter application logs
- **Dark/Light Theme**: System-aware theming

## Prerequisites

- Node.js 18+
- React Native CLI
- Android Studio (for Android development)
- Xcode (for iOS development, macOS only)
- CocoaPods (for iOS)

## Setup

1. Install dependencies:
```bash
npm install
```

2. For iOS, install CocoaPods:
```bash
cd ios && pod install && cd ..
```

3. Start Metro bundler:
```bash
npm start
```

4. Run on Android:
```bash
npm run android
```

5. Run on iOS:
```bash
npm run ios
```

## Testing

### Unit Tests
```bash
npm test
```

### E2E Tests (Detox)

**Android:**
```bash
## Start Android emulator first
npm run e2e:android
```

**iOS:**
```bash
## Start iOS simulator first
npm run e2e:ios
```

## Development with Fake Backend

1. Start the fake CrypRQ backend:
```bash
docker compose -f docker-compose.yml up -d fake-cryprq
```

2. Configure the app to use LOCAL profile (default) or LAN profile with endpoint `http://localhost:9464`

3. Run the app and connect to see metrics updating

## Building

### Android

**Debug:**
```bash
npm run android:build
```

**Release:**
```bash
npm run androidrelease
npm run android:bundle  # For AAB
```

### iOS

**Debug:**
```bash
npm run ios:build
```

**Release:**
```bash
npm run iosrelease
```

## Fastlane

See `fastlane/README.md` for Fastlane setup and usage.

## Project Structure

```
mobile/
 src/
    app/          # Navigation and app root
    screens/      # Screen components
    components/   # Reusable UI components
    store/        # Zustand state management
    services/     # Backend integration
    utils/        # Utilities and validators
    theme/        # Theme definitions
    types/        # TypeScript types
 e2e/              # Detox E2E tests
 tests/            # Unit tests
 fastlane/         # Fastlane configuration
```

## License

MIT

