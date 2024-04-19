use std::sync::Arc;

use anyhow::Result;
use vulkano::{instance::Instance, swapchain::Surface};
use winit::{event_loop::EventLoop, window::{Window, WindowBuilder}};

pub fn create_window_and_surface(event_loop: &EventLoop<()>, instance: Arc<Instance>) -> Result<(Arc<Window>, Arc<Surface>)> {
    let window = Arc::new(WindowBuilder::new().build(&event_loop)?);
    let surface = Surface::from_window(instance.clone(), window.clone())?;

    Ok((window, surface))
}
