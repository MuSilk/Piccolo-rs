use std::{any::Any};

use vulkanalia::{prelude::v1_0::*};

use crate::runtime::function::render::render_type::{RHIAccessFlags, RHIAttachmentDescriptionFlags, RHIAttachmentLoadOp, RHIAttachmentStoreOp, RHIBlendFactor, RHIBlendOp, RHIColorComponentFlags, RHICompareOp, RHICullModeFlags, RHIDependencyFlags, RHIDescriptorSetLayoutCreateFlags, RHIDescriptorType, RHIDeviceSize, RHIDynamicState, RHIFormat, RHIFramebufferCreateFlags, RHIFrontFace, RHIImageLayout, RHILogicOp, RHIPipelineBindPoint, RHIPipelineColorBlendStateCreateFlags, RHIPipelineCreateFlags, RHIPipelineDepthStencilStateCreateFlags, RHIPipelineDynamicStateCreateFlags, RHIPipelineInputAssemblyStateCreateFlags, RHIPipelineLayoutCreateFlags, RHIPipelineMultisampleStateCreateFlags, RHIPipelineRasterizationStateCreateFlags, RHIPipelineShaderStageCreateFlags, RHIPipelineStageFlags, RHIPipelineVertexInputStateCreateFlags, RHIPipelineViewportStateCreateFlags, RHIPolygonMode, RHIPrimitiveTopology, RHIRenderPassCreateFlags, RHISampleCountFlags, RHISampleMask, RHIShaderStageFlags, RHIStencilOp, RHISubpassDescriptionFlags, RHIVertexInputRate};

