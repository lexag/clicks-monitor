use crate::app::ClicksMonitorApp;
use common::{local::status::CombinedStatus, mem::network::IpAddress, protocol::request::Request};

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct NetworkMemory {
    pub target_ip_str: String,
}

pub fn settings(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    if !app.local_memory.security.allow_interaction {
        ui.disable();
    }
    egui::Grid::new("connection_settings")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Host address");
            ui.text_edit_singleline(&mut app.local_memory.network.target_ip_str);
            ui.end_row();

            ui.label("Local address");
            ui.horizontal(|ui| {
                ui.label(format!(
                    "{}:{}",
                    app.udp_client.local.address.str_from_octets(),
                    app.udp_client.local.address.port
                ))
            });
            ui.end_row();

            ui.label("Identifier");
            egui::TextEdit::singleline(&mut app.udp_client.local.identifier.str())
                .char_limit(16)
                .show(ui);
            ui.end_row();

            let button_text = if app.udp_client.active {
                "Apply"
            } else {
                "Connect"
            };
            let button_connect = ui.button(button_text);
            let button_disconnect = ui.button("Disconnect");
            if app.local_memory.security.allow_interaction {
                if button_connect.clicked() {
                    try_connect(app);
                }
                if button_disconnect.clicked() && app.udp_client.active {
                    try_disconnect(app);
                }
            }
        })
        .response
        .on_disabled_hover_text(
            "Connection settings are disabled when client is locked. Unlock to access settings.",
        );
}

fn try_disconnect(app: &mut ClicksMonitorApp) {
    app.udp_client
        .send_msg(Request::Unsubscribe(app.udp_client.local));
    app.udp_client.active = false;
    app.status = CombinedStatus::default();
}

fn try_connect(app: &mut ClicksMonitorApp) {
    let addr_parse = IpAddress::from_address_str(&app.local_memory.network.target_ip_str);
    if let Some(addr) = addr_parse {
        app.host_connection_info.address = addr;
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
}
