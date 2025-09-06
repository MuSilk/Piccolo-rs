use vulkanalia::{prelude::v1_0::*};
pub trait VertexLayout {
    fn binding_description() -> vk::VertexInputBindingDescription;
    fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription>;
}