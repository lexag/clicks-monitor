use chrono::{DateTime, Utc};

use crate::app::TemplateApp;
use common::{
    local::status::CombinedStatus,
    mem::{
        network::IpAddress,
        typeflags::{MessageType, RequestType},
    },
    protocol::request::Request,
};
use egui::Widget;

pub fn display(app: &mut TemplateApp, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Connection").heading());
    settings(app, ui);
}

pub fn clients_table(app: &mut TemplateApp, ui: &mut egui::Ui) {
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

pub fn settings(app: &mut TemplateApp, ui: &mut egui::Ui) {
    let colw = ui.available_width() / 6.0;
    if !app.allow_interaction {
        ui.disable();
    }
    ui.horizontal(|ui| {
        ui.label("Host address");
        egui::DragValue::new(&mut app.host_connection_info.address.addr[0]).ui(ui);
        egui::DragValue::new(&mut app.host_connection_info.address.addr[1]).ui(ui);
        egui::DragValue::new(&mut app.host_connection_info.address.addr[2]).ui(ui);
        egui::DragValue::new(&mut app.host_connection_info.address.addr[3]).ui(ui);
        egui::DragValue::new(&mut app.host_connection_info.address.port).ui(ui);
    });
    ui.horizontal(|ui| {
        ui.label("Local address");
        ui.label(format!(
            "{}.{}.{}.{}:{}",
            app.udp_client.local.address.addr[0],
            app.udp_client.local.address.addr[1],
            app.udp_client.local.address.addr[2],
            app.udp_client.local.address.addr[3],
            app.udp_client.local.address.port
        ))
    });

    egui::Grid::new("connection_settings")
        .num_columns(5)
        .max_col_width(colw)
        .show(ui, |ui| {
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
                && app.allow_interaction
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

pub fn details(app: &mut TemplateApp, ui: &mut egui::Ui) {
    //    egui::Grid::new("connection_details")
    //        .num_columns(2)
    //        .max_col_width(ui.available_width() / 2.0)
    //        .show(ui, |ui| {
    //            ui.label("UDP channel cue");
    //            ui.label(app.rx.len().to_string());
    //            ui.end_row();
    //        });
    //        ui.separator();

    egui::Grid::new("connection-messages").show(ui, |ui| {
        ui.label("Status Message");
        ui.label("# received");
        ui.label("Total data");
        ui.end_row();

        for kind in [
            MessageType::CueData,
            MessageType::ConfigurationChanged,
            MessageType::JACKStateChanged,
            MessageType::NetworkChanged,
            MessageType::ShowData,
            MessageType::TransportData,
            MessageType::Heartbeat,
        ] {
            ui.label(format!("{:?}", kind));
            let tally = app
                .udp_client
                .rx_message_tally
                .get(&kind)
                .unwrap_or(&(0, 0));
            ui.label(tally.0.to_string());
            ui.label(format!(
                "{:0>3} {:0>3} {:0>3} kB",
                tally.1 / 1000000000 % 1000,
                tally.1 / 1000000 % 1000,
                tally.1 / 1000 % 1000
            ));
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
            ui.label(format!(
                "{:0>3} {:0>3} {:0>3} kB",
                tally.1 / 1000000000 % 1000,
                tally.1 / 1000000 % 1000,
                tally.1 / 1000 % 1000
            ));
            ui.end_row();
        }
    });
}
