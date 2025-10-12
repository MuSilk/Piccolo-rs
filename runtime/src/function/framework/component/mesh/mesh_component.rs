use reflection::reflection_derive::ReflectWhiteListFields;

use crate::{function::{framework::component::{component::{Component, ComponentTrait}, transform::transform_component::TransformComponent}, global::global_context::RuntimeGlobalContext, render::{render_object::{GameObjectDesc, GameObjectPartDesc}}}, resource::res_type::components::mesh::MeshComponentRes};



#[derive(ReflectWhiteListFields)]
pub struct MeshComponent {
    m_component: Component,
    #[meta]
    m_mesh_res: MeshComponentRes,
    m_raw_meshes: Vec<GameObjectPartDesc>,
}

impl ComponentTrait for MeshComponent {
    fn get_component(&self) -> &Component {
        &self.m_component
    }
    fn get_component_mut(&mut self) -> &mut Component {
        &mut self.m_component
    }
    fn tick(&mut self, _delta_time: f32) {
        if self.m_component.m_parent_object.upgrade().is_none() {
            return;
        }
        let parent_object = self.m_component.m_parent_object.upgrade().unwrap();
        let transform_component = parent_object.try_get_component::<TransformComponent>("TransformComponent").unwrap();
        if transform_component.is_dirty() {
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
            logic_swap_data.borrow_mut().add_dirty_game_object(&GameObjectDesc::new(parent_object.get_id(), dirty_mesh_parts));
            transform_component.set_dirty_flag(false);
        }
    }
}

impl MeshComponent {
    
}