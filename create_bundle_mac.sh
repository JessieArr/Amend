#!/bin/bash

# Create macOS app bundle for Amend Text Editor
# Run this script after building with: cargo build --release

set -e

# Configuration
APP_NAME="Amend"
BUNDLE_NAME="${APP_NAME}.app"
EXE_NAME="amend-editor"
TARGET_DIR="target/release"
BUNDLE_PATH="${TARGET_DIR}/${BUNDLE_NAME}"

echo "Creating macOS app bundle: ${BUNDLE_PATH}"

# Clean up existing bundle
if [ -d "${BUNDLE_PATH}" ]; then
    rm -rf "${BUNDLE_PATH}"
fi

# Create directory structure
mkdir -p "${BUNDLE_PATH}/Contents/MacOS"
mkdir -p "${BUNDLE_PATH}/Contents/Resources"

# Copy executable
cp "${TARGET_DIR}/${EXE_NAME}" "${BUNDLE_PATH}/Contents/MacOS/${EXE_NAME}"

# Copy Info.plist
cp "Info.plist" "${BUNDLE_PATH}/Contents/"

# Copy icon if it exists
if [ -f "assets/icon.png" ]; then
    cp "assets/icon.png" "${BUNDLE_PATH}/Contents/Resources/icon.icns"
    echo "Icon copied (PNG format - consider converting to ICNS for production)"
fi

# Make executable executable (in case permissions got messed up)
chmod +x "${BUNDLE_PATH}/Contents/MacOS/${EXE_NAME}"

echo "App bundle created successfully at: ${BUNDLE_PATH}"
echo ""
echo "To test the bundle:"
echo "  open ${BUNDLE_PATH}"
echo ""
echo "To install system-wide (optional):"
echo "  cp -r ${BUNDLE_PATH} /Applications/"
echo ""
echo "Note: The app will now appear in 'Open With' menus for text files" 