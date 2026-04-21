use std::any::Any;

use crate::{
    engine::Engine,
    function::{
        framework::{
            component::{component::ComponentTrait, transform_component::TransformComponent},
            object::object::GObject,
            resource::component::mesh::MeshComponentRes,
        },
        render::render_object::{
            GameObjectDesc, GameObjectLazyMeshDesc, GameObjectMeshDesc, GameObjectPartDesc,
        },
    },
    resource::{asset_manager::AssetManager, res_type::data::material::MaterialRes},
};

#[derive(Clone, Default)]
pub struct MeshComponent {
    pub m_raw_meshes: Vec<GameObjectPartDesc>,
}

impl MeshComponent {
    pub fn post_load_resource(
        &mut self,
        asset_manager: &AssetManager,
        mesh_res: &MeshComponentRes,
    ) {
        self.m_raw_meshes
            .resize(mesh_res.m_sub_meshs.len(), GameObjectPartDesc::default());
        for (raw_mesh_index, sub_mesh) in mesh_res.m_sub_meshs.iter().enumerate() {
            let mesh_component = &mut self.m_raw_meshes[raw_mesh_index];
            if let Some(mesh_file) = &sub_mesh.m_obj_file_ref {
                mesh_component.m_mesh_desc =
                    GameObjectMeshDesc::LazyMesh(GameObjectLazyMeshDesc::new(
                        asset_manager
                            .get_full_path(mesh_file)
                            .to_str()
                            .unwrap()
                            .to_string(),
                    ));
            } else {
                mesh_component.m_mesh_desc = GameObjectMeshDesc::DynamicMesh(Default::default());
            }
            mesh_component.m_material_desc.m_with_texture = !sub_mesh.m_material.is_empty();
            if mesh_component.m_material_desc.m_with_texture {
                let material_res: MaterialRes =
                    asset_manager.load_asset(&sub_mesh.m_material).unwrap();
                mesh_component.m_material_desc.m_base_color_texture_file = asset_manager
                    .get_full_path(&material_res.m_base_colour_texture_file)
                    .to_str()
                    .unwrap()
                    .to_string();
                mesh_component
                    .m_material_desc
                    .m_metallic_roughness_texture_file = asset_manager
                    .get_full_path(&material_res.m_metallic_roughness_texture_file)
                    .to_str()
                    .unwrap()
                    .to_string();
                mesh_component.m_material_desc.m_normal_texture_file = asset_manager
                    .get_full_path(&material_res.m_normal_texture_file)
                    .to_str()
                    .unwrap()
                    .to_string();
                mesh_component.m_material_desc.m_occlusion_texture_file = asset_manager
                    .get_full_path(&material_res.m_occlusion_texture_file)
                    .to_str()
                    .unwrap()
                    .to_string();
                mesh_component.m_material_desc.m_emissive_texture_file = asset_manager
                    .get_full_path(&material_res.m_emissive_texture_file)
                    .to_str()
                    .unwrap()
                    .to_string();
            }
            mesh_component.m_transform_desc.m_transform_matrix = sub_mesh.m_transform.get_matrix();
        }
    }
}

impl ComponentTrait for MeshComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn tick(&mut self, engine: &Engine, gobject: &GObject, _delta_time: f32) {
        let mut transform = gobject.get_component_mut::<TransformComponent>().unwrap();
        let dynamic_mesh_dirty = self.m_raw_meshes.iter().any(|part| {
            matches!(
                &part.m_mesh_desc,
                GameObjectMeshDesc::DynamicMesh(d) if d.borrow().m_is_dirty
            )
        });
        if transform.is_dirty() || dynamic_mesh_dirty {
            let mut dirty_mesh_parts = vec![];
            for mesh_part in &mut self.m_raw_meshes {
                let object_transform_matrix = mesh_part.m_transform_desc.m_transform_matrix;

                mesh_part.m_transform_desc.m_transform_matrix =
                    transform.get_matrix() * object_transform_matrix;
                dirty_mesh_parts.push(mesh_part.clone());

                mesh_part.m_transform_desc.m_transform_matrix = object_transform_matrix;
            }

            transform.set_dirty_flag(false);
            engine
                .render_system()
                .borrow()
                .get_logic_swap_data()
                .borrow_mut()
                .add_dirty_game_object(&GameObjectDesc::new(gobject.get_id(), dirty_mesh_parts));
        }
    }
}
