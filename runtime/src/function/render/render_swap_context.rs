use nalgebra_glm::Mat4;

use crate::function::render::render_camera::RenderCameraType;


#[derive(Default, Clone, Copy)]
pub struct CameraSwapData{
    pub m_fov_x: Option<f32>,
    pub m_camera_type: Option<RenderCameraType>,
    pub m_view_matrix: Option<Mat4>,
}

#[derive(Default, Clone, Copy)]
pub struct RenderSwapData{
    pub m_camera_swap_data: Option<CameraSwapData>,
}

pub enum SwapDataType{
    LogicSwapDataType,
    RenderSwapDataType,
    SwapDataTypeCount,
}

pub struct RenderSwapContext{
    m_logic_swap_data_index: usize,
    m_render_swap_data_index: usize,
    m_swap_data: [RenderSwapData; SwapDataType::SwapDataTypeCount as usize],
}

impl Default for RenderSwapContext {
    fn default() -> Self {
        Self {
            m_logic_swap_data_index: SwapDataType::LogicSwapDataType as usize,
            m_render_swap_data_index: SwapDataType::RenderSwapDataType as usize,
            m_swap_data: [RenderSwapData::default(); SwapDataType::SwapDataTypeCount as usize],
        }
    }
}

impl RenderSwapContext{
    pub fn get_logic_swap_data(&self) -> &RenderSwapData {
        &self.m_swap_data[self.m_logic_swap_data_index]
    }

    pub fn get_render_swap_data(&self) -> &RenderSwapData {
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
        self.m_swap_data[self.m_render_swap_data_index].m_camera_swap_data.is_none()
    }

    pub fn reset_camera_swap_data(&mut self){
        self.m_swap_data[self.m_render_swap_data_index].m_camera_swap_data = None;
    }

    fn swap(&mut self) {
        self.reset_camera_swap_data();
        std::mem::swap(&mut self.m_logic_swap_data_index, &mut self.m_render_swap_data_index);
    }
}