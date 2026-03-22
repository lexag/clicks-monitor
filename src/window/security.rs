use crate::app::ClicksMonitorApp;
use egui::RichText;

#[derive(serde::Serialize, serde::Deserialize, Default, Debug)]
pub struct SecurityWindowMemory {
    pub allow_interaction: bool,
    pub require_password: bool,
    pub password: String,
}

pub fn display(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        if !app.local_memory.security.allow_interaction {
            ui.disable();
        }
        ui.label(egui::RichText::new("Client Settings").heading());
        egui::Grid::new("client-settings")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Client locking").underline());
                ui.end_row();
                ui.label("Require password");
                ui.checkbox(&mut app.local_memory.security.require_password, "");
                ui.end_row();
                let textbox = egui::TextEdit::singleline(&mut app.local_memory.security.password)
                    .password(!ui.label("Password [view]").is_pointer_button_down_on());
                ui.add_enabled(app.local_memory.security.require_password, textbox);
                ui.end_row();
            });
    });
}

pub fn lock_slot(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    if app.local_memory.security.allow_interaction {
        let label = ui.colored_label(app.theme.active_prim, RichText::new("󰌿 FREE").monospace());
        if label.clicked() {
            app.local_memory.security.allow_interaction = false;
        }
    } else {
        let color = if app.local_memory.security.require_password {
            app.theme.cued_prim
        } else {
            app.theme.warn_prim
        };
        let label = ui.colored_label(color, RichText::new("󰌾 LOCK").monospace());
        if label.clicked() {
            if app.local_memory.security.require_password {
                // TODO: Handle modal
            } else {
                app.local_memory.security.allow_interaction = true;
            }
        }
    }
}

pub fn try_toggle_lock(app: &mut ClicksMonitorApp) {
    if app.local_memory.security.require_password {
    } else {
        app.local_memory.security.allow_interaction = !app.local_memory.security.allow_interaction;
    }
}
