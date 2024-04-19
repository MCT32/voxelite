use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[derive(BufferContents, Vertex)]
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

impl From<Vec<MyVertex>> for Model {
    fn from(value: Vec<MyVertex>) -> Self {
        Self { vertices: value }
    }
}
