

use std::{cell::RefCell, rc::Rc, slice};

use crate::{function::render::{interface::vulkan::vulkan_rhi::{VulkanRHI, VULKAN_RHI_DESCRIPTOR_STORAGE_BUFFER}, render_mesh::{MeshVertex, VulkanMeshVertexPosition}, render_pass::{RenderPass, RenderPipelineBase, _MAIN_CAMERA_PASS_ATTACHMENT_COUNT, _MAIN_CAMERA_PASS_BACKUP_BUFFER_EVEN, _MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD, _MAIN_CAMERA_PASS_CUSTOM_ATTACHMENT_COUNT, _MAIN_CAMERA_PASS_DEPTH, _MAIN_CAMERA_PASS_GBUFFER_A, _MAIN_CAMERA_PASS_GBUFFER_B, _MAIN_CAMERA_PASS_GBUFFER_C, _MAIN_CAMERA_PASS_POST_PROCESS_ATTACHMENT_COUNT, _MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_EVEN, _MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_ODD, _MAIN_CAMERA_PASS_SWAPCHAIN_IMAGE}, render_pass_base::RenderPassCommonInfo}, shader::generated::shader::{MESH_GBUFFER_FRAG, MESH_VERT}};

use anyhow::Result;
use linkme::distributed_slice;
use nalgebra_glm::Vec3;
use vulkanalia::{prelude::v1_0::*};

pub struct MainCameraPassCreateInfo<'a> {
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
    m_render_pass: RenderPass,
    m_swapchain_framebuffers: Vec<vk::Framebuffer>,

    t_test_triangle_buffer: vk::Buffer,
    t_test_triangle_buffer_memory: vk::DeviceMemory,
}

impl MainCameraPass {
    pub fn create(info: &MainCameraPassCreateInfo) -> Result<Self> {
        let mut main_camera_render_pass = MainCameraPass::default();
        main_camera_render_pass.m_render_pass.set_common_info(&RenderPassCommonInfo{
            rhi: info.rhi,
        });
        let rhi = info.rhi.borrow();
        main_camera_render_pass.setup_attachments(&rhi)?;
        main_camera_render_pass.setup_render_pass(&rhi)?;
        main_camera_render_pass.setup_framebuffer(&rhi)?;
        main_camera_render_pass.setup_descriptor_layout(&rhi)?;
        main_camera_render_pass.setup_pipelines(&rhi)?;

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
        (main_camera_render_pass.t_test_triangle_buffer, main_camera_render_pass.t_test_triangle_buffer_memory) = rhi.create_buffer(
            std::mem::size_of_val(&triangle_positions) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        rhi.copy_buffer(
            staging_buffer, 
            main_camera_render_pass.t_test_triangle_buffer, 
            0,
            0,
            std::mem::size_of_val(&triangle_positions) as u64
        )?;
        rhi.destroy_buffer(staging_buffer);
        rhi.free_memory(staging_buffer_memory);

        Ok(main_camera_render_pass)
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

    pub fn draw(&self, current_swapchain_image_index: usize) -> Result<()> {
        let color = [1.0;4];
        let rhi = self.m_render_pass.m_base.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        let command_buffer = rhi.get_current_command_buffer();
        rhi.push_event(command_buffer, "maincamerabasepass", color);
        let info = rhi.get_swapchain_info();
        rhi.cmd_set_viewport(command_buffer, 0, slice::from_ref(info.viewport));
        rhi.cmd_set_scissor(command_buffer, 0, slice::from_ref(info.scissor));

        self.draw_object(current_swapchain_image_index)?;

        rhi.pop_event(command_buffer);
        Ok(())
    }
}

#[distributed_slice(VULKAN_RHI_DESCRIPTOR_STORAGE_BUFFER)]
static STORAGE_BUFFER_COUNT: u32 = VulkanRHI::get_max_frames_in_flight() as u32;

impl MainCameraPass {
    fn setup_attachments(&mut self, rhi: &VulkanRHI) -> Result<()> {
        let swapchain_info = rhi.get_swapchain_info();

        self.m_render_pass.m_framebuffer.attachments.resize_with(
            _MAIN_CAMERA_PASS_ATTACHMENT_COUNT + _MAIN_CAMERA_PASS_POST_PROCESS_ATTACHMENT_COUNT,
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

        let mut subpasses = [vk::SubpassDescription::default();1];

        {
            let depth_stencil_attachment_ref = vk::AttachmentReference::builder()
                .attachment(1)
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
            let color_attachment_ref = vk::AttachmentReference::builder()
                .attachment(2)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
            let color_attachments = &[color_attachment_ref];
            subpasses[0] = vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(color_attachments)
                .depth_stencil_attachment(&depth_stencil_attachment_ref)
                .build();
        }  

        {
            
        }

        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | 
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT |
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let dependencies = &[dependency];
        let info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(dependencies);

        self.m_render_pass.m_framebuffer.render_pass = rhi.create_render_pass(&info)?;

        Ok(())
    }

    fn setup_framebuffer(&mut self, rhi: &VulkanRHI) -> Result<()> {
        let swapchain_info = rhi.get_swapchain_info();
        let depth_image_info = rhi.get_depth_image_info();
        let framebuffers =  swapchain_info.image_views
            .iter()
            .map(|i| {
                let attachments = &[
                    self.m_render_pass.m_framebuffer.attachments[_MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD].view,
                    *depth_image_info.image_view,
                    *i,
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
        // self.m_base.m_descriptor_infos.resize_with(LayoutType::LayoutTypeCount as usize, Default::default);
        // {
        //     let mesh_mesh_layout_bindings = [
        //         vk::DescriptorSetLayoutBinding::builder()
        //             .binding(0)
        //             .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
        //             .descriptor_count(1)
        //             .stage_flags(vk::ShaderStageFlags::VERTEX)
        //             .build(),
        //     ];
        //     let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
        //         .bindings(&mesh_mesh_layout_bindings)
        //         .build();
        //     self.m_base.m_descriptor_infos[LayoutType::PerMesh as usize].layout = rhi.create_descriptor_set_layout(&create_info)?;
        // }
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

    fn draw_object(&self, current_swapchain_image_index: usize) -> Result<()> {
        let rhi = self.m_render_pass.m_base.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        let swapchain_info = rhi.get_swapchain_info();
        let command_buffer = rhi.get_current_command_buffer();

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(swapchain_info.extent);

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue{ float32: [0.0, 0.0, 0.0, 1.0] },
            },
            vk::ClearValue{ 
                depth_stencil: vk::ClearDepthStencilValue{depth: 1.0, stencil: 0 },
            },
            vk::ClearValue {
                color: vk::ClearColorValue{ float32: [0.0, 0.0, 0.0, 1.0] },
            },
        ];
        
        let pipeline = &self.m_render_pass.m_render_pipeline[0];
        
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.m_render_pass.m_framebuffer.render_pass)
            .framebuffer(self.m_swapchain_framebuffers[current_swapchain_image_index])
            .render_area(render_area)
            .clear_values(&clear_values);

        rhi.cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::INLINE);
        rhi.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline);
        
        // rhi.cmd_bind_vertex_buffers(command_buffer, 0, &[self.t_test_triangle_buffer], &[0]);
        // rhi.cmd_draw(command_buffer, 3, 1, 0, 0);

        rhi.cmd_end_render_pass(command_buffer);
        Ok(())
    }
}


