#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APPLET_NAME="cosmic-applet-acer-thermal"
DESKTOP_ID="com.acer.CosmicAppletThermal.desktop"
DESKTOP_SRC="$SCRIPT_DIR/data/$DESKTOP_ID"
BUILD_BIN="$SCRIPT_DIR/target/release/$APPLET_NAME"
BACKEND_SRC="$SCRIPT_DIR/backend/thermal-control.sh"
DEFAULT_BACKEND="/usr/local/bin/thermal-control.sh"

usage() {
    cat <<EOF
Usage:
  $(basename "$0") [--local | --system]

Options:
  --local   Install into \$HOME/.local (default)
  --system  Install into /usr/local using sudo
EOF
}

install_local() {
    local bin_dir="${HOME}/.local/bin"
    local app_dir="${HOME}/.local/share/applications"

    install -d "$bin_dir" "$app_dir"
    install -m0755 "$BUILD_BIN" "$bin_dir/$APPLET_NAME"
    install -m0644 "$DESKTOP_SRC" "$app_dir/$DESKTOP_ID"
    install -m0755 "$BACKEND_SRC" "$bin_dir/thermal-control.sh"

    cat <<EOF
Installed locally:
  Binary: $bin_dir/$APPLET_NAME
  Desktop file: $app_dir/$DESKTOP_ID
  Backend: $bin_dir/thermal-control.sh
EOF
}

install_system() {
    sudo install -Dm0755 "$BUILD_BIN" "/usr/local/bin/$APPLET_NAME"
    sudo install -Dm0644 "$DESKTOP_SRC" "/usr/local/share/applications/$DESKTOP_ID"
    sudo install -Dm0755 "$BACKEND_SRC" "$DEFAULT_BACKEND"

    cat <<EOF
Installed system-wide:
  Binary: /usr/local/bin/$APPLET_NAME
  Desktop file: /usr/local/share/applications/$DESKTOP_ID
  Backend: $DEFAULT_BACKEND
EOF
}

mode="local"

case "${1:-}" in
    "")
        ;;
    --local)
        mode="local"
        ;;
    --system)
        mode="system"
        ;;
    -h|--help)
        usage
        exit 0
        ;;
    *)
        usage >&2
        exit 1
        ;;
esac

cd "$SCRIPT_DIR"

echo "Building release binary..."
cargo build --release

if [[ "$mode" == "system" ]]; then
    install_system
else
    install_local
fi

cat <<EOF

Next steps:
  1. Ensure the backend script supports:
     thermal-control.sh list --json
     thermal-control.sh set <profile>
  2. Add the applet to your COSMIC panel.

If the backend is installed somewhere else, launch the applet with:
  ACER_THERMAL_CONTROL_CMD=/path/to/thermal-control.sh $APPLET_NAME
EOF
