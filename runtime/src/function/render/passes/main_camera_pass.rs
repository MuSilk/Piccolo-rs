

use std::{cell::RefCell, os::raw::c_void, rc::Rc, slice};

use crate::{function::render::{interface::vulkan::vulkan_rhi::{VulkanRHI, VULKAN_RHI_DESCRIPTOR_STORAGE_BUFFER, VULKAN_RHI_DESCRIPTOR_STORAGE_BUFFER_DYNAMIC}, passes::combine_ui_pass::CombineUIPass, render_common::{MeshPerdrawcallStorageBufferObject, MeshPerframeStorageBufferObject}, render_helper::round_up, render_mesh::{MeshVertex, VulkanMeshVertexPosition, VulkanMeshVertexVarying, VulkanMeshVertexVaryingEnableBlending}, render_pass::{RenderPass, RenderPipelineBase, _MAIN_CAMERA_PASS_BACKUP_BUFFER_EVEN, _MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD, _MAIN_CAMERA_PASS_CUSTOM_ATTACHMENT_COUNT, _MAIN_CAMERA_PASS_GBUFFER_A, _MAIN_CAMERA_PASS_GBUFFER_B, _MAIN_CAMERA_PASS_GBUFFER_C, _MAIN_CAMERA_PASS_POST_PROCESS_ATTACHMENT_COUNT, _MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_EVEN, _MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_ODD}, render_resource::RenderResource}, shader::generated::shader::{MESH_GBUFFER_FRAG, MESH_VERT}};

use anyhow::Result;
use linkme::distributed_slice;
use nalgebra_glm::Vec3;
use vulkanalia::{prelude::v1_0::*};

pub struct MainCameraPassInitInfo<'a> {
    pub rhi: &'a Rc<RefCell<VulkanRHI>>,
}

enum LayoutType {
    PerMesh,
    MeshGlobal,
    SkyBox,
    Axis,
    Partical,
    DeferredLighting,
    LayoutTypeCount,
}

enum RenderPipelineType {
    MeshGBuffer,
    DeferredLighting,
    MeshLighting,
    SkyBox,
    Axis,
    Partical,
    EnumCount,
}

#[derive(Default)]
pub struct MainCameraPass{
    pub m_render_pass: RenderPass,
    pub m_mesh_perframe_storage_buffer_object: MeshPerframeStorageBufferObject,
    m_swapchain_framebuffers: Vec<vk::Framebuffer>,

    t_test_triangle_buffer: vk::Buffer,
    t_test_triangle_buffer_memory: vk::DeviceMemory,
}

