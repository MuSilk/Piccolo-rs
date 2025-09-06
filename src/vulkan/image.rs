use vulkanalia::prelude::v1_0::*;

#[derive(Clone, Debug, Default)]
pub struct Image{
    pub image: vk::Image,
    pub image_view: vk::ImageView,
}

impl Image {
    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_image_view(self.image_view, None);
            device.destroy_image(self.image, None);
        }
    }
}