use std::sync::Arc;

use types::MyVertex;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo};
use vulkano::sync::future::FenceSignalFuture;
use vulkano::{Validated, VulkanError};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::sync::{self, GpuFuture};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::swapchain::{acquire_next_image, SwapchainCreateInfo, SwapchainPresentInfo};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::{Event, WindowEvent};

use crate::types::Model;


mod window;
mod instance;
mod device;
mod swapchain;
mod pipeline;
mod buffer;
mod shaders;
mod types;


fn main() {
    let event_loop = EventLoop::new();

    let instance = instance::create_instance(&event_loop)
        .expect("could not create instance");

    let (window, surface) = window::create_window_and_surface(&event_loop, instance.clone())
        .expect("could not create window or surface");
    
    let (physical_device, device, queue) = device::setup_device(instance.clone(), surface.clone())
        .expect("could not set up device");

    let (mut swapchain, mut images) = swapchain::create_swapchain(device.clone(), physical_device.clone(), window.clone(), surface.clone())
        .expect("could not create swapchain");

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    let command_buffer_allocator = StandardCommandBufferAllocator::new(
        device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default()
    );

    let model: Model = vec![
        MyVertex { position: [-0.5,  0.5], color: [1.0, 0.0, 0.0] },
        MyVertex { position: [ 0.0, -0.5], color: [0.0, 1.0, 0.0] },
        MyVertex { position: [ 0.5,  0.5], color: [0.0, 0.0, 1.0] },
    ].into();

    let render_pass = swapchain::get_render_pass(device.clone(), &swapchain)
        .expect("could not get render pass");

    let mut framebuffers = swapchain::get_framebuffers(&images, &render_pass)
        .expect("could not get framebuffers");

    let (vs, fs) = shaders::load_shaders(device.clone())
        .expect("could not load shaders");

    let mut viewport = Viewport {
        offset: [0.0, 0.0],
        extent: window.inner_size().into(),
        depth_range: 0.0..=1.0,
    };

    let (mut layout, mut pipeline) = pipeline::get_pipeline(
        device.clone(),
        vs.clone(),
        fs.clone(),
        render_pass.clone(),
        viewport.clone(),
    ).expect("could not get pipeline");

    let mut window_resized = false;
    let mut recreate_swapchain = false;

    let frames_in_flight = images.len();
    let mut fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
    let mut previous_fence_i = 0;

    let mut frame: u32 = 0;

    let mut command_buffers: Option<Vec<Arc<PrimaryAutoCommandBuffer>>> = None;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            },
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                window_resized = true;
            },
            Event::MainEventsCleared => {
                if window_resized || recreate_swapchain {
                    recreate_swapchain = false;

                    let new_dimentions = window.inner_size();

                    (swapchain, images) = swapchain
                        .recreate(SwapchainCreateInfo {
                            image_extent: new_dimentions.into(),
                            ..swapchain.create_info()
                        })
                        .expect("failed to recreate swapchain: {e}");
                    framebuffers = swapchain::get_framebuffers(&images, &render_pass)
                        .expect("could not get framebuffers");

                    if window_resized {
                        window_resized = false;

                        viewport.extent = new_dimentions.into();
                        (layout, pipeline) = pipeline::get_pipeline(
                            device.clone(),
                            vs.clone(),
                            fs.clone(),
                            render_pass.clone(),
                            viewport.clone()
                        ).expect("could not get pipeline");
                    }
                }

                command_buffers = Some(framebuffers
                    .iter()
                    .map(|framebuffer| {
                        let mut builder = AutoCommandBufferBuilder::primary(
                            &command_buffer_allocator,
                            queue.queue_family_index(),
                            // Don't forget to write the correct buffer usage.
                            CommandBufferUsage::MultipleSubmit,
                        ).unwrap();
            
                        builder
                            .begin_render_pass(
                                RenderPassBeginInfo {
                                    clear_values: vec![Some([0.1, 0.1, 0.1, 1.0].into())],
                                    ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                                },
                                SubpassBeginInfo {
                                    contents: SubpassContents::Inline,
                                    ..Default::default()
                                },
                            ).unwrap()
                            .bind_pipeline_graphics(pipeline.clone()).unwrap()
                            .push_constants(layout.clone(), 0, shaders::vs::Push {
                                transform: [[1.0, 0.0],
                                            [0.0, 1.0]],
                                offset: [0.0, 0.0].into(),
                                color: [1.0, 0.0, 1.0],
                            }).unwrap();
                        
                        let length = model.bind_vertex_buffer(&mut builder, memory_allocator.clone()).unwrap();
                        model.draw(&mut builder, length).unwrap();
                            
                        builder.end_render_pass(SubpassEndInfo::default()).unwrap();
            
                        builder.build().unwrap()
                    })
                    .collect());

                let (image_i, suboptimal, aquire_future) =
                    match acquire_next_image(swapchain.clone(), None)
                        .map_err(Validated::unwrap)
                    {
                        Ok(r) => r,
                        Err(VulkanError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("failed to aquire next image: {e}"),
                    };

                if suboptimal {
                    recreate_swapchain = true;
                }

                if let Some(image_fence) = &fences[image_i as usize] {
                    image_fence.wait(None).unwrap();
                }

                let previous_future = match fences[previous_fence_i as usize].clone() {
                    None => {
                        let mut now = sync::now(device.clone());
                        now.cleanup_finished();

                        now.boxed()
                    }
                    Some(fence) => fence.boxed()
                };

                let future = previous_future
                    .join(aquire_future)
                    .then_execute(queue.clone(), command_buffers.as_mut().unwrap()[image_i as usize].clone())
                    .unwrap()
                    .then_swapchain_present(
                        queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_i),
                    )
                    .then_signal_fence_and_flush();

                fences[image_i as usize] = match future.map_err(Validated::unwrap) {
                    Ok(value) => Some(Arc::new(value)),
                    Err(VulkanError::OutOfDate) => {
                        recreate_swapchain = true;
                        None
                    }
                    Err(e) => {
                        println!("failed to flush future: {e}");
                        None
                    }
                };

                previous_fence_i = image_i;

                frame = (frame + 1).rem_euclid(100);
            },
            _ => ()
        }
    });
}
