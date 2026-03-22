use common::{
    mem::time::format_hms,
    protocol::request::{ControlAction, Request},
};
use egui::{containers::menu::MenuConfig, Color32, MenuBar, PopupCloseBehavior, RichText, Widget};

use crate::{app::ClicksMonitorApp, theme};

pub fn display(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    MenuBar::new()
        .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
        .ui(ui, |ui| {
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
                ui.separator();
                if ui.button("Lock").clicked() {
                    app.local_memory.security.allow_interaction = false;
                }
                if ui
                    .button(
                        "Unlock".to_owned()
                            + if app.local_memory.security.require_password {
                                "..."
                            } else {
                                ""
                            },
                    )
                    .clicked()
                {
                    if app.local_memory.security.require_password {
                        app.text_entry.open("Password").password(true);
                    } else {
                        app.local_memory.security.allow_interaction = true;
                    }
                }
                if app.text_entry.submitted("Password")
                    && app.text_entry.get_text() == app.local_memory.security.password
                {
                    app.local_memory.security.allow_interaction = true;
                    app.text_entry.done()
                }
            });
            ui.menu_button("Help", |ui| {
                ui.label(format!("Monitor version {}", ClicksMonitorApp::VERSION));
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
                crate::window::connection::settings(app, ui);
                ui.separator();
                if app.rx.len() > 16 {
                    ui.colored_label(app.theme.err_prim, "Living in the past. Clearing cue...");
                } else if !app.udp_client.active {
                    ui.colored_label(app.theme.warn_prim, "Not Connected");
                } else {
                    ui.colored_label(app.theme.active_prim, "Ok");
                }
                ui.label(format!(
                    "Common version: {}",
                    app.last_heartbeat.common_version.str()
                ));
                ui.label(format!(
                    "System version: {}",
                    app.last_heartbeat.system_version.str()
                ));
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
            let host_time = app.last_heartbeat.system_time;
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
                if app.last_heartbeat.cpu_use_audio > 80.0
                    || app.last_heartbeat.process_freq_main < 10000
                {
                    app.theme.err_prim
                } else {
                    app.theme.active_prim
                },
                egui::RichText::new(format!(
                    "PERF: {:2.1}%  {}kHz",
                    app.last_heartbeat.cpu_use_audio,
                    app.last_heartbeat.process_freq_main / 1000
                ))
                .monospace(),
            );

            // Interaction lock
            if !app.local_memory.security.allow_interaction {
                ui.colored_label(
                    if app.local_memory.security.require_password {
                        app.theme.active_prim
                    } else {
                        app.theme.warn_prim
                    },
                    egui::RichText::new("󰌾 LOCK").monospace(),
                );
            }

            crate::window::transport::control_field(app, ui);
        });
}

fn transport_menu(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    if !app.local_memory.security.allow_interaction {
        ui.disable();
    }
    ui.response().on_disabled_hover_text(
        "Transport controls are disabled when client is locked. Unlock client to access.",
    );
    if ui.button("Start").clicked() && app.local_memory.security.allow_interaction {
        app.udp_client
            .send_msg(Request::ControlAction(ControlAction::TransportStart));
    }
    if ui.button("Stop").clicked() && app.local_memory.security.allow_interaction {
        app.udp_client
            .send_msg(Request::ControlAction(ControlAction::TransportStop));
    }
    if ui.button("Zero").clicked() && app.local_memory.security.allow_interaction {
        app.udp_client
            .send_msg(Request::ControlAction(ControlAction::TransportZero));
    }
}

pub fn cues_menu(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
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
            if ui.add_enabled(app.local_memory.security.allow_interaction, egui::Button::new("GOTO").small()).on_disabled_hover_text("Transport controls are disabled when client is locked. Unlock client to change cues.").clicked() {
                app.udp_client.send_msg(Request::ControlAction(
                    ControlAction::LoadCueByIndex(i as u8),
                ));
            }
            ui.end_row();
        }
    });
}
