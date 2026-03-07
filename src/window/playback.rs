use crate::app::TemplateApp;
use common::event::EventDescription;
use common::local::status::{AudioSourceState, PlaybackState};
use common::protocol::request::{ControlAction, Request};
use egui::{Align, Color32, Label, ProgressBar, RichText, Sense};
use egui::{Grid, Widget};
use std::collections::HashMap;

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct PlaybackWindowMemory {
    pub clip_cue_list: HashMap<u8, u16>,
}

pub fn display(app: &mut TemplateApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Playback").heading());
        if !app.layout_settings.playback.clip_cue_list.is_empty() {
            ui.label(format!(
                "{} clip(s) cued",
                app.layout_settings.playback.clip_cue_list.len(),
            ));
            if ui.button("Play").clicked() && app.allow_interaction {
                play_clip_cue(app, ui);
            };
            if ui.button("Once").clicked() && app.allow_interaction {
                play_clip_cue(app, ui);
                app.layout_settings.playback.clip_cue_list.clear();
            };
            if ui.button("Clear").clicked() && app.allow_interaction {
                app.layout_settings.playback.clip_cue_list.clear();
            };
        }
    });
    Grid::new("playback-channel-grid")
        .num_columns(5)
        .striped(true)
        .min_row_height(24.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label("Ch. #");
            ui.label("Channel");
            ui.label("Progress");
            ui.label("Duration");
            ui.label("Clips");
            ui.end_row();
            for i in 0..30 {
                render_channel_slice(app, ui, i);
                ui.end_row();
            }
        });
}

pub fn render_channel_slice(app: &mut TemplateApp, ui: &mut egui::Ui, index: usize) {
    let channel_name = app.system_config.channels[index + 2].name.str().to_string();
    let source = if let AudioSourceState::PlaybackStatus(status) = app.status.sources[index + 2] {
        status
    } else {
        PlaybackState::default()
    };
    let seconds_left = if app.status.jack_status.sample_rate != 0 {
        (source.clip_length as f32 - source.current_sample as f32).abs()
            / (app.status.jack_status.sample_rate as f32)
    } else {
        0.0
    };
    let seconds_total = if app.status.jack_status.sample_rate != 0 {
        source.clip_length as f32 / app.status.jack_status.sample_rate as f32
    } else {
        0.0
    };
    // Index
    ui.label(format!("{:02}", index + 1));

    // Name
    ui.label(channel_name);

    // Time bar
    ProgressBar::new(
        source.current_sample as f32 / source.clip_length as f32 * source.playing as usize as f32,
    )
    .text(format!(
        "{:02.0}:{:02.0}.{:03.0}",
        (seconds_left / 60.0).floor(),
        (seconds_left % 60.0).floor(),
        (seconds_left * 1000.0 % 1000.0).floor()
    ))
    .desired_width(500.0)
    .ui(ui);
    ui.label(format!(
        "{:02.0}:{:02.0}.{:03.0}",
        (seconds_total / 60.0).floor(),
        (seconds_total % 60.0).floor(),
        (seconds_total * 1000.0 % 1000.0).floor()
    ));

    // Clips
    ui.horizontal(|ui| {
        if app.status.playback_status.clips.is_empty() {
            ui.label("No clips");
            return;
        }

        for (i, clip) in app.status.playback_status.clips[index]
            .clone()
            .iter()
            .enumerate()
        {
            let selected = source.playing && source.clip_idx == i as u16;
            render_clip(app, ui, *clip, selected, source);
        }
    });
}

pub fn render_clip(
    app: &mut TemplateApp,
    ui: &mut egui::Ui,
    clip: u16,
    selected: bool,
    status: PlaybackState,
) {
    let real_clip = clip < 2048;
    // FIXME: magic number for "usize::MAX
    // of the core machine, which we can't
    // guarantee is the same as on this
    // machine
    egui::Frame::new()
        .fill(if selected {
            app.theme.active_prim
        } else if app
            .layout_settings
            .playback
            .clip_cue_list
            .get(&status.channel)
            .is_some_and(|c| *c as u16 == clip)
        {
            app.theme.cued_prim
        } else if real_clip {
            app.theme.base_ex
        } else {
            Color32::TRANSPARENT
        })
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.set_width(64.0);
                ui.set_height(16.0);

                if !real_clip {
                    return;
                }

                let cue_button = Label::new("☉")
                    .sense(Sense::click())
                    .halign(Align::RIGHT)
                    .selectable(false)
                    .ui(ui);

                if cue_button.clicked() && app.allow_interaction {
                    app.layout_settings
                        .playback
                        .clip_cue_list
                        .insert(status.channel, clip);
                }

                let play_button = Label::new(if status.playing && selected {
                    "⏹"
                } else {
                    "▶"
                })
                .selectable(false)
                .sense(Sense::click())
                .ui(ui);

                if play_button.double_clicked() && app.allow_interaction {
                    app.udp_client
                        .send_msg(Request::ControlAction(ControlAction::RunEvent(
                            if status.playing && selected {
                                EventDescription::PlaybackStopEvent {
                                    channel_idx: status.channel as u16,
                                }
                            } else {
                                EventDescription::PlaybackEvent {
                                    sample: 0,
                                    channel_idx: status.channel as u16,
                                    clip_idx: clip,
                                }
                            },
                        )));
                }

                ui.label(clip.to_string());
            });
        });
}

pub fn play_clip_cue(app: &mut TemplateApp, ui: &mut egui::Ui) {
    for (channel, clip) in app.layout_settings.playback.clip_cue_list.clone() {
        app.udp_client
            .send_msg(Request::ControlAction(ControlAction::RunEvent(
                EventDescription::PlaybackEvent {
                    sample: 0,
                    channel_idx: channel as u16,
                    clip_idx: clip,
                },
            )));
    }
}
