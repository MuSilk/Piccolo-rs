use std::{cell::RefCell, rc::Rc, time::Duration};

use anyhow::Result;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use winit::event::{Event};

use crate::function::{global::global_context::RuntimeGlobalContext, render::{interface::{rhi::RHICreateInfo, vulkan::vulkan_rhi::VulkanRHI}, passes::main_camera_pass::LayoutType, render_camera::{RenderCamera, RenderCameraType}, render_entity::RenderEntity, render_object::GameObjectPartId, render_pipeline::RenderPipeline, render_pipeline_base::RenderPipelineCreateInfo, render_resource::RenderResource, render_resource_base::RenderResourceBase, render_scene::RenderScene, render_swap_context::{LevelColorGradingResourceDesc, LevelIBLResourceDesc, LevelResourceDesc, RenderSwapContext}, render_type::{MaterialSourceDesc, MeshSourceDesc, RenderMaterialData, RenderMeshData, RenderPipelineType}, window_system::WindowSystem}, ui::window_ui::WindowUI};

pub struct RenderSystemCreateInfo<'a>{
    pub window_system: &'a WindowSystem,
}

pub struct RenderSystem{
    pub m_rhi: Rc<RefCell<VulkanRHI>>,
    m_imgui_context: Rc<RefCell<imgui::Context>>,
    pub m_imgui_platform: Rc<RefCell<WinitPlatform>>,
    m_swap_context: RenderSwapContext,
    m_render_pipeline_type: RenderPipelineType,
    m_render_camera: Rc<RefCell<RenderCamera>>,
    m_render_scene: RenderScene,
    m_render_resource: Rc<RefCell<RenderResource>>,
    m_render_pipeline: RenderPipeline,
}

impl RenderSystem {
    pub fn create(create_info: &RenderSystemCreateInfo) -> Result<Self> {
        let rhi_create_info = RHICreateInfo {
            window_system: create_info.window_system,
        };
        let vulkan_rhi = VulkanRHI::create(&rhi_create_info)?;
        let vulkan_rhi = Rc::new(RefCell::new(vulkan_rhi));

        let mut imgui_context = imgui::Context::create();
        let mut platform = WinitPlatform::new(&mut imgui_context);
        platform.attach_window(imgui_context.io_mut(), create_info.window_system.get_window(), HiDpiMode::Rounded);
        let imgui_context = Rc::new(RefCell::new(imgui_context));
        let imgui_platform = Rc::new(RefCell::new(platform));

        let swap_context = RenderSwapContext::default();

        let mut render_camera = RenderCamera::default();
        render_camera.set_aspect(1024.0/768.0);

        let render_scene = RenderScene::default();
        render_scene.set_visible_nodes_reference();

        let level_resource_desc = LevelResourceDesc{
            m_ibl_resource_desc: LevelIBLResourceDesc{},
            m_color_grading_resource_desc: LevelColorGradingResourceDesc{
                m_color_grading_map: "asset/texture/lut/color_grading_LUT.jpg".to_string(),
            }
        };

        let mut render_resource = RenderResource::default();
        render_resource.upload_global_render_resource(&vulkan_rhi.borrow(), &level_resource_desc);
        let render_resource = Rc::new(RefCell::new(render_resource));

        let create_info = RenderPipelineCreateInfo {
            rhi : &vulkan_rhi,
            render_resource: &render_resource,
            enable_fxaa: true,
            imgui_context: &imgui_context,
            imgui_platform: &imgui_platform,
        };
        let render_pipeline = RenderPipeline::create(&create_info)?;

        render_resource.borrow_mut().m_material_descriptor_set_layout = 
            render_pipeline.m_base.borrow().m_main_camera_pass.m_render_pass.m_descriptor_infos[LayoutType::MeshPerMaterial as usize].layout;

        Ok(Self {
            m_rhi: vulkan_rhi, 
            m_imgui_context: imgui_context,
            m_imgui_platform: imgui_platform,
            m_swap_context: swap_context,
            m_render_pipeline_type: RenderPipelineType::ForwardPipeline,
            m_render_camera: Rc::new(RefCell::new(render_camera)),
            m_render_scene: render_scene,
            m_render_resource: render_resource,
            m_render_pipeline: render_pipeline
        })
    }
    
