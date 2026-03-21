use std::{
    f32,
    ops::{Add, Div},
};

use common::mem::{
    network::{ConnectionEnd, ConnectionInfo, SubscriberInfo},
    str::StaticString,
    typeflags::{MessageType, RequestType},
};
use egui::{Align2, CentralPanel, Color32, FontId, Painter, Pos2, RichText, Sense, Stroke, Vec2};

use crate::app::ClicksMonitorApp;

pub fn display(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    egui::SidePanel::right("network-details-panel").show_inside(ui, |ui| {
        client_data_use(app, ui);
    });
    network_galaxy(app, ui);
}

fn format_data_amount(bytes: usize) -> String {
    const SI: &[&str] = &[
        "B", "kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB", "RB", "QB",
    ];

    let bytesf = bytes as f32;

    let n = bytesf.add(0.5).log10().floor();
    let si_idx = n.div(3.0).floor();

    let mut val = bytesf.div(1000.0_f32.powf(si_idx as f32));

    format!("{} {}", val.round(), SI[si_idx as usize])
}

pub fn client_data_use(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
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
}

pub fn network_galaxy(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    let size = ui.available_size();
    let top_left = ui.cursor().min;
    let (res, p) = ui.allocate_painter(size, Sense::click_and_drag());
    let canvas_rect = res.rect;
    let center = top_left + size * 0.5;

    let stroke = Stroke::new(2.0, app.theme.neutral_prim);

    p.rect_filled(canvas_rect, 0.0, Color32::BLACK);

    let angle_inter = 2.0 * f32::consts::PI / app.status.network_status.subscribers.len() as f32;

    const PLANET_DISTANCE: f32 = 400.0;
    for (i, client) in app.status.network_status.subscribers.iter().enumerate() {
        let direction = Vec2::angled(angle_inter * i as f32);

        let pos = center + PLANET_DISTANCE * direction;
        device_planet(
            &p,
            pos,
            client,
            stroke,
            Color32::from_rgb(
                (direction.x * 256.0) as u8,
                (256.0 - direction.x * 256.0) as u8,
                (direction.y * 256.0) as u8,
            )
            .lerp_to_gamma(Color32::BLACK, 0.5),
        );
    }

    device_planet(
        &p,
        center,
        &SubscriberInfo {
            identifier: StaticString::new("Core"),
            address: app.host_connection_info.address,
            message_kinds: MessageType::all(),
            last_contact: 0,
        },
        stroke,
        Color32::DARK_GRAY,
    );
}

pub fn device_planet(
    p: &Painter,
    pos: Pos2,
    client: &SubscriberInfo,
    stroke: Stroke,
    fill: Color32,
) {
    const RADIUS: f32 = 65.0;

    p.line_segment([pos, p.clip_rect().center()], stroke);
    p.circle(pos, RADIUS, fill, stroke);
    p.text(
        pos + 16.0 * Vec2::UP,
        Align2::CENTER_CENTER,
        client.identifier.str(),
        FontId::monospace(12.0),
        stroke.color,
    );
    p.text(
        pos,
        Align2::CENTER_CENTER,
        client.address.str_from_octets(),
        FontId::monospace(12.0),
        stroke.color,
    );
    p.text(
        pos + 16.0 * Vec2::DOWN,
        Align2::CENTER_CENTER,
        client.address.port.to_string(),
        FontId::monospace(12.0),
        stroke.color,
    );
}
