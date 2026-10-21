#!/usr/bin/env bash
# Builds a standalone Windows bundle (zip) of dupotDevTools. GTK4 /
# libadwaita / GtkSourceView have no usable cross-compile path from Linux,
# so this must run on Windows inside an MSYS2 UCRT64 shell
# (https://www.msys2.org), or on a GitHub Actions windows runner using
# msys2/setup-msys2 with msystem: UCRT64.
#
# Bundle layout (GTK finds its data relative to the directory holding its
# DLLs, so share/ and lib/ must sit next to bin/):
#   DupotDevTools/
#     bin/    dupot-dev-tools.exe + every DLL it (and the loaders) need
#     lib/    gdk-pixbuf loaders (PNG / SVG icons)
#     share/  GSettings schemas, Adwaita + hicolor icons, GtkSourceView
#             language specs and style schemes
#
# Usage: ./build_windows.sh [--no-deps]

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$repo_root"

app_name="DupotDevTools"
exe_name="dupot-dev-tools.exe"
version="$(grep -m1 -oP '^version = "\K[^"]+' Cargo.toml)"
version="${version:-dev}"
build_dir="build-windows"
bundle="$build_dir/$app_name"
dist_dir="export/windows"

# Build date shown in the About dialog.
build_date="$(date -u +%F)"
export DUPOT_BUILD_PACKAGE=windows DUPOT_BUILD_DATE="$build_date"

if [[ "${MSYSTEM:-}" != "UCRT64" ]]; then
    echo "error: run this from an MSYS2 UCRT64 shell (MSYSTEM=${MSYSTEM:-unset})" >&2
    exit 2
fi
prefix="${MINGW_PREFIX:-/ucrt64}"

if [[ "${1:-}" != "--no-deps" ]]; then
    echo "=== Installing dependencies (pacman) ==="
    p="mingw-w64-ucrt-x86_64"
    pacman -S --needed --noconfirm \
        "$p-rust" "$p-gcc" "$p-binutils" "$p-pkgconf" \
        "$p-gtk4" "$p-libadwaita" "$p-gtksourceview5" \
        "$p-adwaita-icon-theme" "$p-hicolor-icon-theme" "$p-librsvg" \
        zip
fi

echo "=== Building (cargo --release) ==="
# build.rs embeds src/assets/app.ico into the .exe (winresource + windres);
# main.rs sets windows_subsystem = "windows" in release, so no console.
cargo build --release --locked

echo "=== Assembling bundle in $bundle ==="
rm -rf "$bundle"
mkdir -p "$bundle/bin" "$bundle/lib" "$bundle/share/glib-2.0/schemas" "$bundle/share/icons"

cp "target/release/$exe_name" "$bundle/bin/"

# gdk-pixbuf loaders + their cache (paths in it are relative to the prefix).
cp -r "$prefix/lib/gdk-pixbuf-2.0" "$bundle/lib/"

# Copies every MSYS2 DLL that the given binaries depend on (ldd lists the
# transitive closure). System DLLs from C:/Windows are left out.
copy_dlls() {
    ldd "$@" 2>/dev/null \
        | awk -v pfx="$prefix/" 'index($3, pfx) == 1 { print $3 }' \
        | sort -u \
        | while read -r dll; do
            [[ -e "$bundle/bin/${dll##*/}" ]] || cp "$dll" "$bundle/bin/"
        done
}
copy_dlls "$bundle/bin/$exe_name"
mapfile -t loaders < <(find "$bundle/lib/gdk-pixbuf-2.0" -name '*.dll')
if ((${#loaders[@]})); then
    copy_dlls "${loaders[@]}"
fi

# GSettings schemas (GTK aborts on some widgets without them).
cp "$prefix"/share/glib-2.0/schemas/*.xml "$bundle/share/glib-2.0/schemas/" 2>/dev/null || true
glib-compile-schemas "$bundle/share/glib-2.0/schemas"

# Icon themes: symbolic icons used throughout the UI come from Adwaita.
cp -r "$prefix/share/icons/Adwaita" "$prefix/share/icons/hicolor" "$bundle/share/icons/"
for size in 16 24 32 48 64 128 256 512; do
    install -Dm644 "export/flatpak/${size}x${size}.png" \
        "$bundle/share/icons/hicolor/${size}x${size}/apps/org.dupot.devtools.png"
done
for theme in Adwaita hicolor; do
    gtk4-update-icon-cache -q -t -f "$bundle/share/icons/$theme" 2>/dev/null || true
done

# GtkSourceView: syntax highlighting definitions and the Adwaita schemes.
cp -r "$prefix/share/gtksourceview-5" "$bundle/share/"

echo "=== Packaging ==="
mkdir -p "$dist_dir"
archive="$dist_dir/${app_name}-${version}-windows-x64.zip"
rm -f "$archive"
(cd "$build_dir" && zip -qr "$repo_root/$archive" "$app_name")

echo "Done: $archive"
echo "Run: $bundle/bin/$exe_name"