impl MainCameraPass {
    pub fn initialize(&mut self, info: &MainCameraPassInitInfo) -> Result<()> {
        self.m_render_pass.m_global_render_resource = 
            Rc::downgrade(&self.m_render_pass.m_base.m_render_resource.upgrade().unwrap().borrow().m_global_render_resource);
        let rhi = info.rhi.borrow();
        self.setup_attachments(&rhi)?;
        self.setup_render_pass(&rhi)?;
        self.setup_descriptor_layout(&rhi)?;
        self.setup_pipelines(&rhi)?;
        self.setup_descriptor_set(&rhi)?;
        self.setup_framebuffer(&rhi)?;

        let triangle_positions = [
            VulkanMeshVertexPosition {
                position: Vec3::new(0.0, -0.5, 0.0),
            },
            VulkanMeshVertexPosition {
                position: Vec3::new(0.5, 0.5, 0.0),
            },
            VulkanMeshVertexPosition {
                position: Vec3::new(-0.5, 0.5, 0.0),
            },
        ];
        let (staging_buffer, staging_buffer_memory) = rhi.create_buffer(
            std::mem::size_of_val(&triangle_positions) as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        let staging_buffer_data =
            rhi.map_memory(staging_buffer_memory, 0, std::mem::size_of_val(&triangle_positions) as u64, vk::MemoryMapFlags::empty())?;
        unsafe {
            std::ptr::copy_nonoverlapping(
                triangle_positions.as_ptr().cast(),
                staging_buffer_data,
                std::mem::size_of::<VulkanMeshVertexPosition>() * triangle_positions.len(),
            );
        }
        rhi.unmap_memory(staging_buffer_memory);
        (self.t_test_triangle_buffer, self.t_test_triangle_buffer_memory) = rhi.create_buffer(
            std::mem::size_of_val(&triangle_positions) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        rhi.copy_buffer(
            staging_buffer, 
            self.t_test_triangle_buffer, 
            0,
            0,
            std::mem::size_of_val(&triangle_positions) as u64
        )?;
        rhi.destroy_buffer(staging_buffer);
        rhi.free_memory(staging_buffer_memory);

        Ok(())
    }

    pub fn prepare_pass_data(&mut self, render_resource: &RenderResource) {
        self.m_mesh_perframe_storage_buffer_object = render_resource.m_mesh_perframe_storage_buffer_object.clone();
    }
    
    pub fn recreate_after_swapchain(&mut self, rhi: &VulkanRHI) -> Result<()>{
        for framebuffer in self.m_swapchain_framebuffers.drain(..){
            rhi.destroy_framebuffer(framebuffer);
        }
        self.setup_framebuffer(rhi)?;
        Ok(())
    }

    pub fn destroy(&self) {
        let rhi = self.m_render_pass.m_base.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        self.m_swapchain_framebuffers.iter().for_each(|f| rhi.destroy_framebuffer(*f));
        rhi.destroy_pipeline(self.m_render_pass.m_render_pipeline[0].pipeline);
        rhi.destroy_pipeline_layout(self.m_render_pass.m_render_pipeline[0].layout);
        // rhi.destroy_descriptor_set_layout(self.m_base.m_descriptor_infos[0].layout);
        rhi.destroy_render_pass(self.m_render_pass.m_framebuffer.render_pass);
    }

    pub fn draw(
        &self, 
        combine_ui_pass: &CombineUIPass,
        current_swapchain_image_index: usize
    ) -> Result<()> {
        let rhi = self.m_render_pass.m_base.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        let command_buffer = rhi.get_current_command_buffer();

        let swapchain_info = rhi.get_swapchain_info();
        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(swapchain_info.extent);

        let mut clear_values = [vk::ClearValue::default(); 3];
        clear_values[0].color.float32 = [0.0, 0.0, 0.0, 1.0];
        clear_values[1].depth_stencil = vk::ClearDepthStencilValue{depth: 1.0, stencil: 0};
        clear_values[2].color.float32 = [0.0, 0.0, 0.0, 1.0];
        
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.m_render_pass.m_framebuffer.render_pass)
            .framebuffer(self.m_swapchain_framebuffers[current_swapchain_image_index])
            .render_area(render_area)
            .clear_values(&clear_values);

        rhi.cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::INLINE);

        let color = [1.0;4];
        rhi.push_event(command_buffer, "BasePass", color);

        self.draw_object()?;

        rhi.pop_event(command_buffer);

        rhi.cmd_next_subpass(command_buffer, vk::SubpassContents::INLINE);

        combine_ui_pass.draw();

        rhi.cmd_end_render_pass(command_buffer);

        Ok(())
    }
}

// #[distributed_slice(VULKAN_RHI_DESCRIPTOR_STORAGE_BUFFER)]
// static STORAGE_BUFFER_COUNT: u32 = 1;
#[distributed_slice(VULKAN_RHI_DESCRIPTOR_STORAGE_BUFFER_DYNAMIC)]
static STORAGE_BUFFER_DYNAMIC_COUNT: u32 = 2;

