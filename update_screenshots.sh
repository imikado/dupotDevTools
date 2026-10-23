#!/usr/bin/env bash
# Regenerates the README screenshots (export/screenshots/*.png) and the demo
# GIF (export/screenshots/promo.gif) from the current code.
#
# The app runs its scripted demo (src/infrastructure/ui/demo.rs, enabled by
# DUPOT_DEMO_DIR) inside a headless sway at a fixed size, with fixed fonts
# and an empty profile; it prints what to capture on stdout:
#   DUPOT_SHOT <name>          -> grim screenshot to <name>.png
#   DUPOT_RECORD start|stop    -> wf-recorder video, turned into the GIF
#   DUPOT_DONE                 -> end
# Nothing shows on screen, the machine stays usable meanwhile.
#
# - Tools: sway, grim, wf-recorder, ffmpeg, sqlite3, dbus, provided by
#   `nix develop .#screenshots` (the script re-enters it on its own).
# - Data: examples/sqlite/library.sql, loaded into a fresh SQLite file.
#
# Usage: ./update_screenshots.sh [--keep]
#   --keep   keep the work directory (video, raw captures, logs)

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$repo_root"

if ! command -v sway >/dev/null 2>&1 || ! command -v wf-recorder >/dev/null 2>&1; then
    nix_bin="$(command -v nix || echo /nix/var/nix/profiles/default/bin/nix)"
    if [[ ! -x "$nix_bin" ]]; then
        echo "error: needs Nix, or a shell providing sway, grim, wf-recorder," >&2
        echo "       ffmpeg, sqlite3 and dbus-run-session." >&2
        exit 2
    fi
    echo "=== Entering the screenshot shell (nix develop .#screenshots) ===" >&2
    exec "$nix_bin" develop "$repo_root#screenshots" --command "$0" "$@"
fi

keep=false
for arg in "$@"; do
    case "$arg" in
        --keep) keep=true ;;
        *) echo "error: unknown option $arg" >&2; exit 2 ;;
    esac
done

# Window size of the screenshots; the GIF is scaled down to gif_width.
width=1600
height=1000
gif_width=1200
gif_fps=12
dist_dir="export/screenshots"

work="$(mktemp -d -t dupot-screenshots.XXXXXX)"
shots="$work/shots"
mkdir -p "$shots" "$work/runtime" "$work/home"
chmod 700 "$work/runtime"

sway_pid=""
app_pid=""
rec_pid=""
cleanup() {
    [[ -n "$rec_pid" ]] && kill "$rec_pid" 2>/dev/null
    [[ -n "$app_pid" ]] && kill -- -"$app_pid" 2>/dev/null
    [[ -n "$sway_pid" ]] && kill "$sway_pid" 2>/dev/null
    wait 2>/dev/null || true
    if $keep; then
        echo "Work directory kept: $work"
    else
        rm -rf "$work"
    fi
}
trap cleanup EXIT

echo "=== Building ==="
cargo build --quiet
app_bin="$repo_root/target/debug/dupot-dev-tools"

echo "=== Demo database ==="
# Opened by a relative path (the app runs from $work): short in the dialog.
sqlite3 "$work/library.db" < examples/sqlite/library.sql

echo "=== Starting headless sway (${width}x${height}) ==="
cat > "$work/sway.conf" <<EOF
output HEADLESS-1 resolution ${width}x${height} position 0 0 scale 1
default_border none
default_floating_border none
seat * hide_cursor 1
xwayland disable
EOF
# An empty profile: no saved connections, light theme, no portal.
export XDG_RUNTIME_DIR="$work/runtime"
export HOME="$work/home"
export XDG_CONFIG_HOME="$work/home/.config"
export XDG_STATE_HOME="$work/home/.local/state"
export XDG_DATA_HOME="$work/home/.local/share"
export XDG_CACHE_HOME="$work/home/.cache"
unset WAYLAND_DISPLAY DISPLAY
WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1 \
    sway -c "$work/sway.conf" >"$work/sway.log" 2>&1 &
sway_pid=$!

socket=""
for _ in $(seq 100); do
    socket="$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'wayland-*' ! -name '*.lock' -printf '%f\n' | head -1)"
    [[ -n "$socket" ]] && break
    kill -0 "$sway_pid" 2>/dev/null || break
    sleep 0.1
done
if [[ -z "$socket" ]]; then
    echo "error: sway did not start, see its log:" >&2
    cat "$work/sway.log" >&2
    exit 1
fi
export WAYLAND_DISPLAY="$socket"

echo "=== Running the demo ==="
mkfifo "$work/app.out"
# A private session bus (GApplication registers on it); its config ships with
# dbus, not necessarily in /etc.
dbus_conf="$(dirname "$(readlink -f "$(command -v dbus-run-session)")")/../share/dbus-1/session.conf"
dbus_args=()
[[ -f "$dbus_conf" ]] && dbus_args=(--config-file="$dbus_conf")
# English UI, no portals nor accessibility bus (they would start services on
# the private bus), software rendering. setsid: one process group, killed
# as a whole at the end.
(
    cd "$work"
    LC_ALL=C.UTF-8 LANGUAGE=en GDK_BACKEND=wayland GSK_RENDERER=cairo \
        GDK_DEBUG=no-portals GTK_A11Y=none ADW_DISABLE_PORTAL=1 \
        DUPOT_DEMO_DIR="$shots" DUPOT_DEMO_DB="library.db" \
        exec setsid dbus-run-session "${dbus_args[@]}" -- "$app_bin"
) >"$work/app.out" 2>"$work/app.log" &
app_pid=$!

done_ok=false
while IFS= read -r -t 120 line; do
    case "$line" in
        "DUPOT_SHOT "*)
            name="${line#DUPOT_SHOT }"
            grim "$shots/$name.tmp.png"
            # Renamed once complete: the app waits for this name.
            mv "$shots/$name.tmp.png" "$shots/$name.png"
            echo "  shot: $name"
            ;;
        "DUPOT_RECORD start")
            # -D: every frame, not only damaged ones; else it waits for one
            # more screen change before stopping, and the screen is still.
            wf-recorder -y -D -r 20 -f "$work/demo.mp4" >"$work/recorder.log" 2>&1 &
            rec_pid=$!
            echo "  recording…"
            ;;
        "DUPOT_RECORD stop")
            # TERM, not INT: background jobs of a script ignore SIGINT.
            kill -TERM "$rec_pid" 2>/dev/null || true
            wait "$rec_pid" 2>/dev/null || true
            rec_pid=""
            echo "  recording stopped"
            ;;
        DUPOT_DONE)
            done_ok=true
            break
            ;;
    esac
done <"$work/app.out"

if ! $done_ok; then
    echo "error: the demo did not finish, app log:" >&2
    tail -n 30 "$work/app.log" >&2
    exit 1
fi

echo "=== Writing $dist_dir ==="
mkdir -p "$dist_dir"
count=0
for png in "$shots"/*.png; do
    [[ -e "$png" ]] || continue
    cp "$png" "$dist_dir/"
    count=$((count + 1))
done
if [[ -s "$work/demo.mp4" ]]; then
    ffmpeg -loglevel error -y -i "$work/demo.mp4" -vf \
        "fps=${gif_fps},scale=${gif_width}:-1:flags=lanczos,split[a][b];[a]palettegen=stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle" \
        -loop 0 "$dist_dir/promo.gif"
    echo "promo.gif: $(du -h "$dist_dir/promo.gif" | cut -f1)"
else
    echo "warning: no video recorded, promo.gif left as is (see recorder log)" >&2
fi
echo "Done: $count screenshot(s) in $dist_dir"
