use crate::{
    app::ClicksMonitorApp,
    widget::cassette::{Cassette, CassetteDeck},
};
use chrono::{DateTime, NaiveTime, Timelike};
use common::mem::smpte::TimecodeInstant;
use eframe::glow::TESS_CONTROL_SHADER_BIT;
use egui::{
    Align2, Color32, FontId, Frame, Grid, Margin, Rect, RichText, Sense, Stroke, Vec2, Widget,
};

const GRID_MARGIN: f32 = 5.0;
const BORDER_MARGIN: f32 = 16.0;

const ASPECT_RATIO: f32 = 180.0 / 800.0;

pub fn display(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    let size = 800.0 * (ui.available_width() - 4.0 * BORDER_MARGIN - 3.0 * GRID_MARGIN) / 1600.0;
    Grid::new("time-grid")
        .num_columns(2)
        .spacing(Vec2::splat(GRID_MARGIN))
        .show(ui, |ui| {
            // Monitor wall time
            let systime = chrono::prelude::Utc::now().time();
            draw_wall_time(app, ui, systime, "Local time".to_string(), size);

            // Core wall time
            let host_time = DateTime::from_timestamp_secs(app.last_heartbeat.system_time as i64)
                .unwrap_or_default()
                .time();
            draw_wall_time(app, ui, host_time, "Core time".to_string(), size);

            ui.end_row();

            // SMPTE Timestamp
            let smpte_time = app.status.time_state().ltc;
            draw_smpte_time(app, ui, smpte_time, "SMPTE Timecode".to_string(), size);

            // Session timer
            draw_session_timer(
                app,
                ui,
                [systime, host_time, systime, host_time],
                "Session timer".to_string(),
                size,
            );

            ui.end_row();

            // Metronome bar:beat
            draw_big_clock_in_frame(
                app,
                ui,
                format!(
                    "{: >6}{: <2}",
                    app.status.beat_state().beat.bar_number,
                    app.status.beat_state().beat.count,
                )
                .as_str(),
                [b' ', b' ', b'.'],
                app.theme.cued_prim,
                size,
                "Metronome (bar / beat)".to_string(),
                |_, _| {},
            );
            draw_big_clock_in_frame(
                app,
                ui,
                format!("{: >8}", app.status.beat_state().beat_idx).as_str(),
                [b' ', b' ', b' '],
                app.theme.cued_prim,
                size,
                "Metronome beat index".to_string(),
                |_, _| {},
            );

            ui.end_row();

            // Monitor uptime
            draw_uptime(app, ui, 0, "Monitor uptime".to_string(), size);
            // Core uptime
            draw_uptime(app, ui, 0, "Core uptime".to_string(), size);
        });
}

pub fn draw_wall_time(
    app: &mut ClicksMonitorApp,
    ui: &mut egui::Ui,
    time: NaiveTime,
    title: String,
    size: f32,
) {
    draw_big_clock_in_frame(
        app,
        ui,
        format!(
            "{:02}{:02}{:02}  ",
            time.hour(),
            time.minute(),
            time.second()
        )
        .as_str(),
        [b':', b':', b' '],
        app.theme.warn_prim,
        size,
        title,
        |_, _| {},
    );
}

pub fn draw_session_timer(
    app: &mut ClicksMonitorApp,
    ui: &mut egui::Ui,
    time: [NaiveTime; 4],
    title: String,
    size: f32,
) {
    draw_big_clock_in_frame(
        app,
        ui,
        format!(
            "{:02}{:02}{:02}  ",
            time[0].hour(),
            time[0].minute(),
            time[0].second()
        )
        .as_str(),
        [b':', b':', b' '],
        app.theme.neutral_prim,
        size,
        title,
        move |app_n, ui_n| {
            for i in 0..3 {
                draw_big_clock(
                    ui_n,
                    format!(
                        "{:02}{:02}{:02}  ",
                        time[i + 1].hour(),
                        time[i + 1].minute(),
                        time[i + 1].second()
                    )
                    .as_str(),
                    [b':', b':', b' '],
                    app_n.theme.base_ex,
                    app_n.theme.neutral_prim,
                    size * ui_n.available_height() / (size / ASPECT_RATIO),
                );
            }
        },
    );
}

