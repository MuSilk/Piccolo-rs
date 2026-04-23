use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::{
    core::math::vector2::Vector2,
    function::{
        render::{
            debugdraw::debug_draw_manager::{DebugDrawManager, DebugDrawManagerCreateInfo},
            interface::{rhi::RHICreateInfo, vulkan::vulkan_rhi::VulkanRHI},
            light::{AmbientLight, DirectionalLight},
            passes::main_camera_pass::{MeshPerMaterialDescriptorLayout, PerMeshDescriptorLayout},
            render_camera::RenderCamera,
            render_entity::RenderEntity,
            render_object::{GameObjectMeshDesc, GameObjectPartId},
            render_pipeline::{RenderPipeline, RenderPipelineCreateInfo},
            render_resource::RenderResource,
            render_resource_base::RenderResourceBase,
            render_scene::RenderScene,
            render_swap_context::{
                LevelColorGradingResourceDesc, LevelIBLResourceDesc, LevelResourceDesc,
                RenderSwapContext, RenderSwapData,
            },
            render_type::{MaterialSourceDesc, MeshSourceDesc, RenderPipelineType},
            window_system::WindowSystem,
        },
        ui::ui2::UiRuntime,
    },
    resource::{
        asset_manager::AssetManager, config_manager::ConfigManager,
        res_type::global::global_rendering::GlobalRenderingRes,
    },
};

pub struct RenderSystemCreateInfo<'a> {
    pub window_system: &'a WindowSystem,
    pub asset_manager: &'a AssetManager,
    pub config_manager: &'a ConfigManager,
}

pub struct RenderSystem {
    m_rhi: RefCell<VulkanRHI>,
    m_swap_context: RenderSwapContext,
    m_render_pipeline_type: RenderPipelineType,
    m_render_camera: Rc<RefCell<RenderCamera>>,
    m_render_scene: RenderScene,
    m_render_resource: RefCell<RenderResource>,
    m_render_pipeline: RefCell<RenderPipeline>,

    m_debugdraw_manager: RefCell<DebugDrawManager>,
}

impl RenderSystem {
    pub fn create(create_info: &RenderSystemCreateInfo) -> Self {
        let rhi_create_info = RHICreateInfo {
            window_system: create_info.window_system,
        };
        let vulkan_rhi = VulkanRHI::create(&rhi_create_info);
        let vulkan_rhi = RefCell::new(vulkan_rhi);

        let asset_manager = create_info.asset_manager;
        let config_manager = create_info.config_manager;
        let global_rendering_res_url = config_manager.get_global_rendering_res_url();
        let global_rendering_res: GlobalRenderingRes =
            asset_manager.load_asset(&global_rendering_res_url).unwrap();

        let swap_context = RenderSwapContext::default();

        let mut render_camera = RenderCamera::default();
        let camera_pose = &global_rendering_res.camera_config.pose;
        render_camera.look_at(camera_pose.position, &camera_pose.target, &camera_pose.up);
        render_camera.set_znear(global_rendering_res.camera_config.z_near);
        render_camera.set_zfar(global_rendering_res.camera_config.z_far);
        render_camera.set_aspect(
            global_rendering_res.camera_config.aspect.x
                / global_rendering_res.camera_config.aspect.y,
        );

        let mut render_scene = RenderScene::default();
        render_scene.m_ambient_light = AmbientLight {
            m_irradiance: global_rendering_res.ambient_light,
        };
        render_scene.m_directional_light = DirectionalLight {
            m_direction: global_rendering_res.directional_light.direction,
            m_color: global_rendering_res.directional_light.color,
        };

        let level_resource_desc = LevelResourceDesc {
            m_ibl_resource_desc: LevelIBLResourceDesc {
                m_skybox_irradiance_map: global_rendering_res.skybox_irradiance_map,
                m_skybox_specular_map: global_rendering_res.skybox_specular_map,
                m_brdf_map: global_rendering_res.brdf_map,
            },
            m_color_grading_resource_desc: LevelColorGradingResourceDesc {
                m_color_grading_map: global_rendering_res.color_grading_map,
            },
        };

        let mut render_resource = RenderResource::default();
        render_resource.upload_global_render_resource(
            asset_manager,
            &vulkan_rhi.borrow(),
            &level_resource_desc,
        );
        let render_resource = RefCell::new(render_resource);

        let render_pipeline = RenderPipeline::create(&RenderPipelineCreateInfo {
            rhi: &vulkan_rhi.borrow(),
            render_resource: &render_resource.borrow(),
            enable_fxaa: global_rendering_res.enable_fxaa,
        })
        .unwrap();

        render_resource.borrow_mut().m_mesh_descriptor_set_layout = render_pipeline
            .get_descriptor_set_layout::<PerMeshDescriptorLayout>(&vulkan_rhi.borrow())
            .unwrap();

        render_resource
            .borrow_mut()
            .m_material_descriptor_set_layout = render_pipeline
            .get_descriptor_set_layout::<MeshPerMaterialDescriptorLayout>(&vulkan_rhi.borrow())
            .unwrap();

        let debugdraw_manager = DebugDrawManager::create(&DebugDrawManagerCreateInfo {
            rhi: &vulkan_rhi.borrow(),
            font_path: config_manager.get_editor_font_path(),
        })
        .unwrap();

        Self {
            m_rhi: vulkan_rhi,
            m_swap_context: swap_context,
            m_render_pipeline_type: RenderPipelineType::DeferredPipeline,
            m_render_camera: Rc::new(RefCell::new(render_camera)),
            m_render_scene: render_scene,
            m_render_resource: render_resource,
            m_render_pipeline: RefCell::new(render_pipeline),
            m_debugdraw_manager: RefCell::new(debugdraw_manager),
        }
    }

