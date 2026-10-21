#!/usr/bin/env bash
# Builds the distributable packages of dupotDevTools, all from Linux:
#   flatpak   -> export/flatpak/DupotDevTools-<version>.flatpak   (build_flatpak.sh)
#   appimage  -> export/appimage/DupotDevTools-<version>-x86_64.AppImage   (build_appimage.sh, Docker)
#   windows   -> export/windows/DupotDevTools-<version>-windows-x64.zip   (build_windows_cross.sh, Nix)
#
# Usage: ./build_all.sh [flatpak] [appimage] [windows]   (default: all three)
# Targets run one after the other; a failing target does not stop the others.

set -uo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$repo_root"

version="$(grep -m1 -oP '^version = "\K[^"]+' Cargo.toml)"
targets=("$@")
((${#targets[@]})) || targets=(flatpak appimage windows)

declare -A outputs=(
    [flatpak]="export/flatpak/DupotDevTools-${version}.flatpak"
    [appimage]="export/appimage/DupotDevTools-${version}-x86_64.AppImage"
    [windows]="export/windows/DupotDevTools-${version}-windows-x64.zip"
)
declare -A results

for target in "${targets[@]}"; do
    case "$target" in
        flatpak) cmd=(./build_flatpak.sh "${outputs[flatpak]}") ;;
        appimage) cmd=(./build_appimage.sh "${outputs[appimage]}") ;;
        windows) cmd=(./build_windows_cross.sh) ;;
        *)
            echo "error: unknown target '$target' (flatpak, appimage, windows)" >&2
            exit 2
            ;;
    esac
    echo
    echo "################ $target ################"
    start=$SECONDS
    if "${cmd[@]}"; then
        results[$target]="OK      ${outputs[$target]} ($(du -h "${outputs[$target]}" | cut -f1), $((SECONDS - start))s)"
    else
        results[$target]="FAILED  (see the log above)"
    fi
done

echo
echo "################ Summary (v$version) ################"
status=0
for target in "${targets[@]}"; do
    printf '%-9s %s\n' "$target" "${results[$target]}"
    [[ "${results[$target]}" == OK* ]] || status=1
done
exit $status
