use crate::{
    app::{ConfigurationEditorTab, TemplateApp},
    window::{WindowCategory, WindowTab},
};
use common::{local::status::CombinedStatus, protocol::request::Request};
use egui::{Button, Color32, Label, RichText, ScrollArea, Sense, Stroke, Vec2, Widget};

pub fn display(app: &mut TemplateApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Navigation").heading());
    ScrollArea::new([false, true])
        .drag_to_scroll(true)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                let mut cat = WindowCategory::None;

                const BUTTON_HEIGHT: f32 = 32.0;

                for tab in [
                    WindowTab::SourcesOverview,
                    WindowTab::SourcesTime,
                    WindowTab::SourcesPlayback,
                    WindowTab::CueTimeline,
                    WindowTab::CueBeats,
                    WindowTab::CueEvents,
                    WindowTab::ControlTransport,
                    WindowTab::ControlRunEvent,
                    WindowTab::ControlSystem,
                    WindowTab::SystemLogs,
                    WindowTab::SystemPerformance,
                    WindowTab::SystemNetwork,
                    WindowTab::SettingsAudio,
                    WindowTab::SettingsNetwork,
                    WindowTab::SettingsLogs,
                    WindowTab::PreferencesAppearance,
                    WindowTab::PreferencesHotkeys,
                    WindowTab::PreferencesSecurity,
                ] {
                    if tab.category() != cat {
                        cat = tab.category();
                        ui.add_space(BUTTON_HEIGHT / 3.0);
                        ui.add(Label::new(RichText::new(cat.name())).selectable(false));
                        ui.add_space(BUTTON_HEIGHT / 3.0);
                    }
                    let label = ui.add(
                        Button::new(RichText::new(tab.name()))
                            .sense(Sense::click())
                            .truncate()
                            //.stroke(Stroke::new(0.0, Color32::TRANSPARENT))
                            //.fill(Color32::TRANSPARENT)
                            .min_size(Vec2::new(ui.available_width(), BUTTON_HEIGHT)),
                    );
                    if label.clicked() {
                        app.tab = tab
                    }
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
