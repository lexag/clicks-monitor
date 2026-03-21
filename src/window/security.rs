use crate::app::ClicksMonitorApp;

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct SecurityWindowMemory {
    pub allow_interaction: bool,
    pub require_password: bool,
    pub password: String,
}

pub fn display(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {}