impl MainCameraPass {
    fn setup_attachments(&mut self, rhi: &VulkanRHI) -> Result<()> {
        let swapchain_info = rhi.get_swapchain_info();

        self.m_render_pass.m_framebuffer.attachments.resize_with(
            _MAIN_CAMERA_PASS_CUSTOM_ATTACHMENT_COUNT + _MAIN_CAMERA_PASS_POST_PROCESS_ATTACHMENT_COUNT,
            Default::default
        );

        self.m_render_pass.m_framebuffer.attachments[_MAIN_CAMERA_PASS_GBUFFER_A].format = vk::Format::R8G8B8A8_SNORM;
        self.m_render_pass.m_framebuffer.attachments[_MAIN_CAMERA_PASS_GBUFFER_B].format = vk::Format::R8G8B8A8_SNORM;
        self.m_render_pass.m_framebuffer.attachments[_MAIN_CAMERA_PASS_GBUFFER_C].format = vk::Format::R8G8B8A8_SRGB;
        self.m_render_pass.m_framebuffer.attachments[_MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD].format = vk::Format::R16G16B16A16_SFLOAT;
        self.m_render_pass.m_framebuffer.attachments[_MAIN_CAMERA_PASS_BACKUP_BUFFER_EVEN].format = vk::Format::R16G16B16A16_SFLOAT;
        for buffer_index in 0.._MAIN_CAMERA_PASS_CUSTOM_ATTACHMENT_COUNT {
            if buffer_index == _MAIN_CAMERA_PASS_GBUFFER_A {
                (
                    self.m_render_pass.m_framebuffer.attachments[buffer_index].image,
                    self.m_render_pass.m_framebuffer.attachments[buffer_index].mem,
                ) = rhi.create_image(
                    swapchain_info.extent.width,
                    swapchain_info.extent.height,
                    self.m_render_pass.m_framebuffer.attachments[buffer_index].format,
                    vk::ImageTiling::OPTIMAL,
                    vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                    vk::ImageCreateFlags::empty(),
                    1,
                    1,
                )?;
            }
            else{
                (
                    self.m_render_pass.m_framebuffer.attachments[buffer_index].image,
                    self.m_render_pass.m_framebuffer.attachments[buffer_index].mem,
                ) = rhi.create_image(
                    swapchain_info.extent.width,
                    swapchain_info.extent.height,
                    self.m_render_pass.m_framebuffer.attachments[buffer_index].format,
                    vk::ImageTiling::OPTIMAL,
                    vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                    vk::ImageCreateFlags::empty(),
                    1,
                    1,
                )?;
            }
            self.m_render_pass.m_framebuffer.attachments[buffer_index].view = rhi.create_image_view(
                self.m_render_pass.m_framebuffer.attachments[buffer_index].image,
                self.m_render_pass.m_framebuffer.attachments[buffer_index].format,
                vk::ImageAspectFlags::COLOR,
                vk::ImageViewType::_2D,
                1,
                1,
            )?;
        }
        
        self.m_render_pass.m_framebuffer.attachments[_MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_ODD].format = vk::Format::R16G16B16A16_SFLOAT;
        self.m_render_pass.m_framebuffer.attachments[_MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_EVEN].format = vk::Format::R16G16B16A16_SFLOAT;

        for buffer_index in _MAIN_CAMERA_PASS_CUSTOM_ATTACHMENT_COUNT..(_MAIN_CAMERA_PASS_CUSTOM_ATTACHMENT_COUNT + _MAIN_CAMERA_PASS_POST_PROCESS_ATTACHMENT_COUNT) {
            (
                self.m_render_pass.m_framebuffer.attachments[buffer_index].image,
                self.m_render_pass.m_framebuffer.attachments[buffer_index].mem,
            ) = rhi.create_image(
                swapchain_info.extent.width,
                swapchain_info.extent.height,
                self.m_render_pass.m_framebuffer.attachments[buffer_index].format,
                vk::ImageTiling::OPTIMAL,
                vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                vk::ImageCreateFlags::empty(),
                1,
                1,
            )?;
            self.m_render_pass.m_framebuffer.attachments[buffer_index].view = rhi.create_image_view(
                self.m_render_pass.m_framebuffer.attachments[buffer_index].image,
                self.m_render_pass.m_framebuffer.attachments[buffer_index].format,
                vk::ImageAspectFlags::COLOR,
                vk::ImageViewType::_2D,
                1,
                1,
            )?;
        }
        Ok(())
    }

