use std::{cell::RefCell, collections::VecDeque, f32::consts::TAU, ptr::copy_nonoverlapping, rc::{Rc, Weak}};

use anyhow::Result;
use nalgebra_glm::{Mat4, Vec3, Vec4};

use crate::runtime::function::{global::global_context::RuntimeGlobalContext, render::{debugdraw::{debug_draw_font::DebugDrawFont, debug_draw_primitive::DebugDrawVertex}, interface::{rhi::{RHI}, rhi_struct::{RHIBuffer, RHIDescriptorBufferInfo, RHIDescriptorImageInfo, RHIDescriptorSet, RHIDescriptorSetAllocateInfo, RHIDescriptorSetLayout, RHIDescriptorSetLayoutBinding, RHIDescriptorSetLayoutCreateInfo, RHIDeviceMemory, RHIWriteDescriptorSet}}, render_type::{RHIBufferUsageFlags, RHIDefaultSamplerType, RHIDescriptorSetLayoutCreateFlags, RHIDescriptorType, RHIImageLayout, RHIMemoryMapFlags, RHIMemoryPropertyFlags, RHIShaderStageFlags}}};

#[derive(Default)]
struct UniformBufferObject{
    proj_view_matrix: Mat4
}

#[repr(align(256))]
#[repr(C)]
struct UniformBufferDynamicObject{
    model_matrix: Mat4,
    color: Vec4
}

struct Resource{
    buffer: Box<dyn RHIBuffer>,
    memory: Box<dyn RHIDeviceMemory>,
}

struct Descriptor{
    layout : Box<dyn RHIDescriptorSetLayout>,
    descriptor_set: Vec<Box<dyn RHIDescriptorSet>>,
}


#[derive(Default)]
pub struct DebugDrawAllocator{
    m_rhi: Weak<RefCell<Box<dyn RHI>>>,

    m_descriptor: Option<Descriptor>,

    m_vertex_resource: Option<Resource>,
    m_vertex_cache: Vec<DebugDrawVertex>,

    m_uniform_resource: Option<Resource>,
    m_uniform_buffer_object: UniformBufferObject,

    m_uniform_dynamic_resource: Option<Resource>,
    m_uniform_buffer_dynamic_object_cache: Vec<UniformBufferDynamicObject>,

    m_sphere_resource: Option<Resource>,
    m_cylinder_resource: Option<Resource>,
    m_capsule_resource: Option<Resource>,

    m_font: Rc<RefCell<DebugDrawFont>>,

    m_current_frame: u32,
    m_deffer_delete_queue: [VecDeque<Resource>;Self::K_DEFERRED_DELETE_RESOURCE_FRAME_COUNT],
}

impl DebugDrawAllocator {
    const K_DEFERRED_DELETE_RESOURCE_FRAME_COUNT: usize = 5;
    const M_CIRCLE_SAMPLE_COUNT: usize = 10;
}

impl DebugDrawAllocator {
    pub fn initialize(&mut self,font: &Rc<RefCell<DebugDrawFont>>) -> Result<()>{
        self.m_rhi = Rc::downgrade(&RuntimeGlobalContext::global().borrow().m_render_system.borrow().get_rhi());
        self.m_font = font.clone();
        self.setup_descriptor_set()?;
        Ok(())
    }

    pub fn destroy(&mut self){
        self.clear();
        self.unload_mesh_buffer();
    }

    pub fn tick(&mut self){
        self.flush_pending_delete();
        self.m_current_frame = (self.m_current_frame + 1) % Self::K_DEFERRED_DELETE_RESOURCE_FRAME_COUNT as u32;
    }

    pub fn get_vertex_buffer(&self) -> Option<&Box<dyn RHIBuffer>>{
        if self.m_vertex_resource.is_none() {
            return None;
        }
        return Some(&self.m_vertex_resource.as_ref().unwrap().buffer);
    }

    pub fn get_descriptor_set(&self) -> &Box<dyn RHIDescriptorSet>{
        &self.m_descriptor.as_ref().unwrap().descriptor_set[self.m_rhi.upgrade().unwrap().borrow().get_current_frame_index() as usize]
    }

    pub fn cache_vertices(&mut self, vertices: &[DebugDrawVertex]) -> usize {
        let offset = self.m_vertex_cache.len();
        self.m_vertex_cache.extend_from_slice(vertices);
        return offset;
    }

