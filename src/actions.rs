use crate::{app::ClicksMonitorApp, window::WindowTab};
use egui::{Key, KeyboardShortcut, Modifiers};
use egui_keybind::Shortcut;
use std::collections::HashMap;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ShortcutMap {
    pub actions: Vec<ActionID>,
    pub shortcuts: Vec<Shortcut>,
    #[serde(skip)]
    map: HashMap<ActionID, usize>,
}

impl ShortcutMap {
    pub fn new() -> Self {
        Self {
            actions: vec![],
            shortcuts: vec![],
            map: HashMap::new(),
        }
    }

    pub fn rebuild(&mut self) {
        for (i, k) in self.actions.iter().enumerate() {
            self.map.insert(k.clone(), i);
        }
    }

    pub fn add(&mut self, action_id: ActionID, shortcut: Shortcut) {
        self.actions.push(action_id.clone());
        self.shortcuts.push(shortcut);
        self.map.insert(action_id, self.actions.len() - 1);
    }

    pub fn get(&mut self, action_id: &ActionID) -> Option<Shortcut> {
        let idx = *self.map.get(action_id)?;
        self.shortcuts.get(idx).copied()
    }
}

#[derive(
    Hash, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Clone, Debug,
)]
pub enum ActionID {
    Tab(WindowTab),
    ToggleInteractionLock,
    TransportStart,
    TransportStop,
    TransportZero,
    LoadNextCue,
    LoadPrevCue,
    ToggleVLT,
    PlayCuedPlayback,
    PlayCuedPlaybackAndClear,
    ReloadNetwork,
}

pub fn exec_action(app: &mut ClicksMonitorApp, action_id: ActionID) {
    let wctx = window_context(action_id.clone());
    if wctx.is_some() && wctx != crate::window::navigation::current_focused_tab(app) {
        return;
    }

    match action_id {
        ActionID::Tab(tab) => crate::window::navigation::switch_to_tab(app, tab),
        ActionID::ToggleInteractionLock => crate::window::security::try_toggle_lock(app),

        ActionID::TransportStart => crate::window::transport::start(app),
        ActionID::TransportStop => crate::window::transport::stop(app),
        ActionID::TransportZero => crate::window::transport::zero(app),
        ActionID::LoadNextCue => crate::window::transport::next(app),
        ActionID::LoadPrevCue => crate::window::transport::prev(app),
        ActionID::ToggleVLT => crate::window::transport::vlt_toggle(app),
        ActionID::PlayCuedPlayback => crate::window::playback::play_clip_cue(app),
        ActionID::PlayCuedPlaybackAndClear => crate::window::playback::play_clip_cue_once(app),
        ActionID::ReloadNetwork => crate::window::connection::try_connect(app),
    };
}

pub fn window_context(action_id: ActionID) -> Option<WindowTab> {
    match action_id {
        ActionID::ToggleVLT => Some(WindowTab::ControlTransport),
        ActionID::PlayCuedPlayback | ActionID::PlayCuedPlaybackAndClear => {
            Some(WindowTab::SourcesPlayback)
        }
        _ => None,
    }
}

pub fn all_default_shortcuts() -> ShortcutMap {
    let mut shortcuts = ShortcutMap::new();

    add_navigation_shortcuts(&mut shortcuts);
    shortcuts.add(ActionID::TransportStart, press(Key::Enter));
    shortcuts.add(ActionID::TransportStop, press(Key::Backspace));
    shortcuts.add(ActionID::TransportZero, press(Key::Home));
    shortcuts.add(ActionID::LoadNextCue, press(Key::PageDown));
    shortcuts.add(ActionID::LoadPrevCue, press(Key::PageUp));
    shortcuts.add(ActionID::ToggleVLT, press(Key::Space));
    shortcuts.add(ActionID::PlayCuedPlayback, press(Key::Space));
    shortcuts.add(ActionID::PlayCuedPlaybackAndClear, shift(Key::Space));

    shortcuts.add(ActionID::ReloadNetwork, shift(Key::F12));

    shortcuts.add(ActionID::ToggleInteractionLock, shift(Key::L));

    shortcuts
}

fn add_navigation_shortcuts(shortcuts: &mut ShortcutMap) {
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

        shortcuts.add(ActionID::Tab(tab), press(*key));
    }
}

fn press(key: Key) -> Shortcut {
    Shortcut::new(Some(KeyboardShortcut::new(Modifiers::NONE, key)), None)
}
fn ctrl(key: Key) -> Shortcut {
    Shortcut::new(Some(KeyboardShortcut::new(Modifiers::CTRL, key)), None)
}
fn shift(key: Key) -> Shortcut {
    Shortcut::new(Some(KeyboardShortcut::new(Modifiers::SHIFT, key)), None)
}
