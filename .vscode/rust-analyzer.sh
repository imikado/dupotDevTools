#!/usr/bin/env bash
# Starts rust-analyzer from the flake's dev shell so it finds cargo, rustc and
# the GTK libraries through pkg-config.
cd "$(dirname "$0")/.."
nix_bin="$(command -v nix || echo /nix/var/nix/profiles/default/bin/nix)"
exec "$nix_bin" develop --command rust-analyzer "$@"
