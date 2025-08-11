#!/bin/bash

# Build and bundle script for Amend Text Editor
# This script builds the app and creates a proper macOS app bundle

set -e

echo "🚀 Building Amend Text Editor..."

# Build the release version
cargo build --release

echo "✅ Build completed successfully!"

# Create the app bundle
echo "📦 Creating macOS app bundle..."
./create_bundle_mac.sh

echo ""
echo "🎉 All done! Your app bundle is ready at: target/release/Amend.app"
echo ""
echo "Next steps:"
echo "1. Test the app: open target/release/Amend.app"
echo "2. Test file opening: target/release/Amend.app/Contents/MacOS/amend-editor test.txt"
echo "3. Install system-wide (optional): cp -r target/release/Amend.app /Applications/"
echo ""
echo "The app will now appear in 'Open With' menus for text files!"
echo "You can right-click any text file → 'Open With' → 'Amend'" 