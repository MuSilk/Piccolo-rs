use std::{cell::RefCell, mem::offset_of, rc::{Rc, Weak}};
use anyhow::Result;
use linkme::distributed_slice;
use vulkanalia::{prelude::v1_0::*};

use crate::{function::{input::input_system::InputSystem, render::{self, font_atlas::create_ascii_font_texture_rgba, interface::vulkan::vulkan_rhi::{K_MAX_FRAMES_IN_FLIGHT, VULKAN_RHI_DESCRIPTOR_COMBINED_IMAGE_SAMPLER, VulkanRHI}, render_pass::{Descriptor, MainCameraSubPass, RenderPass, RenderPipelineBase}, render_system::RenderSystem, render_type::RHISamplerType, window_system::WindowSystem}, ui::{ui2::{UiDrawCmd, UiDrawList, UiRuntime, UiVertex}, window_ui::WindowUI}}, resource::config_manager::ConfigManager, shader::generated::shader::{UI_FRAG, UI_VERT}};

pub struct UIPassInitInfo<'a>{
    pub render_pass: vk::RenderPass,
    pub rhi: &'a Rc<RefCell<VulkanRHI>>,
    pub config_manager: &'a ConfigManager
}

#[derive(Default)]
struct Texture {
    image: vk::Image,
    view: vk::ImageView,
    memory: vk::DeviceMemory,
}

#[derive(Default)]
pub struct UIPass {
    pub m_render_pass: RenderPass,
    m_window_ui: Option<Weak<RefCell<dyn WindowUI>>>,

    font_texture: Texture,
    renderer_data: [RefCell<RendererData>; K_MAX_FRAMES_IN_FLIGHT],
}

impl UIPass {
    pub fn initialize(&mut self, info: &UIPassInitInfo) -> Result<()> {
        self.m_render_pass.initialize();

        self.font_texture = upload_font_texture(&info.rhi.borrow(), info.config_manager)?;

        self.m_render_pass.m_framebuffer.render_pass = info.render_pass;
        self.setup_descriptor_layout(&info.rhi.borrow())?;
        self.setup_pipelines(&info.rhi.borrow())?;
        self.setup_descriptor_set(&info.rhi.borrow())?;
        self.update_after_framebuffer_recreate(&info.rhi.borrow())?;
        Ok(())
    }
    
    pub fn draw(
        &self, 
        render_system: &RefCell<RenderSystem>,
        window_system: &WindowSystem, 
        input_system: &RefCell<InputSystem>,
        ui_runtime: &RefCell<UiRuntime>
    ) {
        if let Some(window_ui) = self.m_window_ui.as_ref().and_then(|w| w.upgrade()) {
            window_ui.borrow_mut().pre_render(
                render_system, 
                window_system, 
                input_system
            );
        }

        let color = [1.0;4];
        let rhi = self.m_render_pass.m_base.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        let command_buffer = rhi.get_current_command_buffer();
        rhi.push_event(command_buffer, "UI\0", color);

        let mut ui_runtime = ui_runtime.borrow_mut();
        let (_frame, draw_list) = ui_runtime.build_frame(1.0 / 60.0);
        self.render_ui_draw_list(&rhi, &draw_list).unwrap();

        rhi.pop_event(command_buffer);
    }
    
    pub fn update_after_framebuffer_recreate(&mut self, _rhi: &VulkanRHI) -> Result<()> {
        Ok(())
    }

    pub fn initialize_ui_render_backend(&mut self, _window_ui: &Rc<RefCell<dyn WindowUI>>) {
        self.m_window_ui = Some(Rc::downgrade(_window_ui));
    }

    pub fn reload_font_texture(
        &mut self, 
        config_manager: &ConfigManager
    ) -> Result<()> {
        let rhi = self.m_render_pass.m_base.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        if self.font_texture.image != vk::Image::null() {
            rhi.destroy_image_view(self.font_texture.view);
            rhi.destroy_image(self.font_texture.image);
            rhi.free_memory(self.font_texture.memory);
        }
        self.font_texture = upload_font_texture(&rhi, config_manager)?;

        let text_sampler_texture_info = [
            vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(self.font_texture.view)
                .sampler(*rhi.get_or_create_default_sampler(
                    RHISamplerType::Linear
                ).unwrap())
                .build()
        ];

        let descriptor_writes_info = [
            vk::WriteDescriptorSet::builder()
                .dst_set(self.m_render_pass.m_descriptor_infos[0].descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&text_sampler_texture_info)
                .build(),
        ];

        rhi.update_descriptor_sets(&descriptor_writes_info)?;

        Ok(())
    }