    pub fn cache_uniform_object(&mut self,proj_view_matrix: &Mat4){
        self.m_uniform_buffer_object.proj_view_matrix = *proj_view_matrix;
    }

    pub fn cache_uniform_dynamic_object(&mut self, model_colors: &[(Mat4,Vec4)]) -> usize{
        let offset = self.m_uniform_buffer_dynamic_object_cache.len();
        self.m_uniform_buffer_dynamic_object_cache.reserve(model_colors.len());
        for i in 0..model_colors.len(){
            self.m_uniform_buffer_dynamic_object_cache.push(UniformBufferDynamicObject{
                model_matrix: model_colors[i].0,
                color: model_colors[i].1
            });
        }
        return offset;
    }

    pub fn get_vertex_cache_offset(&self) -> usize{
        self.m_vertex_cache.len()
    }

    pub fn get_uniform_dynamic_object_cache_offset(&self) -> usize{
        self.m_uniform_buffer_dynamic_object_cache.len()
    }

    pub fn allocator(&mut self)-> Result<()>{
        self.clear_buffer();
        let vertex_buffer_size = self.m_vertex_cache.len() * std::mem::size_of::<DebugDrawVertex>();
        if vertex_buffer_size > 0 {
            let rhi = self.m_rhi.upgrade().unwrap();
            let (buffer,memory) = rhi.borrow().create_buffer(
                vertex_buffer_size as u64,
                RHIBufferUsageFlags::VERTEX_BUFFER, 
                RHIMemoryPropertyFlags::HOST_VISIBLE | RHIMemoryPropertyFlags::HOST_COHERENT,
            )?;
            let data = rhi.borrow().map_memory(
                &memory, 0, vertex_buffer_size as u64, RHIMemoryMapFlags::empty(),
            )?;
            unsafe{
                copy_nonoverlapping(self.m_vertex_cache.as_mut_ptr().cast(), data, vertex_buffer_size);
            }
            rhi.borrow().unmap_memory(&memory);
            
            self.m_vertex_resource = Some(Resource { buffer, memory});
        }
        let uniform_buffer_size = std::mem::size_of::<UniformBufferObject>();
        if uniform_buffer_size > 0 {
            let rhi = self.m_rhi.upgrade().unwrap();
            let (buffer,memory) = rhi.borrow().create_buffer(
                uniform_buffer_size as u64,
                RHIBufferUsageFlags::UNIFORM_BUFFER, 
                RHIMemoryPropertyFlags::HOST_VISIBLE | RHIMemoryPropertyFlags::HOST_COHERENT,
            )?;
            let data = rhi.borrow().map_memory(
                &memory, 0, uniform_buffer_size as u64, RHIMemoryMapFlags::empty(),
            )?;
            unsafe{
                copy_nonoverlapping(self.m_uniform_buffer_object.proj_view_matrix.as_mut_ptr().cast(), data, uniform_buffer_size);
            }
            rhi.borrow().unmap_memory(&memory);

            self.m_uniform_resource = Some(Resource { buffer, memory});
        }
        let uniform_dynamic_buffer_size = self.m_uniform_buffer_dynamic_object_cache.len() * std::mem::size_of::<UniformBufferDynamicObject>();
        if uniform_dynamic_buffer_size > 0 {
            let rhi = self.m_rhi.upgrade().unwrap();
            let (buffer,memory) = rhi.borrow().create_buffer(
                uniform_dynamic_buffer_size as u64,
                RHIBufferUsageFlags::UNIFORM_BUFFER, 
                RHIMemoryPropertyFlags::HOST_VISIBLE | RHIMemoryPropertyFlags::HOST_COHERENT,
            )?;
            let data = rhi.borrow().map_memory(
                &memory, 0, uniform_buffer_size as u64, RHIMemoryMapFlags::empty(),
            )?;
            unsafe{
                copy_nonoverlapping(self.m_uniform_buffer_dynamic_object_cache.as_mut_ptr().cast(), data, uniform_dynamic_buffer_size);
            }
            rhi.borrow().unmap_memory(&memory);

            self.m_uniform_dynamic_resource = Some(Resource { buffer, memory});
        }
        self.update_descriptor_set()?;
        Ok(())
    }

