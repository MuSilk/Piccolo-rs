use std::sync::Arc;

use crate::runtime::function::render::{render_common::MeshPreframeStorageBufferObject, render_resource_base::{RenderResourceBase, RenderResourceBaseTrait}};

pub struct GlobalRenderResource{

}

pub struct RenderResource{
   pub m_render_resource_base: RenderResourceBase,
   pub m_global_render_resource: Arc<GlobalRenderResource>,
   pub m_mesh_preframe_storage_buffer_object: MeshPreframeStorageBufferObject
}

impl RenderResourceBaseTrait for RenderResource {
    
}