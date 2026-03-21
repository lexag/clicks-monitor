use crate::{
    app::ClicksMonitorApp,
    window::{WindowCategory, WindowTab},
};
use common::{local::status::CombinedStatus, protocol::request::Request};
use egui::{Button, Label, RichText, ScrollArea, Sense, Vec2, Widget};

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct NavigationWindowMemory {
    pub current_single_tab: WindowTab,
    pub multiwindow_mode: bool,
    pub lock_navigation: bool,
}

pub fn display(app: &mut ClicksMonitorApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Navigation").heading());
    ui.checkbox(
        &mut app.local_memory.navigation.multiwindow_mode,
        "Enable multiview",
    );
    ui.checkbox(
        &mut app.local_memory.navigation.lock_navigation,
        "Lock layout",
    );
    ScrollArea::new([false, true]).show(ui, |ui| {
        ui.vertical(|ui| {
            let mut cat = WindowCategory::None;

            const BUTTON_HEIGHT: f32 = 32.0;

            for tab in WindowTab::list() {
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
                        .min_size(Vec2::new(ui.available_width(), BUTTON_HEIGHT)),
                );
                if label.clicked() && !app.local_memory.navigation.lock_navigation {
                    if app.local_memory.navigation.multiwindow_mode {
                        app.local_memory.dock_state.push_to_focused_leaf(tab);
                    } else {
                        app.local_memory.navigation.current_single_tab = tab
                    }
                }
            }
            ui.separator();
            if app.udp_client.active
                && egui::Button::new("Shutdown")
                    .fill(app.theme.err_prim_wk)
                    .ui(ui)
                    .clicked()
                && app.local_memory.security.allow_interaction
            {
                app.status = CombinedStatus::default();
                app.udp_client.send_msg(Request::Shutdown)
            }
        });
    });
}
