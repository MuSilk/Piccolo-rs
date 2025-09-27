use std::{cell::RefCell, rc::Rc, slice, sync::{Arc, Mutex, Weak}};

use anyhow::Result;
use nalgebra_glm::{Mat4, Vec4};

use crate::runtime::function::{global::global_context::RuntimeGlobalContext, render::{debugdraw::{debug_draw_buffer::DebugDrawAllocator, debug_draw_context::DebugDrawContext, debug_draw_font::DebugDrawFont, debug_draw_group::DebugDrawGroup, debug_draw_pipeline::{DebugDrawPipeline, DebugDrawPipelineType}}, interface::{rhi::RHI, rhi_struct::{RHIClearColorValue, RHIClearDepthStencilValue, RHIClearValue, RHIOffset2D, RHIRect2D, RHIRenderPassBeginInfo}}, render_resource::RenderResource, render_type::{RHIPipelineBindPoint, RHISubpassContents}}};

#[derive(Default)]
pub struct DebugDrawManager {
    m_mutex : Mutex<()>,
    m_rhi : Weak<Mutex<Box<dyn RHI>>>,

    m_debug_draw_pipeline: [DebugDrawPipeline; DebugDrawPipelineType::EnumCount as usize],
    m_buffer_allocator: DebugDrawAllocator,
    m_debug_context : DebugDrawContext,
    m_debug_draw_group_for_render: DebugDrawGroup,
    m_font: Rc<RefCell<DebugDrawFont>>,
    m_proj_view_matrix: Mat4,

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

unsafe impl Send for DebugDrawManager {}
unsafe impl Sync for DebugDrawManager {}

impl DebugDrawManager {
    pub fn initialize(&mut self) -> Result<()>{
        self.m_rhi = Arc::downgrade(&RuntimeGlobalContext::global().m_render_system.lock().unwrap().get_rhi());
        self.setup_pipelines()?;
        Ok(())
    }

    pub fn setup_pipelines(&mut self) -> Result<()> {
        for i in 0..DebugDrawPipelineType::EnumCount as usize {
            self.m_debug_draw_pipeline[i].m_pipeline_type = DebugDrawPipelineType::from_u8(i as u8).unwrap();
            self.m_debug_draw_pipeline[i].initialize()?;
        }
        self.m_font.borrow_mut().initialize();
        self.m_buffer_allocator.initialize(&self.m_font)?;
        Ok(())
    }

    pub fn prepare_pass_data(&mut self,render_resource: &RenderResource){
        self.m_proj_view_matrix = render_resource.m_mesh_preframe_storage_buffer_object.proj_view_matrix;
    }

    pub fn destroy(&mut self){
        for pipeline in self.m_debug_draw_pipeline.iter_mut(){
            pipeline.destroy();
        }
        self.m_buffer_allocator.destroy();
        self.m_font.borrow_mut().destroy();
    }

    pub fn clear(&mut self){
        let _guard = self.m_mutex.lock().unwrap();
        self.m_debug_context.clear();
    }

    pub fn tick(&mut self,delta_time:f32){
        let _guard = self.m_mutex.lock().unwrap();
        self.m_buffer_allocator.tick();
        self.m_debug_context.tick(delta_time);
    }

    pub fn update_after_recreate_swap_chain(&mut self){
        for pipeline in self.m_debug_draw_pipeline.iter_mut(){
            pipeline.recreate_after_swapchain();
        }
    }

    pub fn try_get_or_create_debug_draw_group(&mut self, name: &str) -> &DebugDrawGroup {
        let _guard = self.m_mutex.lock().unwrap();
        self.m_debug_context.try_get_or_create_debug_draw_group(name)
    }

