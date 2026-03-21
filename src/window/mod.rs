use crate::app::ClicksMonitorApp;

pub mod appearance;
pub mod beats;
pub mod connection;
pub mod cue;
pub mod events;
pub mod file_system;
pub mod hotkeys;
pub mod logs;
pub mod navigation;
pub mod network;
pub mod performance;
pub mod playback;
pub mod run_event;
pub mod security;
pub mod settings_audio;
pub mod sources;
pub mod statusbar;
pub mod time;
pub mod transport;

#[derive(serde::Deserialize, serde::Serialize, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WindowTab {
    SourcesOverview,
    #[default]
    SourcesTime,
    SourcesPlayback,
    CueTimeline,
    CueBeats,
    CueEvents,
    ControlTransport,
    ControlRunEvent,
    ControlFileSystem,
    SystemLogs,
    SystemPerformance,
    SystemNetwork,
    SystemAudio,
    PreferencesAppearance,
    PreferencesHotkeys,
    PreferencesSecurity,
}

impl WindowTab {
    pub fn category(&self) -> WindowCategory {
        match self {
            Self::SourcesTime | Self::SourcesOverview | Self::SourcesPlayback => {
                WindowCategory::Sources
            }
            Self::CueTimeline | Self::CueBeats | Self::CueEvents => WindowCategory::Cue,
            Self::ControlFileSystem | Self::ControlRunEvent | Self::ControlTransport => {
                WindowCategory::Control
            }
            Self::SystemLogs
            | Self::SystemPerformance
            | Self::SystemNetwork
            | Self::SystemAudio => WindowCategory::System,
            Self::PreferencesAppearance | Self::PreferencesHotkeys | Self::PreferencesSecurity => {
                WindowCategory::Preferences
            }
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::SourcesOverview => "Overview",
            Self::SourcesTime => "Time",
            Self::SourcesPlayback => "Playback",
            Self::CueTimeline => "Timeline",
            Self::CueBeats => "Beats",
            Self::CueEvents => "Events",
            Self::ControlTransport => "Transport",
            Self::ControlRunEvent => "Run Event",
            Self::ControlFileSystem => "File System",
            Self::SystemLogs => "Logs",
            Self::SystemPerformance => "Performance",
            Self::SystemNetwork => "Network",
            Self::SystemAudio => "Audio",
            Self::PreferencesAppearance => "Appearance",
            Self::PreferencesHotkeys => "Hotkeys",
            Self::PreferencesSecurity => "Security",
        }
        .to_string()
    }

    pub fn list() -> Vec<WindowTab> {
        vec![
            WindowTab::SourcesOverview,
            WindowTab::SourcesTime,
            WindowTab::SourcesPlayback,
            WindowTab::CueTimeline,
            WindowTab::CueBeats,
            WindowTab::CueEvents,
            WindowTab::ControlTransport,
            WindowTab::ControlRunEvent,
            WindowTab::ControlFileSystem,
            WindowTab::SystemLogs,
            WindowTab::SystemPerformance,
            WindowTab::SystemNetwork,
            WindowTab::SystemAudio,
            WindowTab::PreferencesAppearance,
            WindowTab::PreferencesHotkeys,
            WindowTab::PreferencesSecurity,
        ]
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq)]
pub enum WindowCategory {
    Sources,
    Cue,
    Control,
    System,
    Settings,
    Preferences,
    None,
}

impl WindowCategory {
    pub fn name(&self) -> String {
        match self {
            Self::Sources => "Sources",
            Self::Cue => "Cue",
            Self::Control => "Control",
            Self::System => "System",
            Self::Settings => "Settings (Core)",
            Self::Preferences => "Preferences (Monitor)",
            Self::None => "",
        }
        .to_string()
    }
}

pub struct DockTabRenderer<'a> {
    pub app_state: &'a mut ClicksMonitorApp,
}

impl<'a> egui_dock::TabViewer for DockTabRenderer<'a> {
    type Tab = WindowTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.name().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let renderer = match tab {
            WindowTab::SourcesOverview => crate::window::sources::display,
            WindowTab::CueTimeline => crate::window::cue::display,
            WindowTab::ControlTransport => crate::window::transport::display,
            WindowTab::SourcesTime => crate::window::time::display,
            WindowTab::SourcesPlayback => crate::window::playback::display,
            WindowTab::CueBeats => crate::window::beats::display,
            WindowTab::CueEvents => crate::window::events::display,
            WindowTab::SystemLogs => crate::window::logs::display,
            WindowTab::SystemPerformance => crate::window::performance::display,
            WindowTab::SystemAudio => crate::window::settings_audio::display,
            WindowTab::SystemNetwork => crate::window::network::display,
            WindowTab::ControlRunEvent => crate::window::run_event::display,
            WindowTab::ControlFileSystem => crate::window::file_system::display,
            WindowTab::PreferencesAppearance => crate::window::appearance::display,
            WindowTab::PreferencesHotkeys => crate::window::hotkeys::display,
            WindowTab::PreferencesSecurity => crate::window::security::display,
        };
        (renderer)(self.app_state, ui);
    }
}
