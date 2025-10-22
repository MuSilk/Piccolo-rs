use std::{cell::RefCell, rc::Rc};

use crate::{function::{framework::{component::{component::{Component, ComponentTrait}, transform::transform_component::TransformComponent}, level::level::Level, object::object_id_allocator::GObjectID}, global::global_context::RuntimeGlobalContext, render::{render_object::{GameObjectDesc, GameObjectPartDesc}}}, resource::res_type::components::mesh::MeshComponentRes};



#[derive(Clone, Default)]
pub struct MeshComponent {
    pub m_component: Component,
    pub m_mesh_res: MeshComponentRes,
    pub m_raw_meshes: Vec<GameObjectPartDesc>,
}

impl ComponentTrait for MeshComponent {
    fn get_component(&self) -> &Component {
        &self.m_component
    }
    
    fn get_component_mut(&mut self) -> &mut Component {
        &mut self.m_component
    }
    
    fn post_load_resource(&mut self, parent_level: &Rc<RefCell<Level>>, parent_object: GObjectID) {
        self.get_component_mut().m_parent_level = Rc::downgrade(parent_level);
        self.get_component_mut().m_parent_object = parent_object;

        self.m_raw_meshes.resize(self.m_mesh_res.m_sub_meshs.len(), GameObjectPartDesc::default());
        for (raw_mesh_index, sub_mesh) in self.m_mesh_res.m_sub_meshs.iter().enumerate() {
            let mesh_component = &mut self.m_raw_meshes[raw_mesh_index];
            let asset_manager = RuntimeGlobalContext::get_asset_manager().borrow();
            mesh_component.m_mesh_desc.m_mesh_file = asset_manager.get_full_path(&sub_mesh.m_obj_file_ref).to_str().unwrap().to_string();
            mesh_component.m_material_desc.m_with_texture = !sub_mesh.m_material.is_empty();
            if mesh_component.m_material_desc.m_with_texture {
                //todo: load material
            }
            mesh_component.m_transform_desc.m_transform_matrix = sub_mesh.m_transform.get_matrix();
        }
    }
}

impl MeshComponent {
    
}