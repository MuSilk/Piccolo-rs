use std::{cell::RefCell, rc::{Rc, Weak}, slice, sync::{Mutex}};

use anyhow::Result;
use nalgebra_glm::{Mat4, Vec3, Vec4};
use vulkanalia::{prelude::v1_0::*};

use crate::runtime::function::render::{debugdraw::{debug_draw_buffer::DebugDrawAllocator, debug_draw_context::DebugDrawContext, debug_draw_font::DebugDrawFont, debug_draw_group::DebugDrawGroup, debug_draw_pipeline::{DebugDrawPipeline, DebugDrawPipelineType}, debug_draw_primitive::{FillMode, K_DEBUG_DRAW_ONE_FRAME}}, interface::vulkan::vulkan_rhi::VulkanRHI, render_resource::RenderResource};

#[derive(Default)]
pub struct DebugDrawManagerBase {
    m_point_start_offset: usize,
    m_point_end_offset: usize,
    m_line_start_offset: usize,
    m_line_end_offset: usize,
    m_triangle_start_offset: usize,
    m_triangle_end_offset: usize,
    m_no_depth_test_point_start_offset: usize,
    m_no_depth_test_point_end_offset: usize,
    m_no_depth_test_line_start_offset: usize,
    m_no_depth_test_line_end_offset: usize,
    m_no_depth_test_triangle_start_offset: usize,
    m_no_depth_test_triangle_end_offset: usize,
    m_text_start_offset: usize,
    m_text_end_offset: usize,
}

pub struct DebugDrawManager {
    m_mutex : Mutex<()>,
    m_rhi : Weak<RefCell<VulkanRHI>>,

    m_debug_draw_pipelines: [DebugDrawPipeline; DebugDrawPipelineType::EnumCount as usize],
    m_buffer_allocator: DebugDrawAllocator,
    m_debug_context : DebugDrawContext,
    m_debug_draw_group_for_render: DebugDrawGroup,
    m_font: DebugDrawFont,
    m_proj_view_matrix: Mat4,

    m_base : DebugDrawManagerBase,
}

impl DebugDrawManager {
    pub fn create(rhi: &Rc<RefCell<VulkanRHI>>) -> Result<Self> {
        let m_rhi = Rc::downgrade(rhi);
        let m_font = DebugDrawFont::create(&rhi.borrow())?;
        let m_buffer_allocator = DebugDrawAllocator::create(rhi,&m_font)?;
        let set_layout = m_buffer_allocator.get_descriptor_set_layout();
        let m_debug_draw_pipelines = [
            DebugDrawPipeline::create(DebugDrawPipelineType::Point, rhi, set_layout)?,
            DebugDrawPipeline::create(DebugDrawPipelineType::Line, rhi, set_layout)?,
            DebugDrawPipeline::create(DebugDrawPipelineType::Triangle, rhi, set_layout)?,
            DebugDrawPipeline::create(DebugDrawPipelineType::PointNoDepthTest, rhi, set_layout)?,
            DebugDrawPipeline::create(DebugDrawPipelineType::LineNoDepthTest, rhi, set_layout)?,
            DebugDrawPipeline::create(DebugDrawPipelineType::TriangleNoDepthTest, rhi, set_layout)?,
        ];
        Ok(Self { 
            m_mutex: Mutex::new(()),
            m_rhi: m_rhi,
            m_debug_draw_pipelines,
            m_buffer_allocator,
            m_debug_context: DebugDrawContext::default(),
            m_debug_draw_group_for_render : DebugDrawGroup::default(),
            m_font,
            m_proj_view_matrix: Mat4::identity(),
            m_base: DebugDrawManagerBase::default(),
        })
    }

    pub fn prepare_pass_data(&mut self,render_resource: &RenderResource){
        self.m_proj_view_matrix = render_resource.m_mesh_perframe_storage_buffer_object.proj_view_matrix;
    }

    pub fn destroy(&mut self){
        for pipeline in self.m_debug_draw_pipelines.iter_mut(){
            pipeline.destroy();
        }
        self.m_buffer_allocator.destroy();
        self.m_font.destroy();
    }

    pub fn clear(&mut self){
        let _guard = self.m_mutex.lock().unwrap();
        self.m_debug_context.clear();
    }

