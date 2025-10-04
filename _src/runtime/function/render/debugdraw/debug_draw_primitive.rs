use nalgebra_glm::{Quat, Vec2, Vec3, Vec4};

use crate::runtime::function::render::{interface::rhi_struct::{RHIVertexInputAttributeDescription, RHIVertexInputBindingDescription}, render_type::{RHIFormat, RHIVertexInputRate}};

const K_DEBUG_DRAW_INFINITY_LIFE_TIME: f32 = -2.0;
const K_DEBUG_DRAW_ONE_FRAME: f32 = 0.0;

#[derive(Clone)]
pub enum DebugDrawTimeType {
    Infinity,
    OneFrame,
    Common,
}

#[derive(Clone)]
enum _DebugDrawPrimitiveType {
    Point,
    Line,
    Triangle,
    Quad,
    DrawBox,
    Cylinder,
    Sphere,
    Capsule,
    Text,
    EnumCount,
}

#[derive(Clone, PartialEq)]
pub enum FillMode {
    WireFrame,
    Solid,
    EnumCount,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DebugDrawVertex {
    pub pos: Vec3,
    pub color: Vec4,
    pub texcoord: Vec2,
}

impl Default for DebugDrawVertex {
    fn default() -> Self {
        Self {
            pos: Vec3::default(),
            color: Vec4::default(),
            texcoord: Vec2::new(1.0, 1.0),
        }
    }
    
}

impl DebugDrawVertex {
    pub fn get_binding_descriptions() -> [RHIVertexInputBindingDescription; 1] {
        [RHIVertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<DebugDrawVertex>() as u32,
            input_rate: RHIVertexInputRate::VERTEX,
        }]
    }

    pub fn get_attribute_descriptions() -> [RHIVertexInputAttributeDescription; 3] {
        [
            RHIVertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: RHIFormat::R32G32B32_SFLOAT,
                offset: 0,
            },
            RHIVertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: RHIFormat::R32G32B32A32_SFLOAT,
                offset: std::mem::size_of::<Vec3>() as u32,
            },
            RHIVertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: RHIFormat::R32G32_SFLOAT,
                offset: (std::mem::size_of::<Vec3>() + std::mem::size_of::<Vec4>()) as u32,
            },
        ]
    }

}

#[derive(Clone)]
pub struct DebugDrawPrimitive {
    pub m_time_type: DebugDrawTimeType,
    pub m_life_time: f32,
    pub m_fill_mode: FillMode,
    pub m_no_depth_test: bool,
    pub m_rendered: bool,
}

impl Default for DebugDrawPrimitive {
    fn default() -> Self {
        Self {
            m_time_type: DebugDrawTimeType::Infinity,
            m_life_time: 0.0,
            m_fill_mode: FillMode::WireFrame,
            m_no_depth_test: false,
            m_rendered: false,
        }
    }
}

impl DebugDrawPrimitive {
    pub fn is_time_out(&mut self, delta_time: f32) -> bool {
        match self.m_time_type {
            DebugDrawTimeType::Infinity => false,
            DebugDrawTimeType::OneFrame => if !self.m_rendered {
                self.m_rendered = true;
                false
            } else {
                true
            },
            DebugDrawTimeType::Common => {
                self.m_life_time -= delta_time;
                self.m_life_time <= 0.0
            }
        }
    }

    pub fn set_time(&mut self,in_life_time: f32) {
        if (in_life_time - K_DEBUG_DRAW_INFINITY_LIFE_TIME).abs() < f32::EPSILON {
            self.m_time_type = DebugDrawTimeType::Infinity;
            self.m_life_time = 0.0;
        } else if (in_life_time - K_DEBUG_DRAW_ONE_FRAME).abs() < f32::EPSILON {
            self.m_time_type = DebugDrawTimeType::OneFrame;
            self.m_life_time = 0.03;
        } else {
            self.m_time_type = DebugDrawTimeType::Common;
            self.m_life_time = in_life_time;
        }
    }
}

#[derive(Clone, Default)]
pub struct DebugDrawPoint {
    pub m_base: DebugDrawPrimitive,
    pub m_vertex: DebugDrawVertex,
}

#[derive(Clone, Default)]
pub struct DebugDrawLine {
    pub m_base: DebugDrawPrimitive,
    pub m_vertex: [DebugDrawVertex; 2],
}

#[derive(Clone, Default)]
pub struct DebugDrawTriangle {
    pub m_base: DebugDrawPrimitive,
    pub m_vertex: [DebugDrawVertex; 3],
}

#[derive(Clone, Default)]
pub struct DebugDrawQuad {
    pub m_base: DebugDrawPrimitive,
    pub m_vertex: [DebugDrawVertex; 4],
}

#[derive(Clone, Default)]
pub struct DebugDrawBox { 
    pub m_base: DebugDrawPrimitive,
    pub m_center_point: Vec3,
    pub m_half_extent: Vec3,
    pub m_color: Vec4,
    pub m_rotate: Quat
}

#[derive(Clone, Default)]
pub struct DebugDrawCylinder { 
    pub m_base: DebugDrawPrimitive,
    pub m_center: Vec3,
    pub m_rotate: Quat,
    pub m_radius: f32,
    pub m_height: f32,
    pub m_color: Vec4,
}

#[derive(Clone, Default)]
pub struct DebugDrawSphere { 
    pub m_base: DebugDrawPrimitive,
    pub m_center: Vec3,
    pub m_radius: f32,
    pub m_color: Vec4,
}

#[derive(Clone, Default)]
pub struct DebugDrawCapsule { 
    pub m_base: DebugDrawPrimitive,
    pub m_center: Vec3,
    pub m_rotate: Quat,
    pub m_scale: Vec3,
    pub m_radius: f32,
    pub m_height: f32,
    pub m_color: Vec4,
}

#[derive(Clone, Default)]
pub struct DebugDrawText {
    pub m_base: DebugDrawPrimitive,
    pub m_content: String,
    pub m_color: Vec4,
    pub m_coordinate: Vec3,
    pub m_size: i32,
    pub m_is_screen_text: bool,
}
