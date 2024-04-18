use std::sync::Arc;

use vulkano::{instance::Instance, swapchain::Surface};
use winit::{event_loop::EventLoop, window::{Window, WindowBuilder}};

pub fn create_window_and_surface(event_loop: &EventLoop<()>, instance: Arc<Instance>) -> (Arc<Window>, Arc<Surface>) {
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

    (window, surface)
}
