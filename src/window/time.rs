use crate::app::TemplateApp;
use chrono::{DateTime, Timelike};
use egui::{Align2, Color32, FontId, Grid, Rect, Sense, Vec2};

pub fn display(app: &mut TemplateApp, ui: &mut egui::Ui) {
    Grid::new("time-grid").num_columns(2).show(ui, |ui| {
        let size = Vec2::new(800.0, 180.0);

        // Monitor wall time
        let systime = chrono::prelude::Utc::now().time();
        draw_big_clock(
            app,
            ui,
            format!(
                "{:02}{:02}{:02}  ",
                systime.hour(),
                systime.minute(),
                systime.second()
            )
            .as_str(),
            [b':', b':', b' '],
            app.theme.warn_prim,
            size,
        );

        // Core wall time
        let host_time = DateTime::from_timestamp_secs(app.heartbeat.system_time as i64)
            .unwrap_or_default()
            .time();
        draw_big_clock(
            app,
            ui,
            if app.heartbeat.system_time > 0 {
                format!(
                    "{:02}{:02}{:02}  ",
                    host_time.hour(),
                    host_time.minute(),
                    host_time.second()
                )
            } else {
                "        ".to_string()
            }
            .as_str(),
            if app.heartbeat.system_time > 0 {
                [b':', b':', b' ']
            } else {
                [b' '; 3]
            },
            app.theme.warn_prim,
            size,
        );

        ui.end_row();

        // SMPTE Timestamp
        let smpte_time = app.status.time_state().ltc;
        draw_big_clock(
            app,
            ui,
            format!(
                "{:02}{:02}{:02}{:02}",
                smpte_time.h, smpte_time.m, smpte_time.s, smpte_time.f
            )
            .as_str(),
            [b':', b':', b':'],
            if app.status.time_state().running {
                app.theme.active_prim
            } else {
                app.theme.cued_prim
            },
            size,
        );

        // Session timer
        draw_big_clock(
            app,
            ui,
            "        ",
            [b' ', b' ', b' '],
            app.theme.neutral_prim,
            size,
        );

        ui.end_row();

        // Metronome bar:beat
        draw_big_clock(
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
        );
        draw_big_clock(
            app,
            ui,
            format!("{: >8}", app.status.beat_state().beat_idx).as_str(),
            [b' ', b' ', b' '],
            app.theme.cued_prim,
            size,
        );

        ui.end_row();

        // Monitor uptime
        draw_big_clock(
            app,
            ui,
            "        ",
            [b' ', b' ', b' '],
            app.theme.neutral_prim,
            size,
        );
        // Core uptime
        draw_big_clock(
            app,
            ui,
            "        ",
            [b' ', b' ', b' '],
            app.theme.neutral_prim,
            size,
        );
    });
}

pub fn draw_big_clock(
    app: &mut TemplateApp,
    ui: &mut egui::Ui,
    text: &str,
    separators: [u8; 3],
    color: Color32,
    size: Vec2,
) {
    const OUTER_MARGIN: f32 = 8.0;
    const SEPARATOR_WIDTH: f32 = 32.0;
    let chunk_width: f32 = (size.x - 2.0 * OUTER_MARGIN - 3.0 * SEPARATOR_WIDTH) / 4.0;

    let top_left = ui.cursor().left_top();
    let clock_rect = Rect::from_min_size(top_left, size);
    let p = ui.allocate_painter(size, Sense::click()).1;
    p.rect_filled(clock_rect, 0.0, app.theme.base_ex);
    for i in 0..4 {
        for j in 0..2 {
            let text_char = &text[2 * i + j..2 * i + j + 1];
            p.text(
                top_left
                    + Vec2::new(
                        OUTER_MARGIN
                            + (SEPARATOR_WIDTH + chunk_width) * i as f32
                            + (2 * j + 1) as f32 * chunk_width / 4.0,
                        size.y / 2.0,
                    ),
                Align2::CENTER_CENTER,
                if text_char != " " { text_char } else { "0" },
                FontId::monospace(size.y / 1.414),
                if text_char != " " {
                    color
                } else {
                    app.theme.base_wk
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
                    OUTER_MARGIN
                        + (SEPARATOR_WIDTH + chunk_width) * i as f32
                        + chunk_width
                        + SEPARATOR_WIDTH / 2.0,
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
                app.theme.base_wk
            },
        );
    }
}