    pub fn draw(&mut self, current_swap_chain_image_index: usize) -> Result<()>{ 
        self.swap_data_to_render();
        let color = [1.0;4];
        let rhi = self.m_rhi.upgrade().unwrap();
        let rhi = rhi.lock().unwrap();
        let command_buffer = rhi.get_current_command_buffer();
        rhi.push_event(command_buffer, "DebugDrawManager", color);
        let info = rhi.get_swap_chain_info();
        rhi.cmd_set_viewport(command_buffer, 0, slice::from_ref(info.viewport));
        rhi.cmd_set_scissor(command_buffer, 0, slice::from_ref(info.scissor));

        self.draw_debug_object(current_swap_chain_image_index)?;

        rhi.pop_event(command_buffer);
        Ok(())
    }
}

impl DebugDrawManager {
    fn swap_data_to_render(&mut self){
        let _guard = self.m_mutex.lock().unwrap();

        self.m_debug_draw_group_for_render.clear();
        self.m_debug_context.m_debug_draw_groups.iter().for_each(|group|{
            self.m_debug_draw_group_for_render.merge_from(group);
        });
    }

    fn draw_debug_object(&mut self, current_swap_chain_image_index: usize) -> Result<()>{
        self.prepare_draw_buffer()?;
        self.draw_point_line_triangle_box(current_swap_chain_image_index);
        self.draw_wire_frame_object(current_swap_chain_image_index)?;
        Ok(())
    }

    fn prepare_draw_buffer(&mut self) -> Result<()>{
        self.m_buffer_allocator.clear();

        let mut vertices = Vec::new();
        vertices.extend_from_slice(&self.m_debug_draw_group_for_render.write_point_data(false));
        self.m_point_start_offset = self.m_buffer_allocator.cache_vertices(&vertices);
        self.m_point_end_offset = self.m_buffer_allocator.get_vertex_cache_offset();

        vertices.extend_from_slice(&self.m_debug_draw_group_for_render.write_line_data(false));
        self.m_line_start_offset = self.m_buffer_allocator.cache_vertices(&vertices);
        self.m_line_end_offset = self.m_buffer_allocator.get_vertex_cache_offset();

        vertices.extend_from_slice(&self.m_debug_draw_group_for_render.write_triangle_data(false));
        self.m_triangle_start_offset = self.m_buffer_allocator.cache_vertices(&vertices);
        self.m_triangle_end_offset = self.m_buffer_allocator.get_vertex_cache_offset();

        vertices.extend_from_slice(&self.m_debug_draw_group_for_render.write_point_data(true));
        self.m_point_start_offset = self.m_buffer_allocator.cache_vertices(&vertices);
        self.m_point_end_offset = self.m_buffer_allocator.get_vertex_cache_offset();

        vertices.extend_from_slice(&self.m_debug_draw_group_for_render.write_line_data(true));
        self.m_line_start_offset = self.m_buffer_allocator.cache_vertices(&vertices);
        self.m_line_end_offset = self.m_buffer_allocator.get_vertex_cache_offset();

        vertices.extend_from_slice(&self.m_debug_draw_group_for_render.write_triangle_data(true));
        self.m_triangle_start_offset = self.m_buffer_allocator.cache_vertices(&vertices);
        self.m_triangle_end_offset = self.m_buffer_allocator.get_vertex_cache_offset();

        let font = self.m_font.borrow();
        vertices.extend_from_slice(&&self.m_debug_draw_group_for_render.write_text_data(&font, &self.m_proj_view_matrix));
        self.m_text_start_offset = self.m_buffer_allocator.cache_vertices(&vertices);
        self.m_text_end_offset = self.m_buffer_allocator.get_vertex_cache_offset();

        self.m_buffer_allocator.cache_uniform_object(&self.m_proj_view_matrix);

        let dynamic_object = [
            (Mat4::identity(), Vec4::new(0.0,0.0,0.0,0.0))
        ];
        self.m_buffer_allocator.cache_uniform_dynamic_object(&dynamic_object);

        self.m_buffer_allocator.allocator()?;
        Ok(())
    }

    fn draw_point_line_triangle_box(&self, current_swap_chain_image_index: usize){
        let vertex_buffers = [self.m_buffer_allocator.get_vertex_buffer()];
        if vertex_buffers[0].is_none(){
            return ;
        }
        let vertex_buffers = [vertex_buffers[0].unwrap()];
        let rhi = self.m_rhi.upgrade().unwrap();
        let rhi = rhi.lock().unwrap();
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
            self.m_point_start_offset,
            self.m_line_start_offset,
            self.m_triangle_start_offset,
            self.m_no_depth_test_point_start_offset,
            self.m_no_depth_test_line_start_offset,
            self.m_no_depth_test_triangle_start_offset,
            self.m_text_start_offset
        ];

