use egui::Window;

pub fn new(title: &str) -> Window<'static> {
    Window::new(title)
        .resizable(true)
        .hscroll(true)
        .vscroll(true)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
        .fade_in(true)
        .fade_out(true)
        .title_bar(true)
}
