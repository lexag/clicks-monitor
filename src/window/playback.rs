use crate::app::TemplateApp;
use crate::widget::cassette::Cassette;
use common::local::status::{AudioSourceState, PlaybackState};
use common::mem::time::format_hms;
use egui::{Color32, Frame, ProgressBar, RichText, Stroke, Vec2};
use egui::{Grid, Widget};

pub fn display(app: &mut TemplateApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Playback").heading());
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
    .text(format_hms(seconds_left as u64).str())
    .desired_width(500.0)
    .ui(ui);
    ui.label(format_hms(seconds_total as u64).str());

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
    let frame = egui::Frame::new()
        .fill(if selected {
            app.theme.active_prim
        } else if clip < 2048 {
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
                ui.label(if clip < 2048 {
                    clip.to_string()
                } else {
                    "".to_string()
                });
                if selected {}
            })
        });
    if frame.response.double_clicked() && app.allow_interaction {
        app.udp_client
            .send_msg(common::protocol::request::Request::ControlAction(
                common::protocol::request::ControlAction::RunEvent(
                    common::event::EventDescription::PlaybackEvent {
                        sample: 0,
                        channel_idx: status.channel as u16,
                        clip_idx: clip,
                    },
                ),
            ));
    };
}
