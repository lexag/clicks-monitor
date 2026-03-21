use crate::app::ClicksMonitorApp;
use common::{
    event::JumpModeChange,
    protocol::request::{ControlAction, Request},
};
use egui::{Button, Color32, ProgressBar, Response, RichText, Vec2, Widget};

pub fn big_button(ui: &mut egui::Ui, label: &str, fill: Color32, size: Vec2) -> Response {
    ui.add(
        Button::new(RichText::new(label).color(Color32::BLACK).heading())
            .fill(fill)
            .min_size(size),
    )
}

pub fn display(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        let size = ui.available_size() / Vec2::new(4.0, 2.0);
        egui::Grid::new("control-grid").show(ui, |ui| {
            if !app.local_memory.security.allow_interaction {
                ui.label("Transport control is disabled. Unlock the client to access controls.");
                return;
            }

            if big_button(
                ui,
                format!("VLT: {}", app.status.transport.vlt).as_str(),
                app.theme.neutral_prim,
                size,
            )
            .clicked()
            {
                app.udp_client
                    .send_msg(Request::ControlAction(ControlAction::ChangeJumpMode(
                        JumpModeChange::Toggle,
                    )));
            }

            ui.horizontal_top(|ui| {
                ui.set_width(size.x * 2.0);
                ui.vertical_centered_justified(|ui| {
                    ui.label(
                        RichText::new(app.status.cue.cue.metadata.human_ident.str()).heading(),
                    );
                    ui.label(RichText::new(app.status.cue.cue.metadata.name.str()).size(64.0));
                    ui.label(
                        RichText::new(format!(
                            "{}.{}",
                            app.status.beat_state().beat.bar_number,
                            app.status.beat_state().beat.count
                        ))
                        .size(64.0),
                    );
                    ProgressBar::new(
                        app.status.beat_state().beat_idx as f32
                            / app.status.cue.cue.beats.len() as f32,
                    )
                    .fill(if app.status.transport.running {
                        app.theme.active_prim
                    } else {
                        app.theme.cued_prim
                    })
                    .ui(ui);
                });
            });

            let stop_button = big_button(ui, "Stop", app.theme.err_prim, size);

            if stop_button.clicked() {
                app.udp_client
                    .send_msg(Request::ControlAction(ControlAction::TransportStop));
            }
            if stop_button.double_clicked() {
                app.udp_client
                    .send_msg(Request::ControlAction(ControlAction::TransportZero));
            }

            ui.end_row();

            if big_button(ui, "Goto Zero", app.theme.neutral_prim, size).clicked() {
                app.udp_client
                    .send_msg(Request::ControlAction(ControlAction::TransportZero));
            }

            ui.horizontal_centered(|ui| {
                if big_button(ui, "Prev Cue", app.theme.neutral_prim, size).clicked() {
                    app.udp_client
                        .send_msg(Request::ControlAction(ControlAction::LoadPreviousCue));
                }
                if big_button(ui, "Next Cue", app.theme.neutral_prim, size).clicked() {
                    app.udp_client
                        .send_msg(Request::ControlAction(ControlAction::LoadNextCue));
                }
            });

            if big_button(ui, "Play", app.theme.cued_prim, size).clicked() {
                app.udp_client
                    .send_msg(Request::ControlAction(ControlAction::TransportStart));
            }

            ui.end_row();

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

pub fn control_field(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
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