    pub fn tick(
        &self,
        ui_runtime: &UiRuntime,
        asset_manager: &AssetManager,
        delta_time: f32,
    ) -> Result<()> {
        self.process_swap_data(asset_manager);
        self.m_rhi.borrow_mut().prepare_context();
        self.m_render_resource
            .borrow_mut()
            .update_per_frame_buffer(&self.m_render_scene, &self.m_render_camera.borrow());
        self.m_render_scene.update_visible_objects(
            &self.m_render_resource.borrow(),
            &self.m_render_camera.borrow(),
        );
        self.m_render_pipeline
            .borrow_mut()
            .prepare_pass_data(&self.m_rhi.borrow(), &self.m_render_resource.borrow());
        self.m_debugdraw_manager
            .borrow_mut()
            .prepare_pass_data(&self.m_render_resource.borrow());
        self.m_debugdraw_manager
            .borrow_mut()
            .tick(&self.m_rhi.borrow(), delta_time);
        self.render(
            ui_runtime,
            match self.m_render_pipeline_type {
                RenderPipelineType::ForwardPipeline => true,
                RenderPipelineType::DeferredPipeline => false,
            },
        )?;
        Ok(())
    }

    pub fn destroy(&self) -> Result<()> {
        self.m_debugdraw_manager
            .borrow_mut()
            .destroy(&self.m_rhi.borrow());
        self.m_render_pipeline
            .borrow_mut()
            .destroy(&self.m_rhi.borrow());
        self.m_render_resource
            .borrow_mut()
            .flush_deferred_mesh_destroys(&self.m_rhi.borrow());
        self.m_rhi.borrow_mut().destroy();
        Ok(())
    }

    pub fn clear(&self) {
        // if let Some(rhi) = &self.m_rhi {
        //     let mut rhi_borrow = rhi.borrow_mut();
        //     let vulkan_rhi = rhi_borrow.as_any_mut().downcast_mut::<VulkanRHI>().unwrap();
        //     vulkan_rhi.clear();
        // }
    }

    pub fn swap_logic_render_data(&mut self) {
        self.m_swap_context.swap_logic_render_data();
    }

    pub fn get_logic_swap_data(&self) -> &RefCell<RenderSwapData> {
        &self.m_swap_context.get_logic_swap_data()
    }

    pub fn get_render_camera(&self) -> &Rc<RefCell<RenderCamera>> {
        &self.m_render_camera
    }

    pub fn update_engine_content_viewport(
        &self,
        offset_x: f32,
        offset_y: f32,
        width: f32,
        height: f32,
    ) {
        let mut rhi = self.m_rhi.borrow_mut();
        rhi.m_data.m_viewport.x = offset_x;
        rhi.m_data.m_viewport.y = offset_y;
        rhi.m_data.m_viewport.width = width;
        rhi.m_data.m_viewport.height = height;

        self.m_render_camera.borrow_mut().set_aspect(width / height);
    }

