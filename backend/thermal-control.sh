#!/bin/bash
# /usr/local/bin/thermal-control.sh
# Bare-bones ACPI thermal mode switcher for Acer Swift X 14 (SFX14-71G)
# Requires 'acpi_call' kernel module.

set -euo pipefail

ACPI_METHOD="\_SB.PC00.WMID.WMAA"
STATE_FILE="/run/acer_thermal_mode"
DEFAULT_MODE="normal"

usage() {
    cat <<'EOF'
Usage:
  thermal-control.sh get
  thermal-control.sh get-json
  thermal-control.sh list
  thermal-control.sh list --json
  thermal-control.sh set {quiet|normal|performance|turbo}
EOF
}

mode_label() {
    case "${1:-}" in
        quiet) echo "Quiet" ;;
        normal) echo "Normal" ;;
        performance) echo "Performance" ;;
        turbo) echo "Turbo" ;;
        *) echo "Unknown" ;;
    esac
}

mode_icon() {
    case "${1:-}" in
        quiet) echo "󰌪" ;;
        normal) echo "󰾅" ;;
        performance) echo "󱐋" ;;
        turbo) echo "󱓞" ;;
        *) echo "?" ;;
    esac
}

validate_mode() {
    case "${1:-}" in
        quiet|normal|performance|turbo) return 0 ;;
        *) return 1 ;;
    esac
}

read_mode() {
    local current
    if [ -r "$STATE_FILE" ]; then
        current="$(tr -d '\n' < "$STATE_FILE" 2>/dev/null || true)"
    else
        current=""
    fi

    if validate_mode "$current"; then
        printf '%s\n' "$current"
    else
        printf '%s\n' "$DEFAULT_MODE"
    fi
}

write_mode() {
    local mode="$1"
    install -m 0644 /dev/null "$STATE_FILE"
    printf '%s\n' "$mode" > "$STATE_FILE"
    chmod 0644 "$STATE_FILE"
}

print_mode_json() {
    local current label icon
    current="$(read_mode)"
    label="$(mode_label "$current")"
    icon="$(mode_icon "$current")"
    printf '{"mode":"%s","label":"%s","icon":"%s","state_source":"cache"}\n' \
        "$current" "$label" "$icon"
}

print_waybar_json() {
    local current label icon
    current="$(read_mode)"
    label="$(mode_label "$current")"
    icon="$(mode_icon "$current")"
    printf '{"text":"%s","tooltip":"Thermal Mode: %s","class":"%s"}\n' \
        "$icon" "$label" "$current"
}

print_profiles_text() {
    cat <<'EOF'
quiet
normal
performance
turbo
EOF
}

print_profiles_json() {
    local current
    current="$(read_mode)"
    cat <<EOF
{"current":"$current","profiles":[
  {"id":"quiet","label":"Quiet","icon":"$(mode_icon quiet)","active":$( [ "$current" = "quiet" ] && echo true || echo false )},
  {"id":"normal","label":"Normal","icon":"$(mode_icon normal)","active":$( [ "$current" = "normal" ] && echo true || echo false )},
  {"id":"performance","label":"Performance","icon":"$(mode_icon performance)","active":$( [ "$current" = "performance" ] && echo true || echo false )},
  {"id":"turbo","label":"Turbo","icon":"$(mode_icon turbo)","active":$( [ "$current" = "turbo" ] && echo true || echo false )}
]}
EOF
}

set_mode() {
    local mode="${1:-}"

    if ! validate_mode "$mode"; then
        usage >&2
        exit 1
    fi

    case "$mode" in
        quiet)       echo "$ACPI_METHOD 1 1 {7, 0, 2, 0}" > /proc/acpi/call ;;
        normal)      echo "$ACPI_METHOD 1 1 {7, 0, 0, 0}" > /proc/acpi/call ;;
        performance) echo "$ACPI_METHOD 1 1 {7, 0, 3, 0}" > /proc/acpi/call ;;
        turbo)       echo "$ACPI_METHOD 1 1 {7, 0, 4, 0}" > /proc/acpi/call ;;
    esac

    write_mode "$mode"
}

if [[ "${1:-}" == "set" && "$EUID" -ne 0 ]]; then
    exec sudo "$0" "$@"
fi

case "${1:-}" in
    get)
        print_mode_json
        ;;
    get-json)
        print_waybar_json
        ;;
    list)
        if [[ "${2:-}" == "--json" ]]; then
            print_profiles_json
        else
            print_profiles_text
        fi
        ;;
    set)
        set_mode "${2:-}"
        ;;
    *)
        usage >&2
        exit 1
        ;;
esac
