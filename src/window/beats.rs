use crate::app::ClicksMonitorApp;
use common::{beat::Beat, cue::Cue, event::EventDescription};
use egui::{style::ScrollAnimation, Align, Color32, Frame, Grid, Rect, RichText, ScrollArea, Vec2};
use std::ops::Range;

const NUM_COL: usize = 7;
const COL_W: [f32; NUM_COL] = [64.0, 64.0, 64.0, 64.0, 64.0, 64.0, 256.0];

pub fn display(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    egui::SidePanel::right("cue-list-panel").show_inside(ui, |ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new("Cues").heading());
            crate::window::statusbar::cues_menu(app, ui);
        });
    });

    ui.label(RichText::new("Beats").heading());
    ui.vertical(|ui| {
        ui.label(
            RichText::new(format!(
                "#{} {}",
                app.status.cue.cue.metadata.human_ident, app.status.cue.cue.metadata.name
            ))
            .heading(),
        );
        beat_table(app, ui);
    });
}

pub fn beat_table(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        Grid::new("beats-table-header")
            .striped(true)
            .num_columns(7)
            .min_row_height(16.0)
            .min_col_width(64.0)
            .show(ui, |ui| {
                ui.set_width(COL_W.iter().sum());
                ui.label("Index");
                ui.label("Bar");
                ui.label("Count");
                ui.label("Length");
                ui.label("Tempo");
                ui.label("R. Mark");
                ui.label("Events");
                ui.end_row();
            });
        ScrollArea::new([false, true])
            .animated(false)
            .enable_scrolling(!app.status.transport.running)
            .show_rows(ui, 16.0, app.status.cue.cue.beats.len(), |ui, range| {
                Grid::new("beats-table")
                    .striped(false)
                    .num_columns(7)
                    .min_row_height(16.0)
                    .min_col_width(64.0)
                    .show(ui, |ui| {
                        if app.status.transport.running {
                            ui.scroll_to_rect_animation(
                                Rect::from_pos(
                                    ui.cursor().min
                                        + Vec2::new(
                                            0.0,
                                            (app.status
                                                .beat_state()
                                                .beat_idx
                                                .saturating_sub(
                                                    range.clone().min().unwrap_or_default() as u16,
                                                )
                                                .saturating_add(10))
                                                as f32
                                                * 16.0
                                                + (app.status.beat_state().us_to_next_beat as f32
                                                    / app.status.beat_state().beat.length as f32)
                                                    * 16.0,
                                        ),
                                ),
                                Some(Align::Center),
                                ScrollAnimation::none(),
                            );
                        }
                        beat_table_cue(app, ui, app.status.cue.cue.clone(), range);
                    });
            });
    });
}

pub fn beat_table_cue(
    app: &mut ClicksMonitorApp,
    ui: &mut egui::Ui,
    cue: Cue,
    beat_range: Range<usize>,
) {
    ui.set_width(COL_W.iter().sum());
    for (i, beat) in cue.beats[beat_range.clone()].iter().enumerate() {
        let actual_index = i + beat_range.clone().min().unwrap_or(0);
        ui.label(actual_index.to_string());
        let current_beat_idx = app.status.beat_state().beat_idx as usize;
        let color = if current_beat_idx == actual_index {
            app.theme.active_prim
        } else if app.status.beat_state().next_beat_idx as usize == actual_index {
            app.theme.cued_prim
        } else {
            app.theme.neutral_prim
        };

        beat_table_beat(app, ui, *beat, color, actual_index);
    }
    for _ in 0..5 {
        ui.end_row();
    }
}

pub fn beat_table_beat(
    app: &mut ClicksMonitorApp,
    ui: &mut egui::Ui,
    beat: Beat,
    color: Color32,
    idx: usize,
) {
    ui.label(RichText::new(beat.bar_number.to_string()).color(color));
    ui.label(RichText::new(beat.count.to_string()).color(color));
    ui.label(RichText::new(beat.length.to_string()).color(color));
    ui.label(RichText::new(beat.tempo().to_string()).color(color));

    let events = app.status.cue.cue.events.get_at_location(idx as u16);

    let mut mark_string = String::new();
    let mut event_string = String::new();
    for event in events {
        if let Some(EventDescription::RehearsalMarkEvent { label }) = event.event {
            mark_string.push_str(label.str());
            mark_string.push(' ');
            continue;
        }

        event_string.push_str(
            match event.event {
                Some(EventDescription::TimecodeEvent {
                    time,
                    properties: _,
                }) => {
                    format!("LTC [{}]", time)
                }

                Some(EventDescription::TimecodeStopEvent) => "LTC [STOP]".to_string(),
                Some(other) => other.get_name().to_string(),

                _ => "".to_string(),
            }
            .as_str(),
        );
        event_string.push(' ');
    }
    ui.label(mark_string);

    ui.label(event_string);

    ui.end_row();
}