pub trait RHIBuffer: Any { }
impl dyn RHIBuffer {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
pub trait  RHIBufferView: Any { }
impl dyn RHIBufferView {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
pub trait  RHICommandBuffer: Any { }
impl dyn RHICommandBuffer {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
pub trait  RHICommandPool: Any { }
impl dyn RHICommandPool {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
pub trait  RHIDescriptorPool: Any { }
impl dyn RHIDescriptorPool {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
pub trait  RHIDescriptorSet: Any { }
impl dyn RHIDescriptorSet {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait  RHIDescriptorSetLayout: Any { }
impl dyn RHIDescriptorSetLayout {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
pub trait  RHIDevice { }
pub trait  RHIDeviceMemory: Any { }
impl dyn RHIDeviceMemory {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
pub trait  RHIEvent { }
pub trait  RHIFence { }
pub trait  RHIFramebuffer: Any { }
impl dyn RHIFramebuffer {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
pub trait  RHIImage: Any { }
impl dyn RHIImage {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
pub trait  RHIImageView:Any { }
impl dyn RHIImageView {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
pub trait  RHIInstance { }
pub trait  RHIQueue: Any { }
impl dyn RHIQueue {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
pub trait  RHIPhysicalDevice { }
pub trait  RHIPipeline: Any { }
impl dyn RHIPipeline {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
pub trait  RHIPipelineCache { }
pub trait  RHIPipelineLayout: Any { }
impl dyn RHIPipelineLayout {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
pub trait  RHIRenderPass: Any { }
impl dyn RHIRenderPass {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
pub trait  RHISampler: Any { }
impl dyn RHISampler {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
pub trait  RHISemaphore: Any { }
impl dyn RHISemaphore {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
pub trait  RHIShader: Any { }
impl dyn RHIShader {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct RHIAttachmentDescription {
    pub flags: RHIAttachmentDescriptionFlags,
    pub format: RHIFormat,
    pub samples: RHISampleCountFlags,
    pub load_op: RHIAttachmentLoadOp,
    pub store_op: RHIAttachmentStoreOp,
    pub stencil_load_op: RHIAttachmentLoadOp,
    pub stencil_store_op: RHIAttachmentStoreOp,
    pub initial_layout: RHIImageLayout,
    pub final_layout: RHIImageLayout,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Eq, Hash, PartialEq)]
pub struct RHIAttachmentReference {
    pub attachment: u32,
    pub layout: RHIImageLayout,
}

pub struct RHIDescriptorBufferInfo<'a>{
    pub buffer: &'a Box<dyn RHIBuffer>,
    pub offset: RHIDeviceSize,
    pub range: RHIDeviceSize,
}

pub struct RHIDescriptorImageInfo<'a>{
    pub sampler: &'a Box<dyn RHISampler>,
    pub image_view: &'a Box<dyn RHIImageView>,
    pub image_layout: RHIImageLayout,
}

pub struct RHIDescriptorSetLayoutBinding<'a>{
    pub binding: u32,
    pub descriptor_type: RHIDescriptorType,
    pub descriptor_count: u32,
    pub stage_flags: RHIShaderStageFlags,
    pub p_immutable_samplers: Option<&'a Box<dyn RHISampler>>,
}

pub struct RHIDescriptorSetAllocateInfo<'a>{
    pub descriptor_pool: &'a Box<dyn RHIDescriptorPool>,
    pub set_layouts: &'a [&'a Box<dyn RHIDescriptorSetLayout>],
}

pub struct RHIDescriptorSetLayoutCreateInfo<'a>{
    pub flags: RHIDescriptorSetLayoutCreateFlags,
    pub bindings: &'a [RHIDescriptorSetLayoutBinding<'a>],
}

pub struct RHIFramebufferCreateInfo<'a>{
    pub flags: RHIFramebufferCreateFlags,
    pub render_pass: &'a Box<dyn RHIRenderPass>,
    pub attachments: &'a [&'a Box<dyn RHIImageView>],
    pub width: u32,
    pub height: u32,
    pub layers: u32,
}

pub struct RHIPipelineColorBlendAttachmentState {
    pub blend_enable: bool,
    pub src_color_blend_factor: RHIBlendFactor,
    pub dst_color_blend_factor: RHIBlendFactor,
    pub color_blend_op: RHIBlendOp,
    pub src_alpha_blend_factor: RHIBlendFactor,
    pub dst_alpha_blend_factor: RHIBlendFactor,
    pub alpha_blend_op: RHIBlendOp,
    pub color_write_mask: RHIColorComponentFlags,
}

pub struct RHIPipelineColorBlendStateCreateInfo<'a> {
    pub flags: RHIPipelineColorBlendStateCreateFlags,
    pub logic_op_enable: bool,
    pub logic_op: RHILogicOp,
    pub attachments: &'a [&'a RHIPipelineColorBlendAttachmentState],
    pub blend_constants: [f32; 4],
}

pub struct RHIGraphicsPipelineCreateInfo<'a> {
    pub flags: RHIPipelineCreateFlags,
    pub stages: &'a [RHIPipelineShaderStageCreateInfo<'a>],
    pub vertex_input_state: &'a RHIPipelineVertexInputStateCreateInfo<'a>,
    pub input_assembly_state: &'a RHIPipelineInputAssemblyStateCreateInfo,
    pub tessellation_state: Option<&'a ()>,
    pub viewport_state: &'a RHIPipelineViewportStateCreateInfo<'a>,
    pub rasterization_state: &'a RHIPipelineRasterizationStateCreateInfo,
    pub multisample_state: &'a RHIPipelineMultisampleStateCreateInfo<'a>,
    pub depth_stencil_state: Option<&'a RHIPipelineDepthStencilStateCreateInfo>,
    pub color_blend_state: &'a RHIPipelineColorBlendStateCreateInfo<'a>,
    pub dynamic_state: Option<&'a RHIPipelineDynamicStateCreateInfo<'a>>,
    pub layout: &'a Box<dyn RHIPipelineLayout>,
    pub render_pass: &'a Box<dyn RHIRenderPass>,
    pub subpass: u32,
    pub base_pipeline_handle: Option<&'a Box<dyn RHIPipeline>>,
    pub base_pipeline_index: i32,
}

pub struct RHIPipelineDepthStencilStateCreateInfo {
    pub flags: RHIPipelineDepthStencilStateCreateFlags,
    pub depth_test_enable: bool,
    pub depth_write_enable: bool,
    pub depth_compare_op: RHICompareOp,
    pub depth_bounds_test_enable: bool,
    pub stencil_test_enable: bool,
    pub front: RHIStencilOpState,
    pub back: RHIStencilOpState,
    pub min_depth_bounds: f32,
    pub max_depth_bounds: f32,
}

pub struct RHIPipelineDynamicStateCreateInfo<'a> {
    pub flags: RHIPipelineDynamicStateCreateFlags,
    pub dynamic_states: &'a [RHIDynamicState],
}

pub struct RHIPipelineInputAssemblyStateCreateInfo {
    pub flags: RHIPipelineInputAssemblyStateCreateFlags,
    pub topology: RHIPrimitiveTopology,
    pub primitive_restart_enable: bool,
}

pub struct RHIPipelineLayoutCreateInfo<'a>{
    pub flags: RHIPipelineLayoutCreateFlags,
    pub set_layouts: &'a [&'a Box<dyn RHIDescriptorSetLayout>],
    pub push_constant_ranges: &'a [RHIPushConstantRange],
}

pub struct RHIPipelineVertexInputStateCreateInfo<'a> {
    pub flags: RHIPipelineVertexInputStateCreateFlags,
    pub vertex_binding_descriptions: &'a [RHIVertexInputBindingDescription],
    pub vertex_attribute_descriptions: &'a [RHIVertexInputAttributeDescription],
}

