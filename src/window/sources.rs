use crate::app::ClicksMonitorApp;
use common::{
    local::config::{ChannelAssignment, ChannelConfiguration, SystemConfigurationChange},
    protocol::request::{ControlAction, Request},
};
use egui::{
    Align2, FontId, Frame, Grid, Label, Rect, Response, RichText, ScrollArea, Sense, Stroke,
    StrokeKind, Vec2, Widget,
};

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
}

pub fn channel_strip(app: &mut ClicksMonitorApp, ui: &mut egui::Ui, idx: usize, width: f32) {
    let conf: ChannelConfiguration = app.system_config.channels[idx];
    Frame::group(ui.style()).show(ui, |ui| {
        ui.vertical_centered_justified(|ui| {
            ui.set_width(width);

            Label::new((idx + 1).to_string()).ui(ui);
            Label::new(conf.name.str()).truncate().ui(ui);

            ui.separator();

            hw_io_slots(app, idx, ui);

            ui.separator();

            fader(app, idx, width, ui);

            ui.separator();

            stereo_link_selector(app, idx, ui);

            ui.separator();

            Label::new(conf.name.str()).truncate().ui(ui);
            Label::new((idx + 1).to_string()).ui(ui);
        });
    });
}

fn stereo_link_selector(app: &mut ClicksMonitorApp, idx: usize, ui: &mut egui::Ui) {
    let conf: ChannelConfiguration = app.system_config.channels[idx];
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
}

fn fader(app: &mut ClicksMonitorApp, idx: usize, width: f32, ui: &mut egui::Ui) {
    const MAX_SLIDER: f32 = 12.0f32;
    const MIN_SLIDER: f32 = -48.0f32;

    let conf: ChannelConfiguration = app.system_config.channels[idx];
    let prev_link: ChannelAssignment = app
        .system_config
        .channels
        .get(idx.saturating_sub(1))
        .map_or(ChannelAssignment::Mono, |&cc| cc.channel_assignment);
    let next_link: ChannelAssignment = app
        .system_config
        .channels
        .get(idx.saturating_add(1))
        .map_or(ChannelAssignment::Mono, |&cc| cc.channel_assignment);
    Label::new(format!("{:0>+2.1}dB", &mut app.sources_gains[idx])).ui(ui);

    let mut slider_value = app.sources_gains[idx];
    if custom_volume_slider(
        app,
        ui,
        width,
        500.0,
        &mut slider_value,
        MAX_SLIDER,
        MIN_SLIDER,
    )
    .drag_stopped()
        && app.local_memory.security.allow_interaction
    {
        app.udp_client
            .send_msg(Request::ControlAction(ControlAction::SetChannelGain(
                idx as u8,
                app.sources_gains[idx],
            )));
        if conf.channel_assignment == ChannelAssignment::L && next_link == ChannelAssignment::R {
            app.udp_client
                .send_msg(Request::ControlAction(ControlAction::SetChannelGain(
                    idx as u8 + 1,
                    slider_value,
                )));
        }
        if conf.channel_assignment == ChannelAssignment::R && prev_link == ChannelAssignment::L {
            app.udp_client
                .send_msg(Request::ControlAction(ControlAction::SetChannelGain(
                    idx as u8 - 1,
                    slider_value,
                )));
        }
    }
    app.sources_gains[idx] = slider_value;
}

fn hw_io_slots(app: &mut ClicksMonitorApp, idx: usize, ui: &mut egui::Ui) {
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