pub fn draw_smpte_time(
    app: &mut ClicksMonitorApp,
    ui: &mut egui::Ui,
    time: TimecodeInstant,
    title: String,
    size: f32,
) {
    draw_big_clock_in_frame(
        app,
        ui,
        format!("{:02}{:02}{:02}{:02}", time.h, time.m, time.s, time.f).as_str(),
        [b':', b':', b':'],
        if app.status.time_state().running {
            app.theme.active_prim
        } else {
            app.theme.cued_prim
        },
        size,
        title,
        |_, _| {},
    );
}

pub fn draw_uptime(
    app: &mut ClicksMonitorApp,
    ui: &mut egui::Ui,
    time: u64,
    title: String,
    size: f32,
) {
    draw_big_clock_in_frame(
        app,
        ui,
        format!(
            "{:02}d {:02}{:02}",
            time / 86400,
            time / 3600 % 24,
            time / 60 % 60
        )
        .as_str(),
        [b' ', b' ', b':'],
        app.theme.neutral_prim,
        size,
        title,
        |_, _| {},
    );
}

pub fn draw_big_clock_in_frame<F>(
    app: &mut ClicksMonitorApp,
    ui: &mut egui::Ui,
    text: &str,
    separators: [u8; 3],
    color: Color32,
    width: f32,
    title: String,
    lower_content: F,
) where
    F: FnOnce(&mut ClicksMonitorApp, &mut egui::Ui) + 'static,
{
    let bg_col = app.theme.base_ex;
    let c = Cassette::new()
        .fill(app.theme.base_wk)
        .stroke(Stroke::new(1.0, app.theme.base_ex))
        .outer_width(width);

    let inner_size = c.get_inner_width();
    c.show_with_lower(
        ui,
        |ui| {
            draw_big_clock(ui, text, separators, bg_col, color, inner_size);
        },
        |ui: &mut egui::Ui| {
            ui.label(RichText::new(title).heading());
            lower_content(app, ui);
        },
    );
}

pub fn draw_big_clock(
    ui: &mut egui::Ui,
    text: &str,
    separators: [u8; 3],
    background_color: Color32,
    color: Color32,
    width: f32,
) {
    let height = width * 140.0 / 800.0;
    let size = Vec2::new(width, height);
    let outer_margin: f32 = 8.0 * height / 200.0;
    let separator_width: f32 = 32.0 * height / 200.0;
    let chunk_width: f32 = (width - 2.0 * outer_margin - 3.0 * separator_width) / 4.0;

    let top_left = ui.cursor().left_top();
    let clock_rect = Rect::from_min_size(top_left, size);
    let p = ui.allocate_painter(size, Sense::click()).1;
    p.rect_filled(clock_rect, 5.0, background_color);
    for i in 0..4 {
        for j in 0..2 {
            let text_char = &text[2 * i + j..2 * i + j + 1];
            p.text(
                top_left
                    + Vec2::new(
                        outer_margin
                            + (separator_width + chunk_width) * i as f32
                            + (2 * j + 1) as f32 * chunk_width / 4.0,
                        size.y / 2.0,
                    ),
                Align2::CENTER_CENTER,
                if text_char != " " { text_char } else { "0" },
                FontId::monospace(size.y / 1.414),
                if text_char != " " {
                    color
                } else {
                    background_color.lerp_to_gamma(Color32::GRAY, 0.1)
                },
            );
        }
    }

    for i in 0..3 {
        let text = str::from_utf8(&[separators[i]])
            .unwrap_or_default()
            .to_string();
        p.text(
            top_left
                + Vec2::new(
                    outer_margin
                        + (separator_width + chunk_width) * i as f32
                        + chunk_width
                        + separator_width / 2.0,
                    size.y * 0.48,
                ),
            Align2::CENTER_CENTER,
            if text != " " {
                text.clone()
            } else {
                ":".to_string()
            },
            FontId::monospace(size.y / 1.414),
            if text != " " {
                color
            } else {
                background_color.lerp_to_gamma(Color32::GRAY, 0.1)
            },
        );
    }
}
