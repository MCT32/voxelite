use std::sync::Arc;

use anyhow::Result;
use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo}, device::Queue, memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter}, pipeline::{GraphicsPipeline, PipelineLayout}, render_pass::Framebuffer};

use crate::{shaders, types::MyVertex};


pub fn create_vertex_buffer(memory_allocator: Arc<dyn MemoryAllocator>, vertices: Vec<MyVertex>) -> Result<Subbuffer<[MyVertex]>> {
    Ok(Buffer::from_iter(
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
    )?)
}

pub fn get_command_buffers(
    command_buffer_allocator: &StandardCommandBufferAllocator,
    queue: &Arc<Queue>,
    layout: &Arc<PipelineLayout>,
    pipeline: &Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    vertex_buffer: &Subbuffer<[MyVertex]>,
    frame: &i32,
) -> Result<Vec<Arc<PrimaryAutoCommandBuffer>>> {
    framebuffers
        .iter()
        .map(|framebuffer| {
            let mut builder = AutoCommandBufferBuilder::primary(
                command_buffer_allocator,
                queue.queue_family_index(),
                // Don't forget to write the correct buffer usage.
                CommandBufferUsage::MultipleSubmit,
            )?;

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
                )?
                .bind_pipeline_graphics(pipeline.clone())?
                .bind_vertex_buffers(0, vertex_buffer.clone())?;

            builder.draw(vertex_buffer.len() as u32, 1, 0, 0)?;
                
            builder.end_render_pass(SubpassEndInfo::default())?;

            Ok(builder.build()?)
        })
        .collect::<Result<Vec<Arc<PrimaryAutoCommandBuffer>>>>()
}
