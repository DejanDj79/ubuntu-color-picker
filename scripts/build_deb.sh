#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PACKAGE="ubuntu-color-picker"
APP_ID="com.example.UbuntuColorPicker"
APP_NAME="Ubuntu Color Picker"
VERSION="$(grep -m1 '^version = ' "$ROOT_DIR/Cargo.toml" | sed -E 's/version = "([^"]+)"/\1/')"
ARCH="$(dpkg --print-architecture)"
BUILD_ROOT="$ROOT_DIR/target/deb/${PACKAGE}_${VERSION}_${ARCH}"
DEB_PATH="$ROOT_DIR/dist/${PACKAGE}_${VERSION}_${ARCH}.deb"

cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml"

rm -rf "$BUILD_ROOT"
mkdir -p "$BUILD_ROOT/DEBIAN"
install -Dm755 "$ROOT_DIR/target/release/$PACKAGE" "$BUILD_ROOT/usr/bin/$PACKAGE"
strip --strip-unneeded "$BUILD_ROOT/usr/bin/$PACKAGE" || true

install -Dm644 "$ROOT_DIR/picker-icon.svg" \
    "$BUILD_ROOT/usr/share/icons/hicolor/scalable/apps/picker-icon.svg"

install -d "$BUILD_ROOT/usr/share/applications"
cat > "$BUILD_ROOT/usr/share/applications/${APP_ID}.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=${APP_NAME}
Comment=Pick, copy, save, and explore colors
Exec=${PACKAGE}
Icon=picker-icon
Terminal=false
Categories=Graphics;GTK;
StartupNotify=true
DESKTOP

SHLIBDEPS="$(mktemp -d)"
mkdir -p "$SHLIBDEPS/debian"
cat > "$SHLIBDEPS/debian/control" <<CONTROL
Source: ${PACKAGE}
Section: graphics
Priority: optional
Maintainer: DejanDj79 <deki4679@gmail.com>
Standards-Version: 4.6.2

Package: ${PACKAGE}
Architecture: any
Depends: \${shlibs:Depends}
Description: ${APP_NAME}
 Pick, copy, save, and explore colors.
CONTROL

DEPENDS="$(
    cd "$SHLIBDEPS"
    dpkg-shlibdeps -O -e"$BUILD_ROOT/usr/bin/$PACKAGE" \
        | sed -n 's/^shlibs:Depends=//p'
)"
rm -rf "$SHLIBDEPS"

if [[ -z "$DEPENDS" ]]; then
    DEPENDS="libc6, libadwaita-1-0, libgtk-4-1"
fi

cat > "$BUILD_ROOT/DEBIAN/control" <<CONTROL
Package: ${PACKAGE}
Version: ${VERSION}
Section: graphics
Priority: optional
Architecture: ${ARCH}
Maintainer: DejanDj79 <deki4679@gmail.com>
Depends: ${DEPENDS}
Description: ${APP_NAME}
 Pick, copy, save, and explore colors.
CONTROL

find "$BUILD_ROOT" -type d -exec chmod 755 {} +
find "$BUILD_ROOT" -type f -exec chmod 644 {} +
chmod 755 "$BUILD_ROOT/usr/bin/$PACKAGE"
mkdir -p "$ROOT_DIR/dist"
dpkg-deb --root-owner-group --build "$BUILD_ROOT" "$DEB_PATH"

echo "$DEB_PATH"
