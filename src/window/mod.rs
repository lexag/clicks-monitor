pub mod beats;
pub mod connection;
pub mod cue;
pub mod events;
pub mod jack;
pub mod local_config;
pub mod logs;
pub mod navigation;
pub mod network;
pub mod performance;
pub mod playback;
pub mod settings_audio;
pub mod sources;
pub mod statusbar;
pub mod system_config;
pub mod time;
pub mod transport;

#[derive(serde::Deserialize, serde::Serialize)]
pub enum WindowTab {
    SourcesOverview,
    SourcesTime,
    SourcesPlayback,
    CueTimeline,
    CueBeats,
    CueEvents,
    ControlTransport,
    ControlRunEvent,
    ControlSystem,
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
            Self::ControlSystem | Self::ControlRunEvent | Self::ControlTransport => {
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
            Self::ControlSystem => "File System",
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