    pub fn clear(&mut self){
        self.clear_buffer();
        self.m_vertex_cache.clear();
        self.m_uniform_buffer_object.proj_view_matrix = Mat4::identity();
        self.m_uniform_buffer_dynamic_object_cache.clear();
    }

    pub fn get_size_of_uniform_buffer_object() -> usize {
        std::mem::size_of::<UniformBufferObject>()
    }

    pub fn get_sphere_vertex_buffer(&mut self) -> Result<&Box<dyn RHIBuffer>> {
        if self.m_sphere_resource.is_none() {
            self.load_sphere_mesh_buffer()?;
        }
        Ok(&self.m_sphere_resource.as_ref().unwrap().buffer)
    }

    pub fn get_cylinder_vertex_buffer(&mut self) -> Result<&Box<dyn RHIBuffer>> {
        if self.m_cylinder_resource.is_none() {
            self.load_cylinder_mesh_buffer()?;
        }
        Ok(&self.m_cylinder_resource.as_ref().unwrap().buffer)
    }

    pub fn get_capsule_vertex_buffer(&mut self) -> Result<&Box<dyn RHIBuffer>> {
        if self.m_capsule_resource.is_none() {
            self.load_capsule_mesh_buffer()?;
        }
        Ok(&self.m_capsule_resource.as_ref().unwrap().buffer)
    }

    pub fn get_sphere_vertex_buffer_size() -> usize {
        (Self::M_CIRCLE_SAMPLE_COUNT * 2 + 2) * (Self::M_CIRCLE_SAMPLE_COUNT * 2) * 2 + (Self::M_CIRCLE_SAMPLE_COUNT * 2 + 1) * (Self::M_CIRCLE_SAMPLE_COUNT * 2) * 2
    }

    pub fn get_cylinder_vertex_buffer_size() -> usize {
        Self::M_CIRCLE_SAMPLE_COUNT * 2 * 5 * 2
    }

    pub fn get_capsule_vertex_buffer_size() -> usize {
        Self::M_CIRCLE_SAMPLE_COUNT * 2 * Self::M_CIRCLE_SAMPLE_COUNT * 4 +
            (2 * Self::M_CIRCLE_SAMPLE_COUNT) * 2 + 
            (2 * Self::M_CIRCLE_SAMPLE_COUNT) * Self::M_CIRCLE_SAMPLE_COUNT * 4
    }

    pub fn get_capsule_vertex_buffer_up_size() -> usize {
        Self::M_CIRCLE_SAMPLE_COUNT * 2 * Self::M_CIRCLE_SAMPLE_COUNT * 4
    }

    pub fn get_capsule_vertex_buffer_mid_size() -> usize {
        (2 * Self::M_CIRCLE_SAMPLE_COUNT) * 2
    }

    pub fn get_capsule_vertex_buffer_down_size() -> usize {
        Self::M_CIRCLE_SAMPLE_COUNT * 2 * Self::M_CIRCLE_SAMPLE_COUNT * 4
    }
}

impl DebugDrawAllocator {

    fn clear_buffer(&mut self){ 
        if !self.m_vertex_resource.is_none() {
            self.m_deffer_delete_queue[self.m_current_frame as usize].push_back(self.m_vertex_resource.take().unwrap());
        }
        if !self.m_uniform_resource.is_none() {
            self.m_deffer_delete_queue[self.m_current_frame as usize].push_back(self.m_uniform_resource.take().unwrap());
        }
        if !self.m_uniform_dynamic_resource.is_none() {
            self.m_deffer_delete_queue[self.m_current_frame as usize].push_back(self.m_uniform_dynamic_resource.take().unwrap());
        }
    }

    fn flush_pending_delete(&mut self){
        let current_frame_to_delete = ((self.m_current_frame + 1) % Self::K_DEFERRED_DELETE_RESOURCE_FRAME_COUNT as u32) as usize;
        while let Some(resource_to_delete) = self.m_deffer_delete_queue[current_frame_to_delete].pop_front() {
            self.m_rhi.upgrade().unwrap().borrow().free_memory(resource_to_delete.memory);
            self.m_rhi.upgrade().unwrap().borrow().destroy_buffer(resource_to_delete.buffer);
        }
    }

