use common::{
    local::{
        config::SystemConfiguration,
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
};
use egui::{FontFamily, FontId, TextStyle};
use std::collections::BTreeMap;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    #[serde(skip)]
    pub status: CombinedStatus,
    #[serde(skip)]
    pub heartbeat: Heartbeat,
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
    pub host_connection_info: ConnectionInfo,
    pub tab: TabView,
    pub layout_settings: LayoutSettings,
    pub theme: Theme,
    pub allow_interaction: bool,
    pub require_password: bool,
    pub password: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub enum TabView {
    Control,
    Sources,
    Cue,
    Options,
}

#[derive(serde::Deserialize, serde::Serialize, Default, PartialEq)]
pub enum ConfigurationEditorTab {
    Routing,
    #[default]
    Network,
    Channels,
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct LayoutSettings {
    pub configuration_editor_tab: ConfigurationEditorTab,
    pub routing_window_open: bool,
    pub network_info_window_open: bool,
    pub channel_edit_window_open: bool,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            system_config: SystemConfiguration::default(),
            sources_gains: vec![0.0f32; 32],
            ctx: egui::Context::default(),
            status: CombinedStatus::default(),
            udp_client: UdpClient::new(),
            rx: unbounded().1,
            tab: TabView::Options,
            layout_settings: LayoutSettings::default(),
            theme: theme::DARK,
            host_connection_info: ConnectionInfo::default(),
            allow_interaction: true,
            require_password: true,
            password: String::new(),
            text_entry: TextEntry::new(),
            heartbeat: Heartbeat::default(),
        }
    }
}

impl TemplateApp {
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");

    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, udp_client: UdpClient) -> Self {
        let mut a = if let Some(storage) = cc.storage {
            serde_json::from_str(
                &eframe::Storage::get_string(storage, eframe::APP_KEY).unwrap_or_default(),
            )
            .unwrap_or_default()
        } else {
            Self::default()
        };
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
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

    pub fn handle_cc_message(&mut self, msg: Message, size: usize) {
        self.udp_client.active = true;
        let tally_pre = self
            .udp_client
            .rx_message_tally
            .get(&msg.to_type())
            .unwrap_or(&(0, 0));
        self.udp_client
            .rx_message_tally
            .insert(msg.to_type(), (tally_pre.0 + 1, tally_pre.1 + size));
        //println!("Received Message {:?}", msg.clone());
        match msg {
            Message::Small(SmallMessage::TransportData(status)) => {
                self.status.sources[1] = AudioSourceState::TimeStatus(status.ltc.clone());
                self.status.transport = status;
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
            Message::Small(SmallMessage::Heartbeat(heartbeat)) => self.heartbeat = heartbeat,
            _ => {}
        }
    }

    fn setup_custom_fonts(&self, ctx: &egui::Context) {
        // Load the font from file
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
}

impl eframe::App for TemplateApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.udp_client
            .send_msg(Request::Unsubscribe(self.udp_client.local.clone()));
    }
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(eframe::APP_KEY, serde_json::to_string(&self).unwrap());
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        loop {
            match self.rx.try_recv() {
                Ok((msg, size)) => self.handle_cc_message(msg, size),
                Err(crossbeam_channel::TryRecvError::Empty) => break,
                Err(err) => println!("rx error: {}", err),
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            crate::window::statusbar::display(self, ui);
        });

        egui::SidePanel::left("sources-panel")
            .resizable(false)
            .show_animated(ctx, true, |ui| {
                crate::window::navigation::display(self, ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            TabView::Sources => {
                crate::window::sources::display(self, ui);
            }
            TabView::Cue => {
                crate::window::cue::display(self, ui);
            }
            TabView::Control => {
                crate::window::control::display(self, ui);
            }
            TabView::Options => {
                let width = ui.available_width() / 3.0;
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(width);
                        crate::window::connection::display(self, ui);
                        ui.separator();
                        if self.udp_client.active {
                            crate::window::navigation::configuration_tab_buttons(self, ui);
                            ui.separator();
                            match self.layout_settings.configuration_editor_tab {
                                ConfigurationEditorTab::Routing => {
                                    crate::window::jack::render_routing_matrix(self, ui)
                                }
                                ConfigurationEditorTab::Network => {
                                    crate::window::connection::details(self, ui);
                                    crate::window::connection::clients_table(self, ui);
                                }
                                ConfigurationEditorTab::Channels => {
                                    crate::window::sources::configuration_window(self, ui)
                                }
                            }
                        }
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.set_width(width);
                        crate::window::system_config::display(self, ui);
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.set_width(width);
                        crate::window::local_config::display(self, ui);
                    });
                });
            }
        });

        self.text_entry = self.text_entry.clone().display(self).clone();
    }
}
