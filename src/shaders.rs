use std::sync::Arc;

use anyhow::Result;
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


pub fn load_shaders(device: Arc<Device>) -> Result<(Arc<ShaderModule>, Arc<ShaderModule>)> {
    Ok((
        vs::load(device.clone())?,
        fs::load(device)?,
    ))
}
