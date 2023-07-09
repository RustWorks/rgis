pub struct MeshBuildingJob {
    pub layer_id: rgis_layer_id::LayerId,
    pub color: bevy::render::color::Color,
    pub geometry: geo_projected::Projected<geo::Geometry>,
    pub is_selected: bool,
}

pub struct MeshBuildingJobOutcome {
    pub geometry_mesh: geo_bevy::GeometryMesh,
    pub layer_id: rgis_layer_id::LayerId,
    pub is_selected: bool,
}

impl bevy_jobs::Job for MeshBuildingJob {
    type Outcome = Option<MeshBuildingJobOutcome>;

    fn name(&self) -> String {
        "Building Bevy meshes".to_string()
    }

    fn perform(self, _: bevy_jobs::Context) -> bevy_jobs::AsyncReturn<Self::Outcome> {
        let Some(geometry_mesh) = geo_bevy::geometry_to_mesh(self.geometry.as_raw()) else {
            return Box::pin(async move { None });
        };
        Box::pin(async move {
            Some(MeshBuildingJobOutcome {
                geometry_mesh,
                layer_id: self.layer_id,
                is_selected: self.is_selected,
            })
        })
    }
}
