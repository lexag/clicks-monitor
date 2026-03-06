use crate::app::TemplateApp;
use common::{
    local::{
        config::{ChannelAssignment, SystemConfigurationChange},
        status::AudioSourceState,
    },
    mem::time::format_hms,
    protocol::request::{ControlAction, Request},
};
use egui::{Color32, ProgressBar, Stroke, Widget};

const MSG_NO_CONNECTION: &str = "Unable to find sources. Check connection.";
const MSG_NO_PROCESSOR: &str = "Sources are unavailable when the audio processor is not running. Start the audio processor to access sources.";
const MSG_NO_INTERACTION: &str =
    "Editing sources is disable when client is locked. Unlock client to access settings.";

pub fn display(app: &mut TemplateApp, ui: &mut egui::Ui) {
    let stroke = Stroke::new(1.0, app.theme.neutral_prim);
    let sources = &app.status.sources;

    if sources.is_empty() {
        ui.colored_label(app.theme.err_prim, MSG_NO_CONNECTION);
        ui.disable();
        return;
    }
    if !app.status.jack_status.running {
        ui.colored_label(app.theme.err_prim, MSG_NO_PROCESSOR);
        ui.disable();
        return;
    }

    egui::Grid::new("sources-grid")
        .striped(true)
        .min_row_height(ui.available_height() / 32.0 * 0.9)
        .num_columns(7)
        .show(ui, |ui| {
            for (i, source) in sources.iter().enumerate() {
                let ch_conf = app.system_config.channels[i].clone();
                ui.label(format!("{:0>2}", i + 1));
                ui.horizontal_centered(|ui| {
                    if ui
                        .add_enabled(
                            app.allow_interaction,
                            egui::DragValue::new(&mut app.sources_gains[i])
                                .range(-24.0f32..=12.0f32)
                                .speed(0.01)
                                .custom_formatter(|val, _| format!("{:0>+2.1}dB", val)),
                        )
                        .on_disabled_hover_text(MSG_NO_INTERACTION)
                        .drag_stopped()
                    {
                        app.udp_client.send_msg(Request::ControlAction(
                            ControlAction::SetChannelGain(i as u8, app.sources_gains[i]),
                        ));
                    }
                });
                ui.label(ch_conf.name.str());
                match source {
                    AudioSourceState::BeatStatus(status) => {
                        let beat = app
                            .status
                            .cue
                            .cue
                            .get_beat(status.beat_idx)
                            .unwrap_or_default();
                        let next_beat = app
                            .status
                            .cue
                            .cue
                            .get_beat(status.next_beat_idx)
                            .unwrap_or_default();
                        ui.horizontal_centered(|ui| {
                            ui.label(
                                egui::RichText::new(if status.beat_idx < u16::MAX / 2 {
                                    format!("{:0>3}", status.beat_idx)
                                } else {
                                    "NUL".to_string()
                                })
                                .monospace(),
                            );
                            ui.label(
                                egui::RichText::new(format!("{:0>3}", status.next_beat_idx,))
                                    .monospace()
                                    .weak(),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "({:0>3}.{:0>2})",
                                    beat.bar_number, beat.count
                                ))
                                .monospace(),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "({:0>3}.{:0>2})",
                                    next_beat.bar_number, next_beat.count
                                ))
                                .monospace()
                                .weak(),
                            );
                        });
                    }
                    AudioSourceState::TimeStatus(status) => {
                        ui.horizontal_centered(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "{:0>2}:{:0>2}:{:0>2}:{:0>2}",
                                    status.ltc.h, status.ltc.m, status.ltc.s, status.ltc.f
                                ))
                                .monospace(),
                            );
                            ui.label(
                                egui::RichText::new(format!("({:0>5})", status.ltc.frame_progress))
                                    .monospace()
                                    .weak(),
                            );
                        });
                    }
                    AudioSourceState::PlaybackStatus(status) => {
                        ui.horizontal_centered(|ui| {
                            for (i, clip) in app.status.playback_status.clips
                                [status.channel as usize]
                                .clone()
                                .iter()
                                .enumerate()
                            {
                                let playing = status.playing && status.clip_idx == i as u16;
                                egui::Frame::new()
                                    .stroke(if playing {
                                        Stroke::new(1.0, Color32::WHITE)
                                    } else {
                                        stroke
                                    })
                                    .fill(if playing {
                                        app.theme.active_prim
                                    } else if *clip < 2048 {
                                        // FIXME: magic number for "usize::MAX
                                        // of the core machine, which we can't
                                        // guarantee is the same as on this
                                        // machine
                                        app.theme.cued_prim
                                    } else {
                                        Color32::TRANSPARENT
                                    })
                                    .show(ui, |ui| {
                                        ui.horizontal_centered(|ui| {
                                            ui.set_width(64.0);
                                            ui.set_height(16.0);
                                            ui.label(if *clip < 2048 {
                                                clip.to_string()
                                            } else {
                                                "".to_string()
                                            });
                                            if playing {
                                                ProgressBar::new(
                                                    status.current_sample as f32
                                                        / status.clip_length as f32,
                                                )
                                                .text(
                                                    ((status.clip_length as i32
                                                        - status.current_sample)
                                                        .abs()
                                                        / app.status.jack_status.sample_rate
                                                            as i32)
                                                        .to_string(),
                                                )
                                                .desired_width(ui.available_width())
                                                .ui(ui);
                                            }
                                        })
                                    });
                            }
                        });
                    }
                    _ => {}
                }
                ui.end_row();
            }
        });
}

pub fn configuration_window(app: &mut TemplateApp, ui: &mut egui::Ui) {
    if egui::Button::new("Apply").ui(ui).clicked() {
        for i in 0..32 {
            app.udp_client.send_msg(Request::ChangeConfiguration(
                SystemConfigurationChange::ChangeChannelConfiguration(
                    i as u8,
                    app.system_config.channels[i],
                ),
            ));
        }
    }
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());
        egui::Grid::new("channels-grid")
            .num_columns(5)
            .show(ui, |ui| {
                ui.label("Src. #");
                ui.label("Ch. #");
                ui.label("Side");
                ui.label("Channel Name");
                ui.label("Description");
                ui.end_row();

                for (i, channel) in app.system_config.channels[2..].iter_mut().enumerate() {
                    ui.label(format!("{:0>2}", i + 3));
                    ui.label(format!("{:0>2}", i));
                    egui::ComboBox::new(format!("channel-assignment-selector-{i}"), "")
                        .selected_text(match channel.channel_assignment {
                            ChannelAssignment::L => "L",
                            ChannelAssignment::R => "R",
                            ChannelAssignment::Mono => "Mono",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut channel.channel_assignment,
                                ChannelAssignment::L,
                                "L",
                            );
                            ui.selectable_value(
                                &mut channel.channel_assignment,
                                ChannelAssignment::R,
                                "R",
                            );
                            ui.selectable_value(
                                &mut channel.channel_assignment,
                                ChannelAssignment::Mono,
                                "Mono",
                            );
                        });
                    ui.text_edit_singleline(&mut channel.name.str());
                    ui.end_row();
                }
            });
    });
}
