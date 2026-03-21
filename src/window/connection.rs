use chrono::{DateTime, Utc};

use crate::app::ClicksMonitorApp;
use common::{
    local::status::CombinedStatus,
    mem::{
        network::IpAddress,
        typeflags::{MessageType, RequestType},
    },
    protocol::request::Request,
};
use egui::Widget;

pub fn display(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Connection").heading());
    settings(app, ui);
}

pub fn clients_table(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.label(egui::RichText::new("Active Clients").heading());
        egui::Grid::new("clients-table")
            .striped(true)
            .show(ui, |ui| {
                ui.label("Client name");
                ui.label("IP address");
                ui.label("Port");
                ui.end_row();
                for subscriber in app.status.network_status.subscribers.clone() {
                    ui.label(subscriber.identifier.str());
                    ui.label(subscriber.address.to_string());
                    let timediff = Utc::now().signed_duration_since(
                        DateTime::from_timestamp_secs(subscriber.last_contact as i64).unwrap(),
                    );
                    ui.label(format!("{}m", timediff.num_minutes()));

                    ui.end_row();
                }
            });
    });
}

pub fn settings(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    if !app.local_memory.security.allow_interaction {
        ui.disable();
    }
    egui::Grid::new("connection_settings")
        .num_columns(5)
        .show(ui, |ui| {
            ui.label("Host address");
            ui.horizontal(|ui| {
                egui::DragValue::new(&mut app.host_connection_info.address.addr[0]).ui(ui);
                egui::DragValue::new(&mut app.host_connection_info.address.addr[1]).ui(ui);
                egui::DragValue::new(&mut app.host_connection_info.address.addr[2]).ui(ui);
                egui::DragValue::new(&mut app.host_connection_info.address.addr[3]).ui(ui);
                egui::DragValue::new(&mut app.host_connection_info.address.port).ui(ui);
            });
            ui.end_row();

            ui.label("Local address");
            ui.horizontal(|ui| {
                ui.label(format!(
                    "{}.{}.{}.{}:{}",
                    app.udp_client.local.address.addr[0],
                    app.udp_client.local.address.addr[1],
                    app.udp_client.local.address.addr[2],
                    app.udp_client.local.address.addr[3],
                    app.udp_client.local.address.port
                ))
            });
            ui.end_row();

            ui.label("Identifier");
            egui::TextEdit::singleline(&mut app.udp_client.local.identifier.str())
                .char_limit(16)
                .show(ui);
            ui.end_row();
            if ui
                .button(if app.udp_client.active {
                    "Apply"
                } else {
                    "Connect"
                })
                .clicked()
                && app.local_memory.security.allow_interaction
            {
                match app.udp_client.connect(
                    app.udp_client.local.identifier,
                    app.host_connection_info.address,
                ) {
                    Ok(ci) => {
                        app.host_connection_info = ci;
                    }
                    Err(err) => {
                        println!("Connection error: {}", err);
                    }
                }
            }
            if app.udp_client.active && ui.button("Disconnect").clicked() {
                app.udp_client
                    .send_msg(Request::Unsubscribe(app.udp_client.local.clone()));
                app.udp_client.active = false;
                app.status = CombinedStatus::default();
            }
        })
        .response
        .on_disabled_hover_text(
            "Connection settings are disabled when client is locked. Unlock to access settings.",
        );
}

pub fn details(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    //    egui::Grid::new("connection_details")
    //        .num_columns(2)
    //        .max_col_width(ui.available_width() / 2.0)
    //        .show(ui, |ui| {
    //            ui.label("UDP channel cue");
    //            ui.label(app.rx.len().to_string());
    //            ui.end_row();
    //        });
    //        ui.separator();
}