    fn setup_render_pass(&mut self, rhi: &VulkanRHI) -> Result<()> {
        let mut attachments = [vk::AttachmentDescription::default(); 3];

        let backup_odd_color_attachment_description = &mut attachments[0];
        *backup_odd_color_attachment_description = vk::AttachmentDescription::builder()
            .format(self.m_render_pass.m_framebuffer.attachments[_MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD].format)
            .samples(vk::SampleCountFlags::_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .build();

        let depth_attachment_description = &mut attachments[1];
        *depth_attachment_description = vk::AttachmentDescription::builder()
            .format(rhi.get_depth_image_info().format)
            .samples(vk::SampleCountFlags::_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let swapchain_image_attachment = &mut attachments[2];
        *swapchain_image_attachment = vk::AttachmentDescription::builder()
            .format(rhi.get_swapchain_info().image_format)
            .samples(vk::SampleCountFlags::_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();    

        let mut subpasses = [vk::SubpassDescription::default(); 2];

        //gbuffer subpass
        {
            let depth_stencil_attachment_ref = vk::AttachmentReference::builder()
                .attachment(1)
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
            let color_attachment_ref = vk::AttachmentReference::builder()
                .attachment(0)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
            subpasses[0] = vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&[color_attachment_ref])
                .depth_stencil_attachment(&depth_stencil_attachment_ref)
                .build();
        }  


        //combine ui subpass
        {
            let input_attachment_ref = vk::AttachmentReference::builder()
                .attachment(0)
                .layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
            let color_attachment_ref = vk::AttachmentReference::builder()
                .attachment(2)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
            subpasses[1] = vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&[color_attachment_ref])
                .input_attachments(&[input_attachment_ref])
                .build();
        }

        let dependencies = [
            vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
                .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ)
                .build(),
            vk::SubpassDependency::builder()
                .src_subpass(0)
                .dst_subpass(1)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | 
                    vk::PipelineStageFlags::FRAGMENT_SHADER)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT |
                    vk::PipelineStageFlags::FRAGMENT_SHADER)   
                .src_access_mask(vk::AccessFlags::SHADER_WRITE | vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ | vk::AccessFlags::COLOR_ATTACHMENT_READ)
                .dependency_flags(vk::DependencyFlags::BY_REGION)
                .build()
        ];

        let info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        self.m_render_pass.m_framebuffer.render_pass = rhi.create_render_pass(&info)?;

        Ok(())
    }

    fn setup_framebuffer(&mut self, rhi: &VulkanRHI) -> Result<()> {
        let swapchain_info = rhi.get_swapchain_info();
        let depth_image_info = rhi.get_depth_image_info();
        let framebuffers =  swapchain_info.image_views
            .iter()
            .map(|image_view| {
                let attachments = &[
                    self.m_render_pass.m_framebuffer.attachments[_MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD].view,
                    *depth_image_info.image_view,
                    *image_view,
                ];
                let create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(self.m_render_pass.m_framebuffer.render_pass)
                    .attachments(attachments)
                    .width(swapchain_info.extent.width)
                    .height(swapchain_info.extent.height)
                    .layers(1);

                rhi.create_framebuffer(&create_info)
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.m_swapchain_framebuffers = framebuffers;
        Ok(())
    }

    fn setup_descriptor_layout(&mut self, rhi: &VulkanRHI) -> Result<()> {
        // self.m_render_pass.m_descriptor_infos.resize_with(LayoutType::LayoutTypeCount as usize, Default::default);
        self.m_render_pass.m_descriptor_infos.resize_with(1, Default::default);
        {
            let mesh_global_layout_bindings = [
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER_DYNAMIC)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                    .build(),
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER_DYNAMIC)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::VERTEX)
                    .build(),
            ];
            let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
                .bindings(&mesh_global_layout_bindings)
                .build();
            self.m_render_pass.m_descriptor_infos[0].layout = rhi.create_descriptor_set_layout(&create_info)?;
        }
        Ok(())
    }

    fn setup_pipelines(&mut self ,rhi: &VulkanRHI)-> Result<()> {
        let vert_shader_module = rhi.create_shader_module(&MESH_VERT)?;
        let frag_shader_module = rhi.create_shader_module(&MESH_GBUFFER_FRAG)?;

        let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(b"main\0");

        let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(b"main\0");

        let binding_descriptions = &MeshVertex::get_binding_descriptions();
        let attribute_descriptions = &MeshVertex::get_attribute_descriptions();
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(binding_descriptions)
            .vertex_attribute_descriptions(attribute_descriptions);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let swapchain_info = rhi.get_swapchain_info();

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(std::slice::from_ref(swapchain_info.viewport))
            .scissors(std::slice::from_ref(swapchain_info.scissor));

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::_1);

        let depth_stencil_state: vk::PipelineDepthStencilStateCreateInfoBuilder = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .stencil_test_enable(false);

        let attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::all())
            .blend_enable(false);

        let attachments = &[attachment];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]);

        let set_layouts = &self.m_render_pass.get_descriptor_set_layouts();
        let layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(set_layouts);

        let pipeline_layout = rhi.create_pipeline_layout(&layout_info)?;

        let stages = &[vert_stage, frag_stage];
        let info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(self.m_render_pass.m_framebuffer.render_pass)
            .subpass(0)
            .build();

        let pipeline = rhi.create_graphics_pipelines(vk::PipelineCache::null(), &[info])?[0];

        rhi.destroy_shader_module(vert_shader_module);
        rhi.destroy_shader_module(frag_shader_module);

        self.m_render_pass.m_render_pipeline.push(RenderPipelineBase{
            layout: pipeline_layout,
            pipeline,
        });

        Ok(())
    }

    fn setup_descriptor_set(&mut self, rhi: &VulkanRHI) -> Result<()> {
        self.setup_model_global_descriptor_set(rhi)?;
        Ok(())
    }

    fn setup_model_global_descriptor_set(&mut self, rhi: &VulkanRHI) -> Result<()> {
        let set_layouts = &[self.m_render_pass.m_descriptor_infos[0].layout];
        let mesh_global_descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(rhi.get_descriptor_pool())
            .set_layouts(set_layouts);

        self.m_render_pass.m_descriptor_infos[0].descriptor_set = rhi.allocate_descriptor_sets(&mesh_global_descriptor_set_alloc_info)?[0];

        let global_render_resource = self.m_render_pass.m_global_render_resource.upgrade().unwrap();

        let mesh_perframe_storage_buffer_info = vk::DescriptorBufferInfo::builder()
            .offset(0)
            .range(size_of::<MeshPerframeStorageBufferObject>() as u64)
            .buffer(global_render_resource.borrow()._storage_buffer._global_upload_ringbuffer)
            .build();

        let mesh_perdrawcall_storage_buffer_info = vk::DescriptorBufferInfo::builder()
            .offset(0)
            .range(size_of::<MeshPerdrawcallStorageBufferObject>() as u64) 
            .buffer(global_render_resource.borrow()._storage_buffer._global_upload_ringbuffer)
            .build();

        let mesh_descriptor_writes_info = [
            vk::WriteDescriptorSet::builder()
                .dst_set(self.m_render_pass.m_descriptor_infos[0].descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER_DYNAMIC)
                .buffer_info(&[mesh_perframe_storage_buffer_info])
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(self.m_render_pass.m_descriptor_infos[0].descriptor_set)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER_DYNAMIC)
                .buffer_info(&[mesh_perdrawcall_storage_buffer_info])
                .build()
        ];

        rhi.update_descriptor_sets(&mesh_descriptor_writes_info)?;

        Ok(())
    }
    
    fn draw_object(&self) -> Result<()> {
        let rhi = self.m_render_pass.m_base.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        let command_buffer = rhi.get_current_command_buffer();

        let info = rhi.get_swapchain_info();
        rhi.cmd_set_viewport(command_buffer, 0, slice::from_ref(info.viewport));
        rhi.cmd_set_scissor(command_buffer, 0, slice::from_ref(info.scissor));

        let pipeline = &self.m_render_pass.m_render_pipeline[0];
        rhi.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline);
        
        let perframe_dynamic_offset = round_up(
            self.m_render_pass.m_global_render_resource
                .upgrade().unwrap().borrow()
                ._storage_buffer._global_upload_ringbuffers_end[rhi.get_current_frame_index()], 
            self.m_render_pass.m_global_render_resource
                .upgrade().unwrap().borrow()
                ._storage_buffer._min_storage_buffer_offset_alignment
        );

        self.m_render_pass.m_global_render_resource
            .upgrade().unwrap().borrow_mut()
            ._storage_buffer._global_upload_ringbuffers_end[rhi.get_current_frame_index()] = 
                perframe_dynamic_offset + std::mem::size_of::<MeshPerframeStorageBufferObject>() as u32;
        unsafe{
            std::ptr::copy_nonoverlapping(
                &self.m_mesh_perframe_storage_buffer_object as *const _ as *const c_void,
                self.m_render_pass.m_global_render_resource.upgrade().unwrap().borrow()._storage_buffer._global_upload_ringbuffer_pointer.add(perframe_dynamic_offset as usize), 
                std::mem::size_of::<MeshPerframeStorageBufferObject>()
            );
        }

        let perdrawcall_dynamic_offset = round_up(
            self.m_render_pass.m_global_render_resource
                .upgrade().unwrap().borrow()
                ._storage_buffer._global_upload_ringbuffers_end[rhi.get_current_frame_index()], 
            self.m_render_pass.m_global_render_resource
                .upgrade().unwrap().borrow()
                ._storage_buffer._min_storage_buffer_offset_alignment
        );
        self.m_render_pass.m_global_render_resource
            .upgrade().unwrap().borrow_mut()
            ._storage_buffer._global_upload_ringbuffers_end[rhi.get_current_frame_index()] = 
                perdrawcall_dynamic_offset + std::mem::size_of::<MeshPerdrawcallStorageBufferObject>() as u32;

        rhi.cmd_bind_descriptor_sets(
            command_buffer, 
            vk::PipelineBindPoint::GRAPHICS, 
            self.m_render_pass.m_render_pipeline[0].layout,
            0,
            &[self.m_render_pass.m_descriptor_infos[0].descriptor_set],
            &[perframe_dynamic_offset, perdrawcall_dynamic_offset],
        );
        
        rhi.cmd_bind_vertex_buffers(command_buffer, 0, &[self.t_test_triangle_buffer], &[0]);
        rhi.cmd_draw(command_buffer, 3, 1, 0, 0);
        Ok(())
    }
}


