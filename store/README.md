# Store Listing Assets

This directory contains store listing content and assets for Google Play Store and Apple App Store.

## Structure

```
store/
 play/
    short.txt          # Short description (≤80 chars)
    full.txt           # Full description (≤4000 chars)
    screenshots/       # Screenshots (1080×1920)
 appstore/
    promo.txt         # Promotional text
    keywords.txt      # Keywords (≤100 chars)
    subtitle.txt      # Subtitle
    screenshots/      # Screenshots (6.7", 6.1")
 validate.mjs          # Validation script
```

## Screenshots

### Google Play Store
- **Size**: 1080×1920 (phone)
- **Themes**: Dark + Light
- **Devices**: Phone (portrait)

### Apple App Store
- **Sizes**: 
  - 6.7" (iPhone 14 Pro Max): 1290×2796
  - 6.1" (iPhone 14 Pro): 1179×2556
- **Themes**: Dark + Light
- **Devices**: Phone (portrait)

## Commands to Capture Screenshots

### Android (Emulator)
```bash
## Start emulator
emulator -avd Pixel_6_API_33 &

## Take screenshot
adb shell screencap -p /sdcard/screenshot.png
adb pull /sdcard/screenshot.png store/play/screenshots/
```

### iOS (Simulator)
```bash
## Start simulator
xcrun simctl boot "iPhone 14 Pro Max"

## Take screenshot
xcrun simctl io booted screenshot store/appstore/screenshots/6.7-dark.png
```

## Validation

Run validation script:
```bash
node store/validate.mjs
```

Checks:
- Length limits (short ≤80, full ≤4000, keywords ≤100)
- Missing locales
- Privacy URL validity
- Screenshot dimensions

## Privacy Policy URL

**URL**: [Your Privacy Policy URL]

Must be:
- HTTPS
- Accessible (200 OK)
- Same URL in:
  - Desktop "Privacy" screen
  - Google Play Store listing
  - Apple App Store listing

