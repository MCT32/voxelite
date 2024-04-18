use std::sync::Arc;

use vulkano::{instance::{Instance, InstanceCreateInfo}, swapchain::Surface, VulkanLibrary};
use winit::event_loop::EventLoop;

pub fn create_instance(event_loop: &EventLoop<()>) -> Arc<Instance> {
    let library = VulkanLibrary::new().expect("no Vulkan library");
    let required_extensions = Surface::required_extensions(&event_loop);
    Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            ..Default::default()
        }
    )
    .expect("failed to create instance")
}
