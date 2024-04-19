use std::sync::Arc;

use anyhow::Result;
use vulkano::{buffer::BufferContents, command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer}, memory::allocator::MemoryAllocator, pipeline::graphics::vertex_input::Vertex};

use crate::buffer::create_vertex_buffer;

#[derive(BufferContents, Vertex, Clone)]
#[repr(C)]
pub struct MyVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],
}

pub struct Model {
    pub vertices: Vec<MyVertex>,
}

impl Model {
    pub fn bind_vertex_buffer(&self, builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, memory_allocator: Arc<dyn MemoryAllocator>) -> Result<u32> {
        let vertex_buffer = create_vertex_buffer(memory_allocator, self.vertices.clone())?;

        let length = vertex_buffer.len();

        builder.bind_vertex_buffers(0, vertex_buffer)?;

        Ok(length as u32)
    }

    pub fn draw(&self, builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, buffer_length: u32) -> Result<()> {
        builder.draw(buffer_length, 1, 0, 0)?;

        Ok(())
    }
}

impl From<Vec<MyVertex>> for Model {
    fn from(value: Vec<MyVertex>) -> Self {
        Self { vertices: value }
    }
}
