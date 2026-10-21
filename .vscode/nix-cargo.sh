#!/usr/bin/env bash
# Runs cargo inside the flake's dev shell (VSCode is usually not started from
# `nix develop`), then saves the shell's XDG_DATA_DIRS next to the binaries so
# launch.json can hand GtkSourceView / GSettings their data files.
# Usage: .vscode/nix-cargo.sh <cargo args…>
set -euo pipefail
cd "$(dirname "$0")/.."
nix_bin="$(command -v nix || echo /nix/var/nix/profiles/default/bin/nix)"
exec "$nix_bin" develop --command bash -c '
    cargo "$@"
    mkdir -p target
    printf "XDG_DATA_DIRS=%s\n" "$XDG_DATA_DIRS" > target/vscode.env
' nix-cargo "$@"