    fn render_ui_draw_list(&self, rhi: &VulkanRHI, draw_list: &UiDrawList) -> Result<()> {
        if draw_list.vertices.is_empty() || draw_list.indices.is_empty() || draw_list.commands.is_empty() {
            return Ok(());
        }

        let data = &mut self.renderer_data[rhi.get_current_frame_index()].borrow_mut();
        let vertex_size = draw_list.vertices.len() * std::mem::size_of::<UiVertex>();
        let index_size = draw_list.indices.len() * std::mem::size_of::<u32>();
        data.update_vertex_buffer(rhi, vertex_size)?;
        data.update_index_buffer(rhi, index_size)?;

        let vertex_ptr = rhi.map_memory(data.vertex_buffer_memory, 0, vertex_size as u64, vk::MemoryMapFlags::empty())?;
        let index_ptr = rhi.map_memory(data.index_buffer_memory, 0, index_size as u64, vk::MemoryMapFlags::empty())?;

        unsafe {
            std::ptr::copy_nonoverlapping(
                draw_list.vertices.as_ptr(),
                vertex_ptr as *mut UiVertex,
                draw_list.vertices.len(),
            );
            std::ptr::copy_nonoverlapping(
                draw_list.indices.as_ptr(),
                index_ptr as *mut u32,
                draw_list.indices.len(),
            );
        }

        rhi.unmap_memory(data.vertex_buffer_memory);
        rhi.unmap_memory(data.index_buffer_memory);

        let command_buffer = rhi.get_current_command_buffer();
        let swapchain_info = rhi.get_swapchain_info();
        let fb_width = swapchain_info.extent.width as f32;
        let fb_height = swapchain_info.extent.height as f32;
        if fb_width <= 0.0 || fb_height <= 0.0 {
            return Ok(());
        }

        rhi.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.m_render_pass.m_render_pipeline[0].pipeline);
        rhi.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.m_render_pass.m_render_pipeline[0].layout,
            0,
            &[self.m_render_pass.m_descriptor_infos[0].descriptor_set],
            &[],
        );
        rhi.cmd_bind_vertex_buffers(command_buffer, 0, &[data.vertex_buffer], &[0]);
        rhi.cmd_bind_index_buffer(command_buffer, data.index_buffer, 0, vk::IndexType::UINT32);
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: fb_width,
            height: fb_height,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        rhi.cmd_set_viewport(command_buffer, 0, &[viewport]);
        let transform = [2.0 / fb_width, 2.0 / fb_height, -1.0, -1.0];
        rhi.cmd_push_constants(
            command_buffer,
            self.m_render_pass.m_render_pipeline[0].layout,
            vk::ShaderStageFlags::VERTEX,
            0,
            unsafe {
                std::slice::from_raw_parts(
                    transform.as_ptr() as *const u8,
                    transform.len() * std::mem::size_of::<f32>(),
                )
            }
        );

        for cmd in draw_list.commands.iter() {
            match cmd {
                UiDrawCmd::DrawIndexed {
                    first_index,
                    index_count,
                    vertex_offset,
                    clip_rect,
                    texture_id,
                } => {
                    let _ = texture_id;
                    let scissor = vk::Rect2D {
                        offset: vk::Offset2D {
                            x: clip_rect[0] as i32,
                            y: clip_rect[1] as i32,
                        },
                        extent: vk::Extent2D {
                            width: (clip_rect[2] - clip_rect[0]).max(0.0) as u32,
                            height: (clip_rect[3] - clip_rect[1]).max(0.0) as u32,
                        },
                    };
                    rhi.cmd_set_scissor(command_buffer, 0, &[scissor]);
                    rhi.cmd_draw_indexed(
                        command_buffer,
                        *index_count,
                        1,
                        *first_index,
                        *vertex_offset,
                        0,
                    );
                }
            }
        }

        Ok(())
    }
}

fn use_ui2_backend() -> bool {
    true
}

#[distributed_slice(VULKAN_RHI_DESCRIPTOR_COMBINED_IMAGE_SAMPLER)]
static COMBINED_IMAGE_SAMPLER_COUNT: u32 = 1;

impl UIPass {
    fn setup_descriptor_layout(&mut self, rhi: &VulkanRHI) -> Result<()> {
        self.m_render_pass.m_descriptor_infos.clear();
        let text_texture_binding = [
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];
        let text_texture_binding_layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&text_texture_binding);
        self.m_render_pass.m_descriptor_infos.push(Descriptor {
            layout: rhi.create_descriptor_set_layout(&text_texture_binding_layout_create_info)?,
            descriptor_set: Default::default(),
        });