    pub fn get_rhi(&self) -> &RefCell<VulkanRHI> {
        &self.m_rhi
    }

    pub fn get_guid_of_picked_mesh(&self, picked_uv: &Vector2) -> u32 {
        self.m_render_pipeline
            .borrow()
            .get_guid_of_picked_mesh(picked_uv)
    }
}

impl RenderSystem {
    fn process_swap_data(&self, asset_manager: &AssetManager) {
        let swap_data = self.m_swap_context.get_render_swap_data();
        if swap_data.borrow().m_game_object_resource_descs.is_some() {
            {
                let mut swap_data = swap_data.borrow_mut();
                let game_object_resource_desc =
                    swap_data.m_game_object_resource_descs.as_mut().unwrap();
                while !game_object_resource_desc.is_empty() {
                    let gobject = game_object_resource_desc.get_next_process_object();

                    for (part_index, game_object_part) in
                        gobject.get_object_parts().iter().enumerate()
                    {
                        let part_id = GameObjectPartId {
                            m_go_id: gobject.get_id(),
                            m_part_id: part_index,
                        };
                        let mut render_entity = Box::new(RenderEntity::default());
                        render_entity.m_instance_id =
                            self.m_render_scene.alloc_instance_id(&part_id) as u32;
                        render_entity.m_model_matrix =
                            Rc::new(game_object_part.m_transform_desc.m_transform_matrix);
                        render_entity.m_base_color_factor = game_object_part.m_base_color_factor;

                        self.m_render_scene
                            .add_instance_id_to_map(render_entity.m_instance_id, gobject.get_id());

                        match game_object_part.m_mesh_desc {
                            GameObjectMeshDesc::LazyMesh(ref mesh_desc) => {
                                let mesh_source = MeshSourceDesc {
                                    m_mesh_file: mesh_desc.m_mesh_file.clone(),
                                };
                                let is_mesh_loaded = self
                                    .m_render_scene
                                    .get_mesh_asset_id_allocator()
                                    .borrow_mut()
                                    .has_element(&mesh_source);
                                render_entity.m_mesh_asset_id = self
                                    .m_render_scene
                                    .get_mesh_asset_id_allocator()
                                    .borrow_mut()
                                    .alloc_guid(&mesh_source);
                                if !is_mesh_loaded {
                                    let (mesh_data, bounding_box) = self
                                        .m_render_resource
                                        .borrow_mut()
                                        .load_mesh_data(asset_manager, &mesh_source);
                                    render_entity.m_bounding_box = bounding_box;
                                    self.m_render_resource
                                        .borrow_mut()
                                        .upload_game_object_render_resource_mesh(
                                            &self.m_rhi.borrow(),
                                            &render_entity,
                                            &mesh_data,
                                        );
                                    println!("load mesh data");
                                } else {
                                    render_entity.m_bounding_box = self
                                        .m_render_resource
                                        .borrow_mut()
                                        .get_cached_bounding_box(&mesh_source)
                                        .unwrap()
                                        .clone();
                                }
                            }
                            GameObjectMeshDesc::StaticMesh(ref mesh_desc) => {
                                let mesh_source = MeshSourceDesc {
                                    m_mesh_file: mesh_desc.borrow().m_mesh_file.clone(),
                                };
                                let is_mesh_loaded = self
                                    .m_render_scene
                                    .get_mesh_asset_id_allocator()
                                    .borrow()
                                    .has_element(&mesh_source);
                                render_entity.m_mesh_asset_id = self
                                    .m_render_scene
                                    .get_mesh_asset_id_allocator()
                                    .borrow_mut()
                                    .alloc_guid(&mesh_source);
                                if !is_mesh_loaded {
                                    let (mesh_data, bounding_box) = self
                                        .m_render_resource
                                        .borrow_mut()
                                        .load_mesh_data_from_raw(
                                            &mesh_source,
                                            &mesh_desc.borrow().m_vertices,
                                            &mesh_desc.borrow().m_indices,
                                        );
                                    render_entity.m_bounding_box = bounding_box;
                                    self.m_render_resource
                                        .borrow_mut()
                                        .upload_game_object_render_resource_mesh(
                                            &self.m_rhi.borrow(),
                                            &render_entity,
                                            &mesh_data,
                                        );
                                } else {
                                    render_entity.m_bounding_box = self
                                        .m_render_resource
                                        .borrow_mut()
                                        .get_cached_bounding_box(&mesh_source)
                                        .unwrap()
                                        .clone();
                                }
                            }
                            GameObjectMeshDesc::DynamicMesh(ref mesh_desc) => {
                                let mesh_source = MeshSourceDesc {
                                    m_mesh_file: mesh_desc.borrow().m_mesh_file.clone(),
                                };
                                render_entity.m_mesh_asset_id = self
                                    .m_render_scene
                                    .get_mesh_asset_id_allocator()
                                    .borrow_mut()
                                    .alloc_guid(&mesh_source);
                                if mesh_desc.borrow().m_is_dirty {
                                    let (mesh_data, bounding_box) = self
                                        .m_render_resource
                                        .borrow_mut()
                                        .load_mesh_data_from_raw(
                                            &mesh_source,
                                            &mesh_desc.borrow().m_vertices,
                                            &mesh_desc.borrow().m_indices,
                                        );
                                    render_entity.m_bounding_box = bounding_box;
                                    self.m_render_resource
                                        .borrow_mut()
                                        .upload_game_object_render_resource_mesh(
                                            &self.m_rhi.borrow(),
                                            &render_entity,
                                            &mesh_data,
                                        );
                                    mesh_desc.borrow_mut().m_is_dirty = false;
                                } else {
                                    render_entity.m_bounding_box = self
                                        .m_render_resource
                                        .borrow_mut()
                                        .get_cached_bounding_box(&mesh_source)
                                        .unwrap()
                                        .clone();
                                }
                            }
                        };

                        render_entity.m_enable_vertex_blending = game_object_part
                            .m_skeleton_animation_result
                            .m_transforms
                            .len()
                            > 1;
                        render_entity.m_joint_matrices.resize(
                            game_object_part
                                .m_skeleton_animation_result
                                .m_transforms
                                .len(),
                            Default::default(),
                        );
                        for i in 0..game_object_part
                            .m_skeleton_animation_result
                            .m_transforms
                            .len()
                        {
                            render_entity.m_joint_matrices[i] =
                                game_object_part.m_skeleton_animation_result.m_transforms[i]
                                    .m_matrix;
                        }

                        let mut material_source = MaterialSourceDesc::default();
                        if game_object_part.m_material_desc.m_with_texture {
                            material_source = MaterialSourceDesc {
                                m_base_color_file: game_object_part
                                    .m_material_desc
                                    .m_base_color_texture_file
                                    .clone(),
                                m_metallic_roughness_file: game_object_part
                                    .m_material_desc
                                    .m_metallic_roughness_texture_file
                                    .clone(),
                                m_normal_file: game_object_part
                                    .m_material_desc
                                    .m_normal_texture_file
                                    .clone(),
                                m_emissive_file: game_object_part
                                    .m_material_desc
                                    .m_emissive_texture_file
                                    .clone(),
                                m_occlusion_file: game_object_part
                                    .m_material_desc
                                    .m_occlusion_texture_file
                                    .clone(),
                            }
                        } else {
                            material_source.m_base_color_file =
                                "asset/texture/default/albedo.jpg".to_string();
                            material_source.m_metallic_roughness_file =
                                "asset/texture/default/mr.jpg".to_string();
                            material_source.m_normal_file =
                                "asset/texture/default/normal.jpg".to_string();
                        }
                        let is_material_loaded = self
                            .m_render_scene
                            .get_material_asset_id_allocator()
                            .borrow()
                            .has_element(&material_source);
                        render_entity.m_material_asset_id = self
                            .m_render_scene
                            .get_material_asset_id_allocator()
                            .borrow_mut()
                            .alloc_guid(&material_source);
                        if !is_material_loaded {
                            println!("load material data");
                            let material_data = RenderResourceBase::load_material_data(
                                asset_manager,
                                &material_source,
                            );
                            self.m_render_resource
                                .borrow_mut()
                                .upload_game_object_render_resource_material(
                                    &self.m_rhi.borrow(),
                                    &render_entity,
                                    &material_data,
                                );
                        }

                        self.m_render_scene
                            .insert_or_update_render_entity(render_entity);
                    }
                    game_object_resource_desc.pop();
                }
            }
            self.m_swap_context.reset_game_object_resource_swap_data();
        }

        if swap_data.borrow().m_game_object_to_delete.is_some() {
            {
                let mut swap_data = swap_data.borrow_mut();
                let game_object_to_delete = swap_data.m_game_object_to_delete.as_mut().unwrap();
                while !game_object_to_delete.is_empty() {
                    let gobject = game_object_to_delete.get_next_process_object();
                    for (_part_index, game_object_part) in
                        gobject.get_object_parts().iter().enumerate()
                    {
                        match game_object_part.m_mesh_desc {
                            GameObjectMeshDesc::DynamicMesh(ref mesh_desc) => {
                                let mesh_source = MeshSourceDesc {
                                    m_mesh_file: mesh_desc.borrow().m_mesh_file.clone(),
                                };
                                let asset_id = self
                                    .m_render_scene
                                    .get_mesh_asset_id_allocator()
                                    .borrow_mut()
                                    .alloc_guid(&mesh_source);
                                self.m_render_resource
                                    .borrow_mut()
                                    .destroy_game_object_render_resource(asset_id);
                            }
                            _ => {}
                        }
                    }
                    self.m_render_scene
                        .delete_entity_by_gobject_id(gobject.get_id());
                    game_object_to_delete.pop();
                }
            }
            self.m_swap_context.reset_game_object_to_delete_swap_data();
        }

        if swap_data.borrow().m_camera_swap_data.is_some() {
            {
                let mut swap_data = swap_data.borrow_mut();
                let camera_swap_data = swap_data.m_camera_swap_data.as_mut().unwrap();
                if let Some(m_fov_x) = &camera_swap_data.m_fov_x {
                    self.m_render_camera.borrow_mut().set_fov_x(*m_fov_x);
                }
                if let Some(m_view_matrix) = &camera_swap_data.m_view_matrix {
                    self.m_render_camera
                        .borrow_mut()
                        .set_main_view_matrix(m_view_matrix.clone());
                }
                if let Some(m_camera_type) = &camera_swap_data.m_camera_type {
                    self.m_render_camera
                        .borrow_mut()
                        .set_current_camera_type(*m_camera_type);
                }
            }
            self.m_swap_context.reset_camera_swap_data();
        }
    }

