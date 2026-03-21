use common::{
    local::{
        config::{LogItem, SystemConfiguration},
        status::{AudioSourceState, CombinedStatus},
    },
    mem::network::ConnectionInfo,
    protocol::{
        message::{Heartbeat, LargeMessage, Message, SmallMessage},
        request::Request,
    },
};
use crossbeam_channel::{unbounded, Receiver};

use crate::{
    theme::{self, Theme},
    udp::UdpClient,
    widget::textentry::TextEntry,
    window::{
        logs::LogWindowMemory, performance::PerformanceWindowMemory,
        playback::PlaybackWindowMemory, security::SecurityWindowMemory, WindowTab,
    },
};
use egui::{Context, FontFamily};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ClicksMonitorApp {
    #[serde(skip)]
    pub status: CombinedStatus,
    #[serde(skip)]
    pub last_heartbeat: Heartbeat,
    #[serde(skip)]
    pub udp_client: UdpClient,
    #[serde(skip)]
    pub rx: Receiver<(Message, usize)>,
    #[serde(skip)]
    pub ctx: egui::Context,
    #[serde(skip)]
    pub sources_gains: Vec<f32>,
    #[serde(skip)]
    pub text_entry: TextEntry,
    #[serde(skip)]
    pub system_config: SystemConfiguration,
    #[serde(skip)]
    pub log_entries: Vec<LogItem>,
    pub host_connection_info: ConnectionInfo,
    pub local_memory: LocalMemory,
    pub theme: Theme,
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct LocalMemory {
    pub current_tab: WindowTab,
    pub playback: PlaybackWindowMemory,
    pub log: LogWindowMemory,
    pub performance: PerformanceWindowMemory,
    pub security: SecurityWindowMemory,
}

impl Default for ClicksMonitorApp {
    fn default() -> Self {
        Self {
            system_config: SystemConfiguration::default(),
            sources_gains: vec![0.0f32; 32],
            ctx: egui::Context::default(),
            status: CombinedStatus::default(),
            udp_client: UdpClient::new(),
            rx: unbounded().1,
            local_memory: LocalMemory::default(),
            theme: theme::DARK,
            host_connection_info: ConnectionInfo::default(),
            text_entry: TextEntry::new(),
            last_heartbeat: Heartbeat::default(),
            log_entries: vec![],
        }
    }
}

impl ClicksMonitorApp {
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");

    pub fn new(cc: &eframe::CreationContext<'_>, udp_client: UdpClient) -> Self {
        let mut a = if let Some(storage) = cc.storage {
            serde_json::from_str(
                &eframe::Storage::get_string(storage, eframe::APP_KEY).unwrap_or_default(),
            )
            .unwrap_or_default()
        } else {
            Self::default()
        };
        a.udp_client = udp_client;
        a.rx = a.udp_client.get_receiver();
        a.set_theme(cc.egui_ctx.clone(), a.theme);
        a.ctx = cc.egui_ctx.clone();
        a.setup_custom_fonts(&a.ctx);
        a
    }

    pub fn set_theme(&mut self, ctx: egui::Context, theme: Theme) {
        if theme == theme::NATIVE {
            ctx.set_visuals(egui::Visuals::default());
        } else {
            ctx.set_visuals(theme.visuals(ctx.style().visuals.clone()));
        }
        self.theme = theme;
    }

    pub fn handle_udp_message(&mut self, msg: Message, size: usize) {
        self.udp_client.active = true;
        let tally_pre = self
            .udp_client
            .rx_message_tally
            .get(&msg.to_type())
            .unwrap_or(&(0, 0));
        self.udp_client
            .rx_message_tally
            .insert(msg.to_type(), (tally_pre.0 + 1, tally_pre.1 + size));
        match msg {
            Message::Small(SmallMessage::TransportData(status)) => {
                self.status.transport = status;
            }
            Message::Small(SmallMessage::PlaybackData(status)) => {
                self.status.sources[2 + status.channel as usize] =
                    AudioSourceState::PlaybackStatus(status);
            }
            Message::Small(SmallMessage::TimecodeData(status)) => {
                self.status.sources[1] = AudioSourceState::TimeStatus(status);
            }
            Message::Small(SmallMessage::BeatData(beat)) => {
                self.status.sources[0] = AudioSourceState::BeatStatus(beat);
            }
            Message::Large(LargeMessage::CueData(cue)) => {
                self.status.cue = cue;
            }
            Message::Large(LargeMessage::ShowData(show)) => {
                self.status.show = show;
            }
            Message::Large(LargeMessage::PlaybackHandlerChanged(status)) => {
                self.status.playback_status = status;
            }
            Message::Large(LargeMessage::NetworkChanged(status)) => {
                self.status.network_status = status;
            }
            Message::Large(LargeMessage::JACKStateChanged(status)) => {
                self.status.jack_status = status;
            }
            Message::Small(SmallMessage::ShutdownOccured) => {
                self.udp_client.active = false;
            }
            Message::Large(LargeMessage::ConfigurationChanged(config)) => {
                for i in 0..self.sources_gains.len() {
                    self.sources_gains[i] = config.channels[i].gain;
                }
                self.system_config = config;
            }
            Message::Small(SmallMessage::Heartbeat(heartbeat)) => {
                self.last_heartbeat = heartbeat;
                self.local_memory
                    .performance
                    .heartbeats
                    .push_back(heartbeat);
                while self.local_memory.performance.heartbeats.len() > 300 {
                    self.local_memory.performance.heartbeats.pop_front();
                }
            }
            Message::Large(LargeMessage::Log(item)) => self.log_entries.push(item),
            _ => {}
        }
    }

    fn setup_custom_fonts(&self, ctx: &egui::Context) {
        let font_data =
            include_bytes!("../assets/fonts/DroidSansMNerdFontMono-Regular.otf").to_vec();

        let mut fonts = egui::FontDefinitions::default();

        // Register the custom monospace font under a unique name
        fonts.font_data.insert(
            "mono_custom".to_string(),
            egui::FontData::from_owned(font_data).into(),
        );

        // Set "my_mono" as the font to use for the monospace family
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .clear();

        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .push("mono_custom".to_string());

        // Apply the font definitions
        ctx.set_fonts(fonts);
    }

    fn handle_all_udp_messages(&mut self) {
        loop {
            match self.rx.try_recv() {
                Ok((msg, size)) => self.handle_udp_message(msg, size),
                Err(crossbeam_channel::TryRecvError::Empty) => break,
                Err(err) => println!("rx error: {}", err),
            }
        }
    }
    fn render_navigation_panel(&mut self, ctx: &Context) {
        egui::SidePanel::left("navigation-panel")
            .resizable(false)
            .show_animated(ctx, true, |ui| {
                crate::window::navigation::display(self, ui);
            });
    }

    fn render_statusbar(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            crate::window::statusbar::display(self, ui);
        });
    }

    fn render_main_panel(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| match self.local_memory.current_tab {
            WindowTab::SourcesOverview => {
                crate::window::sources::display(self, ui);
            }
            WindowTab::CueTimeline => {
                crate::window::cue::display(self, ui);
            }
            WindowTab::ControlTransport => {
                crate::window::transport::display(self, ui);
            }
            WindowTab::SourcesTime => {
                crate::window::time::display(self, ui);
            }
            WindowTab::SourcesPlayback => {
                crate::window::playback::display(self, ui);
            }
            WindowTab::CueBeats => {
                crate::window::beats::display(self, ui);
            }
            WindowTab::CueEvents => {
                crate::window::events::display(self, ui);
            }
            WindowTab::SystemLogs => {
                crate::window::logs::display(self, ui);
            }
            WindowTab::SystemPerformance => {
                crate::window::performance::display(self, ui);
            }
            WindowTab::SystemAudio => {
                crate::window::settings_audio::display(self, ui);
            }
            WindowTab::SystemNetwork => {
                crate::window::network::display(self, ui);
            }
            _ => {}
        });
    }
}

impl eframe::App for ClicksMonitorApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.udp_client
            .send_msg(Request::Unsubscribe(self.udp_client.local));
    }
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(eframe::APP_KEY, serde_json::to_string(&self).unwrap());
    }

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.handle_all_udp_messages();

        self.render_statusbar(ctx);
        self.render_navigation_panel(ctx);
        self.render_main_panel(ctx);

        self.text_entry = self.text_entry.clone().display(self).clone();
    }
}
