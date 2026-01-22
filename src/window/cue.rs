use egui::{vec2, Align2, CornerRadius, Rect, RichText, Stroke};

use crate::{app::TemplateApp, theme::Theme};
use common::{
    beat::Beat,
    event::{Event, EventCursor, EventDescription},
    local::status::CombinedStatus,
    protocol::request::{ControlAction, Request},
};

pub fn display(app: &mut TemplateApp, ui: &mut egui::Ui) {
    // Pre-allocate a side panel
    let side_panel = egui::SidePanel::right("beat-details-panel")
        .resizable(false)
        .exact_width(ui.available_width() * 0.3);

    // Render central canvas area, which returns which beat is hovered.
    let mut hovered_idx = usize::MAX;
    egui::CentralPanel::default().show(ui.ctx(), |ui| {
        hovered_idx = render_cue(app, ui);
    });

    // Render details in side panel for hovered beat.
    side_panel.show(ui.ctx(), |ui| {
        if hovered_idx == usize::MAX {
            ui.label("Hover a beat to view details.");
            return;
        }
        let beat = app.status.cue.cue.get_beat(hovered_idx as u16).unwrap();
        egui::Grid::new("beat-details-grid")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Beat Index");
                ui.label(hovered_idx.to_string());
                ui.end_row();
                ui.label("Beat position");
                ui.label(format!("{}.{}", beat.bar_number, beat.count));
                ui.end_row();
                ui.label("Beat length");
                ui.label(format!("{} us ({} ms)", beat.length, beat.length / 1000));
                ui.end_row();
                ui.label("BPM");
                ui.label(format!("{}", 60000 / beat.length))
            });
        ui.label("Beat Events");
        let mut cursor = EventCursor::new(&app.status.cue.cue.events);
        while cursor.at_or_before(hovered_idx as u16)
            && let Some(event) = cursor.get_next()
            && event.location == hovered_idx as u16
        {
            //if beat.events.is_empty() {
            //    ui.vertical_centered(|ui| {
            //        ui.set_width(ui.available_width());
            //        ui.label("No events");
            //    });
            //}
            render_event_info(app.theme, ui, app, hovered_idx, beat, event);
        }
    });
}

fn render_cue(app: &mut TemplateApp, ui: &mut egui::Ui) -> usize {
    let mut hovered_idx = usize::MAX;
    ui.vertical(|ui| {
        // Cue title
        ui.label(egui::RichText::new(app.status.cue.cue.metadata.name.str()).heading());
        egui::ScrollArea::new([false, true]).show(ui, |ui| {
            let max_beats_per_line = 32.0;
            // Cue beat grid
            let size = ui.available_width() / max_beats_per_line * 0.7;

            let mut x: f32 = f32::MAX / 2.0;
            let mut y: f32 = -size;
            let (rect, resp) = ui.allocate_exact_size(
                vec2(
                    ui.available_width(),
                    app.status.cue.cue.get_beats().len() as f32 / max_beats_per_line * size * 3.0,
                ),
                egui::Sense::click(),
            );
            let mut cursor = EventCursor::new(&app.status.cue.cue.events);
            for (i, beat) in app.status.cue.cue.get_beats().iter().enumerate() {
                let mut line_break_flag = false;
                while cursor.at_or_before(i as u16)
                    && let Some(event) = cursor.get_next()
                    && event.location == i as u16
                    && let Some(inner_event) = event.event
                    && let EventDescription::RehearsalMarkEvent { label } = inner_event
                {
                    x = 0.0;
                    y += size;
                    if !label.is_empty() {
                        let p = ui.painter();
                        p.text(
                            rect.min + vec2(0.0, y + size * 0.5),
                            Align2::LEFT_CENTER,
                            label,
                            egui::FontId {
                                size: size * 0.5,
                                family: egui::FontFamily::Proportional,
                            },
                            app.theme.neutral_prim,
                        );
                        y += size;
                    }
                    line_break_flag = true;
                }

                if !line_break_flag {
                    x += size;
                    if x / size > max_beats_per_line - 0.5 {
                        x = 0.0;
                        y += size;
                    }
                }

                let beat_origin = rect.min + egui::vec2(x, y);
                let half_center = egui::vec2(size, size) * 0.5;
                let beat_rect = Rect {
                    min: beat_origin,
                    max: beat_origin + 2.0 * half_center,
                };

                // If this beat is hovered, remember that it is and save idx to display details in side
                // panel later
                let mut hovered = false;
                if ui.rect_contains_pointer(beat_rect) {
                    hovered_idx = i;
                    hovered = true;
                }

                if app.status.beat_state().beat_idx as usize == i && app.status.transport.running {
                    ui.scroll_to_rect(beat_rect, Some(egui::Align::Center));
                }

                // Draw main beat tile
                render_beat(
                    app.theme,
                    ui,
                    i,
                    hovered,
                    beat.clone(),
                    beat_rect,
                    &app.status,
                );
            }
            if app.allow_interaction && resp.clicked() && hovered_idx < usize::MAX / 2 {
                app.udp_client
                    .send_msg(Request::ControlAction(if app.status.transport.running {
                        ControlAction::TransportSeekBeat(hovered_idx as u16)
                    } else {
                        ControlAction::TransportJumpBeat(hovered_idx as u16)
                    }));
            }
        });
    });
    hovered_idx
}

