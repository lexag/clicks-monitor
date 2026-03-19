use std::ops::{Add, Div};

use common::mem::typeflags::{MessageType, RequestType};
use egui::RichText;

use crate::app::TemplateApp;

pub fn display(app: &mut TemplateApp, ui: &mut egui::Ui) {
    egui::SidePanel::right("network-details-panel")
        .resizable(true)
        .show_inside(ui, |ui| {
            ui.label(RichText::new("This client").heading());
            egui::Grid::new("connection-messages").show(ui, |ui| {
                ui.label("Status Message");
                ui.label("# received");
                ui.label("Total data");
                ui.end_row();

                for (label, kind) in MessageType::all().iter_names() {
                    ui.label(label);
                    let tally = app
                        .udp_client
                        .rx_message_tally
                        .get(&kind)
                        .unwrap_or(&(0, 0));
                    ui.label(tally.0.to_string());
                    ui.label(format_data_amount(tally.1));
                    ui.end_row();
                }
                ui.end_row();

                for kind in [
                    RequestType::ControlCommand,
                    RequestType::Initialize,
                    RequestType::NotifySubscribers,
                    RequestType::Ping,
                    RequestType::RoutingChange,
                    RequestType::SetConfiguration,
                    RequestType::Shutdown,
                    RequestType::Subscribe,
                    RequestType::Unsubscribe,
                ] {
                    ui.label(format!("{:?}", kind));
                    let tally = app
                        .udp_client
                        .tx_message_tally
                        .get(&kind)
                        .unwrap_or(&(0, 0));
                    ui.label(tally.0.to_string());
                    ui.label(format_data_amount(tally.1));
                    ui.end_row();
                }
            });
        });
}

fn format_data_amount(bytes: usize) -> String {
    const SI: &[&str] = &[
        "B", "kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB", "RB", "QB",
    ];

    let bytesf = bytes as f32;

    let n = bytesf.add(0.5).log10().floor();
    let si_idx = n.div(3.0).floor();

    let mut val = bytesf.div(1000.0_f32.powf(si_idx as f32));

    format!("{: >3} {}", val.round(), SI[si_idx as usize])
}
