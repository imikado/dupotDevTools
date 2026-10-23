#!/usr/bin/env bash
# Cross-compiles dupotDevTools for Windows (x86_64) from Linux and packages a
# self-contained folder + zip: no Windows machine needed.
#
# - Toolchain: Rust with the x86_64-pc-windows-gnu target and MinGW-w64,
#   provided by `nix develop .#windows` (the script re-enters it on its own).
# - GTK4 / libadwaita / GtkSourceView: prebuilt Windows packages downloaded
#   from the MSYS2 "ucrt64" repository (the one MSYS2 and gtk-rs recommend),
#   dependencies resolved from the repository index.
#
# Bundle layout (GTK finds its data relative to the folder holding its DLLs):
#   DupotDevTools/
#     bin/    dupot-dev-tools.exe + every DLL it needs
#     lib/    gdk-pixbuf loaders + their cache (generated with Wine)
#     share/  GSettings schemas, Adwaita + hicolor icons, GtkSourceView data
#
# Usage: ./build_windows_cross.sh [--refresh] [--fast]
#   --refresh   re-download the MSYS2 index and packages (else cached)
#   --fast      test build: `release-fast` profile (no fat LTO, incremental),
#               much quicker to rebuild but a larger, slightly slower exe

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$repo_root"

target="x86_64-pc-windows-gnu"
if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
    nix_bin="$(command -v nix || echo /nix/var/nix/profiles/default/bin/nix)"
    if [[ ! -x "$nix_bin" ]]; then
        echo "error: needs Nix (for the cross toolchain), or run it from a shell" >&2
        echo "       providing rustc with the $target target and MinGW-w64." >&2
        exit 2
    fi
    echo "=== Entering the Windows cross-compilation shell (nix develop .#windows) ===" >&2
    exec "$nix_bin" develop "$repo_root#windows" --command "$0" "$@"
fi

app_name="DupotDevTools"
exe_name="dupot-dev-tools.exe"
version="$(grep -m1 -oP '^version = "\K[^"]+' Cargo.toml)"
work_dir="build-windows-cross"
cache_dir="$work_dir/cache"
sysroot="$work_dir/sysroot"
prefix="$sysroot/ucrt64"
bundle="$work_dir/$app_name"
dist_dir="export/windows"
mirror="https://repo.msys2.org/mingw/ucrt64"

# Build date shown in the About dialog.
build_date="$(date -u +%F)"
export DUPOT_BUILD_PACKAGE=windows DUPOT_BUILD_DATE="$build_date"

# Runtime packages; their dependencies are added automatically.
roots=(
    mingw-w64-ucrt-x86_64-gtk4
    mingw-w64-ucrt-x86_64-libadwaita
    mingw-w64-ucrt-x86_64-gtksourceview5
    mingw-w64-ucrt-x86_64-adwaita-icon-theme
    mingw-w64-ucrt-x86_64-hicolor-icon-theme
    mingw-w64-ucrt-x86_64-librsvg
    # Rust's windows-gnu std links libpthread.a statically; it comes with
    # the MSYS2 C runtime used for linking (see RUSTFLAGS below).
    mingw-w64-ucrt-x86_64-winpthreads
    mingw-w64-ucrt-x86_64-crt
)

profile="release"
for arg in "$@"; do
    case "$arg" in
        --refresh) rm -rf "$cache_dir" ;;
        --fast) profile="release-fast" ;;
        *) echo "error: unknown option $arg" >&2; exit 2 ;;
    esac
done
mkdir -p "$cache_dir"

echo "=== Resolving MSYS2 packages ==="
if [[ ! -f "$cache_dir/ucrt64.db" ]]; then
    curl -fsSL "$mirror/ucrt64.db" -o "$cache_dir/ucrt64.db"
fi
# The index is a zstd tarball of <pkg>/desc files; walk DEPENDS from the roots.
zstd -dcq "$cache_dir/ucrt64.db" | python3 -c '
import re, sys, tarfile

entries, provides = {}, {}
with tarfile.open(fileobj=sys.stdin.buffer, mode="r|") as tar:
    for member in tar:
        if not member.name.endswith("/desc"):
            continue
        fields, key = {}, None
        for line in tar.extractfile(member).read().decode().splitlines():
            if line.startswith("%") and line.endswith("%"):
                key = line.strip("%")
                fields[key] = []
            elif line and key:
                fields[key].append(line)
        name = fields["NAME"][0]
        entries[name] = fields
        for p in fields.get("PROVIDES", []):
            provides.setdefault(re.split("[<>=]", p)[0], name)

seen, stack = set(), list(sys.argv[1:])
while stack:
    name = re.split("[<>=]", stack.pop())[0]
    name = name if name in entries else provides.get(name, name)
    if name in seen:
        continue
    if name not in entries:
        sys.exit(f"unknown MSYS2 package: {name}")
    seen.add(name)
    stack.extend(entries[name].get("DEPENDS", []))
for name in sorted(seen):
    print(entries[name]["FILENAME"][0])
' "${roots[@]}" > "$cache_dir/packages.txt"
mapfile -t packages < "$cache_dir/packages.txt"
echo "${#packages[@]} packages"

# The sysroot is kept between builds and only re-extracted when the package
# list changes (new versions after --refresh, or new roots).
sysroot_stamp="$sysroot/.packages"
if cmp -s "$cache_dir/packages.txt" "$sysroot_stamp"; then
    echo "=== Reusing $sysroot (same packages) ==="
