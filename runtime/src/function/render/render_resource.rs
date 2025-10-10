use crate::function::render::{render_camera::RenderCamera, render_common::MeshPreframeStorageBufferObject, render_resource_base::RenderResourceBase, render_scene::RenderScene};


#[derive(Default)]
pub struct RenderResource{
    m_base:RenderResourceBase,
    pub m_mesh_perframe_storage_buffer_object: MeshPreframeStorageBufferObject,
}

impl RenderResource {
    pub fn update_per_frame_buffer(&mut self,render_scene: &RenderScene, camera: &RenderCamera){
        let view_matrix = camera.get_view_matrix();
        let proj_matrix = camera.get_pers_proj_matrix();
        let camera_position = camera.position();
        let proj_view_matrix = proj_matrix * view_matrix;

        self.m_mesh_perframe_storage_buffer_object.proj_view_matrix = proj_view_matrix;
        self.m_mesh_perframe_storage_buffer_object.camera_position = *camera_position;
        
    }
}