use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{function::{framework::{component::component::{Component, ComponentTrait}, level::level::Level, object::object_id_allocator::GObjectID}, global::global_context::RuntimeGlobalContext, render::render_object::GameObjectPartDesc}, resource::res_type::{components::mesh::MeshComponentRes, data::material::MaterialRes}};


#[derive(Clone, Default, Serialize, Deserialize)]
pub struct MeshComponent {
    #[serde(skip)]
    pub m_component: Component,
    pub m_mesh_res: MeshComponentRes,
    #[serde(skip)]
    pub m_raw_meshes: Vec<GameObjectPartDesc>,
}

#[typetag::serde]
impl ComponentTrait for MeshComponent {
    fn get_component(&self) -> &Component {
        &self.m_component
    }
    
    fn get_component_mut(&mut self) -> &mut Component {
        &mut self.m_component
    }
    
    fn post_load_resource(&mut self, _parent_level: &Rc<RefCell<Level>>, parent_object: GObjectID) {
        self.get_component_mut().m_parent_object = parent_object;

        self.m_raw_meshes.resize(self.m_mesh_res.m_sub_meshs.len(), GameObjectPartDesc::default());
        for (raw_mesh_index, sub_mesh) in self.m_mesh_res.m_sub_meshs.iter().enumerate() {
            let mesh_component = &mut self.m_raw_meshes[raw_mesh_index];
            let asset_manager = RuntimeGlobalContext::get_asset_manager().borrow();
            mesh_component.m_mesh_desc.m_mesh_file = asset_manager.get_full_path(&sub_mesh.m_obj_file_ref).to_str().unwrap().to_string();
            mesh_component.m_material_desc.m_with_texture = !sub_mesh.m_material.is_empty();
            if mesh_component.m_material_desc.m_with_texture {
                let material_res: MaterialRes = asset_manager.load_asset(&sub_mesh.m_material).unwrap();
                mesh_component.m_material_desc.m_base_color_texture_file =
                    get_full_path(&material_res.m_base_colour_texture_file);
                mesh_component.m_material_desc.m_metallic_roughness_texture_file =
                    get_full_path(&material_res.m_metallic_roughness_texture_file);
                mesh_component.m_material_desc.m_normal_texture_file =
                    get_full_path(&material_res.m_normal_texture_file);
                mesh_component.m_material_desc.m_occlusion_texture_file =
                    get_full_path(&material_res.m_occlusion_texture_file);
                mesh_component.m_material_desc.m_emissive_texture_file =
                    get_full_path(&material_res.m_emissive_texture_file);
            }
            mesh_component.m_transform_desc.m_transform_matrix = sub_mesh.m_transform.get_matrix();
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    
    fn clone_box(&self) -> Box<dyn ComponentTrait> {
        Box::new(self.clone())
    }
}

impl MeshComponent {
    
}

fn get_full_path(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    let asset_manager = RuntimeGlobalContext::get_asset_manager().borrow();
    asset_manager.get_full_path(path).to_str().unwrap().to_string()
}