        let vc_end_offsets = [
            self.m_point_end_offset,
            self.m_line_end_offset,
            self.m_triangle_end_offset,
            self.m_no_depth_test_point_end_offset,
            self.m_no_depth_test_line_end_offset,
            self.m_no_depth_test_triangle_end_offset,
            self.m_text_end_offset
        ];

        let clear_value = [
            RHIClearValue::Color(RHIClearColorValue{float32:[0.0, 0.0, 0.0, 0.0]}),
            RHIClearValue::DepthStencil(RHIClearDepthStencilValue{depth: 1.0, stencil: 0}),
        ];

        for i in 0..vc_pipelines.len() {
            if vc_end_offsets[i] <= vc_start_offsets[i] {
                continue;
            }
            let pipeline = &self.m_debug_draw_pipeline[vc_pipelines[i]];
            let render_pass = &pipeline.get_framebuffer().render_pass;
            let framebuffer = &pipeline.get_framebuffer().framebuffers[current_swap_chain_image_index];
            
            let render_pass_begin_info = RHIRenderPassBeginInfo{
                render_pass: render_pass.as_ref(),
                framebuffer: framebuffer.as_ref(),
                render_area: RHIRect2D{offset: RHIOffset2D{x: 0, y: 0}, extent: rhi.get_swap_chain_info().extent},
                clear_values: &clear_value,
            };

            rhi.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, RHISubpassContents::INLINE);
            rhi.cmd_bind_pipeline(command_buffer, RHIPipelineBindPoint::GRAPHICS, &pipeline.get_pipeline().pipeline);
            
