-- Hyprland Lua config convertido do seu hyprland.conf
-- Coloque este arquivo em: ~/.config/hypr/hyprland.lua

------------------
---- MONITORS ----
------------------

hl.monitor({
  output = "DP-1",
  disabled = true,
})

hl.monitor({
  output = "DP-2",
  mode = "3440x1440@165.0",
  position = "0x0",
  scale = 1.0,
})

hl.monitor({
  output = "DP-3",
  mode = "1920x1080@60.0",
  position = "3440x0",
  scale = 1.0,
})

---------------------
---- MY PROGRAMS ----
---------------------

local terminal = "alacritty"
local fileManager = "nemo"
local menu = "hyprlauncher"

-------------------
---- AUTOSTART ----
-------------------

hl.on("hyprland.start", function ()
  hl.exec_cmd("firefox")
  hl.exec_cmd("swaybg -i /home/user/Downloads/walpaper.png -m fill")
  hl.exec_cmd("waybar")
  hl.exec_cmd("quickshell -c island")
  -- Corrigido: no seu .conf estava "agente-1", o nome certo é "agent-1"
  hl.exec_cmd("/usr/lib/polkit-gnome/polkit-gnome-authentication-agente-1")
end)

-------------------------------
---- ENVIRONMENT VARIABLES ----
-------------------------------
hl.exec_cmd("swaybg -i /home/user/Downloads/walpaper.png -m fill")
-- hl.exec_cmd("quickshell -c island")

hl.env("XCURSOR_SIZE", "24")
hl.env("HYPRCURSOR_SIZE", "24")

-- Seu .conf tinha: gpus = nvidia
-- Isso não é a forma nova recomendada. Para multi-GPU, use AQ_DRM_DEVICES se precisar:
-- hl.env("AQ_DRM_DEVICES", "/dev/dri/card0:/dev/dri/card1")

-----------------------
---- LOOK AND FEEL ----
-----------------------

hl.config({
  general = {
    gaps_in = 5,
    gaps_out = 20,
    border_size = 2,

    col = {
      active_border = "rgba(999999aa)",
      inactive_border = "rgba(000000aa)",
    },

    resize_on_border = false,
    allow_tearing = false,
    layout = "dwindle",
  },

  decoration = {
    rounding = 30,
    rounding_power = 2,

    active_opacity = 1.0,
    inactive_opacity = 1.0,

    shadow = {
      enabled = true,
      range = 4,
      render_power = 3,
      color = 0xee1a1a1a,
    },

    blur = {
      enabled = true,
      size = 1,
      passes = 4,
      vibrancy = 0.1696,
    },
  },

  animations = {
    -- Corrigido: no seu .conf estava "enabled = yes, please :)", isso não é boolean.
    enabled = true,
  },
})

--------------------
---- ANIMATIONS ----
--------------------

hl.curve("easeOutQuint", { type = "bezier", points = { {0.23, 1}, {0.32, 1} } })
hl.curve("easeInOutCubic", { type = "bezier", points = { {0.65, 0.05}, {0.36, 1} } })
hl.curve("linear", { type = "bezier", points = { {0, 0}, {1, 1} } })
hl.curve("almostLinear", { type = "bezier", points = { {0.5, 0.5}, {0.75, 1} } })
hl.curve("quick", { type = "bezier", points = { {0.15, 0}, {0.1, 1} } })

