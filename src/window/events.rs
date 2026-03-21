use crate::app::ClicksMonitorApp;
use common::event::{
    Event, EventDescription, JumpRequirement, PauseEventBehaviour,
};
use egui::{Color32, Grid, ProgressBar, RichText, Widget};

pub fn display(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Events").heading());
        ui.label(format!(
            "This cue has {} events.",
            app.status.cue.cue.events.len()
        ))
    });
    Grid::new("events-grid")
        .striped(true)
        .min_row_height(24.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.set_width(32.0);
                ui.label("#")
            });
            ui.horizontal(|ui| {
                ui.set_width(32.0);
                ui.label("Location")
            });
            ui.horizontal(|ui| {
                ui.set_width(128.0);
                ui.label("Event")
            });
            ui.horizontal(|ui| {
                ui.set_width(256.0);
                ui.label("Parameters")
            });
            ui.horizontal(|ui| {
                ui.set_width(32.0);
                ui.label("Countdown")
            });
            ui.horizontal(|ui| {
                ui.set_width(256.0);
                ui.label("Visual countdown")
            });
            ui.end_row();

            let mut num_drawn = 0;

            for (i, event) in app.status.cue.cue.events.iter().enumerate() {
                let dist = event
                    .location
                    .wrapping_sub(app.status.beat_state().beat_idx);

                if dist > u16::MAX / 2 {
                    continue;
                }

                if num_drawn > DISPLAY_COUNT_LIMIT {
                    break;
                }

                ui.label(i.to_string());
                render_event_slice(app, ui, event);
                ui.end_row();

                num_drawn += 1;
            }
        });
}

const COUNTDOWN_BEATS_LIMIT: u16 = 25;
const DISPLAY_BEATS_LIMIT: u16 = 200;
const DISPLAY_COUNT_LIMIT: usize = 32;

pub fn render_event_slice(app: &ClicksMonitorApp, ui: &mut egui::Ui, event: &Event) {
    let loc = event.location;
    let beat = app.status.cue.cue.get_beat(loc);
    if let Some(beat) = beat {
        ui.label(format!("{}.{}", beat.bar_number, beat.count));
    } else {
        ui.label("ERR");
    }
    if let Some(event) = event.event {
        ui.label(event.get_name());

        ui.horizontal(|ui| {
            ui.set_width(256.0);
            match event {
                EventDescription::TimecodeEvent {
                    time,
                    properties: _,
                } => {
                    ui.label(RichText::new(format!("{}", time)).monospace());
                }
                EventDescription::TimecodeStopEvent => {}
                EventDescription::JumpEvent {
                    destination,
                    requirement,
                    when_jumped: _,
                    when_passed: _,
                } => {
                    let beat = app.status.cue.cue.get_beat(destination);
                    let dest_s = if let Some(beat) = beat {
                        format!("{}.{}", beat.bar_number, beat.count)
                    } else {
                        "ERR".to_string()
                    };

                    let if_s = match requirement {
                        JumpRequirement::None => "ignoring VLT",
                        JumpRequirement::JumpModeOn => "if VLT on",
                        JumpRequirement::JumpModeOff => "if VLT off",
                    };

                    ui.label(format!("To {}, {}", dest_s, if_s));
                }
                EventDescription::TempoChangeEvent { tempo } => {
                    ui.label(format!("{} BPM", tempo));
                }
                EventDescription::GradualTempoChangeEvent {
                    start_tempo: _,
                    end_tempo,
                    length,
                } => {
                    ui.label(format!("{} BPM over {} beats", end_tempo, length));
                }
                EventDescription::PlaybackEvent {
                    sample,
                    channel_idx,
                    clip_idx,
                } => {
                    ui.label(format!(
                        "Channel {}, clip {}, from sample {}",
                        channel_idx, clip_idx, sample,
                    ));
                }
                EventDescription::PlaybackStopEvent { channel_idx } => {
                    ui.label(format!("Channel {}", channel_idx,));
                }
                EventDescription::RehearsalMarkEvent { label } => {
                    ui.label(label.str());
                }
                EventDescription::PauseEvent { behaviour } => {
                    ui.label(format!(
                        "Pause and {}",
                        match behaviour {
                            PauseEventBehaviour::Hold => "hold",
                            PauseEventBehaviour::RestartCue => "restart cue",
                            PauseEventBehaviour::RestartBeat => "restart beat",
                            PauseEventBehaviour::NextCue => "go to next cue",
                            PauseEventBehaviour::Jump { destination: _ } => "jump",
                        }
                    ));
                }
                _ => {}
            }
        });

        let beats_left = loc.saturating_sub(app.status.beat_state().beat_idx);

        ui.label(beats_left.to_string());
        ProgressBar::new(1.0 - (beats_left as f32 / COUNTDOWN_BEATS_LIMIT as f32).min(1.0))
            .fill(if beats_left == 0 {
                app.theme.active_prim
            } else if beats_left == 1 {
                app.theme.warn_prim
            } else if beats_left < COUNTDOWN_BEATS_LIMIT {
                app.theme.cued_prim
            } else {
                Color32::TRANSPARENT
            })
            .ui(ui);
    }
}
