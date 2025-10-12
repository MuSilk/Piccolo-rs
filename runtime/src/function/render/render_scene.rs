use std::{collections::HashMap};

use crate::function::{framework::object::object_id_allocator::GObjectID, render::{render_entity::RenderEntity, render_guid_allocator::GuidAllocator, render_object::GameObjectPartId, render_type::MeshSourceDesc}};



#[derive(Default)]
pub struct RenderScene{
    
    pub m_render_entities: Vec<RenderEntity>,

    m_instance_id_allocator: GuidAllocator<GameObjectPartId>,
    m_mesh_asset_id_allocator: GuidAllocator<MeshSourceDesc>,

    m_mesh_object_id_map: HashMap<u32, GObjectID>,
}

impl RenderScene {
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