use std::f32::consts::PI;

use crate::{core::math::vector3::Vector3, function::render::render_type::BufferData};


pub struct PointLight {
    pub m_position: Vector3,
    pub m_flux: Vector3,
}

impl PointLight {
    pub fn new(position: Vector3, flux: Vector3) -> Self {
        Self {
            m_position: position,
            m_flux: flux,
        }
    }

    pub fn calculate_radius(&self) -> f32 {
        const INTENSITY_CUTOFF: f32 = 1.0;
        const ATTENTUATION_CUTOFF: f32 = 0.05;
        let intensity = self.m_flux / (4.0 * PI);
        let max_intensity = intensity.x.max(intensity.y).max(intensity.z);
        let attenuation = INTENSITY_CUTOFF.max(ATTENTUATION_CUTOFF * max_intensity) / max_intensity;
        1.0 / attenuation.sqrt()
    } 
}

#[derive(Default)]
pub struct AmbientLight {
    pub m_irradiance: Vector3,
}

#[derive(Default)]
pub struct DirectionalLight {
    pub m_direction: Vector3,
    pub m_color: Vector3,
}

pub struct PointLightVertex {
    pub m_position: Vector3,
    _padding: f32,
    pub m_intensity: Vector3,
    pub m_radius: f32,
}

#[derive(Default)]
pub struct PointLightList {
    pub m_lights: Vec<PointLight>,
    pub m_buffer: BufferData,
}