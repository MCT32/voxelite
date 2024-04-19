use std::sync::Arc;

use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo}, device::Queue, memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter}, pipeline::{GraphicsPipeline, PipelineLayout}, render_pass::Framebuffer};

use crate::{shaders, types::MyVertex};


pub fn create_vertex_buffer(memory_allocator: Arc<dyn MemoryAllocator>, vertices: Vec<MyVertex>) -> Subbuffer<[MyVertex]> {
    Buffer::from_iter(
        memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        vertices,
    )
    .expect("failed to create buffer")
}

pub fn get_command_buffers(
    command_buffer_allocator: &StandardCommandBufferAllocator,
    queue: &Arc<Queue>,
    layout: &Arc<PipelineLayout>,
    pipeline: &Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    vertex_buffer: &Subbuffer<[MyVertex]>,
    frame: &i32,
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    framebuffers
        .iter()
        .map(|framebuffer| {
            let mut builder = AutoCommandBufferBuilder::primary(
                command_buffer_allocator,
                queue.queue_family_index(),
                // Don't forget to write the correct buffer usage.
                CommandBufferUsage::MultipleSubmit,
            )
            .unwrap();

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
                )
                .unwrap()
                .bind_pipeline_graphics(pipeline.clone())
                .unwrap()
                .bind_vertex_buffers(0, vertex_buffer.clone())
                .unwrap();

            for i in 0..4 {
                builder.push_constants(layout.clone(), 0, shaders::vs::PushConstantData {
                    offset: [-0.5 + *frame as f32 * 0.01, 0.5 - i as f32 / 3.0].into(),
                    color: [0.0, 0.0, 0.25 + i as f32 / 3.0],
                })
                .unwrap()
                .draw(vertex_buffer.len() as u32, 1, 0, 0)
                .unwrap();
            }
                
            builder.end_render_pass(SubpassEndInfo::default()).unwrap();

            builder.build().unwrap()
        })
        .collect()
}
