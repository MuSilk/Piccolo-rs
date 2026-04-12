use runtime_derive::ComponentTrait;

use crate::{function::{framework::{object::object_id_allocator::GObjectID, resource::component::mesh::MeshComponentRes}, render::render_object::{GameObjectLazyMeshDesc, GameObjectMeshDesc, GameObjectPartDesc}}, resource::{asset_manager::AssetManager, config_manager::ConfigManager, res_type::data::material::MaterialRes}};

#[derive(Clone, Default, ComponentTrait)]
pub struct MeshComponent {
    pub m_parent_object : GObjectID,
    pub m_raw_meshes: Vec<GameObjectPartDesc>,
}

impl MeshComponent {
    pub fn post_load_resource(
        &mut self, 
        parent_object: GObjectID,
        asset_manager: &AssetManager,
        config_manager: &ConfigManager,
        mesh_res: &MeshComponentRes,
    ) { 
        self.m_parent_object = parent_object;

        self.m_raw_meshes.resize(mesh_res.m_sub_meshs.len(), GameObjectPartDesc::default());
        for (raw_mesh_index, sub_mesh) in mesh_res.m_sub_meshs.iter().enumerate() {
            let mesh_component = &mut self.m_raw_meshes[raw_mesh_index];
            if let Some(mesh_file) = &sub_mesh.m_obj_file_ref {
                mesh_component.m_mesh_desc = GameObjectMeshDesc::LazyMesh(
                    GameObjectLazyMeshDesc::new(asset_manager.get_full_path(config_manager, mesh_file).to_str().unwrap().to_string())
                );
            }
            else{
                mesh_component.m_mesh_desc = GameObjectMeshDesc::DynamicMesh(Default::default());
            }
            mesh_component.m_material_desc.m_with_texture = !sub_mesh.m_material.is_empty();
            if mesh_component.m_material_desc.m_with_texture {
                let material_res: MaterialRes = asset_manager.load_asset(config_manager, &sub_mesh.m_material).unwrap();
                mesh_component.m_material_desc.m_base_color_texture_file =
                    get_full_path(
                        asset_manager,
                        config_manager,
                        &material_res.m_base_colour_texture_file
                    );
                mesh_component.m_material_desc.m_metallic_roughness_texture_file =
                    get_full_path(
                        asset_manager,
                        config_manager,
                        &material_res.m_metallic_roughness_texture_file
                    );
                mesh_component.m_material_desc.m_normal_texture_file =
                    get_full_path(
                        asset_manager,
                        config_manager,
                        &material_res.m_normal_texture_file
                    );
                mesh_component.m_material_desc.m_occlusion_texture_file =
                    get_full_path(
                        asset_manager,
                        config_manager,
                        &material_res.m_occlusion_texture_file
                    );
                mesh_component.m_material_desc.m_emissive_texture_file =
                    get_full_path(
                        asset_manager,
                        config_manager,
                        &material_res.m_emissive_texture_file
                    );
            }
            mesh_component.m_transform_desc.m_transform_matrix = sub_mesh.m_transform.get_matrix();
        }
    }
}

fn get_full_path(asset_manager: &AssetManager, config_manager: &ConfigManager, path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    asset_manager.get_full_path(config_manager, path).to_str().unwrap().to_string()
}