    fn render(&self, ui_runtime: &UiRuntime, forward_draw: bool) -> Result<()> {
        let rhi = &self.m_rhi;
        self.m_render_resource
            .borrow_mut()
            .reset_ring_buffer_offset(rhi.borrow().get_current_frame_index());
        {
            let mut rhi = rhi.borrow_mut();
            rhi.wait_for_fence()?;
            rhi.reset_command_pool()?;
            if rhi.prepare_before_pass(&|rhi: &VulkanRHI| {
                self.pass_update_after_recreate_swapchain(&rhi)
            })? {
                return Ok(());
            }
        }
        {
            let rhi = rhi.borrow();

            self.m_render_pipeline.borrow().draw(
                &rhi,
                &self.m_render_scene,
                &mut self.m_render_resource.borrow_mut().m_global_render_resource,
                ui_runtime,
                forward_draw,
            );

            self.m_debugdraw_manager.borrow_mut().draw(&rhi)?;
        }
        {
            let mut rhi = rhi.borrow_mut();
            rhi.submit_rendering(&|rhi: &VulkanRHI| {
                self.pass_update_after_recreate_swapchain(&rhi)
            })?;
        }
        self.m_render_resource
            .borrow_mut()
            .on_main_frame_submit_complete(&self.m_rhi.borrow());
        Ok(())
    }

    fn pass_update_after_recreate_swapchain(&self, rhi: &VulkanRHI) {
        self.m_render_pipeline
            .borrow_mut()
            .recreate_after_swapchain(
                rhi,
                &self.m_render_resource.borrow().m_global_render_resource,
            );
        self.m_debugdraw_manager
            .borrow_mut()
            .update_after_recreate_swap_chain(rhi);
    }
}
