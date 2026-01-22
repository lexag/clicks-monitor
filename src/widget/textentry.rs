use crate::app::TemplateApp;
use egui::Widget;
use egui::{vec2, Align2, RichText};

#[derive(Default, Clone)]
pub struct TextEntry {
    title: String,
    text: String,
    password: bool,
    is_open: bool,
    just_submitted: bool,
}

impl TextEntry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn display(&mut self, app: &mut TemplateApp) -> &mut Self {
        if !self.is_open {
            return self;
        }
        egui::Window::new("text-entry-window")
            .pivot(Align2::CENTER_CENTER)
            .fixed_size(vec2(300.0, 100.0))
            .movable(false)
            .fixed_pos(app.ctx.screen_rect().center())
            .order(egui::Order::Foreground)
            .title_bar(false)
            .show(&app.ctx, |ui| {
                ui.label(RichText::new(format!("Text Entry: {}", self.title)).heading());
                let textbox = egui::TextEdit::singleline(&mut self.text)
                    .password(self.password)
                    .ui(ui);
                if textbox.lost_focus() {
                    self.is_open = false;
                    self.just_submitted = true;
                }
                textbox.request_focus();
            });
        self
    }

    pub fn open(&mut self, title: &str) -> &mut Self {
        if !self.is_open {
            self.title = title.to_string();
            self.is_open = true;
            self.password = false;
            self.just_submitted = false;
        }
        self
    }

    pub fn password(&mut self, password: bool) -> &mut Self {
        self.password = password;
        self
    }

    pub fn submitted(&mut self, title: &str) -> bool {
        if self.just_submitted && self.title == title {
            self.just_submitted = false;
            return true;
        }
        false
    }

    pub fn get_text(&self) -> String {
        self.text.clone()
    }

    pub fn done(&mut self) {
        self.is_open = false;
        self.text = "".to_string();
        self.title = "".to_string();
    }
}