            rhi.cmd_bind_descriptor_sets(
                command_buffer,
                RHIPipelineBindPoint::GRAPHICS,
                &pipeline.get_pipeline().layout,
                0, 
                &[self.m_buffer_allocator.get_descriptor_set()], 
                &[0]
            );
            rhi.cmd_draw(command_buffer, (vc_end_offsets[i] - vc_start_offsets[i]) as u32, 1, vc_start_offsets[i] as u32, 0);
            rhi.cmd_end_render_pass(command_buffer);
        }
    }

    fn draw_wire_frame_object(&mut self, current_swap_chain_image_index: usize) -> Result<()>{
        let rhi = self.m_rhi.upgrade().unwrap();
        let rhi = rhi.lock().unwrap();
        let command_buffer = rhi.get_current_command_buffer();

        let vc_pipelines = [
            DebugDrawPipelineType::Line as usize,
            DebugDrawPipelineType::LineNoDepthTest as usize,
        ];

        let no_depth_tests = [
            true,
            false,
        ];

        let clear_value = [
            RHIClearValue::Color(RHIClearColorValue{float32:[0.0, 0.0, 0.0, 0.0]}),
            RHIClearValue::DepthStencil(RHIClearDepthStencilValue{depth: 1.0, stencil: 0}),
        ];

        for i in 0..vc_pipelines.len() {
            let no_depth_test = no_depth_tests[i];
            let pipeline = &self.m_debug_draw_pipeline[vc_pipelines[i]];
            let render_pass = &pipeline.get_framebuffer().render_pass;
            let framebuffer = &pipeline.get_framebuffer().framebuffers[current_swap_chain_image_index];
            
            let render_pass_begin_info = RHIRenderPassBeginInfo{
                render_pass: render_pass.as_ref(),
                framebuffer: framebuffer.as_ref(),
                render_area: RHIRect2D{offset: RHIOffset2D{x: 0, y: 0}, extent: rhi.get_swap_chain_info().extent},
                clear_values: &clear_value,
            };

            rhi.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, RHISubpassContents::INLINE);
            rhi.cmd_bind_pipeline(command_buffer, RHIPipelineBindPoint::GRAPHICS, &pipeline.get_pipeline().pipeline);

            let uniform_dynamic_size = DebugDrawAllocator::get_size_of_uniform_buffer_object() as u32;
            let mut dynamic_offset = uniform_dynamic_size;

            let sphere_count = self.m_debug_draw_group_for_render.get_sphere_count(no_depth_test);
            let cylinder_count = self.m_debug_draw_group_for_render.get_cylinder_count(no_depth_test);
            let capsule_count = self.m_debug_draw_group_for_render.get_capsule_count(no_depth_test);

            if sphere_count > 0 {
                let buffers = [self.m_buffer_allocator.get_sphere_vertex_buffer()?];
                rhi.cmd_bind_vertex_buffers(command_buffer, 0, &buffers, &[0]);
                for _i in 0..sphere_count {
                    rhi.cmd_bind_descriptor_sets(
                        command_buffer, 
                        RHIPipelineBindPoint::GRAPHICS, 
                        &pipeline.get_pipeline().layout, 
                        0, 
                        &[self.m_buffer_allocator.get_descriptor_set()], 
                        &[dynamic_offset]
                    );
                    rhi.cmd_draw(command_buffer, DebugDrawAllocator::get_sphere_vertex_buffer_size() as u32, 1, 0, 0);
                    dynamic_offset += uniform_dynamic_size;
                }
            }

            if cylinder_count > 0 {
                let buffers = [self.m_buffer_allocator.get_cylinder_vertex_buffer()?];
                rhi.cmd_bind_vertex_buffers(command_buffer, 0, &buffers, &[0]);
                for _i in 0..cylinder_count {
                    rhi.cmd_bind_descriptor_sets(
                        command_buffer, 
                        RHIPipelineBindPoint::GRAPHICS, 
                        &pipeline.get_pipeline().layout, 
                        0, 
                        &[self.m_buffer_allocator.get_descriptor_set()], 
                        &[dynamic_offset]
                    );
                    rhi.cmd_draw(command_buffer, DebugDrawAllocator::get_cylinder_vertex_buffer_size() as u32, 1, 0, 0);
                    dynamic_offset += uniform_dynamic_size;
                }
            }

            if capsule_count > 0 {
                let buffers = [self.m_buffer_allocator.get_capsule_vertex_buffer()?];
                rhi.cmd_bind_vertex_buffers(command_buffer, 0, &buffers, &[0]);
                for _i in 0..capsule_count {
                    rhi.cmd_bind_descriptor_sets(
                        command_buffer, 
                        RHIPipelineBindPoint::GRAPHICS, 
                        &pipeline.get_pipeline().layout, 
                        0, 
                        &[self.m_buffer_allocator.get_descriptor_set()], 
                        &[dynamic_offset]
                    );
                    rhi.cmd_draw(command_buffer, DebugDrawAllocator::get_capsule_vertex_buffer_up_size() as u32, 1, 0, 0);
                    dynamic_offset += uniform_dynamic_size;

                    rhi.cmd_bind_descriptor_sets(
                        command_buffer, 
                        RHIPipelineBindPoint::GRAPHICS, 
                        &pipeline.get_pipeline().layout, 
                        0, 
                        &[self.m_buffer_allocator.get_descriptor_set()], 
                        &[dynamic_offset]
                    );
                    rhi.cmd_draw(command_buffer, DebugDrawAllocator::get_capsule_vertex_buffer_mid_size() as u32, 1, 0, 0);
                    dynamic_offset += uniform_dynamic_size;

                    rhi.cmd_bind_descriptor_sets(
                        command_buffer, 
                        RHIPipelineBindPoint::GRAPHICS, 
                        &pipeline.get_pipeline().layout, 
                        0, 
                        &[self.m_buffer_allocator.get_descriptor_set()], 
                        &[dynamic_offset]
                    );
                    rhi.cmd_draw(command_buffer, DebugDrawAllocator::get_capsule_vertex_buffer_down_size() as u32, 1, 0, 0);
                    dynamic_offset += uniform_dynamic_size;
                }                
            }
            rhi.cmd_end_render_pass(command_buffer);
        }
        Ok(())
    }
}