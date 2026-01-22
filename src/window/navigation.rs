use crate::app::{ConfigurationEditorTab, TabView, TemplateApp};
use common::{local::status::CombinedStatus, protocol::request::Request};
use egui::Widget;

pub fn display(app: &mut TemplateApp, ui: &mut egui::Ui) {
    ui.vertical_centered_justified(|ui| {
        if ui.button("Sources").clicked() {
            app.tab = TabView::Sources
        }
        if ui.button("Cue").clicked() {
            app.tab = TabView::Cue
        }
        if ui.button("Transport").clicked() {
            app.tab = TabView::Control
        }
        if ui.button("Options").clicked() {
            app.tab = TabView::Options
        }
        ui.separator();
        if app.udp_client.active
            && egui::Button::new("Shutdown")
                .fill(app.theme.err_prim_wk)
                .ui(ui)
                .clicked()
            && app.allow_interaction
        {
            app.status = CombinedStatus::default();
            app.udp_client.send_msg(Request::Shutdown)
        }
    });
}

pub fn configuration_tab_buttons(app: &mut TemplateApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        for (label, tab, require_audio_processor) in [
            ("Routing", ConfigurationEditorTab::Routing, true),
            ("Network", ConfigurationEditorTab::Network, false),
            ("Channels", ConfigurationEditorTab::Channels, false),
        ] {
            if ui
                .add_enabled(
                    !require_audio_processor || app.status.jack_status.running,
                    egui::SelectableLabel::new(
                        app.layout_settings.configuration_editor_tab == tab,
                        label,
                    ),
                )
                .on_disabled_hover_text("This tab requires a running audio processor.")
                .clicked()
            {
                app.layout_settings.configuration_editor_tab = tab
            }
        }
    });
}
