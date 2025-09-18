use std::{cell::RefCell, collections::HashMap, fs::File, io::BufReader, rc::Rc};
use anyhow::{Result};
use nalgebra_glm::{Vec2, Vec3};
use vulkanalia::{Instance, prelude::v1_0::*};

use crate::{surface::TexturedMeshVertex, vulkan::{resource_manager::ResourceManager, Destroy, VertexObject, VulkanData}};

#[derive(Default,Clone,Debug)]
pub struct Mesh<T:Default> {
    pub vertex_object: VertexObject,
    pub vertices :Vec<T>,
    pub indices :Vec<u32>
}

impl Mesh<TexturedMeshVertex> {
    pub fn build(&mut self,instance: &Instance, device: &Device, data: &VulkanData) -> Result<()> {
        self.vertex_object = VertexObject::new(instance, device, data, &self.vertices, &self.indices)?;
        Ok(())
    }
    pub fn destroy(&mut self, device: &Device) {
        self.vertex_object.destroy(device);
    }
    pub fn eval_model(filepath: &str) -> Result<Mesh<TexturedMeshVertex>>{
        let mut reader = BufReader::new(File::open(filepath)?);
        let (models, _) = tobj::load_obj_buf(
            &mut reader, 
            &tobj::LoadOptions {triangulate:true, ..Default::default()}, 
            |_| Ok(Default::default()),
        )?;

        let mut unique_vertices = HashMap::new();

        let mut mesh = Mesh::<TexturedMeshVertex>::default();

    for model in &models {
        for index in &model.mesh.indices {

            let pos_offset = (index * 3) as usize;
            let tex_coord_offset = (index * 2) as usize;

            let vertex = TexturedMeshVertex {
                pos: Vec3::new(
                    model.mesh.positions[pos_offset],
                    model.mesh.positions[pos_offset + 1],
                    model.mesh.positions[pos_offset + 2],
                ),
                color: Vec3::new(1.0, 1.0, 1.0),
                tex_coord: Vec2::new(
                    model.mesh.texcoords[tex_coord_offset],
                    1.0 - model.mesh.texcoords[tex_coord_offset + 1],
                )
            };

            if let Some(index) = unique_vertices.get(&vertex) {
                mesh.indices.push(*index as u32);
            }else{
                let index = mesh.vertices.len();
                unique_vertices.insert(vertex, index);
                mesh.vertices.push(vertex);
                mesh.indices.push(index as u32);
            }
        }
    }

        Ok(mesh)
    }
}

pub type MeshManager = ResourceManager<Rc<RefCell<Mesh<TexturedMeshVertex>>>>;

impl Destroy for MeshManager {
    fn destroy(&mut self, device: &Device) {
        for mesh in self.values_mut() {
            mesh.borrow_mut().destroy(device);
        }
        self.clear();
    }
}