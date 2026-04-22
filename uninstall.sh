#!/usr/bin/env bash

set -euo pipefail

APPLET_NAME="cosmic-applet-acer-thermal"
DESKTOP_ID="com.acer.CosmicAppletThermal.desktop"
BACKEND_NAME="thermal-control.sh"

usage() {
    cat <<EOF
Usage:
  $(basename "$0") [--local | --system]

Options:
  --local   Remove files from \$HOME/.local (default)
  --system  Remove files from /usr/local using sudo
EOF
}

uninstall_local() {
    local bin_path="${HOME}/.local/bin/$APPLET_NAME"
    local backend_path="${HOME}/.local/bin/$BACKEND_NAME"
    local desktop_path="${HOME}/.local/share/applications/$DESKTOP_ID"

    rm -f "$bin_path" "$backend_path" "$desktop_path"

    cat <<EOF
Removed local install:
  Binary: $bin_path
  Backend: $backend_path
  Desktop file: $desktop_path
EOF
}

uninstall_system() {
    local bin_path="/usr/local/bin/$APPLET_NAME"
    local backend_path="/usr/local/bin/$BACKEND_NAME"
    local desktop_path="/usr/local/share/applications/$DESKTOP_ID"

    sudo rm -f "$bin_path" "$backend_path" "$desktop_path"

    cat <<EOF
Removed system install:
  Binary: $bin_path
  Backend: $backend_path
  Desktop file: $desktop_path
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

if [[ "$mode" == "system" ]]; then
    uninstall_system
else
    uninstall_local
fi
