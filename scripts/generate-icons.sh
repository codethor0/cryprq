#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Icon Generation Script - All Platforms"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

MASTER_ICON="store-assets/icon_master_1024.png"
REPORT_FILE="artifacts/icons/icon_report.txt"
mkdir -p artifacts/icons

# Check for ImageMagick
if command -v magick &> /dev/null; then
    MAGICK_CMD="magick"
elif command -v convert &> /dev/null; then
    MAGICK_CMD="convert"
else
    echo "⚠️  ImageMagick not found. Install with: brew install imagemagick (macOS) or apt-get install imagemagick (Linux)"
    echo "   Some icon generation steps will be skipped."
    MAGICK_CMD=""
fi

# Check for macOS iconutil
ICONUTIL_AVAILABLE=false
if command -v iconutil &> /dev/null && [[ "$(uname)" == "Darwin" ]]; then
    ICONUTIL_AVAILABLE=true
fi

# Verify master icon exists
if [ ! -f "$MASTER_ICON" ]; then
    echo "❌ Master icon not found: $MASTER_ICON"
    echo "   Please ensure store-assets/icon_master_1024.png exists"
    exit 1
fi

echo "✅ Master icon found: $MASTER_ICON"
echo ""

# Function to generate PNG at specific size
generate_png() {
    local size=$1
    local output=$2
    if [ -n "$MAGICK_CMD" ]; then
        $MAGICK_CMD "$MASTER_ICON" -resize "${size}x${size}" "$output"
        echo "  Generated: $output (${size}x${size})"
    else
        echo "  ⚠️  Skipped: $output (ImageMagick not available)"
    fi
}

# Function to get file checksum
get_checksum() {
    local file=$1
    if [ -f "$file" ]; then
        shasum -a 256 "$file" 2>/dev/null | awk '{print $1}' || md5sum "$file" 2>/dev/null | awk '{print $1}' || echo "N/A"
    else
        echo "N/A"
    fi
}

# Start report
{
    echo "CrypRQ Icon Generation Report"
    echo "Generated: $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
    echo "Master Icon: $MASTER_ICON"
    echo "Master Checksum: $(get_checksum "$MASTER_ICON")"
    echo ""
} > "$REPORT_FILE"

echo "[1/7] Android Icons"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -d "android/app/src/main/res" ]; then
    mkdir -p android/app/src/main/res/mipmap-xxxhdpi
    mkdir -p android/app/src/main/res/mipmap-xxhdpi
    mkdir -p android/app/src/main/res/mipmap-xhdpi
    mkdir -p android/app/src/main/res/mipmap-hdpi
    mkdir -p android/app/src/main/res/mipmap-mdpi
    mkdir -p android/app/src/main/res/mipmap-anydpi-v26
    
    generate_png 192 android/app/src/main/res/mipmap-xxxhdpi/ic_launcher.png
    generate_png 144 android/app/src/main/res/mipmap-xxhdpi/ic_launcher.png
    generate_png 96 android/app/src/main/res/mipmap-xhdpi/ic_launcher.png
    generate_png 72 android/app/src/main/res/mipmap-hdpi/ic_launcher.png
    generate_png 48 android/app/src/main/res/mipmap-mdpi/ic_launcher.png
    
    # Generate Play Store asset
    generate_png 512 store-assets/android_play_512.png
    
    # Create adaptive icon XML (simplified)
    cat > android/app/src/main/res/mipmap-anydpi-v26/ic_launcher.xml <<'EOF'
<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@color/ic_launcher_background"/>
    <foreground android:drawable="@mipmap/ic_launcher_foreground"/>
</adaptive-icon>
EOF
    
    cat > android/app/src/main/res/mipmap-anydpi-v26/ic_launcher_round.xml <<'EOF'
<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@color/ic_launcher_background"/>
    <foreground android:drawable="@mipmap/ic_launcher_foreground"/>
</adaptive-icon>
EOF
    
    echo "  ✅ Android icons generated"
    {
        echo "Android Icons:"
        for size in 192 144 96 72 48; do
            file="android/app/src/main/res/mipmap-$(case $size in 192) echo xxxhdpi;; 144) echo xxhdpi;; 96) echo xhdpi;; 72) echo hdpi;; 48) echo mdpi;; esac)/ic_launcher.png"
            if [ -f "$file" ]; then
                echo "  - $file (${size}x${size}): $(get_checksum "$file")"
            fi
        done
        echo ""
    } >> "$REPORT_FILE"
