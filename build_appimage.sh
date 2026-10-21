#!/usr/bin/env bash
# Builds an x86_64 AppImage of dupotDevTools.
#
# The build runs in an Ubuntu 24.04 container (export/appimage/Dockerfile):
# its GTK 4.14 / libadwaita 1.5 are the app's minimum versions and its glibc
# 2.39 keeps the AppImage runnable on Ubuntu 24.04+, Debian 13+, Fedora 40+…
# linuxdeploy + linuxdeploy-plugin-gtk bundle the libraries and set GTK's
# environment (schemas, pixbuf loaders, XDG_DATA_DIRS) at startup.
#
# Usage: ./build_appimage.sh [output.AppImage]
# Needs Docker (user in the `docker` group).

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$repo_root"

app_id="org.dupot.devtools"
version="$(grep -m1 -oP '^version = "\K[^"]+' Cargo.toml)"
image="dupot-devtools-appimage-builder"
output="${1:-export/appimage/DupotDevTools-${version}-x86_64.AppImage}"

# Build date shown in the About dialog.
build_date="$(date -u +%F)"

if ! command -v docker >/dev/null 2>&1; then
    echo "error: docker is not installed" >&2
    exit 2
fi

echo "=== Preparing the build image ($image) ==="
docker build -q -t "$image" export/appimage >/dev/null

echo "=== Building in Ubuntu 24.04 ==="
mkdir -p "$(dirname "$output")"
# Cargo registry and target dir live in named volumes: fast rebuilds, and the
# host's target/ (built against another glibc) is left alone.
docker run --rm \
    -v "$repo_root:/src:ro" \
    -v "$(cd "$(dirname "$output")" && pwd):/out" \
    -v dupot-devtools-appimage-cargo:/root/.cargo/registry \
    -v dupot-devtools-appimage-target:/target \
    -e CARGO_TARGET_DIR=/target \
    -e DUPOT_BUILD_PACKAGE=appimage \
    -e DUPOT_BUILD_DATE="$build_date" \
    -e APP_ID="$app_id" -e OUT_NAME="$(basename "$output")" \
    -e HOST_UID="$(id -u)" -e HOST_GID="$(id -g)" \
    "$image" bash -euo pipefail -c '
cd /src
cargo build --release --locked

appdir=/tmp/AppDir
rm -rf "$appdir"
install -Dm755 /target/release/dupot-dev-tools "$appdir/usr/bin/dupot-dev-tools"
install -Dm644 export/flatpak/$APP_ID.desktop "$appdir/usr/share/applications/$APP_ID.desktop"
install -Dm644 export/flatpak/$APP_ID.appdata.xml "$appdir/usr/share/metainfo/$APP_ID.appdata.xml"
for size in 16 24 32 48 64 128 256 512; do
    install -Dm644 export/flatpak/${size}x${size}.png \
        "$appdir/usr/share/icons/hicolor/${size}x${size}/apps/$APP_ID.png"
done
# Data the libraries look up at runtime: symbolic icons, syntax definitions.
cp -r /usr/share/icons/Adwaita "$appdir/usr/share/icons/"
cp -r /usr/share/gtksourceview-5 "$appdir/usr/share/"

cd /tmp
export DEPLOY_GTK_VERSION=4 LINUXDEPLOY_OUTPUT_VERSION='"$version"'
linuxdeploy --appdir "$appdir" --plugin gtk \
    --desktop-file "$appdir/usr/share/applications/$APP_ID.desktop" \
    --icon-file "$appdir/usr/share/icons/hicolor/256x256/apps/$APP_ID.png"
# The GTK plugin forces GTK_THEME, which libadwaita does not support (old-style
# widgets); and the host GIO modules (gvfs) are built against a newer GLib.
hook="$appdir/apprun-hooks/linuxdeploy-plugin-gtk.sh"
sed -i "/GTK_THEME/d" "$hook"
echo "export GIO_MODULE_DIR=\"\$APPDIR/usr/lib/gio/modules\"" >> "$hook"
linuxdeploy --appdir "$appdir" --output appimage
mv /tmp/*.AppImage "/out/$OUT_NAME"
chown "$HOST_UID:$HOST_GID" "/out/$OUT_NAME"
'

echo "Done: $output ($(du -h "$output" | cut -f1))"
echo "Run:  chmod +x \"$output\" && \"$output\""
