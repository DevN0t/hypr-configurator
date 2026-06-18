# Hypr Configurator

A GUI configurator for [Hyprland](https://hyprland.org/) built with Rust, GTK4, and libadwaita. Works with the **Lua config format** introduced in Hyprland v0.47.

![screenshot placeholder](https://raw.githubusercontent.com/DevN0t/hypr-configurator/master/assets/screenshot.png)

> Built to make tweaking Hyprland feel like using a settings panel rather than editing config files.

---

## Features

- **Appearance** — gaps, border size, rounding, opacity, shadow, blur, border colors with a visual color picker
- **Animations** — toggle and edit animation rules as raw lines
- **Input** — keyboard layout, variant, mouse sensitivity, natural scroll
- **Startup** — `exec`, `exec-once`, environment variables, custom Lua lines
- **Monitors** — editable monitor rules with auto-detect
- **Workspaces** — workspace rules
- **Window Rules** — window rules with visual and raw modes
- **Keybinds** — visual keybind editor and raw bind lines
- **Presets** — save, load, export, and import named configuration presets
- **Default apps** — terminal, browser, file manager, editor
- **Wallpaper** — path, backend (`swaybg` / `hyprpaper`), and mode
- **Advanced** — preview generated Lua, restore `.bak` backups, safe rollback, health check, session logs

Writes a `configurator-settings.lua` file to `~/.config/hypr/` and auto-injects a `require()` into your `hyprland.lua` on first save.

---

## Requirements

| Dependency | Version |
|---|---|
| [Hyprland](https://hyprland.org/) | v0.47+ (Lua config) |
| GTK4 | 4.10+ |
| libadwaita | 1.2+ |
| Rust | 1.75+ |

---

## Build

```bash
git clone https://github.com/DevN0t/hypr-configurator.git
cd hypr-configurator
cargo build --release
```

The binary ends up at `target/release/hypr-configurator`.

---

## Install

Copy the binary somewhere on your `$PATH`:

```bash
sudo cp target/release/hypr-configurator /usr/local/bin/
```

Or run it directly:

```bash
./target/release/hypr-configurator
```

---

## Usage

Launch the app — it will open as a floating window on Hyprland.

On first save, it creates:

- `~/.config/hypr/configurator-settings.lua` — your settings as Lua
- Appends `require("configurator-settings")` to `~/.config/hypr/hyprland.lua` if not already there

### CLI

```bash
hypr-configurator --backup     # create a safe snapshot
hypr-configurator --restore    # restore last .bak files
hypr-configurator --rollback   # restore last safe snapshot
hypr-configurator --preview    # print generated Lua to stdout
hypr-configurator --health     # check environment
```

---

## Config files

| File | Description |
|---|---|
| `~/.config/hypr/config.json` | App state (internal, managed by the configurator) |
| `~/.config/hypr/configurator-settings.lua` | Generated Hyprland Lua config |
| `~/.config/hypr/hyprland.lua` | Your main config — only touched to inject `require()` |
| `~/.config/hypr/safe-rollback/` | Safe snapshots created on each save |

---

## How it works

The configurator does not replace your `hyprland.lua`. It generates a separate `configurator-settings.lua` and appends one line to your main config so Hyprland loads it. Everything else in your `hyprland.lua` stays untouched.

---

## License

MIT