else
    echo "  ⚠️  Android directory not found, skipping"
fi

echo ""
echo "[2/7] iOS Icons"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -d "apple" ] || [ -d "ios" ]; then
    IOS_DIR="apple"
    [ -d "ios" ] && IOS_DIR="ios"
    
    ASSETS_DIR="${IOS_DIR}/Sources/CrypRQ/Assets.xcassets/AppIcon.appiconset"
    mkdir -p "$ASSETS_DIR"
    
    # Generate iOS icon sizes (pt @2x/@3x)
    # 20pt @2x=40px @3x=60px
    # 29pt @2x=58px @3x=87px
    # 40pt @2x=80px @3x=120px
    # 60pt @2x=120px @3x=180px
    # 76pt @2x=152px
    # 83.5pt @2x=167px
    # 1024px App Store
    
    generate_png 40 "${ASSETS_DIR}/icon_20pt@2x.png"
    generate_png 60 "${ASSETS_DIR}/icon_20pt@3x.png"
    generate_png 58 "${ASSETS_DIR}/icon_29pt@2x.png"
    generate_png 87 "${ASSETS_DIR}/icon_29pt@3x.png"
    generate_png 80 "${ASSETS_DIR}/icon_40pt@2x.png"
    generate_png 120 "${ASSETS_DIR}/icon_40pt@3x.png"
    generate_png 120 "${ASSETS_DIR}/icon_60pt@2x.png"
    generate_png 180 "${ASSETS_DIR}/icon_60pt@3x.png"
    generate_png 152 "${ASSETS_DIR}/icon_76pt@2x.png"
    generate_png 167 "${ASSETS_DIR}/icon_83.5pt@2x.png"
    generate_png 1024 "${ASSETS_DIR}/icon_1024.png"
    
    # Generate Contents.json
    cat > "${ASSETS_DIR}/Contents.json" <<'EOF'
{
  "images" : [
    {
      "idiom" : "iphone",
      "scale" : "2x",
      "size" : "20x20",
      "filename" : "icon_20pt@2x.png"
    },
    {
      "idiom" : "iphone",
      "scale" : "3x",
      "size" : "20x20",
      "filename" : "icon_20pt@3x.png"
    },
    {
      "idiom" : "iphone",
      "scale" : "2x",
      "size" : "29x29",
      "filename" : "icon_29pt@2x.png"
    },
    {
      "idiom" : "iphone",
      "scale" : "3x",
      "size" : "29x29",
      "filename" : "icon_29pt@3x.png"
    },
    {
      "idiom" : "iphone",
      "scale" : "2x",
      "size" : "40x40",
      "filename" : "icon_40pt@2x.png"
    },
    {
      "idiom" : "iphone",
      "scale" : "3x",
      "size" : "40x40",
      "filename" : "icon_40pt@3x.png"
    },
    {
      "idiom" : "iphone",
      "scale" : "2x",
      "size" : "60x60",
      "filename" : "icon_60pt@2x.png"
    },
    {
      "idiom" : "iphone",
      "scale" : "3x",
      "size" : "60x60",
      "filename" : "icon_60pt@3x.png"
    },
    {
      "idiom" : "ipad",
      "scale" : "1x",
      "size" : "20x20",
      "filename" : "icon_20pt@2x.png"
    },
    {
      "idiom" : "ipad",
      "scale" : "2x",
      "size" : "20x20",
      "filename" : "icon_20pt@2x.png"
    },
    {
      "idiom" : "ipad",
      "scale" : "1x",
      "size" : "29x29",
      "filename" : "icon_29pt@2x.png"
    },
    {
      "idiom" : "ipad",
      "scale" : "2x",
      "size" : "29x29",
      "filename" : "icon_29pt@2x.png"
    },
    {
      "idiom" : "ipad",
      "scale" : "1x",
      "size" : "40x40",
      "filename" : "icon_40pt@2x.png"
    },
    {
      "idiom" : "ipad",
      "scale" : "2x",
      "size" : "40x40",
      "filename" : "icon_40pt@2x.png"
    },
    {
      "idiom" : "ipad",
      "scale" : "2x",
      "size" : "76x76",
      "filename" : "icon_76pt@2x.png"
    },
    {
      "idiom" : "ipad",
      "scale" : "2x",
      "size" : "83.5x83.5",
      "filename" : "icon_83.5pt@2x.png"
    },
    {
      "idiom" : "ios-marketing",
      "scale" : "1x",
      "size" : "1024x1024",
      "filename" : "icon_1024.png"
    }
  ],
  "info" : {
    "author" : "xcode",
    "version" : 1
  }
}
EOF
    
    echo "  ✅ iOS icons generated"
    {
        echo "iOS Icons:"
        echo "  - ${ASSETS_DIR}/Contents.json: $(get_checksum "${ASSETS_DIR}/Contents.json")"
        echo ""
    } >> "$REPORT_FILE"
