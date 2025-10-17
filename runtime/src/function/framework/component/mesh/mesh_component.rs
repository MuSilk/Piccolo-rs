use std::{cell::RefCell, rc::Rc};

use reflection::reflection_derive::ReflectWhiteListFields;

use crate::{function::{framework::{component::{component::{Component, ComponentTrait}, transform::transform_component::TransformComponent}, level::level::Level, object::object_id_allocator::GObjectID}, global::global_context::RuntimeGlobalContext, render::render_object::{GameObjectDesc, GameObjectPartDesc}}, resource::res_type::components::mesh::MeshComponentRes};



#[derive(Clone, Default, ReflectWhiteListFields)]
pub struct MeshComponent {
    pub m_component: Component,
    #[meta]
    pub m_mesh_res: MeshComponentRes,
    m_raw_meshes: Vec<GameObjectPartDesc>,
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
            let global = RuntimeGlobalContext::global().borrow();
            let asset_manager = global.m_asset_manager.borrow();
            mesh_component.m_mesh_desc.m_mesh_file = asset_manager.get_full_path(&sub_mesh.m_obj_file_ref).to_str().unwrap().to_string();
            mesh_component.m_material_desc.m_with_texture = !sub_mesh.m_material.is_empty();
            if mesh_component.m_material_desc.m_with_texture {
                //todo: load material
            }
            mesh_component.m_transform_desc.m_transform_matrix = sub_mesh.m_transform.get_matrix();
        }
    }
    
    fn tick(&mut self, _delta_time: f32) {
        let parent_level = self.m_component.m_parent_level.upgrade().unwrap();
        let parent_level = parent_level.borrow();
        let mut transform_component = parent_level.get_component_mut::<TransformComponent>(self.m_component.m_parent_object).unwrap();
        // if transform_component.is_dirty() {
            let mut dirty_mesh_parts = vec![];
            for mesh_part in &mut self.m_raw_meshes {
                let object_transform_matrix = mesh_part.m_transform_desc.m_transform_matrix;

                mesh_part.m_transform_desc.m_transform_matrix = transform_component.get_matrix() * object_transform_matrix;
                dirty_mesh_parts.push(mesh_part.clone());

                mesh_part.m_transform_desc.m_transform_matrix = object_transform_matrix;
            }

            let global = RuntimeGlobalContext::global().borrow();
            let render_system = global.m_render_system.borrow();
            let render_swap_context = render_system.get_swap_context();
            let logic_swap_data = render_swap_context.get_logic_swap_data();
            transform_component.set_dirty_flag(false);
            logic_swap_data.borrow_mut().add_dirty_game_object(&GameObjectDesc::new(self.m_component.m_parent_object, dirty_mesh_parts));
        // }
    }
}

impl MeshComponent {
    
}