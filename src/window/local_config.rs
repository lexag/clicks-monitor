use crate::app::TemplateApp;

pub fn display(app: &mut TemplateApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        if !app.allow_interaction {
            ui.disable();
        }
        ui.label(egui::RichText::new("Client Settings").heading());
        egui::Grid::new("client-settings")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Client locking").underline());
                ui.end_row();
                ui.label("Require password");
                ui.checkbox(&mut app.require_password, "");
                ui.end_row();
                let textbox = egui::TextEdit::singleline(&mut app.password)
                    .password(!ui.label("Password [view]").is_pointer_button_down_on());
                ui.add_enabled(app.require_password, textbox);
                ui.end_row();
            });
    });
}