else
    echo "  ⚠️  iOS directory not found, skipping"
fi

echo ""
echo "[3/7] macOS Icons"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -d "apple" ] && [ "$ICONUTIL_AVAILABLE" = true ]; then
    MACOS_ASSETS_DIR="apple/Sources/CrypRQ/Assets.xcassets/AppIcon.appiconset"
    mkdir -p "$MACOS_ASSETS_DIR"
    
    # Generate macOS icon sizes
    generate_png 16 "${MACOS_ASSETS_DIR}/icon_16.png"
    generate_png 32 "${MACOS_ASSETS_DIR}/icon_16@2x.png"
    generate_png 32 "${MACOS_ASSETS_DIR}/icon_32.png"
    generate_png 64 "${MACOS_ASSETS_DIR}/icon_32@2x.png"
    generate_png 128 "${MACOS_ASSETS_DIR}/icon_128.png"
    generate_png 256 "${MACOS_ASSETS_DIR}/icon_128@2x.png"
    generate_png 256 "${MACOS_ASSETS_DIR}/icon_256.png"
    generate_png 512 "${MACOS_ASSETS_DIR}/icon_256@2x.png"
    generate_png 512 "${MACOS_ASSETS_DIR}/icon_512.png"
    generate_png 1024 "${MACOS_ASSETS_DIR}/icon_512@2x.png"
    
    # Create .icns using iconutil
    ICONSET_DIR="branding/macos.iconset"
    mkdir -p "$ICONSET_DIR"
    
    cp "${MACOS_ASSETS_DIR}/icon_16.png" "${ICONSET_DIR}/icon_16x16.png"
    cp "${MACOS_ASSETS_DIR}/icon_16@2x.png" "${ICONSET_DIR}/icon_16x16@2x.png"
    cp "${MACOS_ASSETS_DIR}/icon_32.png" "${ICONSET_DIR}/icon_32x32.png"
    cp "${MACOS_ASSETS_DIR}/icon_32@2x.png" "${ICONSET_DIR}/icon_32x32@2x.png"
    cp "${MACOS_ASSETS_DIR}/icon_128.png" "${ICONSET_DIR}/icon_128x128.png"
    cp "${MACOS_ASSETS_DIR}/icon_128@2x.png" "${ICONSET_DIR}/icon_128x128@2x.png"
    cp "${MACOS_ASSETS_DIR}/icon_256.png" "${ICONSET_DIR}/icon_256x256.png"
    cp "${MACOS_ASSETS_DIR}/icon_256@2x.png" "${ICONSET_DIR}/icon_256x256@2x.png"
    cp "${MACOS_ASSETS_DIR}/icon_512.png" "${ICONSET_DIR}/icon_512x512.png"
    cp "${MACOS_ASSETS_DIR}/icon_512@2x.png" "${ICONSET_DIR}/icon_512x512@2x.png"
    
    iconutil -c icns "$ICONSET_DIR" -o "branding/CrypRQ.icns" 2>/dev/null || echo "  ⚠️  iconutil failed, .icns not generated"
    
    if [ -f "branding/CrypRQ.icns" ]; then
        echo "  ✅ macOS .icns generated: branding/CrypRQ.icns"
        {
            echo "macOS Icons:"
            echo "  - branding/CrypRQ.icns: $(get_checksum "branding/CrypRQ.icns")"
            echo ""
        } >> "$REPORT_FILE"
    fi
