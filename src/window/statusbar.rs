use common::{
    cue::Cue,
    mem::time::format_hms,
    protocol::request::{ControlAction, Request},
};
use egui::{Color32, RichText, Widget};

use crate::{
    app::{TabView, TemplateApp},
    theme,
};

pub fn display(app: &mut TemplateApp, ui: &mut egui::Ui) {
    egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Quit").clicked() {
                app.ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
        ui.menu_button("View", |ui| {
            ui.menu_button("Theme", |ui| {
                if ui.button("Native").clicked() {
                    app.set_theme(app.ctx.clone(), theme::NATIVE);
                }
                let themes = vec![
                    ("Dark", theme::DARK),
                    ("Light", theme::LIGHT),
                    ("Black", theme::BLACK),
                    ("Black (Monochrome)", theme::BLACK_MONOCHROME),
                ];
                for (name, theme) in themes {
                    if ui.button(name).clicked() {
                        app.set_theme(app.ctx.clone(), theme);
                    }
                }
            });
            ui.menu_button("Tab", |ui| {
                for (tab, text, short_text) in [
                    (TabView::Sources, "Sources", "S"),
                    (TabView::Cue, "Cue", "S"),
                    (TabView::Control, "Transport", "T"),
                    (TabView::Options, "Options", "O"),
                ] {
                    if egui::Button::new(text)
                        .shortcut_text(short_text)
                        .ui(ui)
                        .clicked()
                    {
                        app.tab = tab
                    }
                }
            });
            ui.separator();
            if ui.button("Lock").clicked() {
                app.allow_interaction = false;
            }
            if ui
                .button("Unlock".to_owned() + if app.require_password { "..." } else { "" })
                .clicked()
            {
                if app.require_password {
                    app.text_entry.open("Password").password(true);
                } else {
                    app.allow_interaction = true;
                }
            }
            if app.text_entry.submitted("Password") && app.text_entry.get_text() == app.password {
                app.allow_interaction = true;
                app.text_entry.done()
            }
        });
        ui.menu_button("Help", |ui| {
            ui.label(format!("Monitor version {}", TemplateApp::VERSION));
            ui.label(format!("Common version {}", common::VERSION));
        });

        ui.add_space(16.0);
        // Network status
        let color: Color32;
        if app.rx.len() > 16 {
            color = app.theme.err_prim;
            for _ in 0..app.rx.len() {
                let _ = app.rx.try_recv();
            }
        } else if !app.udp_client.active {
            color = app.theme.warn_prim;
        } else {
            color = app.theme.active_prim;
        }
        ui.menu_button(RichText::new("Host Network").color(color), |ui| {
            if app.rx.len() > 16 {
                ui.colored_label(app.theme.err_prim, "Living in the past. Clearing cue...");
            } else if !app.udp_client.active {
                ui.colored_label(app.theme.warn_prim, "Not Connected");
            } else {
                ui.colored_label(app.theme.active_prim, "Ok");
            }
            ui.label("Local:");
            ui.label(RichText::new(format!("{}", app.udp_client.local.address)).monospace());
            ui.label("Remote");
            ui.label(RichText::new(format!("{}", app.host_connection_info.address)).monospace());
        });

        // Transport status
        let color: Color32;
        if !app.status.transport.running {
            color = app.theme.warn_prim;
        } else {
            color = app.theme.active_prim;
        }
        let beat = app
            .status
            .cue
            .cue
            .get_beat(app.status.beat_state().beat_idx)
            .unwrap_or_default();
        ui.menu_button(
            RichText::new(format!(
                "LTC: {}   BEAT: {}.{}",
                app.status.time_state().ltc,
                beat.bar_number,
                beat.count
            ))
            .monospace()
            .color(color),
            |ui| {
                transport_menu(app, ui);
            },
        );

        // Cue status
        let color: Color32;
        let cue_state = app.status.cue.clone();
        if app.status.beat_state().beat_idx < u16::MAX / 2
            && app.status.beat_state().beat_idx + 8 > app.status.cue.cue.beats.len() as u16
        {
            color = app.theme.warn_prim;
        } else {
            color = app.theme.active_prim;
        }
        ui.menu_button(
            RichText::new(format!(
                "CUE {:0>3}:{: >6} {: <32}",
                cue_state.cue_idx,
                cue_state.cue.metadata.human_ident.str(),
                cue_state.cue.metadata.name.str()
            ))
            .monospace()
            .color(color),
            |ui| {
                cues_menu(app, ui);
            },
        );

        // Clock
        let system_time = chrono::prelude::Utc::now().timestamp_micros() as u64;
        let host_time = app.heartbeat.system_time;
        let diff = system_time.abs_diff(host_time * 1000000);
        let color = if host_time > 0 {
            if diff < 5 * 1000000 {
                app.theme.active_prim
            } else {
                app.theme.err_prim
            }
        } else {
            app.theme.warn_prim
        };
        ui.menu_button(
            RichText::new(format!(
                "󰈈 {}    {}",
                format_hms(system_time / 1000000).str(),
                if host_time > 0 {
                    format_hms(host_time).str().to_string()
                } else {
                    "--:--:--".to_string()
                }
            ))
            .monospace()
            .color(color),
            |ui| {
                ui.label(format!("Host: {}", host_time));
                ui.label(format!("Client: {}", system_time / 1000000));
                ui.label(format!("Diff: {} ms", diff / 1000,));
            },
        );

        // Performance
        ui.colored_label(
            if app.heartbeat.cpu_use_audio > 80.0 || app.heartbeat.process_freq_main < 10000 {
                app.theme.err_prim
            } else {
                app.theme.active_prim
            },
            egui::RichText::new(format!(
                "PERF: {:2.1}%  {}kHz",
                app.heartbeat.cpu_use_audio,
                app.heartbeat.process_freq_main / 1000
            ))
            .monospace(),
        );

        // Interaction lock
        if !app.allow_interaction {
            ui.colored_label(
                if app.require_password {
                    app.theme.active_prim
                } else {
                    app.theme.warn_prim
                },
                egui::RichText::new("󰌾 LOCK").monospace(),
            );
        }

        crate::window::control::control_field(app, ui);
    });
}

