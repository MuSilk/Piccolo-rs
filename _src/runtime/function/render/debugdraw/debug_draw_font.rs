use anyhow::Result;

use crate::runtime::function::{global::global_context::RuntimeGlobalContext, render::{interface::rhi_struct::{RHIDeviceMemory, RHIImage, RHIImageView}, render_type::RHIFormat}};

#[derive(Default)]
pub struct DebugDrawFont{
    m_font_image: Option<Box<dyn RHIImage>>,
    m_font_image_view: Option<Box<dyn RHIImageView>>,
    m_font_image_memory: Option<Box<dyn RHIDeviceMemory>>,
}

impl DebugDrawFont {
    const RANGE_L : u8 = 32;
    const RANGE_R : u8 = 126;
    const SINGLE_CHARACTER_WIDTH : i32 = 32;
    const SINGLE_CHARACTER_HEIGHT : i32 = 64;
    const NUM_OF_CHARACTER_IN_ONE_LINE : i32 = 16;
    const NUM_OF_CHARACTER: i32 = (Self::RANGE_R - Self::RANGE_L + 1) as i32;
    const BITMAP_H: i32 = Self::SINGLE_CHARACTER_WIDTH * Self::NUM_OF_CHARACTER_IN_ONE_LINE;
    const BITMAP_W: i32 = Self::SINGLE_CHARACTER_HEIGHT * ((Self::NUM_OF_CHARACTER + Self::NUM_OF_CHARACTER_IN_ONE_LINE - 1) / Self::NUM_OF_CHARACTER_IN_ONE_LINE);

    pub fn get_character_texture_rect(character: u8) -> (f32, f32, f32, f32) { 
        if character >= Self::RANGE_L && character <= Self::RANGE_R {(
            ((character - Self::RANGE_L) as i32 % Self::NUM_OF_CHARACTER_IN_ONE_LINE * Self::SINGLE_CHARACTER_WIDTH) as f32 / Self::BITMAP_W as f32,
            ((character - Self::RANGE_L) as i32 % Self::NUM_OF_CHARACTER_IN_ONE_LINE * Self::SINGLE_CHARACTER_WIDTH + Self::SINGLE_CHARACTER_WIDTH) as f32 / Self::BITMAP_W as f32,
            ((character - Self::RANGE_L) as i32 / Self::NUM_OF_CHARACTER_IN_ONE_LINE * Self::SINGLE_CHARACTER_HEIGHT) as f32 / Self::BITMAP_H as f32,
            ((character - Self::RANGE_L) as i32 / Self::NUM_OF_CHARACTER_IN_ONE_LINE * Self::SINGLE_CHARACTER_HEIGHT + Self::SINGLE_CHARACTER_HEIGHT) as f32 / Self::BITMAP_H as f32,
        )}
        else{
            (0.0, 0.0, 0.0, 0.0)
        }
    }

    pub fn get_image_view(&self) -> &Box<dyn RHIImageView> {
        &self.m_font_image_view.as_ref().unwrap()
    }

    pub fn initialize(&mut self) -> Result<()>{
        self.load_font()?;
        Ok(())
    }

    pub fn destroy(&mut self){
        let rhi = RuntimeGlobalContext::global().borrow().m_render_system.borrow().get_rhi();
        let rhi = rhi.borrow();
        rhi.free_memory(self.m_font_image_memory.take().unwrap());
        rhi.destroy_image_view(self.m_font_image_view.take().unwrap());
        rhi.destroy_image(self.m_font_image.take().unwrap());
    }

    fn load_font(&mut self) -> Result<()> {
        let mut image_data = Vec::<f32>::new();
        image_data.resize((Self::BITMAP_W*Self::BITMAP_H) as usize, 0.0);
        //todo
        let pixels = unsafe {
            std::slice::from_raw_parts(
                image_data.as_ptr() as *const u8,
                image_data.len() * std::mem::size_of::<f32>()
            )
        };
        let rhi = RuntimeGlobalContext::global().borrow().m_render_system.borrow().get_rhi();
        let rhi = rhi.borrow();
        let (image, image_memory,image_view) = rhi.create_texture_image(
            Self::BITMAP_W as u32,
            Self::BITMAP_H as u32, 
            pixels,
            RHIFormat::R32_SFLOAT, 
            0)?;

        self.m_font_image = Some(image);
        self.m_font_image_memory = Some(image_memory);
        self.m_font_image_view = Some(image_view);
        
        Ok(())
    }
}