else
    echo "  ⚠️  macOS icon generation skipped (iconutil not available or apple/ not found)"
fi

echo ""
echo "[4/7] Windows Icons"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -n "$MAGICK_CMD" ]; then
    mkdir -p windows/Assets
    
    # Generate multi-size ICO (16, 24, 32, 48, 64, 128, 256)
    $MAGICK_CMD "$MASTER_ICON" \
        \( -clone 0 -resize 16x16 \) \
        \( -clone 0 -resize 24x24 \) \
        \( -clone 0 -resize 32x32 \) \
        \( -clone 0 -resize 48x48 \) \
        \( -clone 0 -resize 64x64 \) \
        \( -clone 0 -resize 128x128 \) \
        \( -clone 0 -resize 256x256 \) \
        -delete 0 -alpha off -colors 256 windows/Assets/AppIcon.ico
    
    echo "  ✅ Windows .ico generated: windows/Assets/AppIcon.ico"
    {
        echo "Windows Icons:"
        echo "  - windows/Assets/AppIcon.ico: $(get_checksum "windows/Assets/AppIcon.ico")"
        echo ""
    } >> "$REPORT_FILE"
else
    echo "  ⚠️  Windows icon generation skipped (ImageMagick not available)"
fi

echo ""
echo "[5/7] Linux Desktop Icons"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
mkdir -p packaging/linux/hicolor
for size in 16 24 32 48 64 128 256 512; do
    mkdir -p "packaging/linux/hicolor/${size}x${size}/apps"
    generate_png "$size" "packaging/linux/hicolor/${size}x${size}/apps/cryprq.png"
done

# Create .DirIcon for AppImage
generate_png 512 packaging/linux/.DirIcon

echo "  ✅ Linux desktop icons generated"
{
    echo "Linux Desktop Icons:"
    for size in 16 24 32 48 64 128 256 512; do
        file="packaging/linux/hicolor/${size}x${size}/apps/cryprq.png"
        if [ -f "$file" ]; then
            echo "  - $file (${size}x${size}): $(get_checksum "$file")"
        fi
    done
    echo ""
} >> "$REPORT_FILE"

echo ""
echo "[6/7] GUI App Icons"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -d "gui" ]; then
    mkdir -p gui/build
    
    # Copy icons for Electron
    if [ -f "branding/CrypRQ.icns" ]; then
        cp branding/CrypRQ.icns gui/build/icon.icns
        echo "  ✅ macOS icon copied for Electron"
    fi
    
    if [ -f "windows/Assets/AppIcon.ico" ]; then
        cp windows/Assets/AppIcon.ico gui/build/icon.ico
        echo "  ✅ Windows icon copied for Electron"
    fi
    
    # Generate PNG for Linux Electron
    generate_png 512 gui/build/icon.png
    
    echo "  ✅ GUI icons prepared"
    {
        echo "GUI Icons:"
        [ -f "gui/build/icon.icns" ] && echo "  - gui/build/icon.icns: $(get_checksum "gui/build/icon.icns")"
        [ -f "gui/build/icon.ico" ] && echo "  - gui/build/icon.ico: $(get_checksum "gui/build/icon.ico")"
        [ -f "gui/build/icon.png" ] && echo "  - gui/build/icon.png: $(get_checksum "gui/build/icon.png")"
        echo ""
    } >> "$REPORT_FILE"
else
    echo "  ⚠️  GUI directory not found, skipping"
fi

echo ""
echo "[7/7] Docker Image Logo"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
# Copy icon for Docker README reference
mkdir -p branding
cp "$MASTER_ICON" branding/docker-logo.png
echo "  ✅ Docker logo prepared: branding/docker-logo.png"
{
    echo "Docker Logo:"
    echo "  - branding/docker-logo.png: $(get_checksum "branding/docker-logo.png")"
    echo ""
} >> "$REPORT_FILE"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Icon generation complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Report: $REPORT_FILE"
echo ""
cat "$REPORT_FILE"

