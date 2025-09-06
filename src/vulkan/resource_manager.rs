use std::collections::HashMap;
use vulkanalia::prelude::v1_0::*;

pub type ResourceManager<T> = HashMap<&'static str, T>;

use crate::vulkan::{Destroy};

impl<T: Destroy> Destroy for ResourceManager<T>{
    fn destroy(&mut self, device: &Device) {
        for data in self.values_mut() {
            data.destroy(device);
        }
        self.clear();
    }
}