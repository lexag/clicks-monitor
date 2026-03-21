use crate::app::ClicksMonitorApp;
use common::{
    local::config::{LogContext, LogKind, SystemConfigurationChange},
    protocol::request::Request,
};
use egui::Widget;
use itertools::Itertools;

pub fn display(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("System Settings").heading());
            if !app.udp_client.active {
                ui.colored_label(
                    app.theme.err_prim,
                    "Could not find system host. Check your connection.",
                );
                return;
            }
            if !app.local_memory.security.allow_interaction {
                ui.disable();
            }

            egui::Grid::new("system-settings")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Logs").underline());
                    ui.end_row();

                    ui.vertical(|ui| {
                        ui.set_width(150.0);
                        for kind in LogKind::iter(&LogKind::all()) {
                            let mut val = app.system_config.logger.active_kinds.contains(kind);
                            ui.checkbox(&mut val, format!("{}", kind));
                            app.system_config.logger.active_kinds.set(kind, val);
                        }
                    });
                    ui.vertical(|ui| {
                        for context in LogContext::iter(&LogContext::all()) {
                            let mut val =
                                app.system_config.logger.active_contexts.contains(context);
                            ui.checkbox(&mut val, context.get_name());
                            app.system_config.logger.active_contexts.set(context, val);
                        }
                    });
                    ui.end_row();
                });

            if egui::Button::new("Apply Logger Settings").ui(ui).clicked() {
                app.udp_client.send_msg(Request::ChangeConfiguration(
                    SystemConfigurationChange::ChangeLoggerConfiguration(app.system_config.logger),
                ));
            }
            //ui.separator();
            //if egui::Button::new("Reset to default configuration (destructive)")
            //    .fill(app.theme.err_prim_wk)
            //    .ui(ui)
            //    .clicked()
            //{
            //    app.udp_client
            //        .send_msg(ControlMessage::SetConfigurationRequest(
            //            SystemConfiguration::default(),
            //        ));
            //}
        });
    });
}
