use crate::app::ClicksMonitorApp;
use common::{
    local::{
        config::{ChannelAssignment, ChannelConfiguration, SystemConfigurationChange},
        status::AudioSourceState,
    },
    mem::time::format_hms,
    protocol::request::{ControlAction, Request},
};
use egui::{
    Align2, Color32, FontId, Frame, Grid, Label, Pos2, ProgressBar, Rect, Response, RichText,
    ScrollArea, Sense, Slider, Stroke, StrokeKind, Vec2, Widget,
};

const MSG_NO_CONNECTION: &str = "Unable to find sources. Check connection.";
const MSG_NO_PROCESSOR: &str = "Sources are unavailable when the audio processor is not running. Start the audio processor to access sources.";
const MSG_NO_INTERACTION: &str =
    "Editing sources is disable when client is locked. Unlock client to access settings.";

pub fn display(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    ScrollArea::horizontal()
        .drag_to_scroll(true)
        .show(ui, |ui| {
            Grid::new("sources-mixer").show(ui, |ui| {
                let w = 64.0;
                for i in 0..32 {
                    channel_strip(app, ui, i, w);
                }
            });
        });
    return;
    let stroke = Stroke::new(1.0, app.theme.neutral_prim);
    let sources = &app.status.sources.clone();

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
                            app.local_memory.security.allow_interaction,
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

pub fn configuration_window(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
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

pub fn channel_strip(app: &mut ClicksMonitorApp, ui: &mut egui::Ui, idx: usize, width: f32) {
    let conf: ChannelConfiguration = app.system_config.channels[idx];
    Frame::group(ui.style()).show(ui, |ui| {
        ui.vertical_centered_justified(|ui| {
            ui.set_width(width);

            Label::new((idx + 1).to_string()).ui(ui);
            Label::new(conf.name.str()).truncate().ui(ui);

            ui.separator();

            ui.vertical_centered(|ui| {
                ui.set_height(100.0);
                ui.label("HW OUT");
                let conns = app.status.jack_status.connections[idx];
                let mut count = 0;
                for i in 0..32 {
                    if 0b1 << i & conns > 0 {
                        if count <= 3 {
                            Frame::new().fill(app.theme.base_ex).show(ui, |ui| {
                                ui.label(if count < 3 {
                                    (i + 1).to_string()
                                } else {
                                    "...".to_string()
                                })
                            });
                        }
                        count += 1;
                    };
                }
            });

            ui.separator();

            Label::new(format!("{:0>+2.1}dB", &mut app.sources_gains[idx])).ui(ui);

            let max_slider = 12.0f32;
            let min_slider = -48.0f32;
            let mut slider_value = app.sources_gains[idx];
            if custom_volume_slider(
                app,
                ui,
                width,
                500.0,
                &mut slider_value,
                max_slider,
                min_slider,
            )
            .drag_stopped()
                && app.local_memory.security.allow_interaction
            {
                app.udp_client
                    .send_msg(Request::ControlAction(ControlAction::SetChannelGain(
                        idx as u8,
                        app.sources_gains[idx],
                    )));
                if conf.channel_assignment == ChannelAssignment::L
                    && app
                        .system_config
                        .channels
                        .get(idx + 1)
                        .is_some_and(|c| c.channel_assignment == ChannelAssignment::R)
                {
                    app.udp_client
                        .send_msg(Request::ControlAction(ControlAction::SetChannelGain(
                            idx as u8 + 1,
                            slider_value,
                        )));
                } else if conf.channel_assignment == ChannelAssignment::R
                    && app
                        .system_config
                        .channels
                        .get(idx.saturating_sub(1))
                        .is_some_and(|c| c.channel_assignment == ChannelAssignment::L)
                {
                    app.udp_client
                        .send_msg(Request::ControlAction(ControlAction::SetChannelGain(
                            idx as u8 - 1,
                            slider_value,
                        )));
                }
            }
            app.sources_gains[idx] = slider_value;

            ui.separator();
            ui.horizontal(|ui| {
                for (val, label) in [
                    (ChannelAssignment::L, "L"),
                    (ChannelAssignment::Mono, "M"),
                    (ChannelAssignment::R, "R"),
                ] {
                    if ui
                        .selectable_label(conf.channel_assignment == val, label)
                        .clicked()
                    {
                        app.system_config.channels[idx].channel_assignment = val;
                        app.udp_client.send_msg(Request::ChangeConfiguration(
                            SystemConfigurationChange::ChangeChannelConfiguration(
                                idx as u8,
                                app.system_config.channels[idx],
                            ),
                        ));
                    }
                }
            });

            ui.label(
                RichText::new(
                    if conf.channel_assignment == ChannelAssignment::L
                        && app
                            .system_config
                            .channels
                            .get(idx + 1)
                            .is_some_and(|c| c.channel_assignment == ChannelAssignment::R)
                    {
                        "LINK >"
                    } else if conf.channel_assignment == ChannelAssignment::R
                        && app
                            .system_config
                            .channels
                            .get(idx.saturating_sub(1))
                            .is_some_and(|c| c.channel_assignment == ChannelAssignment::L)
                    {
                        "< LINK"
                    } else {
                        ""
                    },
                )
                .monospace(),
            );

            ui.separator();

            Label::new(conf.name.str()).truncate().ui(ui);
            Label::new((idx + 1).to_string()).ui(ui);
        });
    });
}

pub fn custom_volume_slider(
    app: &mut ClicksMonitorApp,
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    val: &mut f32,
    max_val: f32,
    min_val: f32,
) -> Response {
    let col_handle_fill = app.theme.base_wk;
    let col_handle_strk = app.theme.neutral_prim;
    let col_path_fill = app.theme.base_ex;

    let val_percent = 1.0 - (*val - min_val) / (max_val - min_val);

    let top_left = ui.cursor().min;
    let size = Vec2::new(width, height);
    let top_center = top_left + Vec2::new(size.x / 2.0, 0.0);
    let full_rect = Rect::from_min_size(top_left, size);
    let (resp, p) = ui.allocate_painter(size, Sense::click_and_drag());

    // Path
    let path_fraction = 0.9;
    let path_thickness = 10.0;
    let path_rect = full_rect.scale_from_center2(Vec2::new(path_thickness / width, path_fraction));
    p.rect_filled(path_rect, 5.0, col_path_fill);

    let inset = 0.05;

    let db_steps = 6.0;
    let num_steps = (max_val - min_val) / db_steps;
    let step_length = path_rect.height() / num_steps;
    for (i, value) in (min_val as i32..(max_val + 1.0) as i32)
        .step_by(db_steps as usize)
        .enumerate()
    {
        p.text(
            top_left
                + Vec2::new(
                    width * (1.0 - inset),
                    height - (1.0 - path_fraction) * height * 0.5 - i as f32 * step_length,
                ),
            Align2::RIGHT_CENTER,
            value.to_string(),
            FontId::monospace(11.0),
            col_handle_strk.gamma_multiply(0.5),
        );
    }

    // Handle
    let handle_size = Vec2::new(40.0, 20.0);
    let handle_center = top_center
        + Vec2::new(
            0.0,
            height * (1.0 - path_fraction) * 0.5 + height * val_percent * path_fraction,
        );
    let handle_rect = Rect::from_min_size(handle_center - handle_size * 0.5, handle_size);
    p.rect(
        handle_rect,
        5.0,
        col_handle_fill,
        Stroke::new(1.0, col_handle_strk),
        StrokeKind::Middle,
    );

    if resp.dragged() && app.local_memory.security.allow_interaction {
        *val -= resp.drag_delta().y / path_rect.height() * (max_val - min_val);
        *val = val.clamp(min_val, max_val)
    }

    resp
}
