use std::sync::Arc;

use anyhow::Result;
use vulkano::{device::{physical::PhysicalDevice, Device}, image::{view::ImageView, Image, ImageUsage}, render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass}, swapchain::{Surface, Swapchain, SwapchainCreateInfo}};
use winit::window::Window;

pub fn create_swapchain(device: Arc<Device>, physical_device: Arc<PhysicalDevice>, window: Arc<Window>, surface: Arc<Surface>) -> Result<(Arc<Swapchain>, Vec<Arc<Image>>)> {
    let caps = physical_device
        .surface_capabilities(&surface, Default::default())?;

    let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
    let image_format = physical_device
        .surface_formats(&surface, Default::default())?[0]
        .0;
    
    Ok(Swapchain::new(
        device,
        surface,
        SwapchainCreateInfo {
            min_image_count: caps.min_image_count + 1,
            image_format,
            image_extent: window.inner_size().into(),
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha,
            ..Default::default()
        },
    )?)
}


pub fn get_render_pass(device: Arc<Device>, swapchain: &Arc<Swapchain>) -> Result<Arc<RenderPass>> {
    Ok(vulkano::single_pass_renderpass!(
        device,
        attachments: {
            color: {
                // Set the format the same as the swapchain.
                format: swapchain.image_format(),
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
        },
        pass: {
            color: [color],
            depth_stencil: {},
        },
    )?)
}


pub fn get_framebuffers(
    images: &[Arc<Image>],
    render_pass: &Arc<RenderPass>,
) -> Result<Vec<Arc<Framebuffer>>> {
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone())?;

            Ok(Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )?)
        })
        .collect::<Result<Vec<_>>>()
}