pub struct RHIPipelineViewportStateCreateInfo<'a> {
    pub flags: RHIPipelineViewportStateCreateFlags,
    pub viewports: &'a [&'a RHIViewport],
    pub scissors: &'a [&'a RHIRect2D],
}

pub struct RHIPipelineMultisampleStateCreateInfo<'a> {
    pub flags: RHIPipelineMultisampleStateCreateFlags,
    pub rasterization_samples: RHISampleCountFlags,
    pub sample_shading_enable: bool,
    pub min_sample_shading: f32,
    pub sample_mask: Option<&'a RHISampleMask>,
    pub alpha_to_coverage_enable: bool,
    pub alpha_to_one_enable: bool,
}

pub struct RHIPipelineRasterizationStateCreateInfo {
    pub flags: RHIPipelineRasterizationStateCreateFlags,
    pub depth_clamp_enable: bool,
    pub rasterizer_discard_enable: bool,
    pub polygon_mode: RHIPolygonMode,
    pub cull_mode: RHICullModeFlags,
    pub front_face: RHIFrontFace,
    pub depth_bias_enable: bool,
    pub depth_bias_constant_factor: f32,
    pub depth_bias_clamp: f32,
    pub depth_bias_slope_factor: f32,
    pub line_width: f32,
}

pub struct RHIPipelineShaderStageCreateInfo<'a>{
    pub flags : RHIPipelineShaderStageCreateFlags,
    pub stage: RHIShaderStageFlags,
    pub module: &'a Box<dyn RHIShader>,
    pub name: &'a str,
    pub specialization_info: Option<&'a RHISpecializationInfo<'a>>,
}

pub struct RHIPushConstantRange {
    pub stage_flags: RHIShaderStageFlags,
    pub offset: u32,
    pub size: u32,
}

pub struct RHIRenderPassCreateInfo<'a>{
    pub flags: RHIRenderPassCreateFlags,
    pub attachments: &'a [RHIAttachmentDescription],
    pub subpasses: &'a [RHISubpassDescription<'a>],
    pub dependencies: &'a [RHISubPassDependency],
}

pub struct RHISpecializationInfo<'a> {
    pub map_entries: &'a [RHISpecializationMapEntry],
    pub data: &'a [u8],
}

