use std::{collections::VecDeque, ops::Sub};

use common::protocol::message::Heartbeat;
use egui::{Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};

use crate::app::TemplateApp;

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct PerformanceWindowMemory {
    pub heartbeats: VecDeque<Heartbeat>,
}

pub fn display(app: &mut TemplateApp, ui: &mut egui::Ui) {
    if ui.button("Clear history").clicked() {
        app.layout_settings.performance.heartbeats.clear();
    }

    graph(
        app,
        ui,
        app.layout_settings
            .performance
            .heartbeats
            .iter()
            .map(|&hb| (app.heartbeat.system_time.saturating_sub(hb.system_time)) as f32)
            .collect(),
        app.layout_settings
            .performance
            .heartbeats
            .iter()
            .map(|&hb| hb.cpu_use_audio)
            .collect(),
        "Audio CPU load".to_string(),
        None,
        None,
        Some(100.0),
        Some(0.0),
    );
    graph(
        app,
        ui,
        app.layout_settings
            .performance
            .heartbeats
            .iter()
            .map(|&hb| (app.heartbeat.system_time.saturating_sub(hb.system_time)) as f32)
            .collect(),
        app.layout_settings
            .performance
            .heartbeats
            .iter()
            .map(|&hb| (hb.process_freq_main as f32 / 1000.0).round())
            .collect(),
        "Processing frequency (kHz)".to_string(),
        None,
        None,
        None,
        None,
    );
}

pub fn graph(
    app: &mut TemplateApp,
    ui: &mut egui::Ui,
    x: Vec<f32>,
    y: Vec<f32>,
    label: String,
    x_max_f: Option<f32>,
    x_min_f: Option<f32>,
    y_max_f: Option<f32>,
    y_min_f: Option<f32>,
) {
    ui.vertical(|ui| {
        ui.label(RichText::new(label).heading());

        const SIZE: Vec2 = Vec2::new(1500.0, 640.0);
        const EDGE_SIZE: Vec2 = Vec2::new(48.0, 32.0);
        let graph_size = (SIZE - EDGE_SIZE) * Vec2::new(1.0, -1.0);
        let (res, p) = ui.allocate_painter(SIZE, Sense::hover());
        let origin = res.rect.left_bottom() + EDGE_SIZE * Vec2::new(1.0, -1.0);

        p.rect_filled(res.rect, 0.0, Color32::BLACK);
        //p.rect_filled(
        //    Rect::from_points(&[origin, origin + graph_size]),
        //    0.0,
        //    Color32::BLACK.lerp_to_gamma(Color32::DARK_GRAY, 0.1),
        //);

        if x.len() != y.len() || x.len() == 0 || y.len() == 0 {
            return;
        }

        let mut x_max = *x
            .iter()
            .max_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap())
            .unwrap_or(&0.0);

        let mut x_min = *x
            .iter()
            .min_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap())
            .unwrap_or(&0.0);

        let mut y_max = *y
            .iter()
            .max_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap())
            .unwrap_or(&0.0);

        let mut y_min = *y
            .iter()
            .min_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap())
            .unwrap_or(&0.0);

        // let x_round_factor = 10_f32.powf(x_max.log10().ceil() + 1.0).ceil();
        let y_round_factor = 10_f32.powf(y_max.log10().ceil() + 1.0).ceil();

        x_min = x_min_f.unwrap_or(x_min);
        x_max = x_max_f.unwrap_or(x_max);
        y_min = y_min_f.unwrap_or(y_min);
        y_max = y_max_f.unwrap_or(y_max);
        //y_min = y_min_f.unwrap_or((y_min / y_round_factor).floor() * y_round_factor);
        //y_max = y_max_f.unwrap_or((y_max / y_round_factor).ceil() * y_round_factor);

        let x_range = 1.0_f32.max(x_max - x_min);
        let y_range = 1.0_f32.max(y_max - y_min);

        if !x_range.is_normal() || !y_range.is_normal() {
            return;
        }

        for i in 0..x.len() - 1 {
            let x_i: f32 = x[i].clone().into();
            let x_j: f32 = x[i + 1].clone().into();
            let y_i: f32 = y[i].clone().into();
            let y_j: f32 = y[i + 1].clone().into();
            let p_a = origin
                + (Vec2::new(x_i, y_i) - Vec2::new(x_min, y_min)) * graph_size
                    / Vec2::new(x_range, y_range);
            let p_b = origin
                + (Vec2::new(x_j, y_j) - Vec2::new(x_min, y_min)) * graph_size
                    / Vec2::new(x_range, y_range);
            p.line_segment([p_a, p_b], Stroke::new(2.0, Color32::WHITE));
            p.circle_filled(p_a, 2.0, Color32::WHITE);
        }

        for (loc, anc, tex) in [
            (origin, Align2::LEFT_TOP, x_min.to_string()),
            (
                origin + graph_size * Vec2::RIGHT,
                Align2::RIGHT_TOP,
                x_max.to_string(),
            ),
            (
                origin + graph_size * Vec2::DOWN,
                Align2::RIGHT_TOP,
                y_max.to_string(),
            ),
            (origin, Align2::RIGHT_BOTTOM, y_min.to_string()),
        ] {
            p.text(
                loc,
                anc,
                tex,
                FontId::monospace(16.0),
                app.theme.neutral_prim,
            );
        }
    });
}
