#!/usr/bin/env bash
# Builds export/flatpak/org.dupot.devtools.yml (sources: type: dir, i.e. the
# local working tree) into a standalone .flatpak bundle, for local testing /
# sharing without needing a flatpak remote set up on the target machine.
#
# The manifest lets cargo download crates during the build (--share=network
# in build-args); that is fine locally but refused by Flathub, whose
# submission must ship a generated cargo-sources.json instead.
#
# flatpak-builder isn't required natively: org.flatpak.Builder (the
# Flathub-maintained flatpak bundling it) is used instead.
#
# Usage: ./build_flatpak.sh [output.flatpak]
#   FLATPAK_SCOPE=--user ./build_flatpak.sh   install missing SDKs per-user
#                                             (default: --system)
#   FLATPAK_REPO=/other/disk/repo ./build_flatpak.sh   ostree repo location

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$repo_root"

manifest="export/flatpak/org.dupot.devtools.yml"
app_id="org.dupot.devtools"
build_dir="build-dir"
local_repo="${FLATPAK_REPO:-repo}"
output="${1:-${app_id}.flatpak}"
scope="${FLATPAK_SCOPE:---system}"

if ! command -v flatpak >/dev/null 2>&1; then
    echo "error: flatpak is not installed" >&2
    exit 2
fi

if [[ ! -f "$manifest" ]]; then
    echo "error: manifest not found: $manifest" >&2
    exit 2
fi

if [[ ! -f Cargo.lock ]]; then
    echo "error: Cargo.lock missing (the build runs cargo --locked)" >&2
    exit 2
fi

if ! flatpak info org.flatpak.Builder >/dev/null 2>&1; then
    echo "org.flatpak.Builder (flatpak-builder) not installed — installing from flathub..." >&2
    flatpak install "$scope" -y flathub org.flatpak.Builder
fi

# Keep these in sync with runtime-version / sdk-extensions in the manifest
# (the rust-stable branch is the freedesktop SDK the GNOME runtime is based on).
for ref in org.gnome.Platform//50 org.gnome.Sdk//50 org.freedesktop.Sdk.Extension.rust-stable//25.08; do
    if ! flatpak info "$ref" >/dev/null 2>&1; then
        echo "$ref not installed — installing from flathub..." >&2
        flatpak install "$scope" -y flathub "$ref"
    fi
done

echo "=== Building $app_id from $manifest ==="
flatpak run --command=flatpak-builder org.flatpak.Builder \
    --force-clean --repo="$local_repo" "$build_dir" "$manifest"

echo "=== Bundling -> $output ==="
# --runtime-repo points the bundle at Flathub for org.gnome.Platform, so
# installing it on a machine that doesn't already have that runtime pulls
# it automatically instead of failing.
flatpak build-bundle \
    --runtime-repo=https://flathub.org/repo/flathub.flatpakrepo \
    "$local_repo" "$output" "$app_id"

echo "Done: $output"
echo "Install: flatpak install --user $output"
