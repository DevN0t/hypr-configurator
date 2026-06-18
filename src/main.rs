use adw::prelude::*;
use gtk::glib;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs;
use std::process::Command;
use std::rc::Rc;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HyprConfig {
    gaps_in: u32,
    gaps_out: u32,
    border_size: u32,
    active_border_color: String,
    inactive_border_color: String,
    resize_on_border: bool,
    allow_tearing: bool,
    layout: String,
    rounding: u32,
    active_opacity: f64,
    inactive_opacity: f64,
    shadow_enabled: bool,
    shadow_range: u32,
    blur_enabled: bool,
    blur_size: u32,
    blur_passes: u32,
    animations_enabled: bool,
    kb_layout: String,
    kb_variant: String,
    sensitivity: f64,
    natural_scroll: bool,
    disable_hyprland_logo: bool,
    startup_commands: Vec<String>,
    exec_commands: Vec<String>,
    exec_once_commands: Vec<String>,
    env_vars: Vec<String>,
    monitor_rules: Vec<String>,
    visual_workspaces: Vec<String>,
    workspace_rules: Vec<String>,
    visual_window_rules: Vec<String>,
    window_rules: Vec<String>,
    visual_keybinds: Vec<String>,
    keybinds: Vec<String>,
    mouse_binds: Vec<String>,
    default_terminal: String,
    default_browser: String,
    default_file_manager: String,
    default_editor: String,
    wallpaper_path: String,
    wallpaper_backend: String,
    wallpaper_mode: String,
    animation_lines: Vec<String>,
    decoration_lines: Vec<String>,
    input_extra_lines: Vec<String>,
    gesture_lines: Vec<String>,
    custom_lines: Vec<String>,
}

impl Default for HyprConfig {
    fn default() -> Self {
        Self {
            gaps_in: 5,
            gaps_out: 20,
            border_size: 2,
            active_border_color: "rgba(999999aa)".to_string(),
            inactive_border_color: "rgba(000000aa)".to_string(),
            resize_on_border: false,
            allow_tearing: false,
            layout: "dwindle".to_string(),
            rounding: 30,
            active_opacity: 1.0,
            inactive_opacity: 1.0,
            shadow_enabled: true,
            shadow_range: 4,
            blur_enabled: true,
            blur_size: 1,
            blur_passes: 4,
            animations_enabled: true,
            kb_layout: "us".to_string(),
            kb_variant: "intl".to_string(),
            sensitivity: 0.0,
            natural_scroll: false,
            disable_hyprland_logo: false,
            startup_commands: vec![],
            exec_commands: vec![],
            exec_once_commands: vec!["waybar".to_string()],
            env_vars: vec!["XCURSOR_SIZE,24".to_string(), "QT_QPA_PLATFORMTHEME,qt5ct".to_string()],
            monitor_rules: vec![],
            visual_workspaces: vec![],
            workspace_rules: vec![],
            visual_window_rules: vec!["dev.n0t.hyprconfigurator|float=true|center=true|size=1280x960".to_string()],
            window_rules: vec![],
            visual_keybinds: vec!["SUPER|Return|exec|kitty".to_string(), "SUPER|Q|killactive|".to_string()],
            keybinds: vec![],
            mouse_binds: vec!["SUPER, mouse:272, movewindow".to_string(), "SUPER, mouse:273, resizewindow".to_string()],
            // ^ format: MOD, mouse:BTN, dispatcher
            default_terminal: "kitty".to_string(),
            default_browser: "firefox".to_string(),
            default_file_manager: "nautilus".to_string(),
            default_editor: "code".to_string(),
            wallpaper_path: String::new(),
            wallpaper_backend: "swaybg".to_string(),
            wallpaper_mode: "fill".to_string(),
            animation_lines: vec![
                "hl.animation({ leaf = \"windows\", enabled = true, speed = 7, bezier = \"default\" })".to_string(),
                "hl.animation({ leaf = \"workspaces\", enabled = true, speed = 6, bezier = \"default\" })".to_string(),
            ],
            decoration_lines: vec![],
            input_extra_lines: vec![
                "hl.config({ input = { repeat_rate = 35, repeat_delay = 250, numlock_by_default = true } })".to_string(),
            ],
            gesture_lines: vec![],
            custom_lines: vec![],
        }
    }
}

#[derive(Clone)]
struct TextRows {
    list: gtk::ListBox,
    entries: Rc<RefCell<Vec<gtk::Entry>>>,
}

impl TextRows {
    fn values(&self) -> Vec<String> {
        self.entries
            .borrow()
            .iter()
            .map(|entry| entry.text().trim().to_string())
            .filter(|line| !line.is_empty())
            .collect()
    }

    fn clear(&self) {
        self.entries.borrow_mut().clear();
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
    }

    fn set_values(&self, values: &[String], placeholder: &str) {
        self.clear();
        if values.is_empty() {
            add_text_row(&self.list, self.entries.clone(), "", placeholder);
        } else {
            for value in values {
                add_text_row(&self.list, self.entries.clone(), value, placeholder);
            }
        }
    }
}

fn config_path() -> std::path::PathBuf {
    let mut p = dirs::home_dir().expect("home not found");
    p.push(".config/hypr/config.json");
    p
}

fn lua_config_path() -> std::path::PathBuf {
    let mut p = dirs::home_dir().expect("home not found");
    p.push(".config/hypr/configurator-settings.lua");
    p
}

fn hyprland_lua_path() -> std::path::PathBuf {
    let mut p = dirs::home_dir().expect("home not found");
    p.push(".config/hypr/hyprland.lua");
    p
}

fn exported_preset_path() -> std::path::PathBuf {
    let mut p = dirs::home_dir().expect("home not found");
    p.push(".config/hypr/exported-preset.json");
    p
}

fn presets_dir() -> std::path::PathBuf {
    let mut p = dirs::home_dir().expect("home not found");
    p.push(".config/hypr/presets");
    p
}

fn safe_backup_dir() -> std::path::PathBuf {
    let mut p = dirs::home_dir().expect("home not found");
    p.push(".config/hypr/safe-rollback");
    p
}

fn safe_path(name: &str) -> std::path::PathBuf {
    let mut p = safe_backup_dir();
    p.push(name);
    p
}

