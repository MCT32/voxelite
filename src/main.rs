use std::sync::Arc;

use types::MyVertex;
use vulkano::sync::future::FenceSignalFuture;
use vulkano::{Validated, VulkanError};
use vulkano::memory::allocator::{StandardMemoryAllocator, AllocationCreateInfo, MemoryTypeFilter};
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::sync::{self, GpuFuture};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::swapchain::{acquire_next_image, SwapchainCreateInfo, SwapchainPresentInfo};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::{Event, WindowEvent};


mod window;
mod instance;
mod device;
mod swapchain;
mod pipeline;
mod command_buffers;
mod shaders;
mod types;


fn main() {
    let event_loop = EventLoop::new();

    let instance = instance::create_instance(&event_loop);

    let (window, surface) = window::create_window_and_surface(&event_loop, instance.clone());
    
    let (physical_device, device, queue) = device::setup_device(instance.clone(), surface.clone());

    let (mut swapchain, mut images) = swapchain::create_swapchain(device.clone(), physical_device.clone(), window.clone(), surface.clone());

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    let command_buffer_allocator = StandardCommandBufferAllocator::new(
        device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default()
    );

    let vertex1 = MyVertex { position: [-0.5,  0.5], color: [1.0, 0.0, 0.0] };
    let vertex2 = MyVertex { position: [ 0.0, -0.5], color: [0.0, 1.0, 0.0] };
    let vertex3 = MyVertex { position: [ 0.5,  0.5], color: [0.0, 0.0, 1.0] };

    let vertex_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        vec![vertex1, vertex2, vertex3],
    )
    .expect("failed to create buffer");

    let render_pass = swapchain::get_render_pass(device.clone(), &swapchain);

    let mut framebuffers = swapchain::get_framebuffers(&images, &render_pass);

    let (vs, fs) = shaders::load_shaders(device.clone());

    let mut viewport = Viewport {
        offset: [0.0, 0.0],
        extent: window.inner_size().into(),
        depth_range: 0.0..=1.0,
    };

    let (mut pipeline, mut layout) = pipeline::get_pipeline(
        device.clone(),
        vs.clone(),
        fs.clone(),
        render_pass.clone(),
        viewport.clone(),
    );

    let mut window_resized = false;
    let mut recreate_swapchain = false;

    let frames_in_flight = images.len();
    let mut fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
    let mut previous_fence_i = 0;

    let mut frame = 0;

    let mut command_buffers = command_buffers::get_command_buffers(
        &command_buffer_allocator,
        &queue,
        &layout,
        &pipeline,
        &framebuffers,
        &vertex_buffer,
        &frame,
    );

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
                    framebuffers = swapchain::get_framebuffers(&images, &render_pass);

                    if window_resized {
                        window_resized = false;

                        viewport.extent = new_dimentions.into();
                        (pipeline, layout) = pipeline::get_pipeline(
                            device.clone(),
                            vs.clone(),
                            fs.clone(),
                            render_pass.clone(),
                            viewport.clone()
                        );
                    }
                }

                command_buffers = command_buffers::get_command_buffers(
                    &command_buffer_allocator,
                    &queue,
                    &layout,
                    &pipeline,
                    &framebuffers,
                    &vertex_buffer,
                    &frame,
                );

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
                    .then_execute(queue.clone(), command_buffers[image_i as usize].clone())
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
