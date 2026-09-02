#!/bin/bash

# Helper script to run GPUI photo grid
# Automatically finds library ID and runs the app

# Get library ID from the library.json file
LIBRARY_PATH="$HOME/WingDrive/Libraries"
if [ ! -d "$LIBRARY_PATH" ] && [ -d "$HOME/Spacedrive/Libraries" ]; then
    LIBRARY_PATH="$HOME/Spacedrive/Libraries"
fi
LIBRARY_FILE=$(find "$LIBRARY_PATH" -name "*.sdlibrary" -type d | head -n 1)

if [ -z "$LIBRARY_FILE" ]; then
    echo "Error: No WingDrive library found in $LIBRARY_PATH"
    exit 1
fi

LIBRARY_JSON="$LIBRARY_FILE/library.json"
if [ ! -f "$LIBRARY_JSON" ]; then
    echo "Error: library.json not found at $LIBRARY_JSON"
    exit 1
fi

# Extract library UUID from JSON
if command -v jq &> /dev/null; then
    LIBRARY_ID=$(jq -r '.id' "$LIBRARY_JSON")
else
    # Fallback: extract UUID with grep
    LIBRARY_ID=$(grep -o '"id"\s*:\s*"[^"]*"' "$LIBRARY_JSON" | sed 's/.*"\([^"]*\)"$/\1/')
fi

if [ -z "$LIBRARY_ID" ]; then
    echo "Error: Could not extract library ID from $LIBRARY_JSON"
    exit 1
fi

echo "═══════════════════════════════════════════════════════"
echo "  GPUI Photo Grid for WingDrive"
echo "═══════════════════════════════════════════════════════"
echo ""
echo "Library ID: $LIBRARY_ID"
echo "Library:    $LIBRARY_FILE"
echo ""

export SD_LIBRARY_ID="$LIBRARY_ID"
export SD_SOCKET_ADDR="${SD_SOCKET_ADDR:-127.0.0.1:6969}"
export SD_HTTP_URL="${SD_HTTP_URL:-http://127.0.0.1:58304}"
export SD_INITIAL_PATH="${1:-$HOME/Downloads}"

echo "Starting GPUI Photo Grid..."
echo ""

cargo run --release
