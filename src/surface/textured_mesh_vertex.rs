use std::hash::{Hash, Hasher};

use nalgebra_glm::{Vec2, Vec3};
use vulkanalia::{prelude::v1_0::*};

use crate::{surface::VertexLayout};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TexturedMeshVertex{
    pub pos: Vec3,
    pub color: Vec3,
    pub tex_coord: Vec2,
}

impl Default for TexturedMeshVertex {
    fn default() -> Self {
        Self { 
            pos: Vec3::new(0.0,0.0,0.0), 
            color: Vec3::new(0.0,0.0,0.0), 
            tex_coord: Vec2::new(0.0,0.0) 
        }
    }
}

impl VertexLayout for TexturedMeshVertex {
    fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<TexturedMeshVertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0)
            .build();

        let color = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(std::mem::size_of::<Vec3>() as u32)
            .build();

        let tex_coord = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(2)
            .format(vk::Format::R32G32_SFLOAT)
            .offset((std::mem::size_of::<Vec3>() + std::mem::size_of::<Vec3>()) as u32)
            .build();
        vec![pos, color, tex_coord]
    }

}


impl PartialEq for TexturedMeshVertex {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos && self.color == other.color && self.tex_coord == other.tex_coord
    }
}

impl Eq for TexturedMeshVertex {}

impl Hash for TexturedMeshVertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos[0].to_bits().hash(state);
        self.pos[1].to_bits().hash(state);
        self.pos[2].to_bits().hash(state);
        self.color[0].to_bits().hash(state);
        self.color[1].to_bits().hash(state);
        self.color[2].to_bits().hash(state);
        self.tex_coord[0].to_bits().hash(state);
        self.tex_coord[1].to_bits().hash(state);
    }
}