else
    echo "=== Downloading and extracting into $sysroot ==="
    rm -rf "$sysroot"
    mkdir -p "$sysroot"
    for file in "${packages[@]}"; do
        if [[ ! -f "$cache_dir/$file" ]]; then
            curl -fsSL "$mirror/$file" -o "$cache_dir/$file.part"
            mv "$cache_dir/$file.part" "$cache_dir/$file"
        fi
        zstd -dcq "$cache_dir/$file" | tar -x -C "$sysroot" --exclude='.*'
    done
    cp "$cache_dir/packages.txt" "$sysroot_stamp"
fi

echo "=== Cross-compiling ($target) ==="
# The .pc files say prefix=/ucrt64; the sysroot makes -I/-L point inside it.
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_SYSROOT_DIR="$repo_root/$sysroot"
export PKG_CONFIG_LIBDIR="$repo_root/$prefix/lib/pkgconfig:$repo_root/$prefix/share/pkgconfig"
export PKG_CONFIG_PATH=""
# Link against the MSYS2 C runtime (crt2.o, libmingw32/ex, libpthread) rather
# than the Nix MinGW one, so it matches what the GTK DLLs were built with.
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS="-C link-arg=-B$repo_root/$prefix/lib"
echo "profile: $profile"
# Lowest CPU priority: the machine stays usable during the build.
nice -n 19 cargo build --profile "$profile" --locked --target "$target"

echo "=== Assembling bundle in $bundle ==="
rm -rf "$bundle"
mkdir -p "$bundle/bin" "$bundle/lib" "$bundle/share/glib-2.0/schemas" "$bundle/share/icons"
cp "target/$target/$profile/$exe_name" "$bundle/bin/"
cp -r "$prefix/lib/gdk-pixbuf-2.0" "$bundle/lib/"

# DLLs come from MSYS2 only (the exe links the MSYS2 C runtime statically).
dll_dirs=("$prefix/bin")

# Copies the DLLs imported by the given binaries, recursively. System DLLs
# (kernel32, user32…) are not found in dll_dirs and are left out.
copy_dlls() {
    local queue=("$@") file dll dir
    while ((${#queue[@]})); do
        file="${queue[0]}"
        queue=("${queue[@]:1}")
        while read -r dll; do
            [[ -e "$bundle/bin/$dll" ]] && continue
            for dir in "${dll_dirs[@]}"; do
                if [[ -f "$dir/$dll" ]]; then
                    cp "$dir/$dll" "$bundle/bin/"
                    queue+=("$bundle/bin/$dll")
                    break
                fi
            done
        done < <(x86_64-w64-mingw32-objdump -p "$file" | awk '/DLL Name:/ { print $3 }')
    done
}
mapfile -t loaders < <(find "$bundle/lib/gdk-pixbuf-2.0" -name '*.dll')
copy_dlls "$bundle/bin/$exe_name" "${loaders[@]}"

# gdk-pixbuf loader cache, normally written by a pacman hook on install:
# generated by running the Windows gdk-pixbuf-query-loaders.exe under Wine
# from inside the bundle, so module paths are relative to it.
find "$bundle/lib" -name '*.dll.a' -delete
loaders_dir="$(find "$bundle/lib/gdk-pixbuf-2.0" -type d -name loaders | head -1)"
export WINEPREFIX="$repo_root/$work_dir/wineprefix" WINEDEBUG=-all WINEDLLOVERRIDES="mscoree,mshtml="
cp "$prefix/bin/gdk-pixbuf-query-loaders.exe" "$bundle/bin/"
wine "$bundle/bin/gdk-pixbuf-query-loaders.exe" | tr -d '\r' > "$(dirname "$loaders_dir")/loaders.cache"
rm "$bundle/bin/gdk-pixbuf-query-loaders.exe"
if ! grep -q 'libpixbufloader-png.dll' "$(dirname "$loaders_dir")/loaders.cache"; then
    echo "error: gdk-pixbuf loader cache generation failed" >&2
    exit 1
fi

# GSettings schemas (GTK aborts on some widgets without them).
cp "$prefix"/share/glib-2.0/schemas/*.xml "$bundle/share/glib-2.0/schemas/"
glib-compile-schemas "$bundle/share/glib-2.0/schemas"

# Icon themes and GtkSourceView syntax definitions / style schemes.
cp -r "$prefix/share/icons/Adwaita" "$prefix/share/icons/hicolor" "$bundle/share/icons/"
for size in 16 24 32 48 64 128 256 512; do
    install -Dm644 "export/flatpak/${size}x${size}.png" \
        "$bundle/share/icons/hicolor/${size}x${size}/apps/org.dupot.devtools.png"
done
for theme in Adwaita hicolor; do
    gtk4-update-icon-cache -q -t -f "$bundle/share/icons/$theme" || true
done
cp -r "$prefix/share/gtksourceview-5" "$bundle/share/"

echo "=== Packaging ==="
mkdir -p "$dist_dir"
archive="$dist_dir/${app_name}-${version}-windows-x64.zip"
rm -f "$archive"
(cd "$work_dir" && zip -qr "$repo_root/$archive" "$app_name")

echo "Done: $archive ($(du -h "$archive" | cut -f1))"
echo "Bundle: $bundle/bin/$exe_name ($(find "$bundle/bin" -name '*.dll' | wc -l) DLLs)"
echo "Try it: nix develop .#windows -c wine $bundle/bin/$exe_name"
