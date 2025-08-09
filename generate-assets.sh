#!/bin/sh

# Assume that npm is present and npm install has been invoked
# Assume ImageMagick is present


# Find the source logo file
SOURCE_LOGO=$(find . -name "crypto7world-logo-*.png" | head -n 1)

if [ -z "$SOURCE_LOGO" ]; then
    echo "Error: No crypto7world-logo-*.png file found"
    exit 1
fi

echo "Using source file: $SOURCE_LOGO"

# Create assets directory if it doesn't exist
echo "Cleaning-up assets and icons..."
rm -rf assets 2> /dev/null
rm -rf icons 2> /dev/null
mkdir -p assets
mkdir -p icons

echo "Generating Tailwind CSS"
npx -y @tailwindcss/cli -m -i input.css -o assets/tailwind.css

echo "Generating Icons"
npx -y @tauri-apps/cli icon -o icons/ $SOURCE_LOGO

# Uses convert instead of magick because Github Actions Ubuntu
# still uses an old version of the package
echo "Generating favicon.ico..."
convert -quiet "$SOURCE_LOGO" \( -clone 0 -resize 16x16 \) \( -clone 0 -resize 32x32 \) \( -clone 0 -resize 64x64 \) -delete 0 assets/favicon.ico

echo "Generating in-app Logo..."
convert -quiet "$SOURCE_LOGO" -resize 256x256 assets/crypto7world-logo.png

echo "Generating app window icon..."
convert -quiet "$SOURCE_LOGO" -resize 256x256 assets/crypto7world-logo.rgba