fn lua_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        r#"
        .config-row { background: #ffffff; border-radius: 12px; }
        .config-entry { background: #ffffff; color: #000000; border-radius: 10px; }
        .config-entry text { background: #ffffff; color: #000000; }
        .config-entry:focus, .config-entry:focus-within,
        .config-entry:focus text, .config-entry:focus-within text {
            outline: none; box-shadow: none; border-color: transparent;
        }
        .internal-row-button { min-height: 0 !important; padding: 2px 16px !important; }
        .destructive-action { margin-right: 10px; }
        .add-button { padding: 2px 20px; height: 10px }
        .code-view { font-family: monospace; font-size: 12px; }
        .sidebar { background: alpha(currentColor, 0.04); border-right: 1px solid alpha(currentColor, 0.10); }
        .sidebar-title { font-weight: 700; font-size: 18px; }
        .sidebar-item { padding: 10px 14px; margin: 2px 8px; border-radius: 10px; }
        .mod-key { min-height: 0 !important; padding: 2px 8px !important; font-size: 0.78em; border-radius: 6px; }
        .kb-key-entry { min-width: 60px; max-width: 90px; }
        .kb-args-entry { min-width: 80px; }
        "#,
    );

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("display not found"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn add_text_row(
    list: &gtk::ListBox,
    entries: Rc<RefCell<Vec<gtk::Entry>>>,
    value: &str,
    placeholder: &str,
) -> gtk::Entry {
    let entry = gtk::Entry::builder()
        .text(value)
        .placeholder_text(placeholder)
        .hexpand(true)
        .css_classes(["config-entry"])
        .build();

    let delete_btn = gtk::Button::builder()
        .icon_name("edit-delete-symbolic")
        .tooltip_text("Remove row")
        .css_classes(["flat", "circular", "destructive-action"])
        .valign(gtk::Align::Center)
        .build();

    let row_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .css_classes(["config-row"])
        .margin_top(6)
        .margin_bottom(6)
        .build();

    row_box.append(&entry);
    row_box.append(&delete_btn);

    let row = gtk::ListBoxRow::builder()
        .child(&row_box)
        .selectable(false)
        .activatable(false)
        .build();

    list.append(&row);
    entries.borrow_mut().push(entry.clone());

    let list_delete = list.clone();
    let entries_delete = entries.clone();
    let entry_delete = entry.clone();
    let row_delete = row.clone();

    delete_btn.connect_clicked(move |_| {
        entries_delete
            .borrow_mut()
            .retain(|current| current != &entry_delete);
        list_delete.remove(&row_delete);
    });

    entry
}

const KB_DISPATCHERS: [&str; 13] = [
    "exec",
    "killactive",
    "togglefloating",
    "workspace",
    "movetoworkspace",
    "movefocus",
    "movewindow",
    "fullscreen",
    "pseudo",
    "togglesplit",
    "exit",
    "togglespecialworkspace",
    "pin",
];

fn dispatcher_has_args(d: &str) -> bool {
    d == "exec"
        || d == "workspace"
        || d == "movetoworkspace"
        || d == "movefocus"
        || d == "movewindow"
        || d == "togglespecialworkspace"
}

#[derive(Clone)]
struct KeybindEntry {
    row: gtk::ListBoxRow,
    super_btn: gtk::ToggleButton,
    shift_btn: gtk::ToggleButton,
    ctrl_btn: gtk::ToggleButton,
    alt_btn: gtk::ToggleButton,
    key_entry: gtk::Entry,
    action_drop: gtk::DropDown,
    args_entry: gtk::Entry,
}

impl KeybindEntry {
    fn value(&self) -> Option<String> {
        let key = self.key_entry.text();
        let key = key.trim();
        if key.is_empty() {
            return None;
        }
        let mut mods: Vec<&str> = Vec::new();
        if self.super_btn.is_active() {
            mods.push("SUPER");
        }
        if self.shift_btn.is_active() {
            mods.push("SHIFT");
        }
        if self.ctrl_btn.is_active() {
            mods.push("CTRL");
        }
        if self.alt_btn.is_active() {
            mods.push("ALT");
        }
        let idx = self.action_drop.selected() as usize;
        let action = KB_DISPATCHERS.get(idx).copied().unwrap_or("exec");
        let args = self.args_entry.text();
        Some(format!("{}|{key}|{action}|{}", mods.join(" "), args.trim()))
    }
}

#[derive(Clone)]
struct KeybindBuilder {
    list: gtk::ListBox,
    entries: Rc<RefCell<Vec<KeybindEntry>>>,
}

impl KeybindBuilder {
    fn values(&self) -> Vec<String> {
        self.entries
            .borrow()
            .iter()
            .filter_map(|e| e.value())
            .collect()
    }

    fn set_values(&self, values: &[String]) {
        self.entries.borrow_mut().clear();
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        let source = if values.is_empty() {
            vec![String::new()]
        } else {
            values.to_vec()
        };
        for val in &source {
            let entry = make_keybind_entry(val, self.list.clone(), self.entries.clone());
            self.list.append(&entry.row);
            self.entries.borrow_mut().push(entry);
        }
    }

    fn add_empty(&self) {
        let entry = make_keybind_entry("", self.list.clone(), self.entries.clone());
        self.list.append(&entry.row);
        self.entries.borrow_mut().push(entry);
    }
}

fn make_keybind_entry(
    initial: &str,
    list: gtk::ListBox,
    entries: Rc<RefCell<Vec<KeybindEntry>>>,
) -> KeybindEntry {
    let parts: Vec<&str> = initial.splitn(4, '|').collect();
    let mod_str = parts.first().copied().unwrap_or("");
    let key_str = parts.get(1).copied().unwrap_or("");
    let action_str = parts.get(2).copied().unwrap_or("exec");
    let args_str = parts.get(3).copied().unwrap_or("");

    let mods: Vec<&str> = mod_str.split_whitespace().collect();
    let action_idx = KB_DISPATCHERS
        .iter()
        .position(|&d| d == action_str)
        .unwrap_or(0) as u32;

    let mod_btn = |label: &str, active: bool| {
        gtk::ToggleButton::builder()
            .label(label)
            .active(active)
            .css_classes(["mod-key"])
            .build()
    };

    let super_btn = mod_btn("SUPER", mods.contains(&"SUPER"));
    let shift_btn = mod_btn("SHIFT", mods.contains(&"SHIFT"));
    let ctrl_btn = mod_btn("CTRL", mods.contains(&"CTRL"));
    let alt_btn = mod_btn("ALT", mods.contains(&"ALT"));

    let mods_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(3)
        .build();
    for btn in [&super_btn, &shift_btn, &ctrl_btn, &alt_btn] {
        mods_box.append(btn);
    }

    let plus = gtk::Label::builder()
        .label("+")
        .css_classes(["dim-label"])
        .margin_start(4)
        .margin_end(4)
        .build();

    let key_entry = gtk::Entry::builder()
        .placeholder_text("key")
        .text(key_str)
        .css_classes(["kb-key-entry"])
        .build();

    let arrow = gtk::Label::builder()
        .label("→")
        .css_classes(["dim-label"])
        .margin_start(4)
        .margin_end(4)
        .build();

    let dispatcher_model = gtk::StringList::new(KB_DISPATCHERS.as_slice());
    let action_drop = gtk::DropDown::builder()
        .model(&dispatcher_model)
        .selected(action_idx)
        .valign(gtk::Align::Center)
        .build();

    let args_entry = gtk::Entry::builder()
        .placeholder_text("argument")
        .text(args_str)
        .hexpand(true)
        .css_classes(["kb-args-entry"])
        .visible(dispatcher_has_args(action_str))
        .build();

    let args_vis = args_entry.clone();
    action_drop.connect_selected_notify(move |drop| {
        let idx = drop.selected() as usize;
        let d = KB_DISPATCHERS.get(idx).copied().unwrap_or("exec");
        args_vis.set_visible(dispatcher_has_args(d));
    });

    let del_btn = gtk::Button::builder()
        .icon_name("list-remove-symbolic")
        .css_classes(["flat", "circular"])
        .valign(gtk::Align::Center)
        .build();

    let row_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(12)
        .margin_end(12)
        .build();
    row_box.append(&mods_box);
    row_box.append(&plus);
    row_box.append(&key_entry);
    row_box.append(&arrow);
    row_box.append(&action_drop);
    row_box.append(&args_entry);
    row_box.append(&del_btn);

    let row = gtk::ListBoxRow::builder()
        .child(&row_box)
        .activatable(false)
        .build();

    let entry = KeybindEntry {
        row: row.clone(),
        super_btn,
        shift_btn,
        ctrl_btn,
        alt_btn,
        key_entry,
        action_drop,
        args_entry,
    };

    let list_d = list.clone();
    let entries_d = entries.clone();
    let row_d = row.clone();
    del_btn.connect_clicked(move |_| {
        list_d.remove(&row_d);
        entries_d.borrow_mut().retain(|e| e.row != row_d);
    });

    entry
}

fn make_keybind_builder(
    title: &str,
    initial: &[String],
) -> (adw::PreferencesGroup, KeybindBuilder) {
    let entries: Rc<RefCell<Vec<KeybindEntry>>> = Rc::new(RefCell::new(Vec::new()));
    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();

    let builder = KeybindBuilder {
        list: list.clone(),
        entries: entries.clone(),
    };
    builder.set_values(initial);

    let add_btn = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Add keybind")
        .css_classes(["flat"])
        .build();

    let builder_add = builder.clone();
    add_btn.connect_clicked(move |_| {
        builder_add.add_empty();
    });

    let group = adw::PreferencesGroup::builder().title(title).build();
    group.set_header_suffix(Some(&add_btn));
    group.add(&list);

    (group, builder)
}

fn clear_listbox(list: &gtk::ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}

fn pill_delete_button() -> gtk::Button {
    gtk::Button::builder()
        .icon_name("list-remove-symbolic")
        .css_classes(["flat", "circular"])
        .valign(gtk::Align::Center)
        .build()
}

fn tiny_entry(text: &str, placeholder: &str, hexpand: bool) -> gtk::Entry {
    gtk::Entry::builder()
        .text(text)
        .placeholder_text(placeholder)
        .hexpand(hexpand)
        .css_classes(["config-entry"])
        .build()
}

fn mods_from_text(value: &str) -> Vec<&str> {
    value.split_whitespace().collect()
}

fn make_mod_buttons(
    mods: &[&str],
) -> (
    gtk::ToggleButton,
    gtk::ToggleButton,
    gtk::ToggleButton,
    gtk::ToggleButton,
    gtk::Box,
) {
    let mk = |label: &str| {
        gtk::ToggleButton::builder()
            .label(label)
            .active(mods.contains(&label))
            .css_classes(["mod-key"])
            .build()
    };
    let super_btn = mk("SUPER");
    let shift_btn = mk("SHIFT");
    let ctrl_btn = mk("CTRL");
    let alt_btn = mk("ALT");
    let box_mods = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(3)
        .build();
    for btn in [&super_btn, &shift_btn, &ctrl_btn, &alt_btn] {
        box_mods.append(btn);
    }
    (super_btn, shift_btn, ctrl_btn, alt_btn, box_mods)
}

fn selected_mods(
    super_btn: &gtk::ToggleButton,
    shift_btn: &gtk::ToggleButton,
    ctrl_btn: &gtk::ToggleButton,
    alt_btn: &gtk::ToggleButton,
) -> String {
    let mut mods = Vec::new();
    if super_btn.is_active() {
        mods.push("SUPER");
    }
    if shift_btn.is_active() {
        mods.push("SHIFT");
    }
    if ctrl_btn.is_active() {
        mods.push("CTRL");
    }
    if alt_btn.is_active() {
        mods.push("ALT");
    }
    mods.join(" ")
}

#[derive(Clone)]
struct EnvEntry {
    row: gtk::ListBoxRow,
    key: gtk::Entry,
    value: gtk::Entry,
}

impl EnvEntry {
    fn value(&self) -> Option<String> {
        let key = self.key.text().trim().to_string();
        if key.is_empty() {
            return None;
        }
        Some(format!("{},{}", key, self.value.text().trim()))
    }
}

#[derive(Clone)]
struct EnvBuilder {
    list: gtk::ListBox,
    entries: Rc<RefCell<Vec<EnvEntry>>>,
}

impl EnvBuilder {
    fn values(&self) -> Vec<String> {
        self.entries
            .borrow()
            .iter()
            .filter_map(|e| e.value())
            .collect()
    }
    fn set_values(&self, values: &[String]) {
        self.entries.borrow_mut().clear();
        clear_listbox(&self.list);
        let source = if values.is_empty() {
            vec![String::new()]
        } else {
            values.to_vec()
        };
        for val in source {
            self.add_value(&val);
        }
    }
    fn add_value(&self, value: &str) {
        let entry = make_env_entry(value, self.list.clone(), self.entries.clone());
        self.list.append(&entry.row);
        self.entries.borrow_mut().push(entry);
    }
}

fn make_env_entry(
    initial: &str,
    list: gtk::ListBox,
    entries: Rc<RefCell<Vec<EnvEntry>>>,
) -> EnvEntry {
    let (k, v) = initial.split_once(',').unwrap_or((initial, ""));
    let key = tiny_entry(k.trim(), "KEY", false);
    key.set_width_chars(22);
    let value = tiny_entry(v.trim(), "value", true);
    let del = pill_delete_button();
    let row_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(12)
        .margin_end(12)
        .build();
    row_box.append(&key);
    row_box.append(&value);
    row_box.append(&del);
    let row = gtk::ListBoxRow::builder()
        .child(&row_box)
        .activatable(false)
        .selectable(false)
        .build();
    let entry = EnvEntry {
        row: row.clone(),
        key,
        value,
    };
    let list_d = list.clone();
    let entries_d = entries.clone();
    let row_d = row.clone();
    del.connect_clicked(move |_| {
        list_d.remove(&row_d);
        entries_d.borrow_mut().retain(|e| e.row != row_d);
    });
    entry
}

fn make_env_builder(title: &str, initial: &[String]) -> (adw::PreferencesGroup, EnvBuilder) {
    let entries = Rc::new(RefCell::new(Vec::new()));
    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    let builder = EnvBuilder {
        list: list.clone(),
        entries: entries.clone(),
    };
    builder.set_values(initial);
    let add = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Add env var")
        .css_classes(["flat"])
        .build();
    let b = builder.clone();
    add.connect_clicked(move |_| b.add_value(""));
    let group = adw::PreferencesGroup::builder()
        .title(title)
        .description("Set environment variables as KEY + value.")
        .build();
    group.set_header_suffix(Some(&add));
    group.add(&list);
    (group, builder)
}

#[derive(Clone)]
struct MonitorEntry {
    row: gtk::ListBoxRow,
    output: gtk::Entry,
    mode: gtk::Entry,
    position: gtk::Entry,
    scale: gtk::SpinButton,
    disabled: gtk::Switch,
}

impl MonitorEntry {
    fn value(&self) -> Option<String> {
        let output = self.output.text().trim().to_string();
        if output.is_empty() {
            return None;
        }
        if self.disabled.is_active() {
            return Some(format!("{},disabled", output));
        }
        let mode = self.mode.text().trim().to_string();
        let mode = if mode.is_empty() {
            "preferred".to_string()
        } else {
            mode
        };
        let pos = self.position.text().trim().to_string();
        let pos = if pos.is_empty() {
            "0x0".to_string()
        } else {
            pos
        };
        Some(format!(
            "{},{},{},{:.1}",
            output,
            mode,
            pos,
            self.scale.value()
        ))
    }
}

#[derive(Clone)]
struct MonitorBuilder {
    list: gtk::ListBox,
    entries: Rc<RefCell<Vec<MonitorEntry>>>,
}

impl MonitorBuilder {
    fn values(&self) -> Vec<String> {
        self.entries
            .borrow()
            .iter()
            .filter_map(|e| e.value())
            .collect()
    }
    fn set_values(&self, values: &[String]) {
        self.entries.borrow_mut().clear();
        clear_listbox(&self.list);
        let source = if values.is_empty() {
            vec![String::new()]
        } else {
            values.to_vec()
        };
        for val in source {
            self.add_value(&val);
        }
    }
    fn add_value(&self, value: &str) {
        let entry = make_monitor_entry(value, self.list.clone(), self.entries.clone());
        self.list.append(&entry.row);
        self.entries.borrow_mut().push(entry);
    }
}

fn make_monitor_entry(
    initial: &str,
    list: gtk::ListBox,
    entries: Rc<RefCell<Vec<MonitorEntry>>>,
) -> MonitorEntry {
    let parts: Vec<&str> = initial.split(',').map(str::trim).collect();
    let output = tiny_entry(
        parts.first().copied().unwrap_or(""),
        "DP-1 / HDMI-A-1",
        false,
    );
    output.set_width_chars(16);
    let disabled = switch(parts.get(1).copied().unwrap_or("") == "disabled");
    let mode = tiny_entry(
        parts
            .get(1)
            .filter(|v| **v != "disabled")
            .copied()
            .unwrap_or("preferred"),
        "1920x1080@144",
        false,
    );
    mode.set_width_chars(18);
    let position = tiny_entry(parts.get(2).copied().unwrap_or("0x0"), "0x0", false);
    position.set_width_chars(10);
    let scale = spin(
        parts.get(3).and_then(|v| v.parse().ok()).unwrap_or(1.0),
        0.25,
        4.0,
        0.25,
    );
    scale.set_digits(2);
    let del = pill_delete_button();
    let row_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(12)
        .margin_end(12)
        .build();
    row_box.append(&output);
    row_box.append(&mode);
    row_box.append(&position);
    row_box.append(&scale);
    row_box.append(
        &gtk::Label::builder()
            .label("off")
            .css_classes(["dim-label"])
            .build(),
    );
    row_box.append(&disabled);
    row_box.append(&del);
    let row = gtk::ListBoxRow::builder()
        .child(&row_box)
        .activatable(false)
        .selectable(false)
        .build();
    let entry = MonitorEntry {
        row: row.clone(),
        output,
        mode,
        position,
        scale,
        disabled,
    };
    let list_d = list.clone();
    let entries_d = entries.clone();
    let row_d = row.clone();
    del.connect_clicked(move |_| {
        list_d.remove(&row_d);
        entries_d.borrow_mut().retain(|e| e.row != row_d);
    });
    entry
}

fn make_monitor_builder(
    title: &str,
    initial: &[String],
) -> (adw::PreferencesGroup, MonitorBuilder) {
    let entries = Rc::new(RefCell::new(Vec::new()));
    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    let builder = MonitorBuilder {
        list: list.clone(),
        entries: entries.clone(),
    };
    builder.set_values(initial);
    let add = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Add monitor")
        .css_classes(["flat"])
        .build();
    let b = builder.clone();
    add.connect_clicked(move |_| b.add_value(""));
    let group = adw::PreferencesGroup::builder()
        .title(title)
        .description("Output, resolution/refresh, position, scale and disable toggle.")
        .build();
    group.set_header_suffix(Some(&add));
    group.add(&list);
    (group, builder)
}

#[derive(Clone)]
struct WorkspaceEntry {
    row: gtk::ListBoxRow,
    workspace: gtk::Entry,
    monitor: gtk::Entry,
    name: gtk::Entry,
    persistent: gtk::Switch,
}
impl WorkspaceEntry {
    fn value(&self) -> Option<String> {
        let ws = self.workspace.text().trim().to_string();
        if ws.is_empty() {
            return None;
        }
        Some(format!(
            "{}|{}|{}|{}",
            ws,
            self.monitor.text().trim(),
            self.name.text().trim(),
            if self.persistent.is_active() {
                "true"
            } else {
                "false"
            }
        ))
    }
}
#[derive(Clone)]
struct WorkspaceBuilder {
    list: gtk::ListBox,
    entries: Rc<RefCell<Vec<WorkspaceEntry>>>,
}
impl WorkspaceBuilder {
    fn values(&self) -> Vec<String> {
        self.entries
            .borrow()
            .iter()
            .filter_map(|e| e.value())
            .collect()
    }
    fn set_values(&self, values: &[String]) {
        self.entries.borrow_mut().clear();
        clear_listbox(&self.list);
        let source = if values.is_empty() {
            vec![String::new()]
        } else {
            values.to_vec()
        };
        for v in source {
            self.add_value(&v);
        }
    }
    fn add_value(&self, value: &str) {
        let entry = make_workspace_entry(value, self.list.clone(), self.entries.clone());
        self.list.append(&entry.row);
        self.entries.borrow_mut().push(entry);
    }
}
fn make_workspace_entry(
    initial: &str,
    list: gtk::ListBox,
    entries: Rc<RefCell<Vec<WorkspaceEntry>>>,
) -> WorkspaceEntry {
    let p: Vec<&str> = initial.split('|').map(str::trim).collect();
    let workspace = tiny_entry(p.first().copied().unwrap_or(""), "1", false);
    workspace.set_width_chars(8);
    let monitor = tiny_entry(p.get(1).copied().unwrap_or(""), "monitor", false);
    monitor.set_width_chars(14);
    let name = tiny_entry(p.get(2).copied().unwrap_or(""), "name", true);
    let persistent = switch(matches!(p.get(3).copied(), Some("true") | Some("1")));
    let del = pill_delete_button();
    let row_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(12)
        .margin_end(12)
        .build();
    row_box.append(&workspace);
    row_box.append(&monitor);
    row_box.append(&name);
    row_box.append(
        &gtk::Label::builder()
            .label("persistent")
            .css_classes(["dim-label"])
            .build(),
    );
    row_box.append(&persistent);
    row_box.append(&del);
    let row = gtk::ListBoxRow::builder()
        .child(&row_box)
        .activatable(false)
        .selectable(false)
        .build();
    let entry = WorkspaceEntry {
        row: row.clone(),
        workspace,
        monitor,
        name,
        persistent,
    };
    let list_d = list.clone();
    let entries_d = entries.clone();
    let row_d = row.clone();
    del.connect_clicked(move |_| {
        list_d.remove(&row_d);
        entries_d.borrow_mut().retain(|e| e.row != row_d);
    });
    entry
}
fn make_workspace_builder(
    title: &str,
    initial: &[String],
) -> (adw::PreferencesGroup, WorkspaceBuilder) {
    let entries = Rc::new(RefCell::new(Vec::new()));
    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    let builder = WorkspaceBuilder {
        list: list.clone(),
        entries: entries.clone(),
    };
    builder.set_values(initial);
    let add = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Add workspace")
        .css_classes(["flat"])
        .build();
    let b = builder.clone();
    add.connect_clicked(move |_| b.add_value(""));
    let group = adw::PreferencesGroup::builder()
        .title(title)
        .description("Workspace number, monitor, optional name and persistence.")
        .build();
    group.set_header_suffix(Some(&add));
    group.add(&list);
    (group, builder)
}

const MOUSE_ACTIONS: [&str; 4] = ["movewindow", "resizewindow", "togglefloating", "fullscreen"];
#[derive(Clone)]
struct MouseBindEntry {
    row: gtk::ListBoxRow,
    super_btn: gtk::ToggleButton,
    shift_btn: gtk::ToggleButton,
    ctrl_btn: gtk::ToggleButton,
    alt_btn: gtk::ToggleButton,
    button: gtk::DropDown,
    action: gtk::DropDown,
}
impl MouseBindEntry {
    fn value(&self) -> Option<String> {
        let mods = selected_mods(
            &self.super_btn,
            &self.shift_btn,
            &self.ctrl_btn,
            &self.alt_btn,
        );
        if mods.is_empty() {
            return None;
        }
        let buttons = [
            "mouse:272",
            "mouse:273",
            "mouse:274",
            "mouse:275",
            "mouse:276",
        ];
        let btn = buttons
            .get(self.button.selected() as usize)
            .copied()
            .unwrap_or("mouse:272");
        let action = MOUSE_ACTIONS
            .get(self.action.selected() as usize)
            .copied()
            .unwrap_or("movewindow");
        Some(format!("{}, {}, {}", mods, btn, action))
    }
}
#[derive(Clone)]
struct MouseBindBuilder {
    list: gtk::ListBox,
    entries: Rc<RefCell<Vec<MouseBindEntry>>>,
}
impl MouseBindBuilder {
    fn values(&self) -> Vec<String> {
        self.entries
            .borrow()
            .iter()
            .filter_map(|e| e.value())
            .collect()
    }
    fn set_values(&self, values: &[String]) {
        self.entries.borrow_mut().clear();
        clear_listbox(&self.list);
        let source = if values.is_empty() {
            vec![String::new()]
        } else {
            values.to_vec()
        };
        for v in source {
            self.add_value(&v);
        }
    }
    fn add_value(&self, value: &str) {
        let entry = make_mouse_bind_entry(value, self.list.clone(), self.entries.clone());
        self.list.append(&entry.row);
        self.entries.borrow_mut().push(entry);
    }
}
fn make_mouse_bind_entry(
    initial: &str,
    list: gtk::ListBox,
    entries: Rc<RefCell<Vec<MouseBindEntry>>>,
) -> MouseBindEntry {
    let p: Vec<&str> = initial.split(',').map(str::trim).collect();
    let mods = mods_from_text(p.first().copied().unwrap_or("SUPER"));
    let (super_btn, shift_btn, ctrl_btn, alt_btn, mods_box) = make_mod_buttons(&mods);
    let buttons = gtk::StringList::new(
        [
            "mouse:272 left",
            "mouse:273 right",
            "mouse:274 middle",
            "mouse:275 back",
            "mouse:276 forward",
        ]
        .as_slice(),
    );
    let button_idx = match p.get(1).copied().unwrap_or("mouse:272") {
        "mouse:273" => 1,
        "mouse:274" => 2,
        "mouse:275" => 3,
        "mouse:276" => 4,
        _ => 0,
    };
    let button = gtk::DropDown::builder()
        .model(&buttons)
        .selected(button_idx)
        .build();
    let actions = gtk::StringList::new(MOUSE_ACTIONS.as_slice());
    let action_idx = MOUSE_ACTIONS
        .iter()
        .position(|a| Some(*a) == p.get(2).copied())
        .unwrap_or(0) as u32;
    let action = gtk::DropDown::builder()
        .model(&actions)
        .selected(action_idx)
        .build();
    let del = pill_delete_button();
    let row_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(12)
        .margin_end(12)
        .build();
    row_box.append(&mods_box);
    row_box.append(&button);
    row_box.append(&action);
    row_box.append(&del);
    let row = gtk::ListBoxRow::builder()
        .child(&row_box)
        .activatable(false)
        .selectable(false)
        .build();
    let entry = MouseBindEntry {
        row: row.clone(),
        super_btn,
        shift_btn,
        ctrl_btn,
        alt_btn,
        button,
        action,
    };
    let list_d = list.clone();
    let entries_d = entries.clone();
    let row_d = row.clone();
    del.connect_clicked(move |_| {
        list_d.remove(&row_d);
        entries_d.borrow_mut().retain(|e| e.row != row_d);
    });
    entry
}
fn make_mouse_bind_builder(
    title: &str,
    initial: &[String],
) -> (adw::PreferencesGroup, MouseBindBuilder) {
    let entries = Rc::new(RefCell::new(Vec::new()));
    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    let builder = MouseBindBuilder {
        list: list.clone(),
        entries: entries.clone(),
    };
    builder.set_values(initial);
    let add = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Add mouse bind")
        .css_classes(["flat"])
        .build();
    let b = builder.clone();
    add.connect_clicked(move |_| b.add_value("SUPER, mouse:272, movewindow"));
    let group = adw::PreferencesGroup::builder()
        .title(title)
        .description("Choose modifier, mouse button and action.")
        .build();
    group.set_header_suffix(Some(&add));
    group.add(&list);
    (group, builder)
}

const INPUT_SETTINGS: [&str; 7] = [
    "repeat_rate",
    "repeat_delay",
    "numlock_by_default",
    "follow_mouse",
    "float_switch_override_focus",
    "touchpad.scroll_factor",
    "touchpad.disable_while_typing",
];
#[derive(Clone)]
struct InputExtraEntry {
    row: gtk::ListBoxRow,
    setting: gtk::DropDown,
    value: gtk::Entry,
}
impl InputExtraEntry {
    fn value(&self) -> Option<String> {
        let setting = INPUT_SETTINGS
            .get(self.setting.selected() as usize)
            .copied()
            .unwrap_or("repeat_rate");
        let val = self.value.text().trim().to_string();
        if val.is_empty() {
            return None;
        }
        let lua_val = if val == "true" || val == "false" || val.parse::<f64>().is_ok() {
            val
        } else {
            format!("\"{}\"", lua_string(&val))
        };
        if let Some(child) = setting.strip_prefix("touchpad.") {
            Some(format!(
                "hl.config({{ input = {{ touchpad = {{ {} = {} }} }} }})",
                child, lua_val
            ))
        } else {
            Some(format!(
                "hl.config({{ input = {{ {} = {} }} }})",
                setting, lua_val
            ))
        }
    }
}
#[derive(Clone)]
struct InputExtraBuilder {
    list: gtk::ListBox,
    entries: Rc<RefCell<Vec<InputExtraEntry>>>,
}
impl InputExtraBuilder {
    fn values(&self) -> Vec<String> {
        self.entries
            .borrow()
            .iter()
            .filter_map(|e| e.value())
            .collect()
    }
    fn set_values(&self, values: &[String]) {
        self.entries.borrow_mut().clear();
        clear_listbox(&self.list);
        let source = if values.is_empty() {
            vec![String::new()]
        } else {
            values.to_vec()
        };
        for v in source {
            let expanded = expand_input_extra_line(&v);
            if expanded.is_empty() {
                self.add_value(&v);
            } else {
                for item in expanded {
                    self.add_value(&item);
                }
            }
        }
    }
    fn add_value(&self, value: &str) {
        let entry = make_input_extra_entry(value, self.list.clone(), self.entries.clone());
        self.list.append(&entry.row);
        self.entries.borrow_mut().push(entry);
    }
}
fn lua_field_value(line: &str, field: &str) -> String {
    let Some(after_field) = line.split(field).nth(1) else {
        return String::new();
    };
    let Some(after_eq) = after_field.split('=').nth(1) else {
        return String::new();
    };
    after_eq
        .split([',', '}'])
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches('"')
        .to_string()
}

fn expand_input_extra_line(line: &str) -> Vec<String> {
    INPUT_SETTINGS
        .iter()
        .filter_map(|key| {
            let plain = key.strip_prefix("touchpad.").unwrap_or(key);
            if line.contains(plain) {
                let val = lua_field_value(line, plain);
                if val.is_empty() {
                    None
                } else {
                    Some(format!("{} = {}", plain, val))
                }
            } else {
                None
            }
        })
        .collect()
}

fn parse_input_extra(initial: &str) -> (&'static str, String) {
    for key in INPUT_SETTINGS {
        let plain = key.strip_prefix("touchpad.").unwrap_or(key);
        if initial.contains(plain) {
            return (key, lua_field_value(initial, plain));
        }
    }
    ("repeat_rate", String::new())
}
fn make_input_extra_entry(
    initial: &str,
    list: gtk::ListBox,
    entries: Rc<RefCell<Vec<InputExtraEntry>>>,
) -> InputExtraEntry {
    let (setting_name, value_str) = parse_input_extra(initial);
    let model = gtk::StringList::new(INPUT_SETTINGS.as_slice());
    let idx = INPUT_SETTINGS
        .iter()
        .position(|s| *s == setting_name)
        .unwrap_or(0) as u32;
    let setting = gtk::DropDown::builder().model(&model).selected(idx).build();
    let value = tiny_entry(&value_str, "value", true);
    let del = pill_delete_button();
    let row_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(12)
        .margin_end(12)
        .build();
    row_box.append(&setting);
    row_box.append(&value);
    row_box.append(&del);
    let row = gtk::ListBoxRow::builder()
        .child(&row_box)
        .activatable(false)
        .selectable(false)
        .build();
    let entry = InputExtraEntry {
        row: row.clone(),
        setting,
        value,
    };
    let list_d = list.clone();
    let entries_d = entries.clone();
    let row_d = row.clone();
    del.connect_clicked(move |_| {
        list_d.remove(&row_d);
        entries_d.borrow_mut().retain(|e| e.row != row_d);
    });
    entry
}
fn make_input_extra_builder(
    title: &str,
    initial: &[String],
) -> (adw::PreferencesGroup, InputExtraBuilder) {
    let entries = Rc::new(RefCell::new(Vec::new()));
    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    let builder = InputExtraBuilder {
        list: list.clone(),
        entries: entries.clone(),
    };
    builder.set_values(initial);
    let add = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Add input setting")
        .css_classes(["flat"])
        .build();
    let b = builder.clone();
    add.connect_clicked(move |_| b.add_value(""));
    let group = adw::PreferencesGroup::builder()
        .title(title)
        .description("Extra input settings without writing Lua manually.")
        .build();
    group.set_header_suffix(Some(&add));
    group.add(&list);
    (group, builder)
}

const GESTURE_ACTIONS: [&str; 5] = [
    "workspace",
    "movefocus",
    "resizewindow",
    "movewindow",
    "custom",
];
#[derive(Clone)]
struct GestureEntry {
    row: gtk::ListBoxRow,
    fingers: gtk::SpinButton,
    direction: gtk::DropDown,
    action: gtk::DropDown,
    value: gtk::Entry,
}
impl GestureEntry {
    fn value(&self) -> Option<String> {
        let dirs = ["horizontal", "vertical", "left", "right", "up", "down"];
        let direction = dirs
            .get(self.direction.selected() as usize)
            .copied()
            .unwrap_or("horizontal");
        let action = GESTURE_ACTIONS
            .get(self.action.selected() as usize)
            .copied()
            .unwrap_or("workspace");
        let text = self.value.text();
        let val = text.trim();
        if action == "custom" {
            if val.is_empty() {
                return None;
            }
            return Some(val.to_string());
        }
        let value_part = if val.is_empty() {
            String::new()
        } else {
            format!(", value = \"{}\"", lua_string(val))
        };
        Some(format!(
            "hl.gesture({{ fingers = {}, direction = \"{}\", action = \"{}\"{} }})",
            self.fingers.value() as u32,
            direction,
            action,
            value_part
        ))
    }
}
#[derive(Clone)]
struct GestureBuilder {
    list: gtk::ListBox,
    entries: Rc<RefCell<Vec<GestureEntry>>>,
}
impl GestureBuilder {
    fn values(&self) -> Vec<String> {
        self.entries
            .borrow()
            .iter()
            .filter_map(|e| e.value())
            .collect()
    }
    fn set_values(&self, values: &[String]) {
        self.entries.borrow_mut().clear();
        clear_listbox(&self.list);
        let source = if values.is_empty() {
            vec![String::new()]
        } else {
            values.to_vec()
        };
        for v in source {
            self.add_value(&v);
        }
    }
    fn add_value(&self, value: &str) {
        let entry = make_gesture_entry(value, self.list.clone(), self.entries.clone());
        self.list.append(&entry.row);
        self.entries.borrow_mut().push(entry);
    }
}
fn extract_between<'a>(s: &'a str, key: &str, default: &'a str) -> String {
    let Some(after) = s.split(key).nth(1) else {
        return default.to_string();
    };
    after
        .split('=')
        .nth(1)
        .unwrap_or(default)
        .trim()
        .trim_matches(',')
        .trim()
        .trim_matches('}')
        .trim()
        .trim_matches('"')
        .to_string()
}
fn make_gesture_entry(
    initial: &str,
    list: gtk::ListBox,
    entries: Rc<RefCell<Vec<GestureEntry>>>,
) -> GestureEntry {
    let fingers = spin(
        extract_between(initial, "fingers", "3")
            .parse()
            .unwrap_or(3.0),
        2.0,
        5.0,
        1.0,
    );
    let dirs = ["horizontal", "vertical", "left", "right", "up", "down"];
    let dir = extract_between(initial, "direction", "horizontal");
    let dir_idx = dirs.iter().position(|d| *d == dir).unwrap_or(0) as u32;
    let direction_model = gtk::StringList::new(dirs.as_slice());
    let direction = gtk::DropDown::builder()
        .model(&direction_model)
        .selected(dir_idx)
        .build();
    let action_name = extract_between(initial, "action", "workspace");
    let action_idx = GESTURE_ACTIONS
        .iter()
        .position(|a| *a == action_name)
        .unwrap_or(
            if initial.trim().starts_with("hl.") && !initial.contains("hl.gesture") {
                4
            } else {
                0
            },
        ) as u32;
    let action_model = gtk::StringList::new(GESTURE_ACTIONS.as_slice());
    let action = gtk::DropDown::builder()
        .model(&action_model)
        .selected(action_idx)
        .build();
    let value = tiny_entry(
        if action_idx == 4 { initial } else { "" },
        "value / raw Lua when custom",
        true,
    );
    let del = pill_delete_button();
    let row_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(12)
        .margin_end(12)
        .build();
    row_box.append(&fingers);
    row_box.append(&direction);
    row_box.append(&action);
    row_box.append(&value);
    row_box.append(&del);
    let row = gtk::ListBoxRow::builder()
        .child(&row_box)
        .activatable(false)
        .selectable(false)
        .build();
    let entry = GestureEntry {
        row: row.clone(),
        fingers,
        direction,
        action,
        value,
    };
    let list_d = list.clone();
    let entries_d = entries.clone();
    let row_d = row.clone();
    del.connect_clicked(move |_| {
        list_d.remove(&row_d);
        entries_d.borrow_mut().retain(|e| e.row != row_d);
    });
    entry
}
fn make_gesture_builder(
    title: &str,
    initial: &[String],
) -> (adw::PreferencesGroup, GestureBuilder) {
    let entries = Rc::new(RefCell::new(Vec::new()));
    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    let builder = GestureBuilder {
        list: list.clone(),
        entries: entries.clone(),
    };
    builder.set_values(initial);
    let add = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Add gesture")
        .css_classes(["flat"])
        .build();
    let b = builder.clone();
    add.connect_clicked(move |_| b.add_value(""));
    let group = adw::PreferencesGroup::builder()
        .title(title)
        .description("Gesture rows with fingers, direction and action.")
        .build();
    group.set_header_suffix(Some(&add));
    group.add(&list);
    (group, builder)
}

fn make_text_rows(
    title: &str,
    description: &str,
    placeholder: &str,
    initial: &[String],
) -> (adw::PreferencesGroup, TextRows, gtk::Button) {
    let entries: Rc<RefCell<Vec<gtk::Entry>>> = Rc::new(RefCell::new(Vec::new()));

    let box_root = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(10)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let header = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();

    let hint = gtk::Label::builder()
        .label(description)
        .css_classes(["dim-label"])
        .wrap(true)
        .halign(gtk::Align::Start)
        .hexpand(true)
        .build();

    let add_btn = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Add row")
        .css_classes(["flat"])
        .valign(gtk::Align::Center)
        .build();

    header.append(&hint);
    header.append(&add_btn);

    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();

    if initial.is_empty() {
        add_text_row(&list, entries.clone(), "", placeholder);
    } else {
        for value in initial {
            add_text_row(&list, entries.clone(), value, placeholder);
        }
    }

    let list_add = list.clone();
    let entries_add = entries.clone();
    let placeholder_add = placeholder.to_string();
    add_btn.connect_clicked(move |_| {
        let entry = add_text_row(&list_add, entries_add.clone(), "", &placeholder_add);
        entry.grab_focus();
    });

    box_root.append(&header);
    box_root.append(&list);

    let group = adw::PreferencesGroup::builder()
        .title(title)
        .description(description)
        .build();
    group.add(&box_root);

    (group, TextRows { list, entries }, add_btn)
}

fn copy_if_exists(from: &std::path::PathBuf, to: &std::path::PathBuf) -> Result<(), String> {
    if !from.exists() {
        return Ok(());
    }
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::copy(from, to).map(|_| ()).map_err(|e| e.to_string())
}

fn create_safe_snapshot() -> Result<(), String> {
    fs::create_dir_all(safe_backup_dir()).map_err(|e| e.to_string())?;
    copy_if_exists(&config_path(), &safe_path("config.json"))?;
    copy_if_exists(&lua_config_path(), &safe_path("configurator-settings.lua"))?;
    copy_if_exists(&hyprland_lua_path(), &safe_path("hyprland.lua"))?;
    Ok(())
}

fn restore_safe_snapshot() -> Result<(), String> {
    copy_if_exists(&safe_path("config.json"), &config_path())?;
    copy_if_exists(&safe_path("configurator-settings.lua"), &lua_config_path())?;
    copy_if_exists(&safe_path("hyprland.lua"), &hyprland_lua_path())?;
    Ok(())
}

fn restore_bak_files() -> Result<(), String> {
    for path in [config_path(), lua_config_path(), hyprland_lua_path()] {
        let bak = path.with_extension("bak");
        if bak.exists() {
            fs::copy(&bak, &path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn preset_file(name: &str) -> std::path::PathBuf {
    let safe = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>();
    let mut p = presets_dir();
    p.push(format!(
        "{}.json",
        if safe.is_empty() { "preset" } else { &safe }
    ));
    p
}

fn save_named_preset(name: &str, cfg: &HyprConfig) -> Result<(), String> {
    fs::create_dir_all(presets_dir()).map_err(|e| e.to_string())?;
    fs::write(
        preset_file(name),
        serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn load_named_preset(name: &str) -> Result<HyprConfig, String> {
    serde_json::from_str(&fs::read_to_string(preset_file(name)).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

fn list_named_presets() -> String {
    fs::read_dir(presets_dir())
        .ok()
        .into_iter()
        .flat_map(|it| it.flatten())
        .filter_map(|e| {
            e.path()
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn delete_named_preset(name: &str) -> Result<(), String> {
    let p = preset_file(name);
    if p.exists() {
        fs::remove_file(p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn is_old_conf_format(line: &str) -> bool {
    // old conf-format lines start with a rule keyword or selector pattern
    let t = line.trim();
    t.starts_with("float,")
        || t.starts_with("center,")
        || t.starts_with("size ")
        || t.starts_with("opacity ")
        || t.starts_with("workspace ")
        || t.starts_with("bind")
        || t.starts_with("exec")
        || t.starts_with("monitor")
        || t.starts_with("windowrulev2")
        || (t.contains(" = ") && !t.contains("hl.") && !t.starts_with("--"))
}

fn read_config() -> HyprConfig {
    let path = config_path();
    if !path.exists() {
        return HyprConfig::default();
    }
    let mut cfg: HyprConfig =
        serde_json::from_str(&fs::read_to_string(path).unwrap_or_default()).unwrap_or_default();
    // migrate: drop any raw lines that are in old .conf format (not valid Lua)
    for field in [
        &mut cfg.window_rules,
        &mut cfg.workspace_rules,
        &mut cfg.keybinds,
        &mut cfg.animation_lines,
        &mut cfg.decoration_lines,
        &mut cfg.input_extra_lines,
        &mut cfg.gesture_lines,
        &mut cfg.custom_lines,
    ] {
        field.retain(|l| !is_old_conf_format(l));
    }
    cfg
}

fn backup_file(path: &std::path::PathBuf) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    let backup = path.with_extension("bak");
    fs::copy(path, backup)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn ensure_require_injected() -> Result<(), String> {
    let path = hyprland_lua_path();
    if !path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(&path).unwrap_or_default();
    if !content.contains("configurator-settings") {
        backup_file(&path)?;
        let appended = format!(
            "{}\n-- Hypr Configurator\nrequire(\"configurator-settings\")\n",
            content.trim_end()
        );
        fs::write(&path, appended).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn validate_config(config: &HyprConfig) -> Result<(), String> {
    let is_hypr_color =
        |value: &str| value.starts_with("rgba(") && value.ends_with(')') && value.len() == 14;
    if !is_hypr_color(&config.active_border_color) {
        return Err("Active border color is invalid".to_string());
    }
    if !is_hypr_color(&config.inactive_border_color) {
        return Err("Inactive border color is invalid".to_string());
    }
    if !(0.0..=1.0).contains(&config.active_opacity)
        || !(0.0..=1.0).contains(&config.inactive_opacity)
    {
        return Err("Opacity must be between 0.0 and 1.0".to_string());
    }
    if config.layout.trim().is_empty() {
        return Err("Layout cannot be empty".to_string());
    }
    if config.kb_layout.trim().is_empty() {
        return Err("Keyboard layout cannot be empty".to_string());
    }
    Ok(())
}

fn save_config(config: &HyprConfig) -> Result<String, String> {
    validate_config(config)?;

    let json_path = config_path();
    let lua_path = lua_config_path();

    if let Some(parent) = json_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    backup_file(&json_path)?;
    backup_file(&lua_path)?;

    fs::write(
        &json_path,
        serde_json::to_string_pretty(config).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    fs::write(&lua_path, generate_lua(config)).map_err(|e| e.to_string())?;
    ensure_require_injected()?;

    Ok(format!(
        "Saved: {} and {}",
        json_path.display(),
        lua_path.display()
    ))
}

fn dispatcher_to_lua(dispatcher: &str, args: &str) -> String {
    match dispatcher {
        "exec" => format!("hl.dsp.exec_cmd(\"{}\")", lua_string(args)),
        "killactive" | "closewindow" => "hl.dsp.window.close()".to_string(),
        "togglefloating" => "hl.dsp.window.float({ action = \"toggle\" })".to_string(),
        "fullscreen" => "hl.dsp.window.fullscreen()".to_string(),
        "pseudo" => "hl.dsp.window.pseudo()".to_string(),
        "togglesplit" => "hl.dsp.layout(\"togglesplit\")".to_string(),
        "workspace" => {
            if args.parse::<i64>().is_ok() {
                format!("hl.dsp.focus({{ workspace = {} }})", args)
            } else {
                format!("hl.dsp.focus({{ workspace = \"{}\" }})", args)
            }
        }
        "movetoworkspace" | "movetoworkspacesilent" => {
            if args.parse::<i64>().is_ok() {
                format!("hl.dsp.window.move({{ workspace = {} }})", args)
            } else {
                format!("hl.dsp.window.move({{ workspace = \"{}\" }})", args)
            }
        }
        "movefocus" => format!("hl.dsp.focus({{ direction = \"{}\" }})", args),
        "movewindow" => match args {
            "l" | "left" => "hl.dsp.window.move({ direction = \"left\" })".to_string(),
            "r" | "right" => "hl.dsp.window.move({ direction = \"right\" })".to_string(),
            "u" | "up" => "hl.dsp.window.move({ direction = \"up\" })".to_string(),
            "d" | "down" => "hl.dsp.window.move({ direction = \"down\" })".to_string(),
            _ if args.parse::<i64>().is_ok() => {
                format!("hl.dsp.window.move({{ workspace = {} }})", args)
            }
            _ => format!("hl.dsp.window.move({{ workspace = \"{}\" }})", args),
        },
        "exit" => "hl.dsp.exit()".to_string(),
        "togglespecialworkspace" => {
            if args.is_empty() {
                "hl.dsp.workspace.toggle_special()".to_string()
            } else {
                format!("hl.dsp.workspace.toggle_special(\"{}\")", args)
            }
        }
        "pin" => "hl.dsp.window.pin()".to_string(),
        _ if !args.is_empty() => format!("hl.dsp.{}(\"{}\")", dispatcher, lua_string(args)),
        _ => format!("hl.dsp.{}()", dispatcher),
    }
}

fn lua_monitors(lines: &[String]) -> String {
    lines
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let mut parts = line.splitn(4, ',').map(str::trim);
            let output = parts.next()?;
            if output.is_empty() {
                return None;
            }
            let mode = parts.next().unwrap_or("preferred");
            if mode == "disabled" {
                return Some(format!(
                    "hl.monitor({{ output = \"{output}\", disabled = true }})"
                ));
            }
            let position = parts.next().unwrap_or("0x0");
            let scale: f64 = parts.next().unwrap_or("1").parse().unwrap_or(1.0);
            Some(format!(
                "hl.monitor({{ output = \"{output}\", mode = \"{mode}\", position = \"{position}\", scale = {scale:.1} }})"
            ))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn lua_envs(lines: &[String]) -> String {
    lines
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let (k, v) = line.split_once(',')?;
            let k = k.trim();
            let v = v.trim();
            if k.is_empty() {
                return None;
            }
            Some(format!("hl.env(\"{k}\", \"{v}\")"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn lua_exec_list(lines: &[String]) -> String {
    lines
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|cmd| format!("hl.exec_cmd(\"{}\")", lua_string(cmd)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn lua_visual_keybinds(lines: &[String]) -> String {
    lines
        .iter()
        .filter_map(|line| {
            let p: Vec<&str> = line.split('|').map(str::trim).collect();
            if p.len() < 3 || p[0].is_empty() || p[1].is_empty() || p[2].is_empty() {
                return None;
            }
            let action = dispatcher_to_lua(p[2], p.get(3).copied().unwrap_or(""));
            Some(format!("hl.bind(\"{} + {}\", {action})", p[0], p[1]))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn lua_mouse_binds(lines: &[String]) -> String {
    lines
        .iter()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(3, ',').map(str::trim).collect();
            if parts.len() < 3 {
                return None;
            }
            let action = match parts[2] {
                "movewindow" => "hl.dsp.window.drag()".to_string(),
                "resizewindow" => "hl.dsp.window.resize()".to_string(),
                d => format!("hl.dsp.{}()", d),
            };
            Some(format!(
                "hl.bind(\"{} + {}\", {action}, {{ mouse = true }})",
                parts[0], parts[1]
            ))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn lua_visual_workspace_rules(lines: &[String]) -> String {
    lines
        .iter()
        .filter_map(|line| {
            let p: Vec<&str> = line.split('|').map(str::trim).collect();
            if p.is_empty() || p[0].is_empty() {
                return None;
            }
            let mut fields = vec![format!("workspace = \"{}\"", p[0])];
            if let Some(v) = p.get(1).filter(|v| !v.is_empty()) {
                fields.push(format!("monitor = \"{v}\""));
            }
            if let Some(v) = p.get(2).filter(|v| !v.is_empty()) {
                fields.push(format!("name = \"{v}\""));
            }
            if let Some(v) = p.get(3).filter(|v| !v.is_empty()) {
                let val = if *v == "true" || *v == "1" {
                    "true"
                } else {
                    "false"
                };
                fields.push(format!("persistent = {val}"));
            }
            Some(format!("hl.workspace_rule({{ {} }})", fields.join(", ")))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn lua_visual_window_rules(lines: &[String]) -> String {
    lines
        .iter()
        .enumerate()
        .filter_map(|(idx, line)| {
            let class = line.split('|').next().unwrap_or("").trim();
            if class.is_empty() {
                return None;
            }
            let safe_class = class.replace('(', r"\(").replace(')', r"\)");
            let mut fields = vec![
                format!("name = \"auto-rule-{idx}\""),
                format!("match = {{ class = \"^{safe_class}$\" }}"),
            ];
            for part in line.split('|').skip(1) {
                let Some((k, v)) = part.split_once('=') else {
                    continue;
                };
                let v = v.trim();
                match k.trim() {
                    "float" if v == "true" || v == "1" => fields.push("float = true".to_string()),
                    "center" if v == "true" || v == "1" => fields.push("center = true".to_string()),
                    "size" if !v.is_empty() => {
                        fields.push(format!("size = \"{}\"", v.replace('x', " ")))
                    }
                    "workspace" if !v.is_empty() => fields.push(format!("workspace = \"{v}\"")),
                    "opacity" if !v.is_empty() => fields.push(format!("opacity = {v}")),
                    _ => {}
                }
            }
            Some(format!("hl.window_rule({{ {} }})", fields.join(", ")))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn default_app_keybind_lines(c: &HyprConfig) -> Vec<String> {
    let mut out = Vec::new();
    if !c.default_terminal.trim().is_empty() {
        out.push(format!(
            "hl.bind(\"SUPER + Return\", hl.dsp.exec_cmd(\"{}\"))",
            lua_string(c.default_terminal.trim())
        ));
    }
    if !c.default_browser.trim().is_empty() {
        out.push(format!(
            "hl.bind(\"SUPER + B\", hl.dsp.exec_cmd(\"{}\"))",
            lua_string(c.default_browser.trim())
        ));
    }
    if !c.default_file_manager.trim().is_empty() {
        out.push(format!(
            "hl.bind(\"SUPER + E\", hl.dsp.exec_cmd(\"{}\"))",
            lua_string(c.default_file_manager.trim())
        ));
    }
    if !c.default_editor.trim().is_empty() {
        out.push(format!(
            "hl.bind(\"SUPER + C\", hl.dsp.exec_cmd(\"{}\"))",
            lua_string(c.default_editor.trim())
        ));
    }
    out
}

fn wallpaper_lines(c: &HyprConfig) -> Vec<String> {
    if c.wallpaper_path.trim().is_empty() {
        return vec![];
    }
    match c.wallpaper_backend.as_str() {
        "hyprpaper" => vec!["hyprpaper".to_string()],
        _ => vec![format!(
            "swaybg -i {} -m {}",
            c.wallpaper_path.trim(),
            c.wallpaper_mode.trim()
        )],
    }
}

fn raw_lines(lines: &[String]) -> String {
    lines
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn raw_lines_multi(groups: &[&Vec<String>]) -> String {
    groups
        .iter()
        .flat_map(|v| v.iter())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn generate_lua(c: &HyprConfig) -> String {
    let b = |v: bool| if v { "true" } else { "false" };

    // startup block: startup_commands + exec_once_commands + wallpaper (run once at start)
    let mut all_startup: Vec<String> = c.startup_commands.clone();
    all_startup.extend(c.exec_once_commands.clone());
    all_startup.extend(wallpaper_lines(c));
    let startup_lines = all_startup
        .iter()
        .map(|cmd| cmd.trim())
        .filter(|cmd| !cmd.is_empty())
        .map(|cmd| format!("  hl.exec_cmd(\"{}\")", lua_string(cmd)))
        .collect::<Vec<_>>()
        .join("\n");
    let startup_block = if startup_lines.is_empty() {
        String::new()
    } else {
        format!(
            "hl.on(\"hyprland.start\", function ()\n{}\nend)\n\n",
            startup_lines
        )
    };

    // top-level exec (runs on every reload)
    let exec_block = lua_exec_list(&c.exec_commands);

    let mut extra: Vec<String> = Vec::new();

    macro_rules! push_block {
        ($b:expr) => {
            let s = $b;
            if !s.is_empty() {
                extra.push(s);
            }
        };
    }

    push_block!(lua_monitors(&c.monitor_rules));
    push_block!(lua_envs(&c.env_vars));
    push_block!(lua_visual_workspace_rules(&c.visual_workspaces));
    push_block!(raw_lines(&c.workspace_rules));
    push_block!(lua_visual_window_rules(&c.visual_window_rules));
    push_block!(raw_lines(&c.window_rules));
    push_block!(lua_visual_keybinds(&c.visual_keybinds));
    push_block!(default_app_keybind_lines(c).join("\n"));
    push_block!(lua_mouse_binds(&c.mouse_binds));
    push_block!(raw_lines(&c.keybinds));
    push_block!(raw_lines_multi(
        &[
            &c.animation_lines,
            &c.decoration_lines,
            &c.input_extra_lines,
            &c.gesture_lines,
        ][..]
    ));
    push_block!(raw_lines(&c.custom_lines));

    let extra_block = if extra.is_empty() {
        String::new()
    } else {
        format!("\n{}\n", extra.join("\n\n"))
    };

    let exec_prefix = if exec_block.is_empty() {
        String::new()
    } else {
        format!("{exec_block}\n\n")
    };

    format!(
        r#"-- Generated by Hypr Configurator - do not edit manually

{}{exec_prefix}hl.config({{
  general = {{
    gaps_in = {},
    gaps_out = {},
    border_size = {},
    col = {{
      active_border = "{}",
      inactive_border = "{}",
    }},
    resize_on_border = {},
    allow_tearing = {},
    layout = "{}",
  }},

  decoration = {{
    rounding = {},
    rounding_power = 2,
    active_opacity = {:.2},
    inactive_opacity = {:.2},
    shadow = {{
      enabled = {},
      range = {},
      render_power = 3,
      color = 0xee1a1a1a,
    }},
    blur = {{
      enabled = {},
      size = {},
      passes = {},
      vibrancy = 0.1696,
    }},
  }},

  animations = {{
    enabled = {},
  }},
}})

hl.config({{
  input = {{
    kb_layout = "{}",
    kb_variant = "{}",
    kb_model = "",
    kb_options = "",
    kb_rules = "",
    follow_mouse = 1,
    sensitivity = {:.1},
    touchpad = {{
      natural_scroll = {},
    }},
  }},
}})

hl.config({{
  misc = {{
    force_default_wallpaper = -1,
    disable_hyprland_logo = {},
  }},
}})
{}"#,
        startup_block,
        c.gaps_in,
        c.gaps_out,
        c.border_size,
        c.active_border_color,
        c.inactive_border_color,
        b(c.resize_on_border),
        b(c.allow_tearing),
        c.layout,
        c.rounding,
        c.active_opacity,
        c.inactive_opacity,
        b(c.shadow_enabled),
        c.shadow_range,
        b(c.blur_enabled),
        c.blur_size,
        c.blur_passes,
        b(c.animations_enabled),
        c.kb_layout,
        c.kb_variant,
        c.sensitivity,
        b(c.natural_scroll),
        b(c.disable_hyprland_logo),
        extra_block,
    )
}

fn reload_hyprland() -> Result<(), String> {
    let output = Command::new("hyprctl")
        .arg("reload")
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn command_exists(name: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {} >/dev/null 2>&1", name))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn open_config_folder() {
    if let Some(dir) = config_path().parent() {
        let _ = Command::new("xdg-open").arg(dir).spawn();
    }
}

fn detect_monitor_rules() -> Vec<String> {
    let output = Command::new("hyprctl")
        .args(["-j", "monitors"])
        .output()
        .ok();
    let Some(output) = output else {
        return vec![];
    };
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_default();
    let Some(monitors) = json.as_array() else {
        return vec![];
    };

    monitors
        .iter()
        .filter_map(|m| {
            let name = m["name"].as_str()?;
            let width = m["width"].as_i64().unwrap_or(1920);
            let height = m["height"].as_i64().unwrap_or(1080);
            let refresh = m["refreshRate"].as_f64().unwrap_or(60.0).round() as i64;
            let x = m["x"].as_i64().unwrap_or(0);
            let y = m["y"].as_i64().unwrap_or(0);
            let scale = m["scale"].as_f64().unwrap_or(1.0);
            Some(format!("{name},{width}x{height}@{refresh},{x}x{y},{scale}"))
        })
        .collect()
}

fn detect_window_rules() -> Vec<String> {
    let output = Command::new("hyprctl")
        .args(["-j", "clients"])
        .output()
        .ok();
    let Some(output) = output else {
        return vec![];
    };
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_default();
    let Some(clients) = json.as_array() else {
        return vec![];
    };

    let mut classes = clients
        .iter()
        .filter_map(|c| c["class"].as_str())
        .filter(|class| !class.trim().is_empty())
        .map(|class| {
            format!(
                "float,class:^({})$",
                class.replace('(', r"\(").replace(')', r"\)")
            )
        })
        .collect::<Vec<_>>();
    classes.sort();
    classes.dedup();
    classes
}

fn rgba_to_hex(c: &gtk::gdk::RGBA) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        (c.red() * 255.0).round() as u8,
        (c.green() * 255.0).round() as u8,
        (c.blue() * 255.0).round() as u8,
    )
}

fn hex_to_rgba(hex: &str, alpha: f32) -> Option<gtk::gdk::RGBA> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
    Some(gtk::gdk::RGBA::new(r, g, b, alpha))
}

fn color_preview(color: Rc<RefCell<gtk::gdk::RGBA>>, width: i32, height: i32) -> gtk::DrawingArea {
    let preview = gtk::DrawingArea::builder()
        .width_request(width)
        .height_request(height)
        .valign(gtk::Align::Center)
        .build();

    preview.set_draw_func(move |_, cr, w, h| {
        let c = color.borrow();
        let radius = 10.0;
        let w = w as f64;
        let h = h as f64;
        cr.new_sub_path();
        cr.arc(
            w - radius,
            radius,
            radius,
            -std::f64::consts::FRAC_PI_2,
            0.0,
        );
        cr.arc(
            w - radius,
            h - radius,
            radius,
            0.0,
            std::f64::consts::FRAC_PI_2,
        );
        cr.arc(
            radius,
            h - radius,
            radius,
            std::f64::consts::FRAC_PI_2,
            std::f64::consts::PI,
        );
        cr.arc(
            radius,
            radius,
            radius,
            std::f64::consts::PI,
            std::f64::consts::PI * 1.5,
        );
        cr.close_path();
        cr.set_source_rgba(
            c.red() as f64,
            c.green() as f64,
            c.blue() as f64,
            c.alpha() as f64,
        );
        let _ = cr.fill_preserve();
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.18);
        cr.set_line_width(1.0);
        let _ = cr.stroke();
    });

    preview
}

fn make_color_button(
    initial: gtk::gdk::RGBA,
    parent: &adw::ApplicationWindow,
) -> (gtk::Button, Rc<RefCell<gtk::gdk::RGBA>>) {
    let color = Rc::new(RefCell::new(initial));
    let preview = color_preview(color.clone(), 42, 28);
    let button_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .valign(gtk::Align::Center)
        .build();
    let label = gtk::Label::builder()
        .label(&rgba_to_hex(&color.borrow()))
        .css_classes(["dim-label"])
        .valign(gtk::Align::Center)
        .build();
    button_box.append(&preview);
    button_box.append(&label);
    let button = gtk::Button::builder()
        .child(&button_box)
        .css_classes(["flat"])
        .valign(gtk::Align::Center)
        .build();

    let color_click = color.clone();
    let preview_click = preview.clone();
    let label_click = label.clone();
    let parent_weak = parent.downgrade();

    button.connect_clicked(move |_| {
        let Some(win) = parent_weak.upgrade() else {
            return;
        };
        let dialog = gtk::Window::builder()
            .title("Choose color")
            .transient_for(&win)
            .modal(true)
            .default_width(460)
            .default_height(560)
            .resizable(false)
            .build();

        let temp_color = Rc::new(RefCell::new(*color_click.borrow()));
        let title = gtk::Label::builder()
            .label("Choose color")
            .css_classes(["title-2"])
            .halign(gtk::Align::Start)
            .build();
        let subtitle = gtk::Label::builder()
            .label("Choose a preset color or customize it manually.")
            .css_classes(["dim-label"])
            .wrap(true)
            .halign(gtk::Align::Start)
            .build();
        let big_preview = color_preview(temp_color.clone(), 420, 72);
        let presets = [
            "#FFFFFF", "#999999", "#000000", "#FF5555", "#F97316", "#FACC15", "#22C55E", "#06B6D4",
            "#3B82F6", "#8B5CF6", "#EC4899", "#A855F7",
        ];
        let presets_box = gtk::FlowBox::builder()
            .min_children_per_line(6)
            .max_children_per_line(6)
            .selection_mode(gtk::SelectionMode::None)
            .row_spacing(10)
            .column_spacing(10)
            .halign(gtk::Align::Center)
            .build();
        for preset in presets {
            let rgba = hex_to_rgba(preset, temp_color.borrow().alpha()).unwrap();
            let preset_color = Rc::new(RefCell::new(rgba));
            let preset_preview = color_preview(preset_color, 48, 34);
            let preset_btn = gtk::Button::builder()
                .child(&preset_preview)
                .css_classes(["flat"])
                .tooltip_text(preset)
                .build();
            let temp_color_set = temp_color.clone();
            let big_preview_set = big_preview.clone();
            preset_btn.connect_clicked(move |_| {
                let alpha = temp_color_set.borrow().alpha();
                let mut selected = rgba;
                selected.set_alpha(alpha);
                *temp_color_set.borrow_mut() = selected;
                big_preview_set.queue_draw();
            });
            presets_box.insert(&preset_btn, -1);
        }

        let presets_group = adw::PreferencesGroup::builder()
            .title("Quick colors")
            .description("Click a color to use as base.")
            .build();
        presets_group.add(&presets_box);

        #[allow(deprecated)]
        let chooser = gtk::ColorChooserWidget::new();
        #[allow(deprecated)]
        {
            chooser.set_rgba(&temp_color.borrow());
            chooser.set_use_alpha(true);
        }

        let custom_group = adw::PreferencesGroup::builder()
            .title("Custom")
            .description("Adjust the color manually.")
            .build();
        let chooser_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .margin_top(8)
            .build();
        chooser_box.append(&chooser);
        custom_group.add(&chooser_box);

        let hex_entry = adw::EntryRow::builder()
            .title("HEX")
            .text(&rgba_to_hex(&temp_color.borrow()))
            .build();
        let alpha = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.01);
        alpha.set_value(temp_color.borrow().alpha() as f64);
        alpha.set_hexpand(true);
        alpha.set_draw_value(true);
        alpha.set_digits(2);
        alpha.set_valign(gtk::Align::Center);
        let alpha_row = action_row("Transparency", Some("1.00 = opaque, 0.00 = invisible"));
        alpha_row.add_suffix(&alpha);
        let input_group = adw::PreferencesGroup::builder().title("Values").build();
        input_group.add(&hex_entry);
        input_group.add(&alpha_row);

        let temp_color_hex = temp_color.clone();
        let big_preview_hex = big_preview.clone();
        let chooser_hex = chooser.clone();
        hex_entry.connect_changed(move |entry| {
            let alpha = temp_color_hex.borrow().alpha();
            if let Some(new_color) = hex_to_rgba(&entry.text(), alpha) {
                *temp_color_hex.borrow_mut() = new_color;
                #[allow(deprecated)]
                chooser_hex.set_rgba(&new_color);
                big_preview_hex.queue_draw();
            }
        });

        let temp_color_alpha = temp_color.clone();
        let big_preview_alpha = big_preview.clone();
        let chooser_alpha = chooser.clone();
        alpha.connect_value_changed(move |scale| {
            let mut current = *temp_color_alpha.borrow();
            current.set_alpha(scale.value() as f32);
            *temp_color_alpha.borrow_mut() = current;
            #[allow(deprecated)]
            chooser_alpha.set_rgba(&current);
            big_preview_alpha.queue_draw();
        });

        let temp_color_chooser = temp_color.clone();
        let big_preview_chooser = big_preview.clone();
        let hex_entry_chooser = hex_entry.clone();
        let alpha_chooser = alpha.clone();
        #[allow(deprecated)]
        chooser.connect_rgba_notify(move |c| {
            let new_color = c.rgba();
            *temp_color_chooser.borrow_mut() = new_color;
            hex_entry_chooser.set_text(&rgba_to_hex(&new_color));
            alpha_chooser.set_value(new_color.alpha() as f64);
            big_preview_chooser.queue_draw();
        });

        let cancel_btn = gtk::Button::builder()
            .label("Cancel")
            .css_classes(["pill"])
            .build();
        let apply_btn = gtk::Button::builder()
            .label("Apply")
            .css_classes(["suggested-action", "pill"])
            .build();
        let footer = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .halign(gtk::Align::End)
            .margin_top(6)
            .build();
        footer.append(&cancel_btn);
        footer.append(&apply_btn);

        let stack = gtk::Stack::builder()
            .transition_type(gtk::StackTransitionType::SlideLeftRight)
            .transition_duration(180)
            .vexpand(true)
            .build();
        let main_page = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(18)
            .build();
        let custom_btn = gtk::Button::builder()
            .label("Customize color")
            .css_classes(["pill"])
            .halign(gtk::Align::End)
            .build();
        main_page.append(&title);
        main_page.append(&subtitle);
        main_page.append(&big_preview);
        main_page.append(&presets_group);
        main_page.append(&custom_btn);

        let custom_title_row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .build();
        let back_btn = gtk::Button::builder()
            .label("Back")
            .css_classes(["flat"])
            .build();
        let custom_title = gtk::Label::builder()
            .label("Custom")
            .css_classes(["title-2"])
            .halign(gtk::Align::Start)
            .hexpand(true)
            .build();
        custom_title_row.append(&back_btn);
        custom_title_row.append(&custom_title);
        let custom_page = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(18)
            .build();
        custom_page.append(&custom_title_row);
        custom_page.append(&custom_group);
        custom_page.append(&input_group);
        stack.add_named(&main_page, Some("main"));
        stack.add_named(&custom_page, Some("custom"));
        stack.set_visible_child_name("main");
        let stack_to_custom = stack.clone();
        custom_btn.connect_clicked(move |_| stack_to_custom.set_visible_child_name("custom"));
        let stack_to_main = stack.clone();
        back_btn.connect_clicked(move |_| stack_to_main.set_visible_child_name("main"));

        let content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(18)
            .margin_top(20)
            .margin_bottom(20)
            .margin_start(20)
            .margin_end(20)
            .build();
        content.append(&stack);
        content.append(&footer);
        dialog.set_child(Some(&content));
        let dialog_cancel = dialog.clone();
        cancel_btn.connect_clicked(move |_| dialog_cancel.close());
        let color_apply = color_click.clone();
        let preview_apply = preview_click.clone();
        let label_apply = label_click.clone();
        let dialog_apply = dialog.clone();
        apply_btn.connect_clicked(move |_| {
            let new_color = *temp_color.borrow();
            *color_apply.borrow_mut() = new_color;
            label_apply.set_label(&rgba_to_hex(&new_color));
            preview_apply.queue_draw();
            dialog_apply.close();
        });
        dialog.present();
    });

    (button, color)
}

fn make_window_floating() {
    glib::timeout_add_once(std::time::Duration::from_millis(500), || {
        let output = Command::new("hyprctl")
            .args(["-j", "clients"])
            .output()
            .ok();
        let address = output.and_then(|o| {
            let json: serde_json::Value = serde_json::from_slice(&o.stdout).ok()?;
            json.as_array()?
                .iter()
                .find(|c| c["class"].as_str() == Some("dev.n0t.hyprconfigurator"))
                .and_then(|c| c["address"].as_str().map(String::from))
        });
        if let Some(addr) = address {
            let target = format!("address:{addr}");
            let _ = Command::new("hyprctl")
                .args(["dispatch", "setfloating", &target])
                .output();
            let _ = Command::new("hyprctl")
                .args([
                    "dispatch",
                    "resizewindowpixel",
                    &format!("exact 1280 960,{target}"),
                ])
                .output();
            let _ = Command::new("hyprctl")
                .args(["dispatch", "centerwindow", &target])
                .output();
        }
    });
}

fn hyprland_to_gdk(color: &str) -> gtk::gdk::RGBA {
    let inner = color
        .strip_prefix("rgba(")
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or("");
    if inner.len() == 8 {
        let parse = |s: &str| u8::from_str_radix(s, 16).unwrap_or(0) as f32 / 255.0;
        return gtk::gdk::RGBA::new(
            parse(&inner[0..2]),
            parse(&inner[2..4]),
            parse(&inner[4..6]),
            parse(&inner[6..8]),
        );
    }
    gtk::gdk::RGBA::new(0.6, 0.6, 0.6, 0.93)
}

fn gdk_to_hyprland(c: &gtk::gdk::RGBA) -> String {
    format!(
        "rgba({:02x}{:02x}{:02x}{:02x})",
        (c.red() * 255.0).round() as u8,
        (c.green() * 255.0).round() as u8,
        (c.blue() * 255.0).round() as u8,
        (c.alpha() * 255.0).round() as u8,
    )
}

fn spin(value: f64, min: f64, max: f64, step: f64) -> gtk::SpinButton {
    let s = gtk::SpinButton::with_range(min, max, step);
    s.set_value(value);
    s.set_valign(gtk::Align::Center);
    s
}

fn switch(active: bool) -> gtk::Switch {
    gtk::Switch::builder()
        .active(active)
        .valign(gtk::Align::Center)
        .build()
}

fn action_row(title: &str, subtitle: Option<&str>) -> adw::ActionRow {
    let b = adw::ActionRow::builder().title(title);
    if let Some(sub) = subtitle {
        b.subtitle(sub).build()
    } else {
        b.build()
    }
}

fn switch_row(title: &str, subtitle: Option<&str>, sw: &gtk::Switch) -> adw::ActionRow {
    let row = action_row(title, subtitle);
    row.add_suffix(sw);
    row.set_activatable_widget(Some(sw));
    row
}

fn spin_row(title: &str, subtitle: Option<&str>, sp: &gtk::SpinButton) -> adw::ActionRow {
    let row = action_row(title, subtitle);
    row.add_suffix(sp);
    row
}

fn make_stack_page(title: &str) -> (gtk::ScrolledWindow, gtk::Box) {
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(24)
        .margin_top(20)
        .margin_bottom(28)
        .margin_start(20)
        .margin_end(20)
        .build();

    let title_label = gtk::Label::builder()
        .label(title)
        .css_classes(["title-1"])
        .halign(gtk::Align::Start)
        .build();

    content.append(&title_label);

    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .hexpand(true)
        .child(&content)
        .build();

    (scrolled, content)
}

fn add_sidebar_item(
    list: &gtk::ListBox,
    stack: &gtk::Stack,
    revealer: &gtk::Revealer,
    title: &str,
    page_name: &str,
) {
    let button = gtk::Button::builder()
        .label(title)
        .halign(gtk::Align::Fill)
        .hexpand(true)
        .css_classes(["flat", "sidebar-item"])
        .build();

    let stack_c = stack.clone();
    let revealer_c = revealer.clone();
    let page_name = page_name.to_string();

    button.connect_clicked(move |_| {
        stack_c.set_visible_child_name(&page_name);

        if revealer_c.width() < 240 {
            revealer_c.set_reveal_child(false);
        }
    });

    let row = gtk::ListBoxRow::builder()
        .child(&button)
        .selectable(false)
        .activatable(false)
        .build();

    list.append(&row);
}

fn show_text_window(parent: &adw::ApplicationWindow, title: &str, text: &str) {
    let dialog = gtk::Window::builder()
        .title(title)
        .transient_for(parent)
        .modal(true)
        .default_width(760)
        .default_height(640)
        .build();

    let buffer = gtk::TextBuffer::new(None::<&gtk::TextTagTable>);
    buffer.set_text(text);
    let view = gtk::TextView::builder()
        .buffer(&buffer)
        .editable(false)
        .monospace(true)
        .css_classes(["code-view"])
        .vexpand(true)
        .hexpand(true)
        .build();
    let scrolled = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&view)
        .build();
    let close = gtk::Button::builder()
        .label("Close")
        .css_classes(["pill"])
        .halign(gtk::Align::End)
        .build();
    let root = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();
    root.append(&scrolled);
    root.append(&close);
    dialog.set_child(Some(&root));
    let dialog_c = dialog.clone();
    close.connect_clicked(move |_| dialog_c.close());
    dialog.present();
}

fn preset_config(name: &str, current: &HyprConfig) -> HyprConfig {
    let mut c = current.clone();
    match name {
        "Minimal" => {
            c.gaps_in = 2;
            c.gaps_out = 8;
            c.border_size = 1;
            c.rounding = 8;
            c.blur_enabled = false;
            c.shadow_enabled = false;
            c.animations_enabled = false;
        }
        "Glass" => {
            c.gaps_in = 6;
            c.gaps_out = 22;
            c.border_size = 2;
            c.rounding = 24;
            c.active_opacity = 0.88;
            c.inactive_opacity = 0.78;
            c.blur_enabled = true;
            c.blur_size = 8;
            c.blur_passes = 3;
            c.shadow_enabled = true;
        }
        "Cyberpunk" => {
            c.gaps_in = 5;
            c.gaps_out = 18;
            c.border_size = 3;
            c.rounding = 18;
            c.active_border_color = "rgba(ec4899ff)".to_string();
            c.inactive_border_color = "rgba(3b82f688)".to_string();
            c.blur_enabled = true;
            c.animations_enabled = true;
            c.shadow_enabled = true;
        }
        "Mac-like" => {
            c.gaps_in = 8;
            c.gaps_out = 24;
            c.border_size = 1;
            c.rounding = 16;
            c.blur_enabled = true;
            c.shadow_enabled = true;
            c.animations_enabled = true;
        }
        "Performance" => {
            c.gaps_in = 3;
            c.gaps_out = 8;
            c.border_size = 1;
            c.rounding = 4;
            c.blur_enabled = false;
            c.shadow_enabled = false;
            c.animations_enabled = false;
            c.active_opacity = 1.0;
            c.inactive_opacity = 1.0;
        }
        "No animations" => c.animations_enabled = false,
        _ => {}
    }
    c
}

const ANIMATION_LEAVES: [&str; 8] = [
    "windows",
    "windowsIn",
    "windowsOut",
    "border",
    "borderangle",
    "fade",
    "workspaces",
    "layers",
];

const ANIMATION_BEZIERS: [&str; 7] = [
    "default",
    "linear",
    "easeOutQuint",
    "easeInOutCubic",
    "easeOutBack",
    "myBezier",
    "custom",
];

#[derive(Clone)]
struct AnimationEntry {
    row: gtk::ListBoxRow,
    leaf: gtk::DropDown,
    enabled: gtk::Switch,
    speed: gtk::SpinButton,
    bezier: gtk::DropDown,
    custom_bezier: gtk::Entry,
}

impl AnimationEntry {
    fn value(&self) -> Option<String> {
        let leaf = ANIMATION_LEAVES
            .get(self.leaf.selected() as usize)
            .copied()
            .unwrap_or("windows");

        let selected_bezier = ANIMATION_BEZIERS
            .get(self.bezier.selected() as usize)
            .copied()
            .unwrap_or("default");

        let custom = self.custom_bezier.text();
        let custom = custom.trim();

        let bezier = if selected_bezier == "custom" {
            if custom.is_empty() {
                "default"
            } else {
                custom
            }
        } else {
            selected_bezier
        };

        Some(format!(
            "hl.animation({{ leaf = \"{}\", enabled = {}, speed = {}, bezier = \"{}\" }})",
            leaf,
            if self.enabled.is_active() {
                "true"
            } else {
                "false"
            },
            self.speed.value() as u32,
            lua_string(bezier)
        ))
    }
}

#[derive(Clone)]
struct AnimationBuilder {
    list: gtk::ListBox,
    entries: Rc<RefCell<Vec<AnimationEntry>>>,
}

impl AnimationBuilder {
    fn values(&self) -> Vec<String> {
        self.entries
            .borrow()
            .iter()
            .filter_map(|entry| entry.value())
            .collect()
    }

    fn set_values(&self, values: &[String]) {
        self.entries.borrow_mut().clear();
        clear_listbox(&self.list);

        let source = if values.is_empty() {
            vec![
                "hl.animation({ leaf = \"windows\", enabled = true, speed = 7, bezier = \"default\" })".to_string(),
                "hl.animation({ leaf = \"workspaces\", enabled = true, speed = 6, bezier = \"default\" })".to_string(),
            ]
        } else {
            values.to_vec()
        };

        for value in source {
            self.add_value(&value);
        }
    }

    fn add_value(&self, value: &str) {
        let entry = make_animation_entry(value, self.list.clone(), self.entries.clone());
        self.list.append(&entry.row);
        self.entries.borrow_mut().push(entry);
    }
}

fn anim_value(line: &str, key: &str, default: &str) -> String {
    let Some(after_key) = line.split(key).nth(1) else {
        return default.to_string();
    };

    let Some(after_eq) = after_key.split('=').nth(1) else {
        return default.to_string();
    };

    after_eq
        .split([',', '}'])
        .next()
        .unwrap_or(default)
        .trim()
        .trim_matches('"')
        .to_string()
}

fn parse_animation_line(initial: &str) -> (String, bool, f64, String) {
    let leaf = anim_value(initial, "leaf", "windows");
    let enabled = anim_value(initial, "enabled", "true") != "false";
    let speed = anim_value(initial, "speed", "7")
        .parse::<f64>()
        .unwrap_or(7.0);
    let bezier = anim_value(initial, "bezier", "default");

    (leaf, enabled, speed, bezier)
}

fn make_animation_entry(
    initial: &str,
    list: gtk::ListBox,
    entries: Rc<RefCell<Vec<AnimationEntry>>>,
) -> AnimationEntry {
    let (leaf_value, enabled_value, speed_value, bezier_value) = parse_animation_line(initial);

    let leaf_model = gtk::StringList::new(ANIMATION_LEAVES.as_slice());
    let leaf_idx = ANIMATION_LEAVES
        .iter()
        .position(|item| *item == leaf_value)
        .unwrap_or(0) as u32;

    let leaf = gtk::DropDown::builder()
        .model(&leaf_model)
        .selected(leaf_idx)
        .valign(gtk::Align::Center)
        .build();

    let enabled = switch(enabled_value);

    let speed = spin(speed_value, 1.0, 20.0, 1.0);
    speed.set_digits(0);

    let bezier_model = gtk::StringList::new(ANIMATION_BEZIERS.as_slice());
    let bezier_idx = ANIMATION_BEZIERS
        .iter()
        .position(|item| *item == bezier_value)
        .unwrap_or(6) as u32;

    let bezier = gtk::DropDown::builder()
        .model(&bezier_model)
        .selected(bezier_idx)
        .valign(gtk::Align::Center)
        .build();

    let custom_bezier = tiny_entry(
        if bezier_idx == 6 { &bezier_value } else { "" },
        "custom bezier",
        true,
    );
    custom_bezier.set_visible(bezier_idx == 6);

    let custom_bezier_visibility = custom_bezier.clone();
    bezier.connect_selected_notify(move |drop| {
        let selected = ANIMATION_BEZIERS
            .get(drop.selected() as usize)
            .copied()
            .unwrap_or("default");

        custom_bezier_visibility.set_visible(selected == "custom");
    });

    let del = pill_delete_button();

    let row_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(12)
        .margin_end(12)
        .build();

    row_box.append(
        &gtk::Label::builder()
            .label("target")
            .css_classes(["dim-label"])
            .build(),
    );
    row_box.append(&leaf);

    row_box.append(
        &gtk::Label::builder()
            .label("enabled")
            .css_classes(["dim-label"])
            .build(),
    );
    row_box.append(&enabled);

    row_box.append(
        &gtk::Label::builder()
            .label("speed")
            .css_classes(["dim-label"])
            .build(),
    );
    row_box.append(&speed);

    row_box.append(
        &gtk::Label::builder()
            .label("curve")
            .css_classes(["dim-label"])
            .build(),
    );
    row_box.append(&bezier);
    row_box.append(&custom_bezier);

    row_box.append(&del);

    let row = gtk::ListBoxRow::builder()
        .child(&row_box)
        .activatable(false)
        .selectable(false)
        .build();

    let entry = AnimationEntry {
        row: row.clone(),
        leaf,
        enabled,
        speed,
        bezier,
        custom_bezier,
    };

    let list_d = list.clone();
    let entries_d = entries.clone();
    let row_d = row.clone();

    del.connect_clicked(move |_| {
        list_d.remove(&row_d);
        entries_d.borrow_mut().retain(|entry| entry.row != row_d);
    });

    entry
}

fn make_animation_builder(
    title: &str,
    initial: &[String],
) -> (adw::PreferencesGroup, AnimationBuilder) {
    let entries = Rc::new(RefCell::new(Vec::new()));

    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();

    let builder = AnimationBuilder {
        list: list.clone(),
        entries: entries.clone(),
    };

    builder.set_values(initial);

    let add = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Add animation")
        .css_classes(["flat"])
        .build();

    let fast = gtk::Button::builder()
        .label("Fast")
        .css_classes(["flat", "pill"])
        .build();

    let smooth = gtk::Button::builder()
        .label("Smooth")
        .css_classes(["flat", "pill"])
        .build();

    let snappy = gtk::Button::builder()
        .label("Snappy")
        .css_classes(["flat", "pill"])
        .build();

    let header = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .build();

    header.append(&fast);
    header.append(&smooth);
    header.append(&snappy);
    header.append(&add);

    let builder_add = builder.clone();
    add.connect_clicked(move |_| {
        builder_add.add_value(
            "hl.animation({ leaf = \"windows\", enabled = true, speed = 7, bezier = \"default\" })",
        );
    });

    let builder_fast = builder.clone();
    fast.connect_clicked(move |_| {
        builder_fast.set_values(&[
            "hl.animation({ leaf = \"windows\", enabled = true, speed = 10, bezier = \"easeOutQuint\" })".to_string(),
            "hl.animation({ leaf = \"workspaces\", enabled = true, speed = 9, bezier = \"easeOutQuint\" })".to_string(),
            "hl.animation({ leaf = \"fade\", enabled = true, speed = 8, bezier = \"default\" })".to_string(),
        ]);
    });

    let builder_smooth = builder.clone();
    smooth.connect_clicked(move |_| {
        builder_smooth.set_values(&[
            "hl.animation({ leaf = \"windows\", enabled = true, speed = 6, bezier = \"easeInOutCubic\" })".to_string(),
            "hl.animation({ leaf = \"workspaces\", enabled = true, speed = 5, bezier = \"easeInOutCubic\" })".to_string(),
            "hl.animation({ leaf = \"fade\", enabled = true, speed = 5, bezier = \"default\" })".to_string(),
        ]);
    });

    let builder_snappy = builder.clone();
    snappy.connect_clicked(move |_| {
        builder_snappy.set_values(&[
            "hl.animation({ leaf = \"windows\", enabled = true, speed = 12, bezier = \"easeOutBack\" })".to_string(),
            "hl.animation({ leaf = \"windowsIn\", enabled = true, speed = 10, bezier = \"easeOutBack\" })".to_string(),
            "hl.animation({ leaf = \"windowsOut\", enabled = true, speed = 9, bezier = \"default\" })".to_string(),
            "hl.animation({ leaf = \"workspaces\", enabled = true, speed = 8, bezier = \"easeOutBack\" })".to_string(),
        ]);
    });

    let group = adw::PreferencesGroup::builder()
        .title(title)
        .description("Build animation rows visually instead of writing Lua manually.")
        .build();

    group.set_header_suffix(Some(&header));
    group.add(&list);

    (group, builder)
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() > 1 {
        match args.get(1).map(String::as_str) {
            Some("--backup") => println!("{:?}", create_safe_snapshot()),
            Some("--restore") => println!("{:?}", restore_bak_files().and_then(|_| reload_hyprland())),
            Some("--rollback") => println!("{:?}", restore_safe_snapshot().and_then(|_| reload_hyprland())),
            Some("--preview") => println!("{}", generate_lua(&read_config())),
            Some("--apply-preset") => {
                let name = args.get(2).map(String::as_str).unwrap_or("Minimal");
                let cfg = preset_config(name, &read_config());
                println!("{:?}", create_safe_snapshot().and_then(|_| save_config(&cfg)).and_then(|_| reload_hyprland()));
            }
            Some("--health") => {
                println!("hyprctl: {}", if command_exists("hyprctl") { "found" } else { "missing" });
                println!("config.json: {}", if config_path().exists() { "exists" } else { "missing" });
                println!("configurator-settings.lua: {}", if lua_config_path().exists() { "exists" } else { "missing" });
                println!("hyprland.lua: {}", if hyprland_lua_path().exists() { "exists" } else { "missing" });
            }
            _ => println!("hypr-configurator --apply-preset <name> | --backup | --restore | --rollback | --preview | --health"),
        }
        return;
    }

    let app = adw::Application::builder()
        .application_id("dev.n0t.hyprconfigurator")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    load_css();
    let config = read_config();
    let logs: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Hypr Configurator")
        .default_width(1280)
        .default_height(960)
        .build();

    let header = adw::HeaderBar::new();

    let sidebar_toggle_btn = gtk::Button::builder()
        .icon_name("open-menu-symbolic")
        .tooltip_text("Toggle sidebar")
        .css_classes(["flat"])
        .build();

    let preview_btn = gtk::Button::builder()
        .label("Preview Lua")
        .css_classes(["flat"])
        .build();
    let health_btn = gtk::Button::builder()
        .label("Health")
        .css_classes(["flat"])
        .build();
    let logs_btn = gtk::Button::builder()
        .label("Logs")
        .css_classes(["flat"])
        .build();
    let folder_btn = gtk::Button::builder()
        .icon_name("folder-open-symbolic")
        .tooltip_text("Open config folder")
        .css_classes(["flat"])
        .build();
    let restore_bak_btn = gtk::Button::builder()
        .label("Restore .bak")
        .css_classes(["flat"])
        .build();
    let rollback_btn = gtk::Button::builder()
        .label("Rollback")
        .css_classes(["flat"])
        .build();
    header.pack_start(&sidebar_toggle_btn);
    header.pack_start(&folder_btn);

    let gaps_in = spin(config.gaps_in as f64, 0.0, 100.0, 1.0);
    let gaps_out = spin(config.gaps_out as f64, 0.0, 100.0, 1.0);
    let border_sz = spin(config.border_size as f64, 0.0, 20.0, 1.0);
    let layouts = gtk::StringList::new(vec!["dwindle", "master", "scrolling"].as_slice());
    let layout_idx = match config.layout.as_str() {
        "master" => 1,
        "scrolling" => 2,
        _ => 0,
    };
    let layout_row = adw::ComboRow::builder()
        .title("Layout")
        .model(&layouts)
        .selected(layout_idx)
        .build();
    let resize_sw = switch(config.resize_on_border);
    let tearing_sw = switch(config.allow_tearing);
    let auto_apply_sw = switch(false);

    let group_general = adw::PreferencesGroup::builder().title("General").build();
    group_general.add(&spin_row("Inner gaps", None, &gaps_in));
    group_general.add(&spin_row("Outer gaps", None, &gaps_out));
    group_general.add(&spin_row("Border size", None, &border_sz));
    group_general.add(&layout_row);
    group_general.add(&switch_row("Resize on border", None, &resize_sw));
    group_general.add(&switch_row(
        "Allow tearing",
        Some("May cause visual artifacts"),
        &tearing_sw,
    ));
    group_general.add(&switch_row(
        "Auto apply changes",
        Some("Automatically saves and reloads when common fields change"),
        &auto_apply_sw,
    ));

    let presets = gtk::StringList::new(
        vec![
            "Minimal",
            "Glass",
            "Cyberpunk",
            "Mac-like",
            "Performance",
            "No animations",
        ]
        .as_slice(),
    );
    let preset_row = adw::ComboRow::builder()
        .title("Theme preset")
        .model(&presets)
        .selected(0)
        .build();
    let apply_preset_btn = gtk::Button::builder()
        .label("Apply preset")
        .css_classes(["add-button", "pill"])
        .valign(gtk::Align::Center)
        .build();
    let export_preset_btn = gtk::Button::builder()
        .label("Export current")
        .css_classes(["add-button", "pill"])
        .valign(gtk::Align::Center)
        .build();
    let import_preset_btn = gtk::Button::builder()
        .label("Import exported")
        .css_classes(["add-button", "pill"])
        .valign(gtk::Align::Center)
        .build();
    let preset_action = action_row(
        "Apply selected preset",
        Some("Changes the UI values before saving"),
    );
    preset_action.add_suffix(&apply_preset_btn);
    let export_action = action_row(
        "Export current preset",
        Some("Writes ~/.config/hypr/exported-preset.json"),
    );
    export_action.add_suffix(&export_preset_btn);
    let import_action = action_row(
        "Import exported preset",
        Some("Reads ~/.config/hypr/exported-preset.json into the UI"),
    );
    import_action.add_suffix(&import_preset_btn);
    let named_preset = adw::EntryRow::builder()
        .title("Named preset")
        .text("gaming")
        .build();
    let save_named_preset_btn = gtk::Button::builder()
        .label("Save named")
        .css_classes(["add-button", "pill"])
        .valign(gtk::Align::Center)
        .build();
    let load_named_preset_btn = gtk::Button::builder()
        .label("Load named")
        .css_classes(["add-button", "pill"])
        .valign(gtk::Align::Center)
        .build();
    let delete_named_preset_btn = gtk::Button::builder()
        .label("Delete named")
        .css_classes(["add-button", "pill"])
        .valign(gtk::Align::Center)
        .build();
    let list_named_preset_btn = gtk::Button::builder()
        .label("List")
        .css_classes(["add-button", "pill"])
        .valign(gtk::Align::Center)
        .build();
    let save_named_row = action_row(
        "Save named preset",
        Some("Stores ~/.config/hypr/presets/<name>.json"),
    );
    save_named_row.add_suffix(&save_named_preset_btn);
    let load_named_row = action_row("Load named preset", Some("Loads a preset into the UI"));
    load_named_row.add_suffix(&load_named_preset_btn);
    let delete_named_row = action_row("Delete named preset", Some("Deletes preset file"));
    delete_named_row.add_suffix(&delete_named_preset_btn);
    let list_named_row = action_row("List named presets", None);
    list_named_row.add_suffix(&list_named_preset_btn);

    let group_presets = adw::PreferencesGroup::builder().title("Presets").build();
    group_presets.add(&preset_row);
    group_presets.add(&preset_action);
    group_presets.add(&export_action);
    group_presets.add(&import_action);
    group_presets.add(&named_preset);
    group_presets.add(&save_named_row);
    group_presets.add(&load_named_row);
    group_presets.add(&delete_named_row);
    group_presets.add(&list_named_row);

    let (active_color_btn, active_color) =
        make_color_button(hyprland_to_gdk(&config.active_border_color), &window);
    let active_color_row = adw::ActionRow::builder().title("Active border").build();
    active_color_row.add_suffix(&active_color_btn);
    let (inactive_color_btn, inactive_color) =
        make_color_button(hyprland_to_gdk(&config.inactive_border_color), &window);
    let inactive_color_row = adw::ActionRow::builder().title("Inactive border").build();
    inactive_color_row.add_suffix(&inactive_color_btn);
    let group_colors = adw::PreferencesGroup::builder()
        .title("Border colors")
        .build();
    group_colors.add(&active_color_row);
    group_colors.add(&inactive_color_row);

    let rounding = spin(config.rounding as f64, 0.0, 60.0, 1.0);
    let active_opacity = spin(config.active_opacity, 0.0, 1.0, 0.05);
    let inactive_opacity = spin(config.inactive_opacity, 0.0, 1.0, 0.05);
    active_opacity.set_digits(2);
    inactive_opacity.set_digits(2);
    let group_decoration = adw::PreferencesGroup::builder().title("Decoration").build();
    group_decoration.add(&spin_row("Rounding", Some("Radius in pixels"), &rounding));
    group_decoration.add(&spin_row("Active window opacity", None, &active_opacity));
    group_decoration.add(&spin_row(
        "Inactive window opacity",
        None,
        &inactive_opacity,
    ));

    let shadow_sw = switch(config.shadow_enabled);
    let shadow_range = spin(config.shadow_range as f64, 0.0, 100.0, 1.0);
    let group_shadow = adw::PreferencesGroup::builder().title("Shadow").build();
    group_shadow.add(&switch_row("Enable shadow", None, &shadow_sw));
    group_shadow.add(&spin_row("Range", None, &shadow_range));

    let blur_sw = switch(config.blur_enabled);
    let blur_size = spin(config.blur_size as f64, 1.0, 20.0, 1.0);
    let blur_passes = spin(config.blur_passes as f64, 1.0, 10.0, 1.0);
    let group_blur = adw::PreferencesGroup::builder().title("Blur").build();
    group_blur.add(&switch_row("Enable blur", None, &blur_sw));
    group_blur.add(&spin_row("Size", None, &blur_size));
    group_blur.add(&spin_row(
        "Passes",
        Some("More passes = stronger and heavier blur"),
        &blur_passes,
    ));

    let anim_sw = switch(config.animations_enabled);
    let group_anim = adw::PreferencesGroup::builder().title("Animations").build();
    group_anim.add(&switch_row("Enable animations", None, &anim_sw));

    let kb_layout = adw::EntryRow::builder()
        .title("Keyboard layout")
        .text(&config.kb_layout)
        .build();
    let kb_variant = adw::EntryRow::builder()
        .title("Variant")
        .text(&config.kb_variant)
        .build();
    let sensitivity = spin(config.sensitivity, -1.0, 1.0, 0.1);
    sensitivity.set_digits(1);
    let natural_scroll_sw = switch(config.natural_scroll);
    let group_input = adw::PreferencesGroup::builder().title("Input").build();
    group_input.add(&kb_layout);
    group_input.add(&kb_variant);
    group_input.add(&spin_row(
        "Mouse sensitivity",
        Some("-1.0 to 1.0"),
        &sensitivity,
    ));
    group_input.add(&switch_row(
        "Natural scroll (touchpad)",
        None,
        &natural_scroll_sw,
    ));

    let default_terminal = adw::EntryRow::builder()
        .title("Terminal")
        .text(&config.default_terminal)
        .build();
    let default_browser = adw::EntryRow::builder()
        .title("Browser")
        .text(&config.default_browser)
        .build();
    let default_file_manager = adw::EntryRow::builder()
        .title("File manager")
        .text(&config.default_file_manager)
        .build();
    let default_editor = adw::EntryRow::builder()
        .title("Editor")
        .text(&config.default_editor)
        .build();
    let group_default_apps = adw::PreferencesGroup::builder()
        .title("Default apps")
        .description("Also generates SUPER shortcuts")
        .build();
    group_default_apps.add(&default_terminal);
    group_default_apps.add(&default_browser);
    group_default_apps.add(&default_file_manager);
    group_default_apps.add(&default_editor);

    let wallpaper_path = adw::EntryRow::builder()
        .title("Wallpaper path")
        .text(&config.wallpaper_path)
        .build();
    let wallpaper_backend = adw::EntryRow::builder()
        .title("Wallpaper backend")
        .text(&config.wallpaper_backend)
        .build();
    let wallpaper_mode = adw::EntryRow::builder()
        .title("Wallpaper mode")
        .text(&config.wallpaper_mode)
        .build();
    let group_wallpaper = adw::PreferencesGroup::builder()
        .title("Wallpaper manager")
        .description("swaybg or hyprpaper")
        .build();
    group_wallpaper.add(&wallpaper_path);
    group_wallpaper.add(&wallpaper_backend);
    group_wallpaper.add(&wallpaper_mode);

    let (group_startup, startup_rows, _) = make_text_rows(
        "Startup event",
        "Commands executed on hyprland.start.",
        "Example: waybar",
        &config.startup_commands,
    );
    let (group_exec, exec_rows, _) = make_text_rows(
        "Exec",
        "Commands executed with exec.",
        "Example: nm-applet",
        &config.exec_commands,
    );
    let (group_exec_once, exec_once_rows, _) = make_text_rows(
        "Exec once",
        "Commands executed once during startup.",
        "Example: waybar",
        &config.exec_once_commands,
    );
    let (group_env, env_rows) = make_env_builder("Environment variables", &config.env_vars);
    let (group_monitors, monitor_rows) = make_monitor_builder("Monitors", &config.monitor_rules);
    let detect_monitors_btn = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Auto-detect monitors")
        .label("Auto-detect monitors")
        .css_classes(["add-button", "pill"])
        .valign(gtk::Align::Center)
        .build();

    let detect_monitor_row =
        action_row("Detect current monitors", Some("Uses hyprctl monitors -j"));
    detect_monitor_row.add_suffix(&detect_monitors_btn);
    group_monitors.add(&detect_monitor_row);

    let (group_visual_workspaces, visual_workspace_rows) =
        make_workspace_builder("Workspace builder", &config.visual_workspaces);
    let (group_workspaces, workspace_rows, _) = make_text_rows(
        "Raw workspaces",
        "Raw Lua workspace rule lines.",
        "Example: hl.workspace_rule({ workspace = \"1\", monitor = \"DP-1\" })",
        &config.workspace_rules,
    );
    let (group_visual_window_rules, visual_window_rule_rows, _) = make_text_rows(
        "Window rule builder",
        "Visual format: class|float=true|center=true|size=1200x800|workspace=2|opacity=0.9",
        "Example: firefox|float=true|center=true|size=1200x800|workspace=2|opacity=0.95",
        &config.visual_window_rules,
    );
    let (group_window_rules, window_rule_rows, _) = make_text_rows(
        "Raw window rules",
        "Raw Lua window rule lines.",
        "Example: hl.window_rule({ name = \"my-rule\", match = { class = \"^firefox$\" }, float = true })",
        &config.window_rules,
    );
    let detect_apps_btn = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Add open Apps")
        .label("Add open Apps")
        .css_classes(["add-button", "pill"])
        .valign(gtk::Align::Center)
        .build();
    let detect_apps_row = action_row(
        "Detect open app classes",
        Some("Uses hyprctl clients -j and creates float rules"),
    );
    detect_apps_row.add_suffix(&detect_apps_btn);
    group_window_rules.add(&detect_apps_row);

    let (group_visual_keybinds, visual_keybind_rows) =
        make_keybind_builder("Keybind builder", &config.visual_keybinds);
    let (group_keybinds, keybind_rows, _) = make_text_rows(
        "Raw keybinds",
        "Raw Lua bind lines.",
        "Example: hl.bind(\"SUPER + Return\", hl.dsp.exec_cmd(\"kitty\"))",
        &config.keybinds,
    );
    let (group_mouse_binds, mouse_bind_rows) =
        make_mouse_bind_builder("Mouse binds", &config.mouse_binds);
    let (group_animation_lines, animation_builder) =
        make_animation_builder("Advanced animations", &config.animation_lines);
    let (group_decoration_lines, decoration_rows, _) = make_text_rows(
        "Decoration extras",
        "Extra hl.config({ decoration = { ... } }) lines.",
        "Example: hl.config({ decoration = { dim_inactive = true, dim_strength = 0.5 } })",
        &config.decoration_lines,
    );
    let (group_input_extra_lines, input_extra_rows) =
        make_input_extra_builder("Input extras", &config.input_extra_lines);
    let (group_gesture_lines, gesture_rows) =
        make_gesture_builder("Gestures", &config.gesture_lines);
    let (group_custom, custom_rows, _) = make_text_rows(
        "Custom lines",
        "Raw Lua lines appended at the end of the generated file.",
        "Example: hl.config({ misc = { vfr = true } })",
        &config.custom_lines,
    );

    let logo_sw = switch(config.disable_hyprland_logo);
    let group_misc = adw::PreferencesGroup::builder().title("Misc").build();
    group_misc.add(&switch_row(
        "Disable Hyprland logo",
        Some("Removes the default background"),
        &logo_sw,
    ));

    let save_button = gtk::Button::builder()
        .label("Save and apply")
        .css_classes(["suggested-action", "pill"])
        .margin_top(8)
        .build();
    let reset_button = gtk::Button::builder()
        .label("Reset to defaults")
        .css_classes(["destructive-action", "pill"])
        .margin_top(8)
        .build();
    let buttons = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::End)
        .build();
    buttons.append(&reset_button);
    buttons.append(&save_button);

    let stack = gtk::Stack::builder()
        .hexpand(true)
        .vexpand(true)
        .transition_type(gtk::StackTransitionType::SlideLeftRight)
        .transition_duration(180)
        .build();

    let (page_general, box_general) = make_stack_page("General");
    box_general.append(&group_general);
    box_general.append(&group_default_apps);
    box_general.append(&group_misc);
    box_general.append(&buttons);
    stack.add_named(&page_general, Some("general"));

    let (page_appearance, box_appearance) = make_stack_page("Appearance");
    box_appearance.append(&group_presets);
    box_appearance.append(&group_colors);
    box_appearance.append(&group_decoration);
    box_appearance.append(&group_shadow);
    box_appearance.append(&group_blur);
    box_appearance.append(&group_anim);
    box_appearance.append(&group_wallpaper);
    stack.add_named(&page_appearance, Some("appearance"));

    let (page_input, box_input) = make_stack_page("Input");
    box_input.append(&group_input);
    box_input.append(&group_input_extra_lines);
    box_input.append(&group_gesture_lines);
    stack.add_named(&page_input, Some("input"));

    let (page_startup, box_startup) = make_stack_page("Startup");
    box_startup.append(&group_startup);
    box_startup.append(&group_exec);
    box_startup.append(&group_exec_once);
    box_startup.append(&group_env);
    stack.add_named(&page_startup, Some("startup"));

    let (page_monitors, box_monitors) = make_stack_page("Monitors");
    box_monitors.append(&group_monitors);
    stack.add_named(&page_monitors, Some("monitors"));

    let (page_workspaces, box_workspaces) = make_stack_page("Workspaces");
    box_workspaces.append(&group_visual_workspaces);
    box_workspaces.append(&group_workspaces);
    stack.add_named(&page_workspaces, Some("workspaces"));

    let (page_window_rules, box_window_rules) = make_stack_page("Window Rules");
    box_window_rules.append(&group_visual_window_rules);
    box_window_rules.append(&group_window_rules);
    stack.add_named(&page_window_rules, Some("window-rules"));

    let (page_keybinds, box_keybinds) = make_stack_page("Keybinds");
    box_keybinds.append(&group_visual_keybinds);
    box_keybinds.append(&group_keybinds);
    box_keybinds.append(&group_mouse_binds);
    stack.add_named(&page_keybinds, Some("keybinds"));

    let preview_row = adw::ActionRow::builder()
        .title("Preview Lua")
        .subtitle("Show generated configurator-settings.lua")
        .build();
    preview_btn.set_css_classes(["add-button", "pill"].as_slice());
    preview_btn.set_valign(gtk::Align::Center);
    preview_row.add_suffix(&preview_btn);

    let restore_bak_row = adw::ActionRow::builder()
        .title("Restore .bak")
        .subtitle("Restore last .bak backup files and reload Hyprland")
        .build();
    restore_bak_btn.set_css_classes(["add-button", "pill"].as_slice());
    restore_bak_btn.set_valign(gtk::Align::Center);
    restore_bak_row.add_suffix(&restore_bak_btn);

    let rollback_row = adw::ActionRow::builder()
        .title("Rollback")
        .subtitle("Restore safe snapshot and reload Hyprland")
        .build();
    rollback_btn.set_css_classes(["add-button", "pill"].as_slice());
    rollback_btn.set_valign(gtk::Align::Center);
    rollback_row.add_suffix(&rollback_btn);

    let health_row = adw::ActionRow::builder()
        .title("Health check")
        .subtitle("Check config files and tools are in place")
        .build();
    health_btn.set_css_classes(["add-button", "pill"].as_slice());
    health_btn.set_valign(gtk::Align::Center);
    health_row.add_suffix(&health_btn);

    let logs_row = adw::ActionRow::builder()
        .title("Logs")
        .subtitle("Show session activity log")
        .build();
    logs_btn.set_css_classes(["add-button", "pill"].as_slice());
    logs_btn.set_valign(gtk::Align::Center);
    logs_row.add_suffix(&logs_btn);

    let group_advanced_actions = adw::PreferencesGroup::builder().title("Actions").build();
    group_advanced_actions.add(&preview_row);
    group_advanced_actions.add(&restore_bak_row);
    group_advanced_actions.add(&rollback_row);
    group_advanced_actions.add(&health_row);
    group_advanced_actions.add(&logs_row);

    let (page_advanced, box_advanced) = make_stack_page("Advanced");
    box_advanced.append(&group_advanced_actions);
    box_advanced.append(&group_animation_lines);
    box_advanced.append(&group_decoration_lines);
    box_advanced.append(&group_custom);
    stack.add_named(&page_advanced, Some("advanced"));

    stack.set_visible_child_name("general");

    let sidebar_list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["navigation-sidebar"])
        .vexpand(true)
        .build();

    let sidebar_revealer = gtk::Revealer::builder()
        .transition_type(gtk::RevealerTransitionType::SlideRight)
        .reveal_child(true)
        .build();

    add_sidebar_item(
        &sidebar_list,
        &stack,
        &sidebar_revealer,
        "General",
        "general",
    );
    add_sidebar_item(
        &sidebar_list,
        &stack,
        &sidebar_revealer,
        "Appearance",
        "appearance",
    );
    add_sidebar_item(&sidebar_list, &stack, &sidebar_revealer, "Input", "input");
    add_sidebar_item(
        &sidebar_list,
        &stack,
        &sidebar_revealer,
        "Startup",
        "startup",
    );
    add_sidebar_item(
        &sidebar_list,
        &stack,
        &sidebar_revealer,
        "Monitors",
        "monitors",
    );
    add_sidebar_item(
        &sidebar_list,
        &stack,
        &sidebar_revealer,
        "Workspaces",
        "workspaces",
    );
    add_sidebar_item(
        &sidebar_list,
        &stack,
        &sidebar_revealer,
        "Window Rules",
        "window-rules",
    );
    add_sidebar_item(
        &sidebar_list,
        &stack,
        &sidebar_revealer,
        "Keybinds",
        "keybinds",
    );
    add_sidebar_item(
        &sidebar_list,
        &stack,
        &sidebar_revealer,
        "Advanced",
        "advanced",
    );

    let sidebar_title = gtk::Label::builder()
        .label("Hypr Config")
        .css_classes(["sidebar-title"])
        .halign(gtk::Align::Start)
        .margin_top(18)
        .margin_bottom(12)
        .margin_start(16)
        .margin_end(16)
        .build();

    let sidebar_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .width_request(240)
        .css_classes(["sidebar"])
        .build();
    sidebar_box.append(&sidebar_title);
    sidebar_box.append(&sidebar_list);
    sidebar_revealer.set_child(Some(&sidebar_box));

    let overlay = adw::ToastOverlay::new();
    overlay.set_child(Some(&stack));

    let main_area = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .vexpand(true)
        .hexpand(true)
        .build();
    main_area.append(&sidebar_revealer);
    main_area.append(&overlay);

    let root = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    root.append(&header);
    root.append(&main_area);
    window.set_content(Some(&root));

    let sidebar_revealer_toggle = sidebar_revealer.clone();
    sidebar_toggle_btn.connect_clicked(move |_| {
        sidebar_revealer_toggle.set_reveal_child(!sidebar_revealer_toggle.reveals_child());
    });

    let sidebar_revealer_responsive = sidebar_revealer.clone();
    let sidebar_toggle_responsive = sidebar_toggle_btn.clone();
    let last_compact: Rc<RefCell<Option<bool>>> = Rc::new(RefCell::new(None));
    let last_compact_c = last_compact.clone();

    root.add_tick_callback(move |widget, _| {
        let compact = widget.width() < 820;
        let mut last = last_compact_c.borrow_mut();

        if (*last).map(|value| value != compact).unwrap_or(true) {
            sidebar_revealer_responsive.set_reveal_child(!compact);
            sidebar_toggle_responsive.set_visible(compact);
            *last = Some(compact);
        }

        glib::ControlFlow::Continue
    });

    let collect_config = {
        let gaps_in = gaps_in.clone();
        let gaps_out = gaps_out.clone();
        let border_sz = border_sz.clone();
        let layout_row = layout_row.clone();
        let resize_sw = resize_sw.clone();
        let tearing_sw = tearing_sw.clone();
        let active_color = active_color.clone();
        let inactive_color = inactive_color.clone();
        let rounding = rounding.clone();
        let active_opacity = active_opacity.clone();
        let inactive_opacity = inactive_opacity.clone();
        let shadow_sw = shadow_sw.clone();
        let shadow_range = shadow_range.clone();
        let blur_sw = blur_sw.clone();
        let blur_size = blur_size.clone();
        let blur_passes = blur_passes.clone();
        let anim_sw = anim_sw.clone();
        let kb_layout = kb_layout.clone();
        let kb_variant = kb_variant.clone();
        let sensitivity = sensitivity.clone();
        let natural_scroll_sw = natural_scroll_sw.clone();
        let startup_rows = startup_rows.clone();
        let exec_rows = exec_rows.clone();
        let exec_once_rows = exec_once_rows.clone();
        let env_rows = env_rows.clone();
        let monitor_rows = monitor_rows.clone();
        let visual_workspace_rows = visual_workspace_rows.clone();
        let workspace_rows = workspace_rows.clone();
        let visual_window_rule_rows = visual_window_rule_rows.clone();
        let window_rule_rows = window_rule_rows.clone();
        let visual_keybind_rows = visual_keybind_rows.clone();
        let keybind_rows = keybind_rows.clone();
        let mouse_bind_rows = mouse_bind_rows.clone();
        let animation_builder = animation_builder.clone();
        let decoration_rows = decoration_rows.clone();
        let input_extra_rows = input_extra_rows.clone();
        let gesture_rows = gesture_rows.clone();
        let custom_rows = custom_rows.clone();
        let default_terminal = default_terminal.clone();
        let default_browser = default_browser.clone();
        let default_file_manager = default_file_manager.clone();
        let default_editor = default_editor.clone();
        let wallpaper_path = wallpaper_path.clone();
        let wallpaper_backend = wallpaper_backend.clone();
        let wallpaper_mode = wallpaper_mode.clone();
        let logo_sw = logo_sw.clone();
        move || {
            let layout = match layout_row.selected() {
                1 => "master",
                2 => "scrolling",
                _ => "dwindle",
            }
            .to_string();
            HyprConfig {
                gaps_in: gaps_in.value() as u32,
                gaps_out: gaps_out.value() as u32,
                border_size: border_sz.value() as u32,
                active_border_color: gdk_to_hyprland(&active_color.borrow()),
                inactive_border_color: gdk_to_hyprland(&inactive_color.borrow()),
                resize_on_border: resize_sw.is_active(),
                allow_tearing: tearing_sw.is_active(),
                layout,
                rounding: rounding.value() as u32,
                active_opacity: active_opacity.value(),
                inactive_opacity: inactive_opacity.value(),
                shadow_enabled: shadow_sw.is_active(),
                shadow_range: shadow_range.value() as u32,
                blur_enabled: blur_sw.is_active(),
                blur_size: blur_size.value() as u32,
                blur_passes: blur_passes.value() as u32,
                animations_enabled: anim_sw.is_active(),
                kb_layout: kb_layout.text().to_string(),
                kb_variant: kb_variant.text().to_string(),
                sensitivity: sensitivity.value(),
                natural_scroll: natural_scroll_sw.is_active(),
                disable_hyprland_logo: logo_sw.is_active(),
                startup_commands: startup_rows.values(),
                exec_commands: exec_rows.values(),
                exec_once_commands: exec_once_rows.values(),
                env_vars: env_rows.values(),
                monitor_rules: monitor_rows.values(),
                visual_workspaces: visual_workspace_rows.values(),
                workspace_rules: workspace_rows.values(),
                visual_window_rules: visual_window_rule_rows.values(),
                window_rules: window_rule_rows.values(),
                visual_keybinds: visual_keybind_rows.values(),
                keybinds: keybind_rows.values(),
                mouse_binds: mouse_bind_rows.values(),
                default_terminal: default_terminal.text().to_string(),
                default_browser: default_browser.text().to_string(),
                default_file_manager: default_file_manager.text().to_string(),
                default_editor: default_editor.text().to_string(),
                wallpaper_path: wallpaper_path.text().to_string(),
                wallpaper_backend: wallpaper_backend.text().to_string(),
                wallpaper_mode: wallpaper_mode.text().to_string(),
                animation_lines: animation_builder.values(),
                decoration_lines: decoration_rows.values(),
                input_extra_lines: input_extra_rows.values(),
                gesture_lines: gesture_rows.values(),
                custom_lines: custom_rows.values(),
            }
        }
    };
    let collect_config = Rc::new(collect_config);

    let overlay_c = overlay.clone();
    let logs_save = logs.clone();
    let collect_save = collect_config.clone();
    save_button.connect_clicked(move |_| {
        let cfg = collect_save();
        let _ = create_safe_snapshot();
        match save_config(&cfg) {
            Ok(saved) => match reload_hyprland() {
                Ok(_) => {
                    logs_save
                        .borrow_mut()
                        .push(format!("{}; Hyprland reloaded", saved));
                    overlay_c
                        .add_toast(adw::Toast::new("Configuration saved and Hyprland reloaded"));
                }
                Err(e) => {
                    logs_save
                        .borrow_mut()
                        .push(format!("{}; Reload failed: {}", saved, e));
                    overlay_c.add_toast(adw::Toast::new("Saved, but Hyprland reload failed"));
                }
            },
            Err(e) => {
                logs_save.borrow_mut().push(format!("Save failed: {}", e));
                overlay_c.add_toast(adw::Toast::new(&format!("Save failed: {e}")));
            }
        }
    });

    let collect_preview = collect_config.clone();
    let window_preview = window.clone();
    preview_btn.connect_clicked(move |_| {
        let cfg = collect_preview();
        show_text_window(
            &window_preview,
            "Generated Lua preview",
            &generate_lua(&cfg),
        );
    });

    let window_health = window.clone();
    health_btn.connect_clicked(move |_| {
        let mut lines = Vec::new();
        lines.push(format!(
            "hyprctl: {}",
            if command_exists("hyprctl") {
                "found"
            } else {
                "missing"
            }
        ));
        lines.push(format!(
            "xdg-open: {}",
            if command_exists("xdg-open") {
                "found"
            } else {
                "missing"
            }
        ));
        lines.push(format!(
            "config.json: {}",
            if config_path().exists() {
                "exists"
            } else {
                "missing"
            }
        ));
        lines.push(format!(
            "configurator-settings.lua: {}",
            if lua_config_path().exists() {
                "exists"
            } else {
                "missing"
            }
        ));
        lines.push(format!(
            "hyprland.lua: {}",
            if hyprland_lua_path().exists() {
                "exists"
            } else {
                "missing"
            }
        ));
        let injected = fs::read_to_string(hyprland_lua_path())
            .unwrap_or_default()
            .contains("configurator-settings");
        lines.push(format!(
            "require injected: {}",
            if injected { "yes" } else { "no" }
        ));
        show_text_window(&window_health, "Health check", &lines.join("\n"));
    });

    let window_logs = window.clone();
    let logs_show = logs.clone();
    logs_btn.connect_clicked(move |_| {
        let text = if logs_show.borrow().is_empty() {
            "No logs yet".to_string()
        } else {
            logs_show.borrow().join("\n")
        };
        show_text_window(&window_logs, "Logs", &text);
    });

    folder_btn.connect_clicked(move |_| open_config_folder());

    let overlay_restore_bak = overlay.clone();
    restore_bak_btn.connect_clicked(move |_| {
        match restore_bak_files().and_then(|_| reload_hyprland()) {
            Ok(_) => overlay_restore_bak
                .add_toast(adw::Toast::new(".bak restored and Hyprland reloaded")),
            Err(e) => {
                overlay_restore_bak.add_toast(adw::Toast::new(&format!("Restore failed: {e}")))
            }
        }
    });

    let overlay_rollback = overlay.clone();
    rollback_btn.connect_clicked(move |_| {
        match restore_safe_snapshot().and_then(|_| reload_hyprland()) {
            Ok(_) => overlay_rollback.add_toast(adw::Toast::new("Safe rollback restored")),
            Err(e) => overlay_rollback.add_toast(adw::Toast::new(&format!("Rollback failed: {e}"))),
        }
    });

    let monitor_rows_detect = monitor_rows.clone();
    let overlay_monitors = overlay.clone();
    detect_monitors_btn.connect_clicked(move |_| {
        let rules = detect_monitor_rules();
        if rules.is_empty() {
            overlay_monitors.add_toast(adw::Toast::new("No monitors detected"));
        } else {
            monitor_rows_detect.set_values(&rules);
            overlay_monitors.add_toast(adw::Toast::new("Monitor rules detected"));
        }
    });

    let window_rule_rows_detect = window_rule_rows.clone();
    let overlay_apps = overlay.clone();
    detect_apps_btn.connect_clicked(move |_| {
        let mut rules = window_rule_rows_detect.values();
        rules.extend(detect_window_rules());
        rules.sort();
        rules.dedup();
        if rules.is_empty() {
            overlay_apps.add_toast(adw::Toast::new("No app classes detected"));
        } else {
            window_rule_rows_detect.set_values(&rules, "Example: float,class:^(firefox)$");
            overlay_apps.add_toast(adw::Toast::new("Open app rules added"));
        }
    });

    let reset_values = {
        let gaps_in = gaps_in.clone();
        let gaps_out = gaps_out.clone();
        let border_sz = border_sz.clone();
        let layout_row = layout_row.clone();
        let resize_sw = resize_sw.clone();
        let tearing_sw = tearing_sw.clone();
        let rounding = rounding.clone();
        let active_opacity = active_opacity.clone();
        let inactive_opacity = inactive_opacity.clone();
        let shadow_sw = shadow_sw.clone();
        let shadow_range = shadow_range.clone();
        let blur_sw = blur_sw.clone();
        let blur_size = blur_size.clone();
        let blur_passes = blur_passes.clone();
        let anim_sw = anim_sw.clone();
        let kb_layout = kb_layout.clone();
        let kb_variant = kb_variant.clone();
        let sensitivity = sensitivity.clone();
        let natural_scroll_sw = natural_scroll_sw.clone();
        let startup_rows = startup_rows.clone();
        let exec_rows = exec_rows.clone();
        let exec_once_rows = exec_once_rows.clone();
        let env_rows = env_rows.clone();
        let monitor_rows = monitor_rows.clone();
        let visual_workspace_rows = visual_workspace_rows.clone();
        let workspace_rows = workspace_rows.clone();
        let visual_window_rule_rows = visual_window_rule_rows.clone();
        let window_rule_rows = window_rule_rows.clone();
        let visual_keybind_rows = visual_keybind_rows.clone();
        let keybind_rows = keybind_rows.clone();
        let mouse_bind_rows = mouse_bind_rows.clone();
        let animation_builder = animation_builder.clone();
        let decoration_rows = decoration_rows.clone();
        let input_extra_rows = input_extra_rows.clone();
        let gesture_rows = gesture_rows.clone();
        let custom_rows = custom_rows.clone();
        let default_terminal = default_terminal.clone();
        let default_browser = default_browser.clone();
        let default_file_manager = default_file_manager.clone();
        let default_editor = default_editor.clone();
        let wallpaper_path = wallpaper_path.clone();
        let wallpaper_backend = wallpaper_backend.clone();
        let wallpaper_mode = wallpaper_mode.clone();
        let logo_sw = logo_sw.clone();
        move |cfg: HyprConfig| {
            gaps_in.set_value(cfg.gaps_in as f64);
            gaps_out.set_value(cfg.gaps_out as f64);
            border_sz.set_value(cfg.border_size as f64);
            layout_row.set_selected(match cfg.layout.as_str() {
                "master" => 1,
                "scrolling" => 2,
                _ => 0,
            });
            resize_sw.set_active(cfg.resize_on_border);
            tearing_sw.set_active(cfg.allow_tearing);
            rounding.set_value(cfg.rounding as f64);
            active_opacity.set_value(cfg.active_opacity);
            inactive_opacity.set_value(cfg.inactive_opacity);
            shadow_sw.set_active(cfg.shadow_enabled);
            shadow_range.set_value(cfg.shadow_range as f64);
            blur_sw.set_active(cfg.blur_enabled);
            blur_size.set_value(cfg.blur_size as f64);
            blur_passes.set_value(cfg.blur_passes as f64);
            anim_sw.set_active(cfg.animations_enabled);
            kb_layout.set_text(&cfg.kb_layout);
            kb_variant.set_text(&cfg.kb_variant);
            sensitivity.set_value(cfg.sensitivity);
            natural_scroll_sw.set_active(cfg.natural_scroll);
            startup_rows.set_values(&cfg.startup_commands, "Example: waybar");
            exec_rows.set_values(&cfg.exec_commands, "Example: nm-applet");
            exec_once_rows.set_values(&cfg.exec_once_commands, "Example: waybar");
            env_rows.set_values(&cfg.env_vars);
            monitor_rows.set_values(&cfg.monitor_rules);
            visual_workspace_rows.set_values(&cfg.visual_workspaces);
            workspace_rows.set_values(
                &cfg.workspace_rules,
                "Example: 1, monitor:DP-1, persistent:true",
            );
            visual_window_rule_rows.set_values(
                &cfg.visual_window_rules,
                "Example: firefox|float=true|center=true|size=1200x800|workspace=2|opacity=0.95",
            );
            window_rule_rows.set_values(&cfg.window_rules, "Example: float,class:^(firefox)$");
            visual_keybind_rows.set_values(&cfg.visual_keybinds);
            keybind_rows.set_values(&cfg.keybinds, "Example: SUPER, Return, exec, kitty");
            mouse_bind_rows.set_values(&cfg.mouse_binds);
            default_terminal.set_text(&cfg.default_terminal);
            default_browser.set_text(&cfg.default_browser);
            default_file_manager.set_text(&cfg.default_file_manager);
            default_editor.set_text(&cfg.default_editor);
            wallpaper_path.set_text(&cfg.wallpaper_path);
            wallpaper_backend.set_text(&cfg.wallpaper_backend);
            wallpaper_mode.set_text(&cfg.wallpaper_mode);
            animation_builder.set_values(&cfg.animation_lines);
            decoration_rows.set_values(&cfg.decoration_lines, "Example: dim_inactive = true");
            input_extra_rows.set_values(&cfg.input_extra_lines);
            gesture_rows.set_values(&cfg.gesture_lines);
            custom_rows.set_values(&cfg.custom_lines, "Example: exec-once = waybar");
            logo_sw.set_active(cfg.disable_hyprland_logo);
        }
    };
    let reset_values = Rc::new(reset_values);

    let reset_values_btn = reset_values.clone();
    let overlay_reset = overlay.clone();
    reset_button.connect_clicked(move |_| {
        reset_values_btn(HyprConfig::default());
        overlay_reset.add_toast(adw::Toast::new(
            "Values reset to defaults. Click Save and apply to persist.",
        ));
    });

    let collect_preset = collect_config.clone();
    let reset_preset = reset_values.clone();
    let overlay_preset = overlay.clone();
    apply_preset_btn.connect_clicked(move |_| {
        let names = [
            "Minimal",
            "Glass",
            "Cyberpunk",
            "Mac-like",
            "Performance",
            "No animations",
        ];
        let selected = names
            .get(preset_row.selected() as usize)
            .unwrap_or(&"Minimal");
        let next = preset_config(selected, &collect_preset());
        reset_preset(next);
        overlay_preset.add_toast(adw::Toast::new(&format!("Preset applied: {selected}")));
    });

    let collect_export = collect_config.clone();
    let overlay_export = overlay.clone();
    export_preset_btn.connect_clicked(move |_| {
        let path = exported_preset_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(&collect_export())
            .map_err(|e| e.to_string())
            .and_then(|json| fs::write(&path, json).map_err(|e| e.to_string()))
        {
            Ok(_) => overlay_export
                .add_toast(adw::Toast::new(&format!("Exported to {}", path.display()))),
            Err(e) => overlay_export.add_toast(adw::Toast::new(&format!("Export failed: {e}"))),
        }
    });

    let reset_import = reset_values.clone();
    let overlay_import = overlay.clone();
    import_preset_btn.connect_clicked(move |_| {
        let path = exported_preset_path();
        match fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|text| serde_json::from_str::<HyprConfig>(&text).map_err(|e| e.to_string()))
        {
            Ok(cfg) => {
                reset_import(cfg);
                overlay_import.add_toast(adw::Toast::new(
                    "Exported preset imported. Click Save and apply to persist.",
                ));
            }
            Err(e) => overlay_import.add_toast(adw::Toast::new(&format!("Import failed: {e}"))),
        }
    });

    let named_for_save = named_preset.clone();
    let collect_named_save = collect_config.clone();
    let overlay_named_save = overlay.clone();
    save_named_preset_btn.connect_clicked(move |_| {
        match save_named_preset(&named_for_save.text(), &collect_named_save()) {
            Ok(_) => overlay_named_save.add_toast(adw::Toast::new("Named preset saved")),
            Err(e) => overlay_named_save
                .add_toast(adw::Toast::new(&format!("Named preset save failed: {e}"))),
        }
    });

    let named_for_load = named_preset.clone();
    let reset_named_load = reset_values.clone();
    let overlay_named_load = overlay.clone();
    load_named_preset_btn.connect_clicked(move |_| {
        match load_named_preset(&named_for_load.text()) {
            Ok(cfg) => {
                reset_named_load(cfg);
                overlay_named_load.add_toast(adw::Toast::new("Named preset loaded"));
            }
            Err(e) => overlay_named_load
                .add_toast(adw::Toast::new(&format!("Named preset load failed: {e}"))),
        }
    });

    let named_for_delete = named_preset.clone();
    let overlay_named_delete = overlay.clone();
    delete_named_preset_btn.connect_clicked(move |_| {
        match delete_named_preset(&named_for_delete.text()) {
            Ok(_) => overlay_named_delete.add_toast(adw::Toast::new("Named preset deleted")),
            Err(e) => overlay_named_delete
                .add_toast(adw::Toast::new(&format!("Named preset delete failed: {e}"))),
        }
    });

    let window_named_list = window.clone();
    list_named_preset_btn.connect_clicked(move |_| {
        let text = list_named_presets();
        show_text_window(
            &window_named_list,
            "Named presets",
            if text.is_empty() {
                "No named presets"
            } else {
                &text
            },
        );
    });

    let save_for_auto = save_button.clone();
    let auto_apply = auto_apply_sw.clone();
    let auto = move || {
        if auto_apply.is_active() {
            save_for_auto.emit_clicked();
        }
    };
    let auto = Rc::new(auto);
    for sp in [
        &gaps_in,
        &gaps_out,
        &border_sz,
        &rounding,
        &active_opacity,
        &inactive_opacity,
        &shadow_range,
        &blur_size,
        &blur_passes,
        &sensitivity,
    ] {
        let auto_c = auto.clone();
        sp.connect_value_changed(move |_| auto_c());
    }
    for sw in [
        &resize_sw,
        &tearing_sw,
        &shadow_sw,
        &blur_sw,
        &anim_sw,
        &natural_scroll_sw,
        &logo_sw,
    ] {
        let auto_c = auto.clone();
        sw.connect_active_notify(move |_| auto_c());
    }
    let auto_c = auto.clone();
    layout_row.connect_selected_notify(move |_| auto_c());

    window.present();
    make_window_floating();
}
