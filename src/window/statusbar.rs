use common::{
    mem::time::format_hms,
    protocol::request::{ControlAction, Request},
};
use egui::{containers::menu::MenuConfig, MenuBar, PopupCloseBehavior, RichText};

use crate::app::ClicksMonitorApp;

pub fn display(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    MenuBar::new()
        .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
        .ui(ui, |ui| {
            // Network status
            network_slot(app, ui);

            // Transport status
            transport_slot(app, ui);

            // Cue status
            cue_slot(app, ui);

            // Clock
            clock_slot(app, ui);

            // Performance
            performance_slot(app, ui);

            // Interaction lock
            crate::window::security::lock_slot(app, ui);

            crate::window::transport::control_field(app, ui);
        });
}

fn performance_slot(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    ui.colored_label(
        if app.last_heartbeat.cpu_use_audio > 80.0 || app.last_heartbeat.process_freq_main < 10000 {
            app.theme.err_prim
        } else {
            app.theme.active_prim
        },
        egui::RichText::new(format!(
            "PERF: {:2.1}%  {: >4}kHz",
            app.last_heartbeat.cpu_use_audio,
            app.last_heartbeat.process_freq_main / 1000
        ))
        .monospace(),
    );
}

fn clock_slot(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    let system_time = chrono::prelude::Utc::now().timestamp_micros() as u64;
    let host_time = app.last_heartbeat.system_time;
    let diff = system_time.abs_diff(host_time * 1000000);
    let color = if host_time > 0 {
        if diff < 5 * 1000000 {
            app.theme.active_prim
        } else {
            app.theme.warn_prim
        }
    } else {
        app.theme.err_prim
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
}

fn cue_slot(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    let cue_state = app.status.cue.clone();
    let color = if app.status.beat_state().beat_idx < u16::MAX / 2
        && app.status.beat_state().beat_idx + 8 > app.status.cue.cue.beats.len() as u16
    {
        app.theme.warn_prim
    } else {
        app.theme.active_prim
    };
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
}

fn transport_slot(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    let color = if app.status.transport.running {
        app.theme.active_prim
    } else {
        app.theme.warn_prim
    };
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
}

fn network_slot(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    let color = if app.rx.len() > 16 {
        for _ in 0..app.rx.len() {
            let _ = app.rx.try_recv();
        }
        app.theme.err_prim
    } else if !app.udp_client.active {
        app.theme.warn_prim
    } else {
        app.theme.active_prim
    };
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
