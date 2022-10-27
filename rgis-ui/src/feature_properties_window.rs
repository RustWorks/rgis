use bevy_egui::egui;

pub(crate) struct FeaturePropertiesWindow<'a> {
    pub bevy_egui_ctx: &'a mut bevy_egui::EguiContext,
    pub state: &'a mut crate::FeaturePropertiesWindowState,
}

impl<'a> FeaturePropertiesWindow<'a> {
    pub(crate) fn render(&mut self) {
        if let Some(ref properties) = self.state.properties {
            egui::Window::new("Layer Feature Properties")
                .id(egui::Id::new("Layer Feature Properties Window"))
                .open(&mut self.state.is_visible)
                .anchor(egui::Align2::LEFT_TOP, [5., 5.])
                .show(self.bevy_egui_ctx.ctx_mut(), |ui| {
                    egui::Grid::new("feature_properties_window_grid")
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            let mut sorted = properties.iter().collect::<Vec<_>>();
                            sorted.sort_unstable_by_key(|n| n.0);
                            for (k, v) in sorted.iter() {
                                ui.label(*k);
                                ui.label(format!("{:?}", v));
                                ui.end_row();
                            }
                        });
                });
        }
    }
}