    pub fn tick(&mut self, delta_time: f32) -> Result<()>{
        self.process_swap_data();
        self.m_rhi.borrow_mut().prepare_context();
        self.m_render_resource.borrow_mut().update_per_frame_buffer(&self.m_render_scene, &self.m_render_camera.borrow());
        self.m_render_scene.update_visible_objects(&self.m_render_resource.borrow(), &self.m_render_camera.borrow());
        self.m_render_pipeline.m_base.borrow_mut().prepare_pass_data(&self.m_render_resource.borrow());
        RuntimeGlobalContext::get_debugdraw_manager().borrow_mut().tick(delta_time);
        let window_system = RuntimeGlobalContext::get_window_system().borrow();
        self.m_imgui_context.borrow_mut().io_mut().update_delta_time(Duration::from_secs_f32(delta_time));
        self.m_imgui_platform.borrow_mut().prepare_frame(self.m_imgui_context.borrow_mut().io_mut(), window_system.get_window()).unwrap();
        match self.m_render_pipeline_type {
            RenderPipelineType::ForwardPipeline => {
                self.m_render_pipeline.forward_render(&mut self.m_render_resource.borrow_mut())?;
            },
            RenderPipelineType::DeferredPipeline => {
                self.m_render_pipeline.defferred_render(&mut self.m_render_resource.borrow_mut())?;
            },
            _ => {panic!("Unknown render pipeline type")}
        }
        Ok(())
    }
    
    pub fn destroy(&self) -> Result<()> {
        self.m_render_pipeline.m_base.borrow_mut().destroy();
        self.m_rhi.borrow_mut().destroy();
        Ok(())
    }
    
    pub fn clear(&mut self){
        // if let Some(rhi) = &self.m_rhi {
        //     let mut rhi_borrow = rhi.borrow_mut();
        //     let vulkan_rhi = rhi_borrow.as_any_mut().downcast_mut::<VulkanRHI>().unwrap();
        //     vulkan_rhi.clear();
        // }   
    }

    pub fn swap_logic_render_data(&mut self){
        self.m_swap_context.swap_logic_render_data();
    }

    pub fn get_swap_context(&self) -> &RenderSwapContext {
        &self.m_swap_context
    }

    pub fn get_render_camera(&self) -> &Rc<RefCell<RenderCamera>> {
        &self.m_render_camera
    }

    pub fn update_engine_content_viewport(&mut self, offset_x: f32, offset_y: f32, width: f32, height: f32){
        let mut rhi = self.m_rhi.as_ref().borrow_mut();
        rhi.m_data.m_viewport.x = offset_x;
        rhi.m_data.m_viewport.y = offset_y;
        rhi.m_data.m_viewport.width = width;
        rhi.m_data.m_viewport.height = height;

        self.m_render_camera.borrow_mut().set_aspect(width/height);
    }

    pub fn get_rhi(&self) -> &Rc<RefCell<VulkanRHI>> {
        &self.m_rhi
    }

    pub fn initialize_ui_render_backend(&mut self, window_ui: &Rc<RefCell<dyn WindowUI>>) {
        self.m_render_pipeline.m_base.borrow_mut().initialize_ui_render_backend(window_ui);
    }

    pub fn handle_event<T>(&self, event: &Event<T>) {
        self.m_imgui_platform.borrow_mut().handle_event(
            self.m_imgui_context.borrow_mut().io_mut(), 
            RuntimeGlobalContext::get_window_system().borrow().get_window(), 
            event
        );
    }
}