    fn setup_descriptor_set(&mut self) -> Result<()>{
        let ubo_layout_binding = [
            RHIDescriptorSetLayoutBinding{
                binding: 0,
                descriptor_type: RHIDescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: RHIShaderStageFlags::VERTEX.into(),
                p_immutable_samplers: None,
            },
            RHIDescriptorSetLayoutBinding{
                binding: 1,
                descriptor_type: RHIDescriptorType::UNIFORM_BUFFER_DYNAMIC,
                descriptor_count: 1,
                stage_flags: RHIShaderStageFlags::VERTEX.into(),
                p_immutable_samplers: None,
            },
            RHIDescriptorSetLayoutBinding{
                binding: 2,
                descriptor_type: RHIDescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: RHIShaderStageFlags::FRAGMENT.into(),
                p_immutable_samplers: None,
            },
        ];

        let create_info = RHIDescriptorSetLayoutCreateInfo{
            flags: RHIDescriptorSetLayoutCreateFlags::empty(),
            bindings: &ubo_layout_binding,
        };

        let rhi = self.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();

        let layout = rhi.create_descriptor_set_layout(&create_info)?;
        let descriptor_set = (0..rhi.get_max_frames_in_flight()).map(|_| {
            let alloc_info = RHIDescriptorSetAllocateInfo {
                descriptor_pool: rhi.get_descriptor_pool().unwrap(),
                set_layouts: &[&layout],
            };
            rhi.allocate_descriptor_sets(&alloc_info).unwrap()
        }).collect::<Vec<_>>().into_iter().flatten().collect::<Vec<_>>();

        self.m_descriptor =Some(Descriptor { 
            layout: layout, 
            descriptor_set: descriptor_set 
        });

        self.prepare_descriptor_set()?;

        Ok(())
    }

    fn prepare_descriptor_set(&mut self) -> Result<()> {
        let rhi = self.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        let sampler = rhi.get_or_create_default_sampler(RHIDefaultSamplerType::Linear)?;
        
        let font = self.m_font.borrow();
        let image_info = [
            RHIDescriptorImageInfo{
                image_view: font.get_image_view(),
                sampler: sampler,
                image_layout: RHIImageLayout::SHADER_READ_ONLY_OPTIMAL,
            }
        ];
        for i in 0..rhi.get_max_frames_in_flight() {
            let descriptor_write = RHIWriteDescriptorSet{
                dst_set: &self.m_descriptor.as_ref().unwrap().descriptor_set[i as usize],
                dst_binding: 2,
                dst_array_element: 0,
                descriptor_type: RHIDescriptorType::COMBINED_IMAGE_SAMPLER,
                image_info: &image_info,
                buffer_info: &[],
                texel_buffer_view: &[],
            };

            rhi.update_descriptor_sets(&[descriptor_write])?;
        }
        Ok(())
    } 

    fn update_descriptor_set(&mut self) -> Result<()> {
        let buffer_info = [
            RHIDescriptorBufferInfo{
                buffer: &self.m_uniform_resource.as_ref().unwrap().buffer,
                offset: 0,
                range: std::mem::size_of::<UniformBufferObject>() as u64,
            },
            RHIDescriptorBufferInfo{
                buffer: &self.m_uniform_dynamic_resource.as_ref().unwrap().buffer,
                offset: 0,
                range: std::mem::size_of::<UniformBufferDynamicObject>() as u64,
            },
        ];
        
        let rhi = self.m_rhi.upgrade().unwrap();
        let rhi  = rhi.borrow();

        let descriptor_write = [
            RHIWriteDescriptorSet{
                dst_set: &self.m_descriptor.as_ref().unwrap().descriptor_set[rhi.get_current_frame_index() as usize],
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_type: RHIDescriptorType::UNIFORM_BUFFER,
                image_info: &[],
                buffer_info: &buffer_info[0..1],
                texel_buffer_view: &[],
            },
            RHIWriteDescriptorSet{
                dst_set: &self.m_descriptor.as_ref().unwrap().descriptor_set[rhi.get_current_frame_index() as usize],
                dst_binding: 1,
                dst_array_element: 0,
                descriptor_type: RHIDescriptorType::UNIFORM_BUFFER_DYNAMIC,
                image_info: &[],
                buffer_info: &buffer_info[1..2],
                texel_buffer_view: &[],
            }
        ];

        rhi.update_descriptor_sets( &descriptor_write)?;

        Ok(())
    }

