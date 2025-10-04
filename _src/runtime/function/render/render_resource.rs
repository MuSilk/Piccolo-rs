use std::{cell::RefCell, rc::Rc};

use crate::runtime::function::render::{interface::rhi_struct::{RHIBuffer, RHIDeviceMemory, RHIImage, RHIImageView, RHISampler}, render_common::MeshPreframeStorageBufferObject, render_resource_base::{RenderResourceBase, RenderResourceBaseTrait}, render_type::RHIFormat};

struct IBLResource{
	_brdf_lut_texture_image : Box<dyn RHIImage>,
	_brdf_lut_texture_image_view : Box<dyn RHIImageView>,
	_brdf_lut_texture_sampler : Box<dyn RHISampler>,
	_brdf_lut_texture_image_allocation: Box<dyn RHIDeviceMemory>,

	_irradiance_texture_image: Box<dyn RHIImage>,
	_irradiance_texture_image_view: Box<dyn RHIImageView>,
	_irradiance_texture_sampler: Box<dyn RHISampler>,
	_irradiance_texture_image_allocation: Box<dyn RHIDeviceMemory>,

	_specular_texture_image: Box<dyn RHIImage>,
	_specular_texture_image_view: Box<dyn RHIImageView>,
	_specular_texture_sampler: Box<dyn RHISampler>,
	_specular_texture_image_allocation: Box<dyn RHIDeviceMemory>,
}

struct IBLResourceData{
	_brdf_lut_texture_image_pixels : Vec<u8>,
	_brdf_lut_texture_image_width: u32,
	_brdf_lut_texture_image_height: u32,
	_brdf_lut_texture_image_format: RHIFormat,

	_irradiance_texture_image_pixels: [Vec<u8>;6],
	_irradiance_texture_image_width: u32,
	_irradiance_texture_image_height: u32,
	_irradiance_texture_image_format: RHIFormat,

	_prefilter_texture_image_pixels: [Vec<u8>;6],
	_prefilter_texture_image_width: u32,
	_prefilter_texture_image_height: u32,
	_prefilter_texture_image_format: RHIFormat,
}

struct ColorGradingResource {
	_color_grading_lut_texture_image: Box<dyn RHIImage>,
	_color_grading_lut_texture_image_view: Box<dyn RHIImageView>,
	_color_grading_lut_texture_image_allocation: Box<dyn RHIDeviceMemory>,
}

struct ColorGradingResourceData {
	_color_grading_lut_texture_image_pixels: Vec<u8>,
	_color_grading_lut_texture_image_width: u32,
	_color_grading_lut_texture_image_height: u32,
	_color_grading_lut_texture_image_format: RHIFormat,
}

struct StorageBuffer {
	_min_uniform_buffer_offset_alignment : u32,
	_min_storage_buffer_offset_alignment : u32,
	_max_storage_buffer_range : u32,
	_non_coherent_atom_size : u32,

	_global_upload_ringbuffer: Option<Box<dyn RHIBuffer>>,
	_global_upload_ringbuffer_memory: Option<Box<dyn RHIDeviceMemory>>,
	_global_upload_ringbuffer_memory_pointer: *mut std::ffi::c_void,
	_global_upload_ringbuffers_begin: Vec<u32>,
	_global_upload_ringbuffers_end: Vec<u32>,
	_global_upload_ringbuffers_size: Vec<u32>,

	_global_null_descriptor_storage_buffer: Option<Box<dyn RHIBuffer>>,
	_global_null_descriptor_storage_buffer_memory: Option<Box<dyn RHIDeviceMemory>>,

	_axis_inefficient_storage_buffer: Option<Box<dyn RHIBuffer>>,
	_axis_inefficient_storage_buffer_memory: Option<Box<dyn RHIDeviceMemory>>,
	_axis_inefficient_storage_buffer_memory_pointer: *mut std::ffi::c_void,
}

impl Default for StorageBuffer {
	fn default() -> Self {
		StorageBuffer {
			_min_uniform_buffer_offset_alignment: 256,
			_min_storage_buffer_offset_alignment: 256,
			_max_storage_buffer_range: 1<<27,
			_non_coherent_atom_size: 256,

			_global_upload_ringbuffer: None,
			_global_upload_ringbuffer_memory: None,
			_global_upload_ringbuffer_memory_pointer: std::ptr::null_mut(),
			_global_upload_ringbuffers_begin: vec![],
			_global_upload_ringbuffers_end: vec![],
			_global_upload_ringbuffers_size: vec![],

			_global_null_descriptor_storage_buffer: None,
			_global_null_descriptor_storage_buffer_memory: None,


			_axis_inefficient_storage_buffer: None,
			_axis_inefficient_storage_buffer_memory: None,
			_axis_inefficient_storage_buffer_memory_pointer: std::ptr::null_mut(),
		}

	}
}

#[derive(Default)]
pub struct GlobalRenderResource{
	_ibl_resource: Option<IBLResource>,
	_color_grading_resource: Option<ColorGradingResource>,
	_storage_buffer: StorageBuffer,
}

#[derive(Default)]
pub struct RenderResource{
    // pub m_render_resource_base: RenderResourceBase,
    pub m_global_render_resource: Rc<RefCell<GlobalRenderResource>>,
    pub m_mesh_preframe_storage_buffer_object: MeshPreframeStorageBufferObject
}

impl RenderResourceBaseTrait for RenderResource {
    
}

impl RenderResource {
	pub fn reset_ring_buffer_offset(&mut self, _current_frame_index: usize){
		// let mut buffer = self.m_global_render_resource.borrow_mut();
		// buffer._storage_buffer._global_upload_ringbuffers_end[current_frame_index] =
		// 	buffer._storage_buffer._global_upload_ringbuffers_begin[current_frame_index];
	}
}