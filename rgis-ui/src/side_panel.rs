use bevy_egui::egui;
#[cfg(not(target_arch = "wasm32"))]
use geo::algorithm::transform::Transform;

const MAX_SIDE_PANEL_WIDTH: f32 = 200.0f32;

pub(crate) struct SidePanel<'a> {
    pub egui_ctx: &'a egui::CtxRef,
    pub state: &'a mut crate::UiState,
    pub rgis_layers_resource: &'a rgis_layers::ArcLayers,
    pub toggle_events: &'a mut bevy::app::Events<rgis_events::ToggleLayerVisibilityEvent>,
    pub toggle_material_events: &'a mut bevy::app::Events<rgis_events::ToggleMaterialEvent>,
    pub center_layer_events: &'a mut bevy::app::Events<rgis_events::CenterCameraEvent>,
    pub thread_pool: &'a bevy::tasks::AsyncComputeTaskPool,
    pub opened_file_bytes_sender: &'a crate::OpenedFileBytesSender,
    pub mouse_pos: &'a rgis_mouse::MousePos,
}

impl<'a> SidePanel<'a> {
    pub fn render(&mut self) {
        egui::SidePanel::left("left-side-panel")
            .max_width(MAX_SIDE_PANEL_WIDTH)
            .show(self.egui_ctx, |ui| {
                self.render_mouse_position_window(ui);
                self.render_layers_window(ui);
            });
    }

    fn render_mouse_position_window(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("🖱 Mouse Position", |ui| {
            ui.label(format!("CRS: {}", self.state.target_srs));
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(format!("X: {}", self.mouse_pos.projected.x));
                ui.label(format!("Y: {}", self.mouse_pos.projected.y));
            });
        });
    }

    fn render_layers_window(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("🗺 Layers", |ui| {
            if ui.button("Add GeoJSON Layer").clicked() {
                let hi = self.opened_file_bytes_sender.clone();
                self.thread_pool
                    .spawn(async move {
                        let task = rfd::AsyncFileDialog::new().pick_file();
                        let file_handle = task.await;
                        if let Some(n) = file_handle {
                            hi.send((n.file_name(), n.read().await)).await.unwrap();
                        }
                    })
                    .detach();
            }

            let rgis_layers_resource = match self.rgis_layers_resource.read() {
                Ok(r) => r,
                Err(_) => {
                    // TODO log failure
                    return;
                }
            };
            for layer in &rgis_layers_resource.data {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    egui::CollapsingHeader::new(layer.name.to_owned())
                        .id_source(layer.id) // Instead of using the layer name as the ID (which is not unique), use the layer ID
                        .show(ui, |ui| {
                            if ui.button("✏ Manage").clicked() {
                                self.state.layer_window_visible = true;
                                self.state.managing_layer = Some(layer.id);
                            }

                            if layer.visible {
                                if ui.button("👁 Hide").clicked() {
                                    self.toggle_events
                                        .send(rgis_events::ToggleLayerVisibilityEvent(layer.id));
                                    self.toggle_material_events
                                        .send(rgis_events::ToggleMaterialEvent::Hide(layer.id));
                                }
                            } else if ui.button("👁 Show").clicked() {
                                self.toggle_events
                                    .send(rgis_events::ToggleLayerVisibilityEvent(layer.id));
                                self.toggle_material_events
                                    .send(rgis_events::ToggleMaterialEvent::Show(layer.id));
                            }

                            if ui.button("🔎 Zoom to extent").clicked() {
                                self.center_layer_events
                                    .send(rgis_events::CenterCameraEvent(layer.id))
                            }

                            // if ui.button("❌ Remove").clicked() {
                            // }
                        });
                });
            }
        });
    }
}
