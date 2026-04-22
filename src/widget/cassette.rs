use egui::{Color32, Frame, InnerResponse, Margin, Stroke, Ui};

#[derive()]
pub struct Cassette {
    border: i8,
    fill: Color32,
    stroke: Stroke,
    inner_width: f32,
}

impl Default for Cassette {
    fn default() -> Self {
        Self {
            border: 8,
            fill: Color32::default(),
            stroke: Stroke::default(),
            inner_width: 32.0,
        }
    }
}

impl Cassette {
    fn show<R, U, L>(
        self,
        ui: &mut Ui,
        upper: Option<U>,
        add_contents: impl FnOnce(&mut Ui) -> R,
        lower: Option<L>,
    ) -> InnerResponse<R>
    where
        U: FnOnce(&mut Ui),
        L: FnOnce(&mut Ui),
    {
        Frame::new()
            .stroke(self.stroke)
            .fill(self.fill)
            .inner_margin(Margin::same(self.border))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.set_width(self.inner_width);
                    if let Some(func) = upper {
                        ui.horizontal(|ui| {
                            (func)(ui);
                        });
                    }
                    let r = add_contents(ui);
                    if let Some(func) = lower {
                        ui.horizontal(|ui| {
                            (func)(ui);
                        });
                    }
                    r
                })
                .inner
            })
    }

    pub fn show_simple<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.show(ui, None::<fn(&mut Ui)>, add_contents, None::<fn(&mut Ui)>)
    }

    pub fn show_with_lower<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
        lower_contents: impl FnOnce(&mut Ui),
    ) -> InnerResponse<R> {
        self.show(ui, None::<fn(&mut Ui)>, add_contents, Some(lower_contents))
    }

    pub fn show_with_upper<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
        upper_contents: impl FnOnce(&mut Ui),
    ) -> InnerResponse<R> {
        self.show(ui, Some(upper_contents), add_contents, None::<fn(&mut Ui)>)
    }

    pub fn new() -> Self {
        Cassette::default()
    }

    pub fn border(mut self, border: i8) -> Self {
        self.border = border;
        self
    }
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = fill;
        self
    }
    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }

    pub fn get_inner_width(&self) -> f32 {
        self.inner_width
    }

    pub fn inner_width(mut self, width: f32) -> Self {
        self.inner_width = width;
        self
    }

    pub fn outer_width(mut self, width: f32) -> Self {
        self.inner_width = width - 2.0 * self.border as f32;
        self
    }
}

#[derive(Default)]
pub struct CassetteDeck {
    total_width: f32,
    wanted_child_width: f32,
}

impl CassetteDeck {
    pub fn fullwidth(ui: &Ui) -> Self {
        Self::new(ui.available_width())
    }

    pub fn new(total_width: f32) -> Self {
        Self {
            total_width,
            ..Default::default()
        }
    }

    pub fn child_width(mut self, width: f32) -> Self {
        self.wanted_child_width = width.max(1.0);
        self
    }

    pub fn child_actual_width(&self) -> f32 {
        self.total_width / (self.total_width / self.wanted_child_width).floor()
    }

    //pub fn show<R>(self, ui: &mut Ui, add_contents: &[Box<impl Fn(&mut Ui) -> R>]) {
    //    let num_wide = (self.total_width / self.wanted_child_width).floor() as usize;
    //    ui.vertical(|ui| {
    //        for y in 0..add_cassettes.len() / num_wide + 1 {
    //            ui.horizontal(|ui| {
    //                for x in 0..num_wide {
    //                    (add_cassettes[y * num_wide + x])(ui);
    //                }
    //            });
    //        }
    //    });
    //}
}