    pub fn tick(&mut self, delta_time: f32){
        let group = self.try_get_or_create_debug_draw_group("test");
        static mut TOTAL_TIME: f32 = 0.0;
        unsafe{TOTAL_TIME += delta_time;}
        group.borrow_mut().add_quad(
            &Vec3::new(0.5,0.5,0.0),
            &Vec3::new(-0.5,0.5,0.0),
            &Vec3::new(-0.5,-0.5,0.0),
            &Vec3::new(0.5,-0.5,0.0),
            &Vec4::new(1.0, 0.0, 0.0, 1.0),
            &Vec4::new(0.0, 1.0, 0.0, 1.0),
            &Vec4::new(0.0, 0.0, 1.0, 1.0),
            &Vec4::new(1.0, 1.0, 1.0, 1.0),
            K_DEBUG_DRAW_ONE_FRAME,
            true,
            FillMode::Solid
        );
        group.borrow_mut().add_sphere(
            &Vec3::new(0.5, 0.0, 0.0), 
            unsafe{TOTAL_TIME.sin()} * 0.5, 
            &Vec4::new(unsafe{TOTAL_TIME.sin()} , 0.0, unsafe{TOTAL_TIME.cos()} , 1.0), 
            K_DEBUG_DRAW_ONE_FRAME, 
            true
        );
        group.borrow_mut().add_sphere(
            &Vec3::new(0.0, 0.5, 0.0), 
            unsafe{TOTAL_TIME.sin()} * 0.5, 
            &Vec4::new(unsafe{TOTAL_TIME.sin()} , 0.0, unsafe{TOTAL_TIME.cos()} , 1.0), 
            K_DEBUG_DRAW_ONE_FRAME, 
            true
        );
        group.borrow_mut().add_sphere(
            &Vec3::new(0.0, 0.0, 0.5), 
            unsafe{TOTAL_TIME.sin()} * 0.5, 
            &Vec4::new(unsafe{TOTAL_TIME.sin()} , 0.0, unsafe{TOTAL_TIME.cos()} , 1.0), 
            K_DEBUG_DRAW_ONE_FRAME, 
            true
        );
        let _guard = self.m_mutex.lock().unwrap();
        self.m_buffer_allocator.tick();
        self.m_debug_context.tick(delta_time);
    }

    pub fn update_after_recreate_swap_chain(&mut self, rhi: &VulkanRHI){
        for pipeline in self.m_debug_draw_pipelines.iter_mut(){
            pipeline.recreate_after_swapchain(rhi).unwrap();
        }
    }

    pub fn try_get_or_create_debug_draw_group(&mut self, name: &str) -> &RefCell<DebugDrawGroup> {
        let _guard = self.m_mutex.lock().unwrap();
        self.m_debug_context.try_get_or_create_debug_draw_group(name)
    }

    pub fn draw(&mut self, current_swapchain_image_index: usize) -> Result<()>{ 
        self.swap_data_to_render();
        let color = [1.0;4];
        let rhi = self.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        let command_buffer = rhi.get_current_command_buffer();
        rhi.push_event(command_buffer, "DebugDrawManager", color);
        let info = rhi.get_swapchain_info();
        rhi.cmd_set_viewport(command_buffer, 0, slice::from_ref(info.viewport));
        rhi.cmd_set_scissor(command_buffer, 0, slice::from_ref(info.scissor));

        self.draw_debug_object(current_swapchain_image_index)?;

        rhi.pop_event(command_buffer);
        Ok(())
    }
}

impl DebugDrawManager {
    fn swap_data_to_render(&mut self){
        let _guard = self.m_mutex.lock().unwrap();

        self.m_debug_draw_group_for_render.clear();
        self.m_debug_context.m_debug_draw_groups.iter().for_each(|group|{
            self.m_debug_draw_group_for_render.merge_from(&group.borrow());
        });
    }

    fn draw_debug_object(&mut self, current_swapchain_image_index: usize) -> Result<()>{
        self.prepare_draw_buffer()?;

        self.draw_point_line_triangle_box(current_swapchain_image_index);
        self.draw_wire_frame_object(current_swapchain_image_index)?;
        Ok(())
    }

