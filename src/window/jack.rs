use egui::{vec2, Align2, CornerRadius, Rect, Stroke};

use crate::app::TemplateApp;
use common::protocol::request::Request;

pub fn display(app: &mut TemplateApp, ui: &mut egui::Ui) {
    ui.horizontal_top(|ui| {
        render_details(app, ui);
        ui.separator();
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Routing Matrix").heading());
            render_routing_matrix(app, ui);
        });
    });
}

pub fn render_details(app: &mut TemplateApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        if !app.allow_interaction {
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
            ui.label(format!("{:.2}%", app.heartbeat.cpu_use_audio));
            ui.end_row();
        });
    });
}

pub fn render_routing_matrix(app: &mut TemplateApp, ui: &mut egui::Ui) {
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
    app: &mut TemplateApp,
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
        CornerRadius::ZERO,
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
