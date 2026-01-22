use crate::app::TemplateApp;
use common::{
    local::config::{LogContext, LogKind, SystemConfigurationChange},
    protocol::request::Request,
};
use egui::Widget;
use itertools::Itertools;

pub fn display(app: &mut TemplateApp, ui: &mut egui::Ui) {
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("System Settings").heading());
            if !app.udp_client.active {
                ui.colored_label(
                    app.theme.err_prim,
                    "Could not find system host. Check your connection.",
                );
                return;
            }
            if !app.allow_interaction {
                ui.disable();
            }

            ui.label(format!(
                "Common version: {}",
                app.heartbeat.common_version.str()
            ));
            ui.label(format!(
                "System version: {}",
                app.heartbeat.system_version.str()
            ));

            egui::Grid::new("audio-system-settings")
                .num_columns(2)
                .show(ui, |ui| {
                    if app.status.jack_status.running {
                        ui.disable();
                    }
                    ui.label(egui::RichText::new("Audio").underline());
                    ui.end_row();

                    ui.label("Device name");
                    let mut devices = app.status.jack_status.available_devices.iter();
                    egui::ComboBox::new("device-selector", "")
                        .selected_text(
                            match devices.position(|d| {
                                d.unwrap_or_default().id == app.system_config.audio.server.device_id
                            }) {
                                Some(idx) => app.status.jack_status.available_devices[idx]
                                    .map_or("".to_string(), |v| v.name.str().to_string()),
                                None => "No audio device selected".to_string(),
                            },
                        )
                        .truncate()
                        .width(400.0)
                        .show_ui(ui, |ui| {
                            for device in app.status.jack_status.available_devices {
                                ui.selectable_value(
                                    &mut app.system_config.audio.server.device_id,
                                    device.unwrap_or_default().id,
                                    device.unwrap_or_default().name.str(),
                                );
                            }
                        });
                    ui.end_row();

                    ui.label("Sample rate");
                    ui.horizontal(|ui| {
                        for (rate, label) in [
                            (44100, "44.1kHz"),
                            (48000, "48kHz"),
                            //(96000, "96kHz"),
                            //(192000, "192kHz"),
                        ] {
                            if ui
                                .selectable_label(
                                    app.system_config.audio.server.sample_rate == rate,
                                    label,
                                )
                                .clicked()
                            {
                                app.system_config.audio.server.sample_rate = rate;
                            }
                        }
                    });
                    ui.end_row();

                    ui.label("Period size (samples)");
                    ui.horizontal(|ui| {
                        for size in [128, 256, 512, 1024, 2048, 4096] {
                            if ui
                                .selectable_label(
                                    app.system_config.audio.server.period_size == size,
                                    size.to_string(),
                                )
                                .clicked()
                            {
                                app.system_config.audio.server.period_size = size;
                            }
                        }
                    });
                    ui.end_row();

                    if egui::Button::new("Launch Audio Processor")
                        .fill(app.theme.cued_prim)
                        .ui(ui)
                        .clicked()
                    {
                        app.udp_client.send_msg(Request::ChangeConfiguration(
                            SystemConfigurationChange::ChangeAudioConfiguration(
                                app.system_config.audio,
                            ),
                        ));
                        app.udp_client.send_msg(Request::Initialize);
                    }
                });
            egui::Grid::new("system-settings")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Logs").underline());
                    ui.end_row();

                    ui.vertical(|ui| {
                        ui.set_width(150.0);
                        for kind in LogKind::iter(&LogKind::all()) {
                            let mut val = app.system_config.logger.active_kinds.contains(kind);
                            ui.checkbox(&mut val, format!("{}", kind));
                            app.system_config.logger.active_kinds.set(kind, val);
                        }
                    });
                    ui.vertical(|ui| {
                        for context in LogContext::iter(&LogContext::all()) {
                            let mut val =
                                app.system_config.logger.active_contexts.contains(context);
                            ui.checkbox(&mut val, context.get_name());
                            app.system_config.logger.active_contexts.set(context, val);
                        }
                    });
                    ui.end_row();
                });

            if egui::Button::new("Apply Logger Settings").ui(ui).clicked() {
                app.udp_client.send_msg(Request::ChangeConfiguration(
                    SystemConfigurationChange::ChangeLoggerConfiguration(app.system_config.logger),
                ));
            }
            //ui.separator();
            //if egui::Button::new("Reset to default configuration (destructive)")
            //    .fill(app.theme.err_prim_wk)
            //    .ui(ui)
            //    .clicked()
            //{
            //    app.udp_client
            //        .send_msg(ControlMessage::SetConfigurationRequest(
            //            SystemConfiguration::default(),
            //        ));
            //}
        });
    });
}
