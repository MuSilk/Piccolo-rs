use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::function::{framework::object::object_id_allocator::GObjectID, render::{render_camera::RenderCamera, render_common::RenderMeshNode, render_entity::RenderEntity, render_guid_allocator::GuidAllocator, render_object::GameObjectPartId, render_pass::RenderPass, render_resource::RenderResource, render_type::MeshSourceDesc}};



#[derive(Default)]
pub struct RenderScene{
    
    pub m_render_entities: Vec<RenderEntity>,

    m_main_camera_visible_mesh_nodes: Rc<RefCell<Vec<RenderMeshNode>>>,

    m_instance_id_allocator: GuidAllocator<GameObjectPartId>,
    m_mesh_asset_id_allocator: GuidAllocator<MeshSourceDesc>,

    m_mesh_object_id_map: HashMap<u32, GObjectID>,
}

impl RenderScene {

    pub fn update_visible_objects(&mut self, render_resource: &RenderResource, camera: &RenderCamera) {
        self.update_visible_objects_main_camera(render_resource, camera);
    }

    pub fn set_visible_nodes_reference(&self) {
        RenderPass::m_visiable_nodes().borrow_mut().p_main_camera_visible_mesh_nodes = 
            Rc::downgrade(&self.m_main_camera_visible_mesh_nodes);
    }
    pub fn get_instance_id_allocator(&mut self) -> &mut GuidAllocator<GameObjectPartId> {
        &mut self.m_instance_id_allocator
    }

    pub fn get_mesh_asset_id_allocator(&mut self) -> &mut GuidAllocator<MeshSourceDesc> {
        &mut self.m_mesh_asset_id_allocator
    }

    pub fn add_instance_id_to_map(&mut self, instance_id: u32, go_id: GObjectID) {
        self.m_mesh_object_id_map.insert(instance_id, go_id);
    }
}

impl RenderScene {
    fn update_visible_objects_main_camera(&mut self, render_resource: &RenderResource, _camera: &RenderCamera) {
        self.m_main_camera_visible_mesh_nodes.borrow_mut().clear();

        for entity in &self.m_render_entities {
            let mut temp_node = RenderMeshNode::default();
            temp_node.model_matrix = entity.m_model_matrix.clone();
            temp_node.node_id = entity.m_instance_id;

            let mesh_asset = render_resource.get_entity_mesh(entity);
            temp_node.ref_mesh = Rc::downgrade(mesh_asset);
            temp_node.enable_vertex_blending = entity.m_enable_vertex_blending;

            self.m_main_camera_visible_mesh_nodes.borrow_mut().push(temp_node);
        }
    }
}   