fn render_beat(
    theme: Theme,
    ui: &mut egui::Ui,
    idx: usize,
    hovered: bool,
    beat: Beat,
    rect: egui::Rect,
    process_status: &CombinedStatus,
) {
    let stroke_width = 1.0;
    let font_large = egui::FontId {
        size: rect.width() * 0.5,
        family: egui::FontFamily::Monospace,
    };
    let font_medium = egui::FontId {
        size: rect.width() * 0.3,
        family: egui::FontFamily::Monospace,
    };

    let p = ui.painter();
    let fg = if hovered {
        theme.cued_prim
    } else {
        theme.neutral_prim
    };
    p.rect(
        rect,
        CornerRadius::ZERO,
        if idx == process_status.beat_state().beat_idx as usize {
            theme.active_prim
        } else if idx == process_status.beat_state().next_beat_idx as usize {
            theme.cued_prim
        } else {
            theme.base_wk
        },
        Stroke::new(stroke_width, fg),
        egui::StrokeKind::Inside,
    );
    p.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        beat.count.to_string(),
        font_large.clone(),
        fg,
    );
    if beat.count == 1 {
        p.text(
            rect.min + vec2(stroke_width, stroke_width),
            egui::Align2::LEFT_TOP,
            format!("{}", beat.bar_number),
            font_medium.clone(),
            fg,
        );
    }

    let mut event_strings: Vec<&str> = vec![];
    let mut cursor = EventCursor::new(&process_status.cue.cue.events);
    while let Some(event) = cursor.get_next()
        && event.location == idx as u16
        && let Some(inner_event) = event.event
    {
        let event_icon = match inner_event {
            EventDescription::JumpEvent {
                destination,
                requirement,
                when_passed,
                when_jumped,
            } => {
                if destination as usize > idx {
                    "J"
                } else {
                    "R"
                }
            }
            EventDescription::PlaybackEvent { .. } => "P",
            EventDescription::PlaybackStopEvent { .. } => "(P)",
            EventDescription::TimecodeEvent { .. } => "LTC",
            EventDescription::TempoChangeEvent { .. } => "T",
            _ => "",
        };
        if !event_icon.is_empty() {
            event_strings.push(event_icon);
        }
    }
    event_strings.sort();
    p.text(
        rect.left_bottom() + vec2(stroke_width, -stroke_width),
        egui::Align2::LEFT_BOTTOM,
        event_strings.join(" "),
        font_medium.clone(),
        theme.warn_prim,
    );
}

fn render_event_info(
    theme: Theme,
    ui: &mut egui::Ui,
    app: &TemplateApp,
    idx: usize,
    beat: Beat,
    event: Event,
) {
    let _ = egui::Frame::new()
        .stroke(Stroke::new(2.0, theme.neutral_prim))
        .corner_radius(2.0)
        .outer_margin(10.0)
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width() * 0.8);
            let event_name = if let Some(event_desc) = event.event {
                event_desc.get_name().to_string()
            } else {
                "Unknown event".to_string()
            };
            ui.label(egui::RichText::new(event_name).heading().underline());
            match event.event {
                Some(EventDescription::JumpEvent {
                    destination,
                    requirement,
                    when_jumped,
                    when_passed,
                }) => {
                    let dest_beat = app.status.cue.cue.get_beat(destination).unwrap();
                    ui.label("Destination:");
                    ui.label(format!("{}.{}", dest_beat.bar_number, dest_beat.count));
                    let bar_diff = beat.bar_number as i16 - dest_beat.bar_number as i16;
                    ui.label(format!(
                        "({} {} {})",
                        bar_diff.abs() + bar_diff.signum(),
                        if bar_diff.abs() + bar_diff.signum() == 1 {
                            "bar"
                        } else {
                            "bars"
                        },
                        if bar_diff < 0 { "forward" } else { "backward" }
                    ));
                }

                Some(EventDescription::PlaybackEvent {
                    sample,
                    channel_idx,
                    clip_idx,
                }) => {
                    let secs = sample / i32::max(app.status.jack_status.sample_rate as i32, 1);
                    ui.label(format!("Channel {} clip {}", channel_idx, clip_idx,));
                    ui.label(format!(
                        "At sample {} ({} m {} s)",
                        sample,
                        secs / 60,
                        secs % 60
                    ));
                }

                Some(EventDescription::TimecodeEvent { time }) => {
                    ui.label("Timestamp:");
                    ui.label(
                        RichText::new(format!("{}:{}:{}:{}", time.h, time.m, time.s, time.f))
                            .monospace()
                            .strong(),
                    );
                }
                Some(EventDescription::TempoChangeEvent { tempo }) => {
                    ui.label(format!("Tempo: {}", tempo));
                }
                _ => {}
            }
        });
}