pub struct RHISpecializationMapEntry {
    pub constant_id: u32,
    pub offset: u32,
    pub size: usize,
}

#[derive(Default)]
pub struct RHIStencilOpState {
    pub fail_op: RHIStencilOp,
    pub pass_op: RHIStencilOp,
    pub depth_fail_op: RHIStencilOp,
    pub compare_op: RHICompareOp,
    pub compare_mask: u32,
    pub write_mask: u32,
    pub reference: u32,
}

pub struct RHISubPassDependency {
    pub src_subpass: u32,
    pub dst_subpass: u32,
    pub src_stage_mask: RHIPipelineStageFlags,
    pub dst_stage_mask: RHIPipelineStageFlags,
    pub src_access_mask: RHIAccessFlags,
    pub dst_access_mask: RHIAccessFlags,
    pub dependency_flags: RHIDependencyFlags,
}

pub struct RHISubpassDescription<'a> {
    pub flags: RHISubpassDescriptionFlags,
    pub pipeline_bind_point: RHIPipelineBindPoint,
    pub input_attachments: &'a [RHIAttachmentReference],
    pub color_attachments: &'a [RHIAttachmentReference],
    pub resolve_attachments: &'a [RHIAttachmentReference],
    pub depth_stencil_attachment: &'a RHIAttachmentReference,
    pub preserve_attachments: &'a [u32],
}

pub struct RHIWriteDescriptorSet<'a>{
    pub dst_set: &'a Box<dyn RHIDescriptorSet>,
    pub dst_binding: u32,
    pub dst_array_element: u32,
    pub descriptor_type: RHIDescriptorType,
    pub image_info: &'a [RHIDescriptorImageInfo<'a>],
    pub buffer_info: &'a [RHIDescriptorBufferInfo<'a>],
    pub texel_buffer_view: &'a [&'a Box<dyn RHIBufferView>],
}

#[derive(Default)]
pub struct RHIClearDepthStencilValue {
    pub depth: f32,
    pub stencil: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union RHIClearColorValue {
    pub float32: [f32; 4],
    pub int32: [i32; 4],
    pub uint32: [u32; 4],
}

pub enum RHIClearValue {
    Color(RHIClearColorValue),
    DepthStencil(RHIClearDepthStencilValue),
}

#[derive(Default, Clone, Copy)]
pub struct RHIExtent2D {
    pub width: u32,
    pub height: u32,
}

#[derive(Default)]
pub struct RHIOffset2D {
    pub x:i32,
    pub y:i32,
}

#[derive(Default)]
pub struct RHIRect2D {
    pub offset: RHIOffset2D,
    pub extent: RHIExtent2D,
}

pub struct RHIRenderPassBeginInfo<'a>{
    pub render_pass: &'a dyn RHIRenderPass,
    pub framebuffer: &'a dyn RHIFramebuffer,
    pub render_area: RHIRect2D,
    pub clear_values: &'a [RHIClearValue],
}

pub struct RHIVertexInputAttributeDescription{
    pub location: u32,
    pub binding: u32,
    pub format: RHIFormat,
    pub offset: u32,
}

pub struct RHIVertexInputBindingDescription{
    pub binding: u32,
    pub stride: u32,
    pub input_rate: RHIVertexInputRate,
}

#[derive(Default)]
pub struct RHIViewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

#[derive(Clone, Debug, Default)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

pub struct SwapChainSupportDetails { 
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub struct RHISwapChainDesc<'a>{
    pub extent: vk::Extent2D,
    pub image_format: vk::Format,
    pub viewport: &'a vk::Viewport,
    pub scissor: &'a vk::Rect2D,
    pub image_views: &'a [vk::ImageView],
}

pub struct RHIDepthImageDesc<'a>{
    pub image: &'a vk::Image,
    pub image_view: &'a vk::ImageView,
    pub format: vk::Format,
}