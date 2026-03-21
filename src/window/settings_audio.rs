use crate::app::ClicksMonitorApp;
use common::{local::config::SystemConfigurationChange, protocol::request::Request};
use egui::Widget;

pub fn display(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    egui::Grid::new("audio-system-settings")
        .num_columns(2)
        .show(ui, |ui| {
            if app.status.jack_status.running {
                ui.disable();
            }
            ui.label(egui::RichText::new("Audio").underline());
            ui.end_row();

            ui.label("Device name");
            let mut devices = app.status.jack_status.available_devices.iter();
            egui::ComboBox::new("device-selector", "")
                .selected_text(
                    match devices.position(|d| {
                        d.unwrap_or_default().id == app.system_config.audio.server.device_id
                    }) {
                        Some(idx) => app.status.jack_status.available_devices[idx]
                            .map_or("".to_string(), |v| v.name.str().to_string()),
                        None => "No audio device selected".to_string(),
                    },
                )
                .truncate()
                .width(400.0)
                .show_ui(ui, |ui| {
                    for device in app.status.jack_status.available_devices {
                        ui.selectable_value(
                            &mut app.system_config.audio.server.device_id,
                            device.unwrap_or_default().id,
                            device.unwrap_or_default().name.str(),
                        );
                    }
                });
            ui.end_row();

            ui.label("Sample rate");
            ui.horizontal(|ui| {
                for (rate, label) in [
                    (44100, "44.1kHz"),
                    (48000, "48kHz"),
                    //(96000, "96kHz"),
                    //(192000, "192kHz"),
                ] {
                    if ui
                        .selectable_label(app.system_config.audio.server.sample_rate == rate, label)
                        .clicked()
                    {
                        app.system_config.audio.server.sample_rate = rate;
                    }
                }
            });
            ui.end_row();

            ui.label("Period size (samples)");
            ui.horizontal(|ui| {
                for size in [128, 256, 512, 1024, 2048, 4096] {
                    if ui
                        .selectable_label(
                            app.system_config.audio.server.period_size == size,
                            size.to_string(),
                        )
                        .clicked()
                    {
                        app.system_config.audio.server.period_size = size;
                    }
                }
            });
            ui.end_row();

            if egui::Button::new("Launch Audio Processor")
                .fill(app.theme.cued_prim)
                .ui(ui)
                .clicked()
            {
                app.udp_client.send_msg(Request::ChangeConfiguration(
                    SystemConfigurationChange::ChangeAudioConfiguration(app.system_config.audio),
                ));
                app.udp_client.send_msg(Request::Initialize);
            }
        });
}
