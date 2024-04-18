use std::sync::Arc;

use vulkano::{device::Device, shader::ShaderModule};

pub mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "src/shaders/shader.vert",
    }
}

pub mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "src/shaders/shader.frag",
    }
}


pub fn load_shaders(device: Arc<Device>) -> (Arc<ShaderModule>, Arc<ShaderModule>) {
    (
        vs::load(device.clone()).expect("failed to create shader module"),
        fs::load(device).expect("failed to create shader module"),
    )
}
