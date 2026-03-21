use crate::app::ClicksMonitorApp;
use common::{local::config::SystemConfigurationChange, protocol::request::Request};
use egui::{vec2, Align2, Rect, SidePanel, Stroke, Widget};

pub fn display(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    SidePanel::left("audio-settings-panel").show_inside(ui, |ui| {
        audio_settings(app, ui);
        audio_details(app, ui);
    });
    routing_matrix(app, ui);
}

pub fn audio_details(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        if !app.local_memory.security.allow_interaction {
            ui.disable();
        }
        ui.label(egui::RichText::new("JACK").heading());
        if app.status.jack_status.io_size == (0, 0) {
            ui.horizontal(|ui| {
                ui.colored_label(
                    app.theme.err_prim,
                    "Unable to find JACK status. Check connection.",
                )
            });
            ui.disable();
            return;
        }
        egui::Grid::new("jack-details").show(ui, |ui| {
            ui.label("Sample rate");
            ui.label(format!("{} Hz", app.status.jack_status.sample_rate));
            ui.end_row();

            ui.label("Buffer size");
            ui.label(format!("{} samples", app.status.jack_status.buffer_size));
            ui.end_row();

            ui.label("Frame size");
            ui.label(format!("{} samples", app.status.jack_status.frame_size));
            ui.end_row();

            ui.label("Client name");
            ui.label(format!("{}", app.status.jack_status.client_name.str()));
            ui.end_row();

            ui.label("Output name");
            ui.label(format!("{}", app.status.jack_status.output_name.str()));
            ui.end_row();

            ui.label("CPU load");
            ui.label(format!("{:.2}%", app.last_heartbeat.cpu_use_audio));
            ui.end_row();
        });
    });
}
pub fn audio_settings(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
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
                        .selectable_label(app.system_config.audio.server.sample_rate == rate, label)
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
                    SystemConfigurationChange::ChangeAudioConfiguration(app.system_config.audio),
                ));
                app.udp_client.send_msg(Request::Initialize);
            }
        });
}

pub fn routing_matrix(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    let size = 25.0;
    let mut hovered_connection = (0_usize, 0_usize);
    let io = app.status.jack_status.io_size;

    let (rect, resp) = ui.allocate_exact_size(
        egui::Vec2 {
            x: size * (io.1 + 1) as f32,
            y: size * (io.0 + 1) as f32,
        },
        egui::Sense::click(),
    );

    let half_center = egui::vec2(size, size) * 0.5;

    // loop one index extra, because we use the 0 column and 0 row for displaying numbers, and 1-32
    // and 1-X respective for actual grid
    for from in 0..33 {
        for to in 0..(io.1 + 1) {
            let x: f32 = to as f32 * size;
            let y: f32 = from as f32 * size;
            let tile_origin = rect.min + egui::vec2(x, y);
            let rect = Rect {
                min: tile_origin,
                max: tile_origin + 2.0 * half_center,
            };

            if from == 0 && to == 0 {
                continue;
            }
            if (from == 0) != (to == 0) {
                ui.painter().text(
                    tile_origin + half_center,
                    Align2::CENTER_CENTER,
                    from.max(to).to_string(),
                    egui::FontId {
                        size: size * 0.5,
                        family: egui::FontFamily::Monospace,
                    },
                    app.theme.neutral_prim,
                );
                continue;
            }

            let mut hovered = false;
            if ui.rect_contains_pointer(rect) {
                hovered_connection = (from - 1, to - 1);
                hovered = true;
            }
            render_routing_tile(app, ui, hovered, rect, (from - 1, to - 1));

            if hovered {
                ui.painter().text(
                    tile_origin + half_center - vec2(0.0, size),
                    Align2::CENTER_CENTER,
                    to,
                    egui::FontId {
                        size: size * 0.5,
                        family: egui::FontFamily::Monospace,
                    },
                    app.theme.neutral_prim,
                );
                ui.painter().text(
                    tile_origin + half_center - vec2(size, 0.0),
                    Align2::CENTER_CENTER,
                    from,
                    egui::FontId {
                        size: size * 0.5,
                        family: egui::FontFamily::Monospace,
                    },
                    app.theme.neutral_prim,
                );
            }
        }
    }

    if resp.clicked() {
        app.udp_client.send_msg(Request::ChangeRouting(
            hovered_connection.0 as u8,
            hovered_connection.1 as u8,
            (app.status.jack_status.connections[hovered_connection.0]
                & (0x01 << hovered_connection.1))
                == 0,
        ))
    }
}

fn render_routing_tile(
    app: &mut ClicksMonitorApp,
    ui: &mut egui::Ui,
    hovered: bool,
    rect: egui::Rect,
    connection: (usize, usize),
) {
    let p = ui.painter();
    let stroke_col = if hovered {
        app.theme.cued_prim
    } else {
        app.theme.base
    };
    p.rect(
        rect,
        0.0,
        if hovered {
            app.theme.cued_prim
        } else if (app.status.jack_status.connections[connection.0] & (0x01 << connection.1)) > 0 {
            app.theme.active_prim
        } else {
            app.theme.base_wk
        },
        Stroke::new(1.0, stroke_col),
        egui::StrokeKind::Inside,
    );
}
