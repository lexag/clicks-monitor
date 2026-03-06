use crate::app::TemplateApp;
use egui::{Align2, Color32, FontId, Frame, Grid, Rect, Sense, Vec2};
use itertools::Itertools;

pub fn display(app: &mut TemplateApp, ui: &mut egui::Ui) {
    Grid::new("time-grid").num_columns(2).show(ui, |ui| {
        for i in 0..8 {
            draw_big_clock(
                app,
                ui,
                "   34567".to_string(),
                [b' ', b':', b'.'],
                app.theme.active_prim,
                Vec2::new(800.0, 200.0),
            );
            if i % 2 == 1 {
                ui.end_row();
            }
        }
    });
}

pub fn draw_big_clock(
    app: &mut TemplateApp,
    ui: &mut egui::Ui,
    text: String,
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