    fn unload_mesh_buffer(&mut self){
        if let Some(resource) = self.m_sphere_resource.take() {
            self.m_deffer_delete_queue[self.m_current_frame as usize].push_back(resource);
        }
        if let Some(resource) = self.m_cylinder_resource.take() {
            self.m_deffer_delete_queue[self.m_current_frame as usize].push_back(resource);
        }
        if let Some(resource) = self.m_capsule_resource.take() {
            self.m_deffer_delete_queue[self.m_current_frame as usize].push_back(resource);
        }
    }

    fn load_sphere_mesh_buffer(&mut self) -> Result<()> {
        let param = Self::M_CIRCLE_SAMPLE_COUNT as i32;
        let vertex_count = (param * 2 + 2) * (param * 2) * 2 + (param * 2 + 1) * (param * 2) * 2;
        let mut vertices = Vec::<DebugDrawVertex>::new();
        vertices.reserve(vertex_count as usize);

        for i in -param-1..param+1 {
            let k = (param + 1) as f32;
            let h = (TAU /4.0 * (i as f32) / k).sin();
            let h1 = (TAU /4.0 * ((i + 1) as f32) / k).sin();
            let r = (1.0 - h * h).sqrt();
            let r1 = (1.0 - h1 * h1).sqrt();
            for j in 0..2 * param {
                let p = Vec3::new(
                    r * (TAU / (2.0 * param as f32) * j as f32).cos() * r,
                    r * (TAU / (2.0 * param as f32) * j as f32).sin() * r,
                    h, 
                );
                let p1 = Vec3::new(
                    r1 * (TAU / (2.0 * param as f32) * j as f32).cos() * r1,
                    r1 * (TAU / (2.0 * param as f32) * j as f32).sin() * r1,
                    h1, 
                );
                vertices.push(DebugDrawVertex{
                    pos: p,
                    color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                    ..Default::default()
                });
                vertices.push(DebugDrawVertex{
                    pos: p1,
                    color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                    ..Default::default()
                });
            }
            if i != -param - 1 {
                for j in 0..2 * param {
                    let p = Vec3::new(
                        r * (TAU / (2.0 * param as f32) * j as f32).cos() * r,
                        r * (TAU / (2.0 * param as f32) * j as f32).sin() * r,
                        h, 
                    );
                    let p1 = Vec3::new(
                        r * (TAU / (2.0 * param as f32) * (j + 1) as f32).cos() * r,
                        r * (TAU / (2.0 * param as f32) * (j + 1) as f32).sin() * r,
                        h, 
                    );

                    vertices.push(DebugDrawVertex {
                        pos: p,
                        color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                        ..Default::default()
                    });
                    vertices.push(DebugDrawVertex {
                        pos: p1,
                        color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                        ..Default::default()
                    });
                }
            }
        }
        
        let buffer_size = vertices.len() * std::mem::size_of::<DebugDrawVertex>();

        let rhi = self.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        let (buffer, memory) = rhi.create_buffer(
            buffer_size as u64, 
            RHIBufferUsageFlags::VERTEX_BUFFER | RHIBufferUsageFlags::TRANSFER_DST,
            RHIMemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        self.m_sphere_resource = Some(Resource{buffer, memory});
        let (buffer, memory) = rhi.create_buffer(
            buffer_size as u64, 
            RHIBufferUsageFlags::TRANSFER_SRC,
            RHIMemoryPropertyFlags::HOST_VISIBLE | RHIMemoryPropertyFlags::HOST_COHERENT,
        )?;
        let data = rhi.map_memory(&memory, 0, buffer_size as u64, RHIMemoryMapFlags::empty())?;
        unsafe{copy_nonoverlapping(vertices.as_ptr(), data.cast(), buffer_size);}
        rhi.unmap_memory(&memory);

        rhi.copy_buffer(&buffer, &self.m_sphere_resource.as_ref().unwrap().buffer, 0, 0, buffer_size as u64)?;
        rhi.destroy_buffer(buffer);
        rhi.free_memory(memory);
        Ok(())
    }

    fn load_cylinder_mesh_buffer(&mut self) -> Result<()> {
        let param = Self::M_CIRCLE_SAMPLE_COUNT as i32;
        let vertex_count = 2 * param * 5 * 2;
        let mut vertices = Vec::<DebugDrawVertex>::new();
        vertices.reserve(vertex_count as usize);

        for i in 0..2*param {
            let p = Vec3::new(
                TAU / (2.0 * param as f32) * (i as f32).cos(),
                TAU / (2.0 * param as f32) * (i as f32).sin(),
                1.0
            );
            let p_ = Vec3::new(
                TAU / (2.0 * param as f32) * ((i + 1) as f32).cos(),
                TAU / (2.0 * param as f32) * ((i + 1) as f32).sin(),
                1.0
            );
            let p1 = Vec3::new(
                TAU / (2.0 * param as f32) * (i as f32).cos(),
                TAU / (2.0 * param as f32) * (i as f32).sin(),
                -1.0
            );
            let p1_ = Vec3::new(
                TAU / (2.0 * param as f32) * ((i + 1) as f32).cos(),
                TAU / (2.0 * param as f32) * ((i + 1) as f32).sin(),
                -1.0
            );

            vertices.push(
                DebugDrawVertex {
                    pos: p,
                    color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                    ..Default::default()
                }
            );
            vertices.push(
                DebugDrawVertex {
                    pos: p_,
                    color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                    ..Default::default()
                }
            );
            vertices.push(
                DebugDrawVertex {
                    pos: p1,
                    color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                    ..Default::default()
                }
            );
            vertices.push(
                DebugDrawVertex {
                    pos: p1_,
                    color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                    ..Default::default()
                }
            );
        }

        let buffer_size = vertices.len() * std::mem::size_of::<DebugDrawVertex>();

        let rhi = self.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        let (buffer, memory) = rhi.create_buffer(
            buffer_size as u64, 
            RHIBufferUsageFlags::VERTEX_BUFFER | RHIBufferUsageFlags::TRANSFER_DST,
            RHIMemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        self.m_cylinder_resource = Some(Resource{buffer, memory});
        let (buffer, memory) = rhi.create_buffer(
            buffer_size as u64, 
            RHIBufferUsageFlags::TRANSFER_SRC,
            RHIMemoryPropertyFlags::HOST_VISIBLE | RHIMemoryPropertyFlags::HOST_COHERENT,
        )?;
        let data = rhi.map_memory(&memory, 0, buffer_size as u64, RHIMemoryMapFlags::empty())?;
        unsafe{copy_nonoverlapping(vertices.as_ptr(), data.cast(), buffer_size);}
        rhi.unmap_memory(&memory);

        rhi.copy_buffer(&buffer, &self.m_cylinder_resource.as_ref().unwrap().buffer, 0, 0, buffer_size as u64)?;
        rhi.destroy_buffer(buffer);
        rhi.free_memory(memory);
        Ok(())
    }

    fn load_capsule_mesh_buffer(&mut self) -> Result<()> {
        let param = Self::M_CIRCLE_SAMPLE_COUNT as i32;
        let vertex_count = 2 * param * param * 4 + 2 * param * param * 4 + 2 * param * 2;
        let mut vertices = Vec::<DebugDrawVertex>::new();
        vertices.reserve(vertex_count as usize);

        for i in 0..param {
            let h = TAU / 4.0 / (param as f32) * (i as f32);
            let h1 = TAU / 4.0 / (param as f32) * ((i + 1) as f32);
            let r = (1.0 - h * h).sqrt();
            let r1 = (1.0 - h1 * h1).sqrt();
            for j in 0..2*param {

                let p = Vec3::new(
                    TAU / (2.0 * param as f32) * (j as f32).cos() * r,
                    TAU / (2.0 * param as f32) * (j as f32).sin() * r,
                    h + 1.0
                );
                let p_ = Vec3::new(
                    TAU / (2.0 * param as f32) * ((j + 1) as f32).cos() * r,
                    TAU / (2.0 * param as f32) * ((j + 1) as f32).sin() * r,
                    h + 1.0
                );
                let p1 = Vec3::new(
                    TAU / (2.0 * param as f32) * (j as f32).cos() * r1,
                    TAU / (2.0 * param as f32) * (j as f32).sin() * r1,
                    h1 + 1.0
                );

                vertices.push(DebugDrawVertex { pos: p, color: Vec4::new(1.0, 0.0, 0.0, 1.0), ..Default::default() });
                vertices.push(DebugDrawVertex { pos: p1, color: Vec4::new(1.0, 0.0, 0.0, 1.0), ..Default::default() });
                vertices.push(DebugDrawVertex { pos: p, color: Vec4::new(1.0, 0.0, 0.0, 1.0), ..Default::default() });
                vertices.push(DebugDrawVertex { pos: p_, color: Vec4::new(1.0, 0.0, 0.0, 1.0), ..Default::default() });
            }
        }

        for j in 0..2*param {
            let p = Vec3::new(
                TAU / (2.0 * param as f32) * (j as f32).cos(),
                TAU / (2.0 * param as f32) * (j as f32).sin(),
                1.0
            );
            let p1 = Vec3::new(
                TAU / (2.0 * param as f32) * (j as f32).cos(),
                TAU / (2.0 * param as f32) * (j as f32).sin(),
                -1.0
            );
            vertices.push(DebugDrawVertex { pos: p, color: Vec4::new(1.0, 0.0, 0.0, 1.0), ..Default::default() });
            vertices.push(DebugDrawVertex { pos: p1, color: Vec4::new(1.0, 0.0, 0.0, 1.0), ..Default::default() });
        }

        for i in (-param+1..=0).rev() {
            let h = (TAU / 4.0 / param as f32 * i as f32).sin();
            let h1 = (TAU / 4.0 / param as f32 * (i - 1) as f32).sin();
            let r = (1.0 - h * h).sqrt();
            let r1 = (1.0 - h1 * h1).sqrt();
            for j in 0..2*param {
                let p = Vec3::new(
                    TAU / (2.0 * param as f32) * (j as f32).cos() * r,
                    TAU / (2.0 * param as f32) * (j as f32).sin() * r,
                    h - 1.0
                );
                let p_ = Vec3::new(
                    TAU / (2.0 * param as f32) * ((j + 1) as f32).cos() * r,
                    TAU / (2.0 * param as f32) * ((j + 1) as f32).sin() * r,
                    h - 1.0
                );
                let p1 = Vec3::new(
                    TAU / (2.0 * param as f32) * (j as f32).cos() * r1,
                    TAU / (2.0 * param as f32) * (j as f32).sin() * r1,
                    h1 - 1.0
                );
                vertices.push(DebugDrawVertex { pos: p, color: Vec4::new(1.0, 0.0, 0.0, 1.0), ..Default::default() });
                vertices.push(DebugDrawVertex { pos: p1, color: Vec4::new(1.0, 0.0, 0.0, 1.0), ..Default::default() });
                vertices.push(DebugDrawVertex { pos: p, color: Vec4::new(1.0, 0.0, 0.0, 1.0), ..Default::default() });
                vertices.push(DebugDrawVertex { pos: p_, color: Vec4::new(1.0, 0.0, 0.0, 1.0), ..Default::default() });
            }
        }

        let buffer_size = vertices.len() * std::mem::size_of::<DebugDrawVertex>();

        let rhi = self.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        let (buffer, memory) = rhi.create_buffer(
            buffer_size as u64, 
            RHIBufferUsageFlags::VERTEX_BUFFER | RHIBufferUsageFlags::TRANSFER_DST,
            RHIMemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        self.m_capsule_resource = Some(Resource{buffer, memory});
        let (buffer, memory) = rhi.create_buffer(
            buffer_size as u64, 
            RHIBufferUsageFlags::TRANSFER_SRC,
            RHIMemoryPropertyFlags::HOST_VISIBLE | RHIMemoryPropertyFlags::HOST_COHERENT,
        )?;
        let data = rhi.map_memory(&memory, 0, buffer_size as u64, RHIMemoryMapFlags::empty())?;
        unsafe{copy_nonoverlapping(vertices.as_ptr(), data.cast(), buffer_size);}
        rhi.unmap_memory(&memory);

        rhi.copy_buffer(&buffer, &self.m_capsule_resource.as_ref().unwrap().buffer, 0, 0, buffer_size as u64)?;
        rhi.destroy_buffer(buffer);
        rhi.free_memory(memory);
        Ok(())
    }

}