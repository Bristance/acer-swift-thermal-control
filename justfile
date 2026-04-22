set dotenv-load := false

backend := justfile_directory() + "/../acer-swift-thermal-control/thermal-control.sh"
desktop_file := justfile_directory() + "/data/com.acer.CosmicAppletThermal.desktop"
local_bin_dir := env_var_or_default("HOME", "") + "/.local/bin"
local_app_dir := env_var_or_default("HOME", "") + "/.local/share/applications"

default:
  @just --list

backend-check:
  bash "{{backend}}" get
  bash "{{backend}}" list --json

fmt:
  cargo fmt --all

check:
  cargo check

run:
  ACER_THERMAL_CONTROL_CMD="{{backend}}" cargo run

build:
  cargo build --release

install-local: build
  install -d "{{local_bin_dir}}" "{{local_app_dir}}"
  install -m0755 target/release/cosmic-applet-acer-thermal "{{local_bin_dir}}/cosmic-applet-acer-thermal"
  install -m0644 "{{desktop_file}}" "{{local_app_dir}}/com.acer.CosmicAppletThermal.desktop"

install-system: build
  sudo install -Dm0755 target/release/cosmic-applet-acer-thermal /usr/local/bin/cosmic-applet-acer-thermal
  sudo install -Dm0644 "{{desktop_file}}" /usr/local/share/applications/com.acer.CosmicAppletThermal.desktop