    fn prepare_draw_buffer(&mut self) -> Result<()>{
        self.m_buffer_allocator.clear();

        let vertices = self.m_debug_draw_group_for_render.write_point_data(false);
        self.m_base.m_point_start_offset = self.m_buffer_allocator.cache_vertices(&vertices);
        self.m_base.m_point_end_offset = self.m_buffer_allocator.get_vertex_cache_offset();

        let vertices = self.m_debug_draw_group_for_render.write_line_data(false);
        self.m_base.m_line_start_offset = self.m_buffer_allocator.cache_vertices(&vertices);
        self.m_base.m_line_end_offset = self.m_buffer_allocator.get_vertex_cache_offset();

        let vertices = self.m_debug_draw_group_for_render.write_triangle_data(false);
        self.m_base.m_triangle_start_offset = self.m_buffer_allocator.cache_vertices(&vertices);
        self.m_base.m_triangle_end_offset = self.m_buffer_allocator.get_vertex_cache_offset();

        let vertices = self.m_debug_draw_group_for_render.write_point_data(true);
        self.m_base.m_no_depth_test_point_start_offset = self.m_buffer_allocator.cache_vertices(&vertices);
        self.m_base.m_no_depth_test_point_end_offset = self.m_buffer_allocator.get_vertex_cache_offset();

        let vertices = self.m_debug_draw_group_for_render.write_line_data(true);
        self.m_base.m_no_depth_test_line_start_offset = self.m_buffer_allocator.cache_vertices(&vertices);
        self.m_base.m_no_depth_test_line_end_offset = self.m_buffer_allocator.get_vertex_cache_offset();

        let vertices = self.m_debug_draw_group_for_render.write_triangle_data(true);
        self.m_base.m_no_depth_test_triangle_start_offset = self.m_buffer_allocator.cache_vertices(&vertices);
        self.m_base.m_no_depth_test_triangle_end_offset = self.m_buffer_allocator.get_vertex_cache_offset();

        let rhi = self.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        let swap_chain_desc = rhi.get_swapchain_info();
        let screen_width = swap_chain_desc.viewport.width;
        let screen_height = swap_chain_desc.viewport.height;

        let vertices = self.m_debug_draw_group_for_render.write_text_data(&self.m_font, &self.m_proj_view_matrix, screen_width, screen_height);
        self.m_base.m_text_start_offset = self.m_buffer_allocator.cache_vertices(&vertices);
        self.m_base.m_text_end_offset = self.m_buffer_allocator.get_vertex_cache_offset();

        self.m_buffer_allocator.cache_uniform_object(&self.m_proj_view_matrix);

        let dynamic_objects = self.m_debug_draw_group_for_render.write_uniform_dynamic_data_to_cache();
        self.m_buffer_allocator.cache_uniform_dynamic_object(&dynamic_objects);

        self.m_buffer_allocator.allocator()?;
        Ok(())
    }

    fn draw_point_line_triangle_box(&self, current_swapchain_image_index: usize){
        let vertex_buffers = [self.m_buffer_allocator.get_vertex_buffer()];
        let rhi = self.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        let swapchain_info = rhi.get_swapchain_info();
        let command_buffer = rhi.get_current_command_buffer();
        rhi.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &[0]);

        let vc_pipelines = [
            DebugDrawPipelineType::Point as usize,
            DebugDrawPipelineType::Line as usize,
            DebugDrawPipelineType::Triangle as usize,
            DebugDrawPipelineType::PointNoDepthTest as usize,
            DebugDrawPipelineType::LineNoDepthTest as usize,
            DebugDrawPipelineType::TriangleNoDepthTest as usize,
            DebugDrawPipelineType::TriangleNoDepthTest as usize,
        ];

        let vc_start_offsets = [
            self.m_base.m_point_start_offset,
            self.m_base.m_line_start_offset,
            self.m_base.m_triangle_start_offset,
            self.m_base.m_no_depth_test_point_start_offset,
            self.m_base.m_no_depth_test_line_start_offset,
            self.m_base.m_no_depth_test_triangle_start_offset,
            self.m_base.m_text_start_offset
        ];

