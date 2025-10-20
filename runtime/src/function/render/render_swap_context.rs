use std::{cell::RefCell, collections::VecDeque};

use nalgebra_glm::Mat4;

use crate::function::render::{render_camera::RenderCameraType, render_object::GameObjectDesc};

pub struct LevelIBLResourceDesc{
    // m_skybox_irradiance_map: SkyBoxIrradianceMap,
    // m_skybox_specular_map: SkyBoxSpecularMap,
    // m_brdf_map: String,
}

pub struct LevelColorGradingResourceDesc {
    pub m_color_grading_map: String,
}

pub struct LevelResourceDesc {
    pub m_ibl_resource_desc: LevelIBLResourceDesc,
    pub m_color_grading_resource_desc: LevelColorGradingResourceDesc,
}

#[derive(Default, Clone, Copy)]
pub struct CameraSwapData{
    pub m_fov_x: Option<f32>,
    pub m_camera_type: Option<RenderCameraType>,
    pub m_view_matrix: Option<Mat4>,
}

#[derive(Default, Clone)]
pub struct GameObjectResourceDesc{
    m_game_object_descs: VecDeque<GameObjectDesc>,
}

impl GameObjectResourceDesc{
    fn add(&mut self, desc: GameObjectDesc){
        self.m_game_object_descs.push_back(desc);
    }

    pub fn pop(&mut self){
        self.m_game_object_descs.pop_front();
    }

    pub fn is_empty(&self) -> bool{
        self.m_game_object_descs.is_empty()
    }

    pub fn get_next_process_object(&self) -> &GameObjectDesc {
        self.m_game_object_descs.front().unwrap()
    }
}

#[derive(Default, Clone)]
pub struct RenderSwapData{
    pub m_camera_swap_data: Option<CameraSwapData>,
    pub m_game_object_resource_descs: Option<GameObjectResourceDesc>,
}

impl RenderSwapData{
    pub fn add_dirty_game_object(&mut self, desc: &GameObjectDesc){
        match &mut self.m_game_object_resource_descs{
            Some(resource_desc) => {
                resource_desc.add(desc.clone());
            },
            None => {
                let mut resource_desc = GameObjectResourceDesc::default();
                resource_desc.add(desc.clone());
                self.m_game_object_resource_descs = Some(resource_desc);
            }
        }
    }
}

pub enum SwapDataType{
    LogicSwapDataType,
    RenderSwapDataType,
    SwapDataTypeCount,
}

pub struct RenderSwapContext{
    m_logic_swap_data_index: usize,
    m_render_swap_data_index: usize,
    m_swap_data: [RefCell<RenderSwapData>; SwapDataType::SwapDataTypeCount as usize],
}

impl Default for RenderSwapContext {
    fn default() -> Self {
        Self {
            m_logic_swap_data_index: SwapDataType::LogicSwapDataType as usize,
            m_render_swap_data_index: SwapDataType::RenderSwapDataType as usize,
            m_swap_data: std::array::from_fn(|_| RefCell::new(RenderSwapData::default())),
        }
    }
}

impl RenderSwapContext{
    pub fn get_logic_swap_data(&self) -> &RefCell<RenderSwapData> {
        &self.m_swap_data[self.m_logic_swap_data_index]
    }

    pub fn get_render_swap_data(&self) -> &RefCell<RenderSwapData> {
        &self.m_swap_data[self.m_render_swap_data_index]
    }

    pub fn swap_logic_render_data(&mut self) {
        if self.is_ready_to_swap() {
            self.swap();
        }
    }
}

impl RenderSwapContext{
    fn is_ready_to_swap(&self) -> bool {
        self.m_swap_data[self.m_render_swap_data_index].borrow().m_camera_swap_data.is_none() &&
        self.m_swap_data[self.m_render_swap_data_index].borrow().m_game_object_resource_descs.is_none()
    }

    pub fn reset_game_object_resource_swap_data(&self){
        self.m_swap_data[self.m_render_swap_data_index].borrow_mut().m_game_object_resource_descs = None;
    }

    pub fn reset_camera_swap_data(&self){
        self.m_swap_data[self.m_render_swap_data_index].borrow_mut().m_camera_swap_data = None;
    }

    fn swap(&mut self) {
        self.reset_game_object_resource_swap_data();
        self.reset_camera_swap_data();
        std::mem::swap(&mut self.m_logic_swap_data_index, &mut self.m_render_swap_data_index);
    }
}