fn transport_menu(app: &mut TemplateApp, ui: &mut egui::Ui) {
    if !app.allow_interaction {
        ui.disable();
    }
    ui.response().on_disabled_hover_text(
        "Transport controls are disabled when client is locked. Unlock client to access.",
    );
    if ui.button("Start").clicked() && app.allow_interaction {
        app.udp_client
            .send_msg(Request::ControlAction(ControlAction::TransportStart));
    }
    if ui.button("Stop").clicked() && app.allow_interaction {
        app.udp_client
            .send_msg(Request::ControlAction(ControlAction::TransportStop));
    }
    if ui.button("Zero").clicked() && app.allow_interaction {
        app.udp_client
            .send_msg(Request::ControlAction(ControlAction::TransportZero));
    }
}

fn cues_menu(app: &mut TemplateApp, ui: &mut egui::Ui) {
    egui::Grid::new("cues-menu-grid").show(ui, |ui| {
        for (i, cue) in app.status.show.cues.iter().enumerate() {
            let color = if i == app.status.cue.cue_idx as usize {
                app.theme.active_prim
            } else {
                app.theme.neutral_prim
            };
            //println!("{:?}", app.status.show.cues);
            ui.label(format!("{:0>3}", i));
            ui.colored_label(color, cue.metadata.human_ident.str());
            ui.colored_label(color, cue.metadata.name.str());
            if ui.add_enabled(app.allow_interaction, egui::Button::new("GOTO").small()).on_disabled_hover_text("Transport controls are disabled when client is locked. Unlock client to change cues.").clicked() {
                app.udp_client.send_msg(Request::ControlAction(
                    ControlAction::LoadCueByIndex(i as u8),
                ));
            }
            ui.end_row();
        }
    });
}
