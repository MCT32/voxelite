use std::sync::Arc;

use anyhow::Result;
use vulkano::{instance::{Instance, InstanceCreateInfo}, swapchain::Surface, VulkanLibrary};
use winit::event_loop::EventLoop;

pub fn create_instance(event_loop: &EventLoop<()>) -> Result<Arc<Instance>> {
    let library = VulkanLibrary::new()?;
    let required_extensions = Surface::required_extensions(&event_loop);
    
    Ok(Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            ..Default::default()
        }
    )?)
}
