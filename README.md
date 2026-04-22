# COSMIC Acer Thermal Applet

This standalone COSMIC applet exposes the Acer Swift thermal profiles from
`acer-swift-thermal-control` in the panel.

## What it does

- Shows the currently selected fan profile in the panel
- Opens a popup with the four available profiles
- Calls the backend script to switch profiles

## Backend expectations

The applet expects a `thermal-control.sh` command in `PATH` that supports:

- `thermal-control.sh list --json`
- `thermal-control.sh set <profile>`

You can override the command path with `ACER_THERMAL_CONTROL_CMD`.

## Desktop integration

Install the applet binary so `Exec=cosmic-applet-acer-thermal` resolves, then
install the desktop file from:

- `data/com.acer.CosmicAppletThermal.desktop`

After that, add the applet to your COSMIC panel configuration.

## Easy Install
Install the applet by simply running the install script from the repository folder:

- `./install.sh --system`

# Linux Build Recipe

This applet targets the COSMIC/Wayland stack and should be built on Linux.

## 1. System packages

Install the usual Rust/C toolchain plus Wayland/XKB development libraries.

### Pop!_OS / Ubuntu

```bash
sudo apt update
sudo apt install -y \
  build-essential \
  curl \
  git \
  pkg-config \
  libxkbcommon-dev \
  libwayland-dev \
  wayland-protocols
```

### Fedora

```bash
sudo dnf install -y \
  gcc \
  gcc-c++ \
  make \
  curl \
  git \
  pkgconf-pkg-config \
  libxkbcommon-devel \
  wayland-devel \
  wayland-protocols-devel
```

### Arch Linux

```bash
sudo pacman -S --needed \
  base-devel \
  curl \
  git \
  pkgconf \
  libxkbcommon \
  wayland \
  wayland-protocols
```

## 2. Rust toolchain

If Rust is not already installed:

```bash
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
rustup default stable
```

## 3. Check the backend script

From the workspace root:

```bash
bash /path/to/acer-thermal-cosmic/acer-swift-thermal-control/thermal-control.sh get
bash /path/to/acer-thermal-cosmic/acer-swift-thermal-control/thermal-control.sh list --json
```

If you want the applet to invoke the script directly later, also make it executable:

```bash
chmod +x /path/to/acer-thermal-cosmic/acer-swift-thermal-control/thermal-control.sh
```

## 4. Build and check the applet

```bash
cd /path/to/acer-thermal-cosmic/cosmic-applet-acer-thermal
cargo fmt --all
cargo check
```

## 5. Run a local development build

Point the applet at the backend script explicitly:

```bash
cd /path/to/acer-thermal-cosmic/cosmic-applet-acer-thermal
ACER_THERMAL_CONTROL_CMD=/path/to/acer-thermal-cosmic/acer-swift-thermal-control/thermal-control.sh \
  cargo run
```

## 6. Install for COSMIC testing

Build the release binary:

```bash
cd /path/to/acer-thermal-cosmic/cosmic-applet-acer-thermal
cargo build --release
```

Install the binary and desktop file:

```bash
sudo install -Dm0755 \
  target/release/cosmic-applet-acer-thermal \
  /usr/local/bin/cosmic-applet-acer-thermal

sudo install -Dm0644 \
  data/com.acer.CosmicAppletThermal.desktop \
  /usr/local/share/applications/com.acer.CosmicAppletThermal.desktop
```

If you prefer a user-local install:

```bash
install -Dm0755 \
  target/release/cosmic-applet-acer-thermal \
  "$HOME/.local/bin/cosmic-applet-acer-thermal"

install -Dm0644 \
  data/com.acer.CosmicAppletThermal.desktop \
  "$HOME/.local/share/applications/com.acer.CosmicAppletThermal.desktop"
```

## 7. Backend privilege model

The applet calls:

- `thermal-control.sh list --json`
- `thermal-control.sh set <profile>`

`set` currently re-execs through `sudo` when not already root, so for panel use you will usually want a narrowly scoped `sudoers` rule for the script on your Linux machine.

Example shape:

```text
youruser ALL=(root) NOPASSWD: /usr/local/bin/thermal-control.sh
```

Review that carefully before using it.

## 8. COSMIC panel integration

After installing the desktop file, add the applet to your COSMIC panel setup in the same way other `X-CosmicApplet=true` entries are added.

The desktop entry included here is:

- `com.acer.CosmicAppletThermal.desktop`

## 9. Recommended verification

Once running under COSMIC:

1. Open the applet popup from the panel.
2. Confirm the current profile label matches `thermal-control.sh list --json`.
3. Switch through `Quiet`, `Normal`, `Performance`, and `Turbo`.
4. After each switch, re-run:

```bash
/path/to/acer-thermal-cosmic/acer-swift-thermal-control/thermal-control.sh list --json
```

5. Confirm the panel label updates after each successful change.

## Notes

- The backend currently reports cached state from `/run/acer_thermal_mode`, not a hardware readback.
- For real deployment, testing on the target Acer model under Linux is important before relying on the displayed state.
