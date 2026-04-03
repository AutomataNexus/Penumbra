use penumbra_backend::*;
use std::collections::HashMap;

#[allow(dead_code)]
pub struct MeshData {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub vertex_count: u32,
    pub index_count: u32,
    pub aabb: Aabb,
}

pub struct TextureData {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub format: TextureFormat,
}

#[allow(dead_code)]
pub struct Resources {
    pub meshes: HashMap<u64, MeshData>,
    pub textures: HashMap<u64, TextureData>,
    pub buffers: HashMap<u64, wgpu::Buffer>,
    pub render_pipelines: HashMap<u64, wgpu::RenderPipeline>,
    pub compute_pipelines: HashMap<u64, wgpu::ComputePipeline>,
    pub bind_groups: HashMap<u64, wgpu::BindGroup>,
    pub bind_group_layouts: HashMap<u64, wgpu::BindGroupLayout>,
    pub samplers: HashMap<u64, wgpu::Sampler>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
            textures: HashMap::new(),
            buffers: HashMap::new(),
            render_pipelines: HashMap::new(),
            compute_pipelines: HashMap::new(),
            bind_groups: HashMap::new(),
            bind_group_layouts: HashMap::new(),
            samplers: HashMap::new(),
        }
    }
}

/// Bytes per pixel for a given texture format.
pub fn bytes_per_pixel(format: TextureFormat) -> u32 {
    match format {
        TextureFormat::R8Unorm => 1,
        TextureFormat::Rg8Unorm => 2,
        TextureFormat::Rgba8Unorm
        | TextureFormat::Rgba8UnormSrgb
        | TextureFormat::Bgra8Unorm
        | TextureFormat::Bgra8UnormSrgb => 4,
        TextureFormat::Rgba16Float => 8,
        TextureFormat::Rgba32Float => 16,
        TextureFormat::Depth32Float => 4,
        TextureFormat::Depth24PlusStencil8 => 4,
    }
}