        let vc_end_offsets = [
            self.m_base.m_point_end_offset,
            self.m_base.m_line_end_offset,
            self.m_base.m_triangle_end_offset,
            self.m_base.m_no_depth_test_point_end_offset,
            self.m_base.m_no_depth_test_line_end_offset,
            self.m_base.m_no_depth_test_triangle_end_offset,
            self.m_base.m_text_end_offset
        ];

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
        ];

        for i in 0..vc_pipelines.len() {
            if vc_end_offsets[i] <= vc_start_offsets[i] {
                continue;
            }
            let pipeline = &self.m_debug_draw_pipelines[vc_pipelines[i]];
            
            let info = vk::RenderPassBeginInfo::builder()
                .render_pass(pipeline.get_framebuffer().render_pass)
                .framebuffer(pipeline.get_framebuffer().framebuffers[current_swapchain_image_index])
                .render_area(render_area)
                .clear_values(&clear_values);

            rhi.cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::INLINE);
            rhi.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline.get_pipeline().pipeline);
            
            rhi.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.get_pipeline().layout,
                0, 
                &[*self.m_buffer_allocator.get_descriptor_set()], 
                &[0]
            );
            rhi.cmd_draw(command_buffer, (vc_end_offsets[i] - vc_start_offsets[i]) as u32, 1, vc_start_offsets[i] as u32, 0);
            rhi.cmd_end_render_pass(command_buffer);
        }
    }

    fn draw_wire_frame_object(&mut self, current_swapchain_image_index: usize) -> Result<()>{
        let vertex_buffers = [self.m_buffer_allocator.get_sphere_vertex_buffer()?];
        let rhi = self.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        let swapchain_info = rhi.get_swapchain_info();
        let command_buffer = rhi.get_current_command_buffer();
        rhi.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &[0]);

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
        ];

        let vc_pipelines = [
            DebugDrawPipelineType::Line as usize,
            DebugDrawPipelineType::LineNoDepthTest as usize,
        ];

        let no_depth_tests = [
            false,
            true,
        ];
        
        let uniform_dynamic_size = DebugDrawAllocator::get_size_of_uniform_buffer_dynamic_object() as u32;
        let mut dynamic_offset = 0;

        for i in 0..vc_pipelines.len() {
            let pipeline = &self.m_debug_draw_pipelines[vc_pipelines[i]];
            let no_depth_test = no_depth_tests[i];
            let info = vk::RenderPassBeginInfo::builder()
                .render_pass(pipeline.get_framebuffer().render_pass)
                .framebuffer(pipeline.get_framebuffer().framebuffers[current_swapchain_image_index])
                .render_area(render_area)
                .clear_values(&clear_values);

            rhi.cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::INLINE);
            rhi.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline.get_pipeline().pipeline);

            let sphere_count = self.m_debug_draw_group_for_render.get_sphere_count(no_depth_test);
            if sphere_count > 0 {    
                for _i in 0..sphere_count {
                    rhi.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline.get_pipeline().layout,
                        0, 
                        &[*self.m_buffer_allocator.get_descriptor_set()], 
                        &[dynamic_offset]
                    );
                    dynamic_offset += uniform_dynamic_size;
                    rhi.cmd_draw(command_buffer, DebugDrawAllocator::get_sphere_vertex_buffer_size() as u32, 1, 0, 0);
                }
            }    
            let cylinder_count = self.m_debug_draw_group_for_render.get_cylinder_count(no_depth_test);
            if cylinder_count > 0 {
                for _i in 0..cylinder_count {
                    rhi.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline.get_pipeline().layout,
                        0, 
                        &[*self.m_buffer_allocator.get_descriptor_set()], 
                        &[dynamic_offset]
                    );
                    dynamic_offset += uniform_dynamic_size;
                    rhi.cmd_draw(command_buffer, DebugDrawAllocator::get_cylinder_vertex_buffer_size() as u32, 1, 0, 0);
                } 
            }      
            let capsule_count = self.m_debug_draw_group_for_render.get_capsule_count(no_depth_test);
            if capsule_count > 0 {
                for _i in 0..capsule_count {
                    rhi.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline.get_pipeline().layout,
                        0, 
                        &[*self.m_buffer_allocator.get_descriptor_set()], 
                        &[dynamic_offset]
                    );
                    dynamic_offset += uniform_dynamic_size;
                    rhi.cmd_draw(command_buffer, DebugDrawAllocator::get_capsule_vertex_buffer_up_size() as u32, 1, 0, 0);
                     rhi.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline.get_pipeline().layout,
                        0, 
                        &[*self.m_buffer_allocator.get_descriptor_set()], 
                        &[dynamic_offset]
                    );
                    dynamic_offset += uniform_dynamic_size;
                    rhi.cmd_draw(command_buffer, DebugDrawAllocator::get_capsule_vertex_buffer_mid_size() as u32, 1, 0, 0);
                     rhi.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline.get_pipeline().layout,
                        0, 
                        &[*self.m_buffer_allocator.get_descriptor_set()], 
                        &[dynamic_offset]
                    );
                    dynamic_offset += uniform_dynamic_size;
                    rhi.cmd_draw(command_buffer, DebugDrawAllocator::get_capsule_vertex_buffer_down_size() as u32, 1, 0, 0);
                } 
            }  
            rhi.cmd_end_render_pass(command_buffer);    
        }
        
        Ok(())
    }
}