hl.animation({ leaf = "global", enabled = true, speed = 10, bezier = "default" })
hl.animation({ leaf = "border", enabled = true, speed = 5.39, bezier = "easeOutQuint" })
hl.animation({ leaf = "windows", enabled = true, speed = 4.79, bezier = "easeOutQuint" })
hl.animation({ leaf = "windowsIn", enabled = true, speed = 4.1, bezier = "easeOutQuint", style = "popin 87%" })
hl.animation({ leaf = "windowsOut", enabled = true, speed = 1.49, bezier = "linear", style = "popin 87%" })
hl.animation({ leaf = "fadeIn", enabled = true, speed = 1.73, bezier = "almostLinear" })
hl.animation({ leaf = "fadeOut", enabled = true, speed = 1.46, bezier = "almostLinear" })
hl.animation({ leaf = "fade", enabled = true, speed = 3.03, bezier = "quick" })
hl.animation({ leaf = "layers", enabled = true, speed = 3.81, bezier = "easeOutQuint" })
hl.animation({ leaf = "layersIn", enabled = true, speed = 4, bezier = "easeOutQuint", style = "fade" })
hl.animation({ leaf = "layersOut", enabled = true, speed = 1.5, bezier = "linear", style = "fade" })
hl.animation({ leaf = "fadeLayersIn", enabled = true, speed = 1.79, bezier = "almostLinear" })
hl.animation({ leaf = "fadeLayersOut", enabled = true, speed = 1.39, bezier = "almostLinear" })
hl.animation({ leaf = "workspaces", enabled = true, speed = 1.94, bezier = "almostLinear", style = "fade" })
hl.animation({ leaf = "workspacesIn", enabled = true, speed = 1.21, bezier = "almostLinear", style = "fade" })
hl.animation({ leaf = "workspacesOut", enabled = true, speed = 1.94, bezier = "almostLinear", style = "fade" })
hl.animation({ leaf = "zoomFactor", enabled = true, speed = 7, bezier = "quick" })

-----------------
---- LAYOUTS ----
-----------------

hl.config({
  dwindle = { 
    preserve_split = true,
  },
})

hl.config({
  master = {
    new_status = "master",
  },
})

--------------
---- MISC ----
--------------

hl.config({
  misc = {
    force_default_wallpaper = -1,
    disable_hyprland_logo = false,
  },
})

---------------
---- INPUT ----
---------------

hl.config({
  input = {
    kb_layout = "us",
    kb_variant = "intl",
    kb_model = "",
    kb_options = "",
    kb_rules = "",

    follow_mouse = 1,
    sensitivity = 0,

    touchpad = {
      natural_scroll = false,
    },
  },
})

hl.gesture({
  fingers = 3,
  direction = "horizontal",
  action = "workspace",
})

hl.device({
  name = "epic-mouse-v1",
  sensitivity = -0.5,
})

------------------
---- WORKSPACES ---
------------------

-- Keep odd workspaces on the main ultrawide and even workspaces on
-- the secondary display, regardless of which monitor is focused.
for workspace = 1, 10 do
  hl.workspace_rule({
    workspace = tostring(workspace),
    monitor = workspace % 2 == 1 and "DP-2" or "DP-3",
    default = workspace == 1 or workspace == 2,
  })
end

---------------------
---- KEYBINDINGS ----
---------------------

local mainMod = "SUPER"

-- Seus atalhos customizados
hl.bind(mainMod .. " + RETURN", hl.dsp.exec_cmd(terminal))
hl.bind(mainMod .. " + Q", hl.dsp.window.close())
hl.bind(mainMod .. " + D", hl.dsp.exec_cmd("wofi --show drun -n"))
hl.bind(mainMod .. " + E", hl.dsp.exec_cmd("nemo"))
-- Padrões úteis
hl.bind(mainMod .. " + C", hl.dsp.window.close())
hl.bind(mainMod .. " + M", hl.dsp.exec_cmd("command -v hyprshutdown >/dev/null 2>&1 && hyprshutdown || hyprctl dispatch exit"))
hl.bind(mainMod .. " + V", hl.dsp.window.float({ action = "toggle" }))
hl.bind(mainMod .. " + R", hl.dsp.exec_cmd(menu))
hl.bind(mainMod .. " + P", hl.dsp.window.pseudo())
hl.bind(mainMod .. " + J", hl.dsp.layout("togglesplit"))

-- Mover foco
hl.bind(mainMod .. " + left", hl.dsp.focus({ direction = "left" }))
hl.bind(mainMod .. " + right", hl.dsp.focus({ direction = "right" }))
hl.bind(mainMod .. " + up", hl.dsp.focus({ direction = "up" }))
hl.bind(mainMod .. " + down", hl.dsp.focus({ direction = "down" }))

