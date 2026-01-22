use crate::app::TemplateApp;
use common::{
    event::JumpModeChange,
    protocol::request::{ControlAction, Request},
};

pub fn display(app: &mut TemplateApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        egui::Grid::new("control-grid").show(ui, |ui| {
            if !app.allow_interaction {
                ui.label("Transport control is disabled. Unlock the client to access controls.");
                return;
            }
            if ui.button("Play").clicked() {
                app.udp_client
                    .send_msg(Request::ControlAction(ControlAction::TransportStart));
            }
            if ui.button("Stop").clicked() {
                app.udp_client
                    .send_msg(Request::ControlAction(ControlAction::TransportStop));
            }

            ui.end_row();

            if ui.button("Goto Zero").clicked() {
                app.udp_client
                    .send_msg(Request::ControlAction(ControlAction::TransportZero));
            }
            ui.end_row();

            if ui.button("Next Cue").clicked() {
                app.udp_client
                    .send_msg(Request::ControlAction(ControlAction::LoadNextCue));
            }
            if ui.button("Prev Cue").clicked() {
                app.udp_client
                    .send_msg(Request::ControlAction(ControlAction::LoadPreviousCue));
            }

            ui.end_row();

            if ui
                .button(format!("VLT: {}", app.status.transport.vlt))
                .clicked()
            {
                app.udp_client
                    .send_msg(Request::ControlAction(ControlAction::ChangeJumpMode(
                        JumpModeChange::Toggle,
                    )));
            }

            let pr_val = ui.add(
                egui::DragValue::new(&mut app.status.transport.playrate_percent)
                    .range(50..=150)
                    .prefix("Playrate: ")
                    .suffix("%"),
            );
            if pr_val.drag_stopped() || pr_val.lost_focus() {
                app.udp_client
                    .send_msg(Request::ControlAction(ControlAction::ChangePlayrate(
                        app.status.transport.playrate_percent,
                    )));
            }

            ui.end_row();
        });
        control_field(app, ui);
    });
}

pub fn control_field(app: &mut TemplateApp, ui: &mut egui::Ui) {
    let mut keyboard_input = String::new();
    ui.add(egui::TextEdit::singleline(&mut keyboard_input));
    //FIXME: ugly but i'm in a hurry
    app.udp_client.send_msg(match keyboard_input.split_at(0).1 {
        " " => {
            if app.status.transport.running {
                Request::ControlAction(ControlAction::TransportStop)
            } else {
                Request::ControlAction(ControlAction::TransportStart)
            }
        }
        "." => Request::ControlAction(ControlAction::LoadNextCue),
        "," => Request::ControlAction(ControlAction::LoadPreviousCue),
        "0" => Request::ControlAction(ControlAction::TransportZero),
        "v" => Request::ControlAction(ControlAction::ChangeJumpMode(JumpModeChange::Toggle)),
        _ => return,
    });
}
