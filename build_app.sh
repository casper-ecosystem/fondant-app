#!/bin/bash

APP_NAME="Fondant"

# Remove existing app and distribution folder if they exist
rm -rf "${APP_NAME}.app" "${APP_NAME}.dmg"

# Create the app bundle structure
mkdir -p "${APP_NAME}.app/Contents/MacOS"
mkdir -p "${APP_NAME}.app/Contents/Resources"

# Copy application files into the app bundle
cp -R ./* "${APP_NAME}.app/Contents/Resources"
cp docker-compose.yml "${APP_NAME}.app/Contents/MacOS/"


# Create the start script within the app bundle
cat > "${APP_NAME}.app/Contents/MacOS/start_app.sh" << 'EOF'
#!/bin/bash

LOG_FILE="/tmp/fondant_app_log.txt"
exec > "$LOG_FILE" 2>&1

set -x

export PATH="/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"

PROJECT_DIR="${HOME}/Documents/Fondant"

if [ ! -d "$PROJECT_DIR" ]; then
    echo "Copying project files to $PROJECT_DIR..."
    mkdir -p "$PROJECT_DIR"
    cp -R "$(dirname "$0")/../Resources/." "$PROJECT_DIR/"
fi

cleanup() {
    echo "Stopping Docker containers..."
    docker-compose down
    exit 0
}

trap cleanup SIGINT SIGTERM SIGQUIT TERM

if ! docker info >/dev/null 2>&1; then
    echo "Docker Desktop is not running. Please start Docker Desktop and try again."
    exit 1
fi

cd "$PROJECT_DIR" || { echo "Failed to change directory to $PROJECT_DIR"; exit 1; }

docker-compose down

docker-compose up -d

caffeinate -i -w $$ &

wait
EOF

chmod +x "${APP_NAME}.app/Contents/MacOS/start_app.sh"

APP_NAME_LOWER=$(echo "$APP_NAME" | tr '[:upper:]' '[:lower:]')

# Create Info.plist file
cat > "${APP_NAME}.app/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
 <dict>
   <key>CFBundleExecutable</key>
   <string>start_app.sh</string>
   <key>CFBundleIdentifier</key>
   <string>network.casper.${APP_NAME_LOWER}</string>
   <key>CFBundleName</key>
   <string>${APP_NAME}</string>
   <key>CFBundleVersion</key>
   <string>1.0</string>
   <key>CFBundlePackageType</key>
   <string>APPL</string>
 </dict>
</plist>
EOF

# ICONSET_DIR="/tmp/logo.iconset"
# PNG_FILE="setup/assets/logo.png"
# ICNS_FILE="${APP_NAME}.app/Contents/Resources/logo.icns"

# sed -i '' "/<key>CFBundlePackageType<\/key>/a\\
#   <key>CFBundleIconFile<\/key>\\
#   <string>logo.icns<\/string>\\
# " "${APP_NAME}.app/Contents/Info.plist"

# Create the disk image
hdiutil create -volname "${APP_NAME}" -srcfolder "${APP_NAME}.app" -ov -format UDZO "${APP_NAME}.dmg"

rm -rf "${APP_NAME}.app"

echo "Disk image ${APP_NAME}.dmg has been created."