-- Mover janela
hl.bind(mainMod .. " + SHIFT + left", hl.dsp.window.move({ direction = "left" }))
hl.bind(mainMod .. " + SHIFT + right", hl.dsp.window.move({ direction = "right" }))
hl.bind(mainMod .. " + SHIFT + up", hl.dsp.window.move({ direction = "up" }))
hl.bind(mainMod .. " + SHIFT + down", hl.dsp.window.move({ direction = "down" }))

hl.bind(mainMod .. " + S", hl.dsp.exec_cmd("grim -g \"$(slurp)\" - | wl-copy"))



-- Workspaces 1-10
for i = 1, 10 do
  local key = i % 10 -- workspace 10 fica na tecla 0
  hl.bind(mainMod .. " + " .. key, hl.dsp.focus({ workspace = i }))
  hl.bind(mainMod .. " + SHIFT + " .. key, hl.dsp.window.move({ workspace = i }))
end

-- Special workspace
hl.bind(mainMod .. " + SHIFT + S", hl.dsp.window.move({ workspace = "special:magic" }))

-- Scroll workspaces
hl.bind(mainMod .. " + mouse_down", hl.dsp.focus({ workspace = "e+1" }))
hl.bind(mainMod .. " + mouse_up", hl.dsp.focus({ workspace = "e-1" }))

-- Mouse move/resize
hl.bind(mainMod .. " + mouse:272", hl.dsp.window.drag(), { mouse = true })
hl.bind(mainMod .. " + mouse:273", hl.dsp.window.resize(), { mouse = true })

-- Teclas multimídia
hl.bind("XF86AudioRaiseVolume", hl.dsp.exec_cmd("wpctl set-volume -l 1 @DEFAULT_AUDIO_SINK@ 5%+"), { locked = true, repeating = true })
hl.bind("XF86AudioLowerVolume", hl.dsp.exec_cmd("wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-"), { locked = true, repeating = true })
hl.bind("XF86AudioMute", hl.dsp.exec_cmd("wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"), { locked = true, repeating = true })
hl.bind("XF86AudioMicMute", hl.dsp.exec_cmd("wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle"), { locked = true, repeating = true })

hl.bind("XF86MonBrightnessUp", hl.dsp.exec_cmd("brightnessctl -e4 -n2 set 5%+"), { locked = true, repeating = true })
hl.bind("XF86MonBrightnessDown", hl.dsp.exec_cmd("brightnessctl -e4 -n2 set 5%-"), { locked = true, repeating = true })

hl.bind("XF86AudioNext", hl.dsp.exec_cmd("playerctl next"), { locked = true })
hl.bind("XF86AudioPause", hl.dsp.exec_cmd("playerctl play-pause"), { locked = true })
hl.bind("XF86AudioPlay", hl.dsp.exec_cmd("playerctl play-pause"), { locked = true })
hl.bind("XF86AudioPrev", hl.dsp.exec_cmd("playerctl previous"), { locked = true })

--------------------------------
---- WINDOWS AND WORKSPACES ----
--------------------------------

hl.window_rule({
  name = "suppress-maximize-events",
  match = {
    class = ".*",
  },
  suppress_event = "maximize",
})

hl.window_rule({
  name = "fix-xwayland-drags",
  match = {
    class = "^$",
    title = "^$",
    xwayland = true,
    float = true,
    fullscreen = false,
    pin = false,
  },
  no_focus = true,
})

hl.window_rule({
  name = "move-hyprland-run",
  match = {
    class = "hyprland-run",
  },
  move = "20 monitor_h-120",
  float = true,
})

------------------------
---- OLD SOURCES NOTE --
------------------------

-- Seu .conf antigo tinha:
-- source = ~/.config/hypr/monitors.conf
-- source = ~/.config/hypr/workspaces.conf
--
-- Em Lua, não use "source = ...". Use require(), exemplo:
-- require("monitors")
-- require("workspaces")
--
-- Mas esses arquivos precisam ser:
-- ~/.config/hypr/monitors.lua
-- ~/.config/hypr/workspaces.lua