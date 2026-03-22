use common::{
    local::{
        config::{LogItem, SystemConfiguration},
        status::{AudioSourceState, CombinedStatus},
    },
    mem::{network::ConnectionInfo, typeflags::MessageType},
    protocol::{
        message::{Heartbeat, LargeMessage, Message, SmallMessage},
        request::Request,
    },
};
use crossbeam_channel::{unbounded, Receiver};

use crate::{
    actions::{ ActionID},
    theme::{self, Theme},
    udp::UdpClient,
    widget::textentry::TextEntry,
    window::{
        connection::NetworkMemory, logs::LogWindowMemory, navigation::NavigationWindowMemory,
        performance::PerformanceWindowMemory, playback::PlaybackWindowMemory,
        security::SecurityWindowMemory, DockTabRenderer, WindowTab,
    },
};
use egui::{Context, FontFamily};
use egui_dock::{DockState, TabViewer};
use egui_keybind::{Shortcut};
use std::collections::HashMap;

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
    pub shortcuts: HashMap<ActionID, Shortcut>,
    pub host_connection_info: ConnectionInfo,
    pub local_memory: LocalMemory,
    pub theme: Theme,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct LocalMemory {
    pub playback: PlaybackWindowMemory,
    pub log: LogWindowMemory,
    pub performance: PerformanceWindowMemory,
    pub security: SecurityWindowMemory,
    pub navigation: NavigationWindowMemory,
    pub network: NetworkMemory,
    pub dock_state: egui_dock::DockState<WindowTab>,
}

impl Default for LocalMemory {
    fn default() -> Self {
        Self {
            playback: PlaybackWindowMemory::default(),
            log: LogWindowMemory::default(),
            navigation: NavigationWindowMemory::default(),
            performance: PerformanceWindowMemory::default(),
            security: SecurityWindowMemory::default(),
            dock_state: DockState::new(vec![WindowTab::SourcesTime]),
            network: NetworkMemory::default(),
        }
    }
}

impl Default for ClicksMonitorApp {
    fn default() -> Self {
        Self {
            shortcuts: crate::actions::all_default_shortcuts(),
            system_config: SystemConfiguration::default(),
            sources_gains: vec![0.0f32; 32],
            ctx: egui::Context::default(),
            status: CombinedStatus::default(),
            udp_client: UdpClient::new(),
            rx: unbounded().1,
            local_memory: LocalMemory::default(),
            theme: theme::NATIVE,
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

        egui_extras::install_image_loaders(&cc.egui_ctx);

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

    fn update_rx_tally(&mut self, msg_type: MessageType, size: usize) {
        let entry = self
            .udp_client
            .rx_message_tally
            .entry(msg_type)
            .or_insert((0, 0));

        entry.0 += 1;
        entry.1 += size;
    }
    fn handle_small_message(&mut self, msg: SmallMessage) {
        match msg {
            SmallMessage::TransportData(status) => {
                self.status.transport = status;
            }
            SmallMessage::PlaybackData(status) => {
                self.status.sources[2 + status.channel as usize] =
                    AudioSourceState::PlaybackStatus(status);
            }
            SmallMessage::TimecodeData(status) => {
                self.status.sources[1] = AudioSourceState::TimeStatus(status);
            }
            SmallMessage::BeatData(beat) => {
                self.status.sources[0] = AudioSourceState::BeatStatus(beat);
            }
            SmallMessage::ShutdownOccured => {
                self.udp_client.active = false;
            }
            SmallMessage::Heartbeat(heartbeat) => {
                self.handle_heartbeat(heartbeat);
            }
            _ => {}
        }
    }

    fn handle_large_message(&mut self, msg: LargeMessage) {
        match msg {
            LargeMessage::CueData(cue) => {
                self.status.cue = cue;
            }
            LargeMessage::ShowData(show) => {
                self.status.show = show;
            }
            LargeMessage::PlaybackHandlerChanged(status) => {
                self.status.playback_status = status;
            }
            LargeMessage::NetworkChanged(status) => {
                self.status.network_status = status;
            }
            LargeMessage::JACKStateChanged(status) => {
                self.status.jack_status = status;
            }
            LargeMessage::ConfigurationChanged(config) => {
                self.apply_config(config);
            }
            LargeMessage::Log(item) => {
                self.log_entries.push(item);
            }
        }
    }

    fn handle_heartbeat(&mut self, heartbeat: Heartbeat) {
        self.last_heartbeat = heartbeat;

        let heartbeats = &mut self.local_memory.performance.heartbeats;
        heartbeats.push_back(heartbeat);

        while heartbeats.len() > 300 {
            heartbeats.pop_front();
        }
    }

    fn apply_config(&mut self, config: SystemConfiguration) {
        for (i, channel) in config.channels.iter().enumerate() {
            self.sources_gains[i] = channel.gain;
        }

        self.system_config = config;
    }

    fn handle_udp_message(&mut self, msg: Message, size: usize) {
        self.udp_client.active = true;
        self.update_rx_tally(msg.to_type(), size);

        match msg {
            Message::Small(s) => self.handle_small_message(s),
            Message::Large(l) => self.handle_large_message(l),
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
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.local_memory.navigation.multiwindow_mode {
                let mut added_nodes = vec![];
                let mut dock_state = self.local_memory.dock_state.clone();
                let dock = egui_dock::DockArea::new(&mut dock_state);
                egui::CentralPanel::default().show(ctx, |ui| {
                    dock.show_add_popup(!self.local_memory.navigation.lock_navigation)
                        .show_add_buttons(!self.local_memory.navigation.lock_navigation)
                        .draggable_tabs(!self.local_memory.navigation.lock_navigation)
                        .show_close_buttons(!self.local_memory.navigation.lock_navigation)
                        .show_leaf_close_all_buttons(!self.local_memory.navigation.lock_navigation)
                        .show_leaf_collapse_buttons(!self.local_memory.navigation.lock_navigation)
                        .show_inside(
                            ui,
                            &mut DockTabRenderer {
                                app_state: self,
                                added_nodes: &mut added_nodes,
                            },
                        );
                });
                added_nodes.drain(..).for_each(|(tab, surface, node)| {
                    dock_state.set_focused_node_and_surface((surface, node));
                    dock_state.push_to_focused_leaf(tab);
                });

                self.local_memory.dock_state = dock_state;
            } else {
                let mut tab = self.local_memory.navigation.current_single_tab.clone();
                (DockTabRenderer {
                    app_state: self,
                    added_nodes: &mut vec![],
                })
                .ui(ui, &mut tab);
            }
        });
    }

    pub fn handle_keybinds(&mut self) {
        let keys = self.shortcuts.keys().cloned().collect::<Vec<ActionID>>();
        for action in keys {
            let shortcut = self.shortcuts.get(&action);
        
            if let Some(shortcut) = shortcut &&

            let Some(kbd) = shortcut.keyboard() && !self.ctx.wants_keyboard_input()
                && self.ctx.input(|i| {
                    i.modifiers == kbd.modifiers
                        && i.key_pressed(kbd.logical_key)
                })
            {
                crate::actions::exec_action(self, action);
            }
        }
            
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
        // Business logic
        self.handle_all_udp_messages();
        self.handle_keybinds();

        self.render_statusbar(ctx);
        self.render_navigation_panel(ctx);
        self.render_main_panel(ctx);

        self.text_entry = self.text_entry.clone().display(self).clone();
        ctx.request_repaint();
    }
}