        Ok(())
    }

    fn setup_pipelines(&mut self, rhi: &VulkanRHI) -> Result<()> {
        self.m_render_pass.m_render_pipeline.clear();
        let vert_shader_module = rhi.create_shader_module(&UI_VERT)?;
        let frag_shader_module = rhi.create_shader_module(&UI_FRAG)?;

        let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(b"main\0");

        let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(b"main\0");

        let binding = ImguiDrawVertex::get_binding_descriptions();
        let attribute = ImguiDrawVertex::get_attribute_descriptions();
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&binding)
            .vertex_attribute_descriptions(&attribute);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::_1);

        let attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::all())
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD);

        let attachments = &[attachment];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]);

        let set_layouts = &[self.m_render_pass.m_descriptor_infos[0].layout];
        let push_constant_ranges = [
            vk::PushConstantRange::builder()
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .offset(0)
                .size(4 * std::mem::size_of::<f32>() as u32)
        ];
        let layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(set_layouts)
            .push_constant_ranges(&push_constant_ranges);

        let pipeline_layout = rhi.create_pipeline_layout(&layout_info)?;

        let stages = &[vert_stage, frag_stage];
        let info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(self.m_render_pass.m_framebuffer.render_pass)
            .subpass(MainCameraSubPass::UI as u32)
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
        let set_layouts = [self.m_render_pass.m_descriptor_infos[0].layout];
        let post_process_global_descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(rhi.get_descriptor_pool())
            .set_layouts(&set_layouts);

        self.m_render_pass.m_descriptor_infos[0].descriptor_set = rhi.allocate_descriptor_sets(&post_process_global_descriptor_set_alloc_info)?[0];

        let text_sampler_texture_info = [
            vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(self.font_texture.view)
                .sampler(*rhi.get_or_create_default_sampler(
                    RHISamplerType::Linear
                ).unwrap())
                .build()
        ];

        let descriptor_writes_info = [
            vk::WriteDescriptorSet::builder()
                .dst_set(self.m_render_pass.m_descriptor_infos[0].descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&text_sampler_texture_info)
                .build(),
        ];

        rhi.update_descriptor_sets(&descriptor_writes_info)?;
        Ok(())
    }
}

#[derive(Default)]
pub struct RendererData {
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    vertex_buffer_size: usize,

    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,
    index_buffer_size: usize,
}

impl RendererData {
    fn update_vertex_buffer(&mut self, rhi: &VulkanRHI, data_size: usize) -> Result<()> {
        if data_size > self.vertex_buffer_size {
            let data_size = data_size.next_power_of_two();
            rhi.destroy_buffer(self.vertex_buffer);
            rhi.free_memory(self.vertex_buffer_memory);
            (self.vertex_buffer, self.vertex_buffer_memory) = rhi.create_buffer(
                data_size as u64,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk::MemoryPropertyFlags::DEVICE_LOCAL | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;
            self.vertex_buffer_size = data_size;
            
        }
        Ok(())
    }

    fn update_index_buffer(&mut self, rhi: &VulkanRHI, data_size: usize) -> Result<()> { 
        if data_size > self.index_buffer_size {
            let data_size = data_size.next_power_of_two();
            rhi.destroy_buffer(self.index_buffer);
            rhi.free_memory(self.index_buffer_memory);
            (self.index_buffer, self.index_buffer_memory)= rhi.create_buffer(
                data_size as u64,
                vk::BufferUsageFlags::INDEX_BUFFER,
                vk::MemoryPropertyFlags::DEVICE_LOCAL | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;
            self.index_buffer_size = data_size;
        }
        Ok(())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
struct ImguiDrawVertex {
    pos: [f32; 2],
    uv: [f32; 2],
    col: [u8; 4],
}

impl ImguiDrawVertex {
    pub fn get_binding_descriptions() -> [vk::VertexInputBindingDescription; 1]{
        [
            vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<ImguiDrawVertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
        ]
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3]{
        [
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(ImguiDrawVertex, pos) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(ImguiDrawVertex, uv) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(2)
                .format(vk::Format::R8G8B8A8_UNORM)
                .offset(offset_of!(ImguiDrawVertex, col) as u32)
                .build(),
        ]
    }
}

fn upload_font_texture(
    rhi: &VulkanRHI,
    config_manager: &ConfigManager
) -> Result<Texture> {
    let font_path = config_manager.get_editor_font_path().to_path_buf();
    let (texture_image, texture_image_memory, texture_image_view) =
        create_ascii_font_texture_rgba(rhi, font_path.as_path())?;
    Ok(Texture { image: texture_image, view: texture_image_view, memory: texture_image_memory })
}