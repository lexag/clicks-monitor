use crate::app::ClicksMonitorApp;
use common::{
    local::config::{LogContext, LogKind},
    mem::time::format_hms,
};
use egui::{Grid, Label, RichText, ScrollArea, Widget};

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct LogWindowMemory {
    pub kind_filter: LogKind,
    pub context_filter: LogContext,
}

pub fn display(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.label(RichText::new("System Log").heading());
        ui.horizontal(|ui| {
            ui.label("Filter by log type:");
            for (label, kind) in LogKind::all().iter_names() {
                if ui
                    .selectable_label(app.local_memory.log.kind_filter.contains(kind), label)
                    .clicked()
                {
                    app.local_memory.log.kind_filter.toggle(kind);
                }
            }
            ui.label("Filter by log context:");
            for (label, context) in LogContext::all().iter_names() {
                if ui
                    .selectable_label(app.local_memory.log.context_filter.contains(context), label)
                    .clicked()
                {
                    app.local_memory.log.context_filter.toggle(context);
                };
            }
        });

        ui.separator();

        ScrollArea::vertical()
            .animated(true)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                Grid::new("logs-grid")
                    .striped(true)
                    .min_col_width(96.0)
                    .num_columns(4)
                    .show(ui, |ui| {
                        ui.label("Time");
                        ui.label("Type");
                        ui.label("Context");
                        ui.label("Message");
                        ui.end_row();
                        for entry in &app.log_entries {
                            if !app.local_memory.log.kind_filter.contains(entry.kind)
                                || !app.local_memory.log.context_filter.contains(entry.context)
                            {
                                continue;
                            }

                            ui.label(format_hms(entry.time / 1000).str());
                            ui.label(entry.kind.to_string());
                            ui.label(entry.context.get_name());
                            ui.horizontal(|ui| {
                                ui.set_width(ui.available_width());
                                Label::new(entry.message.clone()).ui(ui);
                            });
                            ui.end_row();
                        }
                    });
            });
    });
}
