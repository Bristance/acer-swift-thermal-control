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
