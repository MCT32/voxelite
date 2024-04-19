use std::sync::Arc;

use anyhow::Result;
use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter}};

use crate::types::MyVertex;


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
