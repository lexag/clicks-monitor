use crate::{app::ClicksMonitorApp, window::WindowTab};
use egui::{Key, KeyboardShortcut, Modifiers};
use egui_keybind::Shortcut;
use std::collections::HashMap;

type Map = HashMap<ActionID, Shortcut>;

#[derive(
    Hash, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Clone, Debug,
)]
pub enum ActionID {
    Tab(WindowTab),
}

pub fn exec_action(app: &mut ClicksMonitorApp, action_id: ActionID) {
    match action_id {
        ActionID::Tab(tab) => crate::window::navigation::switch_to_tab(app, tab),
    };
}

pub fn all_default_shortcuts() -> Map {
    let mut shortcuts = Map::new();

    add_navigation_shortcuts(&mut shortcuts);

    shortcuts
}

fn add_navigation_shortcuts(shortcuts: &mut Map) {
    for (tab, key) in WindowTab::list().iter().zip(
        [
            Key::F1,
            Key::F2,
            Key::F3,
            Key::F4,
            Key::F5,
            Key::F6,
            Key::F7,
            Key::F8,
            Key::F9,
            Key::F10,
            Key::F11,
            Key::F12,
        ]
        .iter(),
    ) {
        let tab = *tab;

        shortcuts.insert(ActionID::Tab(tab), single_hotkey(*key));
    }
}

fn single_hotkey(key: Key) -> Shortcut {
    Shortcut::new(Some(KeyboardShortcut::new(Modifiers::NONE, key)), None)
}
