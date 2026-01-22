use egui::{
    Color32,
    epaint::CornerRadiusF32,
    style::{HandleShape, WidgetVisuals, Widgets},
};

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub base: Color32,
    pub base_ex: Color32,
    pub base_wk: Color32,
    pub err_prim: Color32,
    pub err_prim_wk: Color32,
    pub err_sec: Color32,
    pub warn_prim: Color32,
    pub warn_sec: Color32,
    pub active_prim: Color32,
    pub active_sec: Color32,
    pub cued_prim: Color32,
    pub cued_sec: Color32,
    pub neutral_prim: Color32,
    pub neutral_sec: Color32,

    pub handle_shape: HandleShape,
    pub corner_radius: f32,

    pub is_dark_mode: bool,
}
impl Theme {
    pub fn visuals(&self, old: egui::Visuals) -> egui::Visuals {
        egui::Visuals {
            button_frame: true,
            clip_rect_margin: 0.0,
            code_bg_color: self.base,
            collapsing_header_frame: true,
            dark_mode: self.is_dark_mode,
            error_fg_color: self.err_prim,
            extreme_bg_color: self.base_ex,
            faint_bg_color: self.base_wk,
            handle_shape: self.handle_shape,
            indent_has_left_vline: false,
            menu_corner_radius: CornerRadiusF32 {
                nw: self.corner_radius,
                ne: self.corner_radius,
                sw: self.corner_radius,
                se: self.corner_radius,
            }
            .into(),
            numeric_color_space: egui::style::NumericColorSpace::GammaByte,
            panel_fill: self.base,
            resize_corner_size: self.corner_radius,
            selection: egui::style::Selection {
                bg_fill: self.cued_sec,
                stroke: egui::Stroke {
                    width: 1.0,
                    color: self.cued_prim,
                },
            },
            slider_trailing_fill: false,
            striped: false,
            warn_fg_color: self.warn_prim,
            widgets: Widgets {
                noninteractive: self.make_widget_visual(
                    old.widgets.noninteractive,
                    self.base,
                    self.neutral_prim,
                ),
                inactive: self.make_widget_visual(
                    old.widgets.inactive,
                    self.base_wk,
                    self.neutral_prim,
                ),
                hovered: self.make_widget_visual(
                    old.widgets.hovered,
                    self.cued_sec,
                    self.cued_prim,
                ),
                active: self.make_widget_visual(
                    old.widgets.active,
                    self.active_sec,
                    self.active_prim,
                ),
                open: self.make_widget_visual(old.widgets.open, self.base_ex, self.neutral_prim),
            },
            window_fill: self.base,
            ..old
        }
    }

    fn make_widget_visual(
        &self,
        old: WidgetVisuals,
        bg_fill: egui::Color32,
        fg: egui::Color32,
    ) -> WidgetVisuals {
        WidgetVisuals {
            bg_fill,
            weak_bg_fill: bg_fill,
            bg_stroke: egui::Stroke {
                color: self.neutral_prim,
                ..old.bg_stroke
            },
            fg_stroke: egui::Stroke {
                color: fg,
                ..old.fg_stroke
            },
            ..old
        }
    }
}

pub const DARK: Theme = Theme {
    base: Color32::from_gray(30),
    base_ex: Color32::from_gray(15),
    base_wk: Color32::from_gray(45),
    err_prim: Color32::RED,
    err_prim_wk: Color32::DARK_RED,
    err_sec: Color32::BLACK,
    warn_prim: Color32::YELLOW,
    warn_sec: Color32::BLACK,
    active_prim: Color32::GREEN,
    active_sec: Color32::BLACK,
    cued_prim: Color32::DARK_GREEN,
    cued_sec: Color32::GRAY,
    neutral_prim: Color32::from_gray(170),
    neutral_sec: Color32::from_gray(30),

    handle_shape: HandleShape::Circle,
    corner_radius: 2.0,

    is_dark_mode: true,
};

pub const LIGHT: Theme = Theme {
    base: Color32::from_gray(230),
    base_ex: Color32::from_gray(255),
    base_wk: Color32::from_gray(200),
    err_prim: Color32::RED,
    err_prim_wk: Color32::DARK_RED,
    err_sec: Color32::BLACK,
    warn_prim: Color32::from_rgb(204, 102, 0),
    warn_sec: Color32::BLACK,
    active_prim: Color32::from_rgb(0, 204, 0),
    active_sec: Color32::from_gray(230),
    cued_prim: Color32::DARK_GREEN,
    cued_sec: Color32::from_gray(230),
    neutral_prim: Color32::BLACK,
    neutral_sec: Color32::WHITE,

    handle_shape: HandleShape::Circle,
    corner_radius: 5.0,

    is_dark_mode: false,
};

pub const BLACK: Theme = Theme {
    base: Color32::BLACK,
    base_ex: Color32::BLACK,
    base_wk: Color32::BLACK,
    err_prim: Color32::RED,
    err_prim_wk: Color32::DARK_RED,
    err_sec: Color32::BLACK,
    warn_prim: Color32::YELLOW,
    warn_sec: Color32::BLACK,
    active_prim: Color32::GREEN,
    active_sec: Color32::BLACK,
    cued_prim: Color32::DARK_GREEN,
    cued_sec: Color32::BLACK,
    neutral_prim: Color32::WHITE,
    neutral_sec: Color32::BLACK,

    handle_shape: HandleShape::Rect { aspect_ratio: 0.5 },
    corner_radius: 5.0,

    is_dark_mode: false,
};

pub const BLACK_MONOCHROME: Theme = Theme {
    base: Color32::BLACK,
    base_ex: Color32::BLACK,
    base_wk: Color32::BLACK,
    err_prim: Color32::WHITE,
    err_prim_wk: Color32::BLACK,
    err_sec: Color32::BLACK,
    warn_prim: Color32::WHITE,
    warn_sec: Color32::BLACK,
    active_prim: Color32::WHITE,
    active_sec: Color32::BLACK,
    cued_prim: Color32::GRAY,
    cued_sec: Color32::BLACK,
    neutral_prim: Color32::WHITE,
    neutral_sec: Color32::BLACK,

    handle_shape: HandleShape::Rect { aspect_ratio: 0.5 },
    corner_radius: 5.0,

    is_dark_mode: false,
};

pub const NATIVE: Theme = Theme {
    err_prim: Color32::RED,
    warn_prim: Color32::YELLOW,
    active_prim: Color32::GREEN,
    cued_prim: Color32::DARK_GREEN,
    base: Color32::DEBUG_COLOR,
    ..DARK
};