impl RenderSystem {
    fn process_swap_data(&mut self) {
        let swap_data = self.m_swap_context.get_render_swap_data();

        if swap_data.borrow().m_game_object_resource_descs.is_some() {
            {
                let mut swap_data = swap_data.borrow_mut();
                let game_object_resource_desc = swap_data.m_game_object_resource_descs.as_mut().unwrap();
                while !game_object_resource_desc.is_empty() {
                    let gobject = game_object_resource_desc.get_next_process_object();

                    for (part_index, game_object_part) in gobject.get_object_parts().iter().enumerate() {
                        let part_id = GameObjectPartId{
                            m_go_id: gobject.get_id(),
                            m_part_id: part_index
                        };
                        
                        let is_entity_in_scene = self.m_render_scene.get_instance_id_allocator().has_element(&part_id);
                        let mut render_entity = RenderEntity::default();
                        render_entity.m_instance_id = 
                            self.m_render_scene.get_instance_id_allocator().alloc_guid(&part_id) as u32;
                        render_entity.m_model_matrix = game_object_part.m_transform_desc.m_transform_matrix;

                        self.m_render_scene.add_instance_id_to_map(render_entity.m_instance_id, gobject.get_id());

                        let mesh_source = MeshSourceDesc {
                            m_mesh_file: game_object_part.m_mesh_desc.m_mesh_file.clone(),
                        };
                        let is_mesh_loaded = self.m_render_scene.get_mesh_asset_id_allocator().has_element(&mesh_source);

                        let mut mesh_data = RenderMeshData::default();
                        if !is_mesh_loaded {
                            mesh_data = self.m_render_resource.borrow_mut().m_base.load_mesh_data(&mesh_source, &mut render_entity.m_bounding_box);
                        }
                        else{
                            render_entity.m_bounding_box = self.m_render_resource.borrow_mut().m_base.get_cached_bounding_box(&mesh_source).unwrap().clone();
                        }

                        render_entity.m_mesh_asset_id = self.m_render_scene.get_mesh_asset_id_allocator().alloc_guid(&mesh_source);
                        render_entity.m_enable_vertex_blending = 
                            game_object_part.m_skeleton_animation_result.m_transforms.len() > 1;
                        render_entity.m_joint_matrices.resize(game_object_part.m_skeleton_animation_result.m_transforms.len(), Default::default());
                        for i in 0..game_object_part.m_skeleton_animation_result.m_transforms.len() {
                            render_entity.m_joint_matrices[i] = game_object_part.m_skeleton_animation_result.m_transforms[i].m_matrix;
                        }

                        let mut material_source = MaterialSourceDesc::default();
                        if game_object_part.m_material_desc.m_with_texture {
                            material_source = MaterialSourceDesc {
                                m_base_color_file: game_object_part.m_material_desc.m_base_color_texture_file.clone(),
                                m_metallic_roughness_file: game_object_part.m_material_desc.m_metallic_roughness_texture_file.clone(),
                                m_normal_file: game_object_part.m_material_desc.m_normal_texture_file.clone(),
                                m_emissive_file: game_object_part.m_material_desc.m_emissive_texture_file.clone(),
                                m_occlusion_file: game_object_part.m_material_desc.m_occlusion_texture_file.clone(),
                            }
                        }
                        else{
                            material_source.m_base_color_file = "asset/texture/default/albedo.jpg".to_string();
                            material_source.m_metallic_roughness_file = "asset/texture/default/mr.jpg".to_string();
                            material_source.m_normal_file = "asset/texture/default/normal.jpg".to_string();
                        }
                        let is_material_loaded = self.m_render_scene.get_material_asset_id_allocator().has_element(&material_source);
                        
                        let mut material_data = RenderMaterialData::default();
                        if !is_material_loaded {
                            material_data = RenderResourceBase::load_material_data(&material_source);
                        }

                        render_entity.m_material_asset_id = self.m_render_scene.get_material_asset_id_allocator().alloc_guid(&material_source);

                        if !is_mesh_loaded {
                            self.m_render_resource.borrow_mut().upload_game_object_render_resource_mesh(&self.m_rhi.borrow(), &render_entity, &mesh_data);
                        }

                        if !is_material_loaded {
                            self.m_render_resource.borrow_mut().upload_game_object_render_resource_material(&self.m_rhi.borrow(), &render_entity, &material_data);
                        }

                        if !is_entity_in_scene {
                            self.m_render_scene.m_render_entities.push(render_entity);
                        }
                        else{
                            for entity in &mut self.m_render_scene.m_render_entities {
                                if entity.m_instance_id == render_entity.m_instance_id {
                                    *entity = render_entity;
                                    break;
                                }
                            }
                        } 
                    }
                    game_object_resource_desc.pop();
                }
            }
            self.m_swap_context.reset_game_object_resource_swap_data();
        }

        if let Some(camera_swap_data) = &swap_data.borrow().m_camera_swap_data {
            if let Some(m_fov_x) = &camera_swap_data.m_fov_x {
                self.m_render_camera.borrow_mut().set_fov_x(*m_fov_x);
            }
            if let Some(m_view_matrix) = &camera_swap_data.m_view_matrix {
                self.m_render_camera.borrow_mut().set_main_view_matrix(m_view_matrix.clone(),RenderCameraType::Editor);
            }
            if let Some(m_camera_type) = &camera_swap_data.m_camera_type {
                self.m_render_camera.borrow_mut().set_current_camera_type(*m_camera_type);
            }
            self.m_swap_context.reset_camera_swap_data();
        }
    }
}