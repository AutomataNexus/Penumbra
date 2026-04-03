use std::collections::HashMap;
use std::time::Instant;
use tracing::info;

use penumbra_backend::{
    BackendCapabilities, BackendError, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BufferDescriptor, BufferUsage, ClearColor, ColorAttachment, ColorTargetState,
    ColorWrites, CompareFunction, DepthAttachment, DepthStencilState, GpuBuffer, GpuMesh,
    GpuTexture, LoadOp, MeshDescriptor, MeshId, MultisampleState, PipelineDescriptor, PipelineId,
    PrimitiveState, RenderBackend, RenderPassDescriptor, ShaderSource, ShaderStages, StoreOp,
    TextureDescriptor, TextureFormat, TextureId, TextureUsage, Vertex, VertexAttribute,
    VertexFormat, VertexLayout, VertexStepMode,
};

use crate::frame::RenderFrame;
use crate::material::{Material, MaterialId};

/// Renderer configuration.
#[derive(Debug, Clone)]
pub struct RendererConfig {
    pub width: u32,
    pub height: u32,
    pub msaa_samples: u32,
    pub hdr: bool,
    pub vsync: bool,
    pub max_instances: u32,
    pub tile_cache_mb: u32,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            msaa_samples: 1,
            hdr: false,
            vsync: true,
            max_instances: 65536,
            tile_cache_mb: 256,
        }
    }
}

/// Per-frame performance statistics.
#[derive(Debug, Clone, Default)]
pub struct FrameStats {
    pub frame_time_ms: f32,
    pub fps: f32,
    pub draw_calls: u32,
    pub triangles: u64,
    pub instances: u32,
    pub tiles_loaded: u32,
    pub tiles_streaming: u32,
    pub gpu_memory_mb: u32,
}

/// The central renderer. Owns the backend and manages frame lifecycle.
pub struct Renderer {
    backend: Box<dyn RenderBackend>,
    config: RendererConfig,
    frame_stats: FrameStats,
    start_time: Instant,
    last_frame_time: Instant,
    frame_count: u64,
    materials: HashMap<MaterialId, Material>,
    next_material_id: u64,
    // Render resources
    default_pipeline: Option<PipelineId>,
    depth_texture: Option<GpuTexture>,
    camera_buffer: Option<GpuBuffer>,
    transform_buffer: Option<GpuBuffer>,
}

impl Renderer {
    pub fn new(backend: impl RenderBackend + 'static, config: RendererConfig) -> Self {
        let now = Instant::now();
        info!(
            backend = backend.backend_name(),
            width = config.width,
            height = config.height,
            "Renderer created"
        );
        Self {
            backend: Box::new(backend),
            config,
            frame_stats: FrameStats::default(),
            start_time: now,
            last_frame_time: now,
            frame_count: 0,
            materials: HashMap::new(),
            next_material_id: 1,
            default_pipeline: None,
            depth_texture: None,
            camera_buffer: None,
            transform_buffer: None,
        }
    }

    /// Initialize the default rendering pipeline. Must be called after the backend
    /// has a surface (for windowed rendering) so the surface format is known.
    pub fn init_pipeline(&mut self) -> Result<(), BackendError> {
        let format = self.backend.surface_format();

        // Create depth texture
        let depth = self.backend.create_texture(TextureDescriptor {
            width: self.config.width,
            height: self.config.height,
            format: TextureFormat::Depth32Float,
            usage: TextureUsage::RENDER_ATTACHMENT,
            data: None,
            label: Some("depth".to_string()),
            mip_levels: 1,
        })?;
        self.depth_texture = Some(depth);

        // Camera uniform buffer (CameraUniforms = 352 bytes)
        let cam_buf = self.backend.create_buffer(BufferDescriptor {
            size: std::mem::size_of::<crate::frame::CameraUniforms>() as u64,
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            label: Some("camera_uniforms".to_string()),
            mapped_at_creation: false,
        })?;
        self.camera_buffer = Some(cam_buf);

        // Per-object transform buffer (Mat4 + Mat4 normal = 128 bytes)
        let transform_buf = self.backend.create_buffer(BufferDescriptor {
            size: 128,
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            label: Some("transform_uniforms".to_string()),
            mapped_at_creation: false,
        })?;
        self.transform_buffer = Some(transform_buf);

        // Default PBR pipeline
        let vertex_shader = ShaderSource::Wgsl(DEFAULT_VERTEX_SHADER.to_string());
        let fragment_shader = ShaderSource::Wgsl(DEFAULT_FRAGMENT_SHADER.to_string());

        let pipeline = self.backend.create_pipeline(PipelineDescriptor {
            vertex_shader,
            fragment_shader,
            vertex_layout: VertexLayout {
                stride: std::mem::size_of::<Vertex>() as u64,
                attributes: vec![
                    VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    },
                    VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: 12,
                        shader_location: 1,
                    },
                    VertexAttribute {
                        format: VertexFormat::Float32x2,
                        offset: 24,
                        shader_location: 2,
                    },
                    VertexAttribute {
                        format: VertexFormat::Float32x4,
                        offset: 32,
                        shader_location: 3,
                    },
                ],
                step_mode: VertexStepMode::Vertex,
            },
            primitive: PrimitiveState::default(),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write: true,
                depth_compare: CompareFunction::Less,
            }),
            multisample: MultisampleState::default(),
            color_targets: vec![ColorTargetState {
                format,
                blend: None,
                write_mask: ColorWrites::ALL,
            }],
            bind_group_layouts: vec![
                // Group 0: Camera uniforms
                BindGroupLayoutDescriptor {
                    entries: vec![BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::UniformBuffer,
                    }],
                    label: Some("camera_layout".to_string()),
                },
                // Group 1: Model transform
                BindGroupLayoutDescriptor {
                    entries: vec![BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::UniformBuffer,
                    }],
                    label: Some("model_layout".to_string()),
                },
            ],
            label: Some("default_pbr".to_string()),
        })?;
        self.default_pipeline = Some(pipeline);

        info!("Default render pipeline initialized");
        Ok(())
    }

    pub fn begin_frame(&mut self) -> Result<RenderFrame, BackendError> {
        let now = Instant::now();
        let time = now.duration_since(self.start_time).as_secs_f64();
        let delta = now.duration_since(self.last_frame_time).as_secs_f32();

        self.backend.begin_frame()?;

        Ok(RenderFrame::new(
            self.config.width,
            self.config.height,
            time,
            delta,
        ))
    }

    pub fn end_frame(&mut self, frame: RenderFrame) -> Result<(), BackendError> {
        let now = Instant::now();
        let frame_time_ms = now.duration_since(self.last_frame_time).as_secs_f32() * 1000.0;

        // Execute draw calls if we have a pipeline and surface
        if let (Some(pipeline), Some(depth_tex), Some(cam_buf)) = (
            self.default_pipeline,
            &self.depth_texture,
            &self.camera_buffer,
        ) {
            // Upload camera uniforms
            let cam_data = bytemuck::bytes_of(&frame.camera);
            self.backend.write_buffer(cam_buf.id, 0, cam_data);

            // Get the surface texture to render to
            if let Ok(surface_tex) = self.backend.surface_texture() {
                // Begin render pass
                let pass = self.backend.begin_render_pass(RenderPassDescriptor {
                    color_attachments: vec![ColorAttachment {
                        texture: surface_tex,
                        resolve_target: None,
                        load_op: LoadOp::Clear(ClearColor {
                            r: 0.05,
                            g: 0.05,
                            b: 0.08,
                            a: 1.0,
                        }),
                        store_op: StoreOp::Store,
                    }],
                    depth_attachment: Some(DepthAttachment {
                        texture: depth_tex.id,
                        depth_load_op: LoadOp::Clear(ClearColor {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        depth_store_op: StoreOp::Store,
                    }),
                    label: Some("main_pass".to_string()),
                });

                self.backend.set_pipeline(pass, pipeline);

                // Create camera bind group
                let cam_bg = self.backend.create_bind_group(
                    &BindGroupLayoutDescriptor {
                        entries: vec![BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::VERTEX_FRAGMENT,
                            ty: BindingType::UniformBuffer,
                        }],
                        label: Some("camera_bg".to_string()),
                    },
                    &[penumbra_backend::traits::BindGroupEntry::Buffer {
                        binding: 0,
                        buffer: cam_buf.id,
                        offset: 0,
                        size: std::mem::size_of::<crate::frame::CameraUniforms>() as u64,
                    }],
                )?;
                self.backend.set_bind_group(pass, 0, cam_bg);

                // Execute each draw call
                let transform_buf_id = self.transform_buffer.as_ref().map(|b| b.id);
                for draw in frame.draw_calls() {
                    if let Some(tb_id) = transform_buf_id {
                        // Upload model + normal matrix
                        let model = draw.transform;
                        let normal = model.inverse().transpose();
                        let mut transform_data = [0u8; 128];
                        transform_data[..64]
                            .copy_from_slice(bytemuck::bytes_of(&model.to_cols_array_2d()));
                        transform_data[64..128]
                            .copy_from_slice(bytemuck::bytes_of(&normal.to_cols_array_2d()));
                        self.backend.write_buffer(tb_id, 0, &transform_data);

                        // Create model bind group
                        let model_bg = self.backend.create_bind_group(
                            &BindGroupLayoutDescriptor {
                                entries: vec![BindGroupLayoutEntry {
                                    binding: 0,
                                    visibility: ShaderStages::VERTEX,
                                    ty: BindingType::UniformBuffer,
                                }],
                                label: Some("model_bg".to_string()),
                            },
                            &[penumbra_backend::traits::BindGroupEntry::Buffer {
                                binding: 0,
                                buffer: tb_id,
                                offset: 0,
                                size: 128,
                            }],
                        )?;
                        self.backend.set_bind_group(pass, 1, model_bg);
                    }

                    // Draw the mesh
                    self.backend
                        .draw_mesh(pass, draw.mesh, 0..draw.instance_count);
                }

                self.backend.end_render_pass(pass);
            }
        }

        self.frame_stats.frame_time_ms = frame_time_ms;
        self.frame_stats.fps = if frame_time_ms > 0.0 {
            1000.0 / frame_time_ms
        } else {
            0.0
        };
        self.frame_stats.draw_calls = frame.draw_count();
        self.last_frame_time = now;
        self.frame_count += 1;

        self.backend.end_frame()?;
        self.backend.present()?;
        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.backend.resize(width, height);

        // Recreate depth texture at new size
        if self.depth_texture.is_some() {
            if let Ok(depth) = self.backend.create_texture(TextureDescriptor {
                width,
                height,
                format: TextureFormat::Depth32Float,
                usage: TextureUsage::RENDER_ATTACHMENT,
                data: None,
                label: Some("depth".to_string()),
                mip_levels: 1,
            }) {
                self.depth_texture = Some(depth);
            }
        }

        info!(width, height, "Renderer resized");
    }

    pub fn stats(&self) -> &FrameStats {
        &self.frame_stats
    }

    pub fn config(&self) -> &RendererConfig {
        &self.config
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn capabilities(&self) -> BackendCapabilities {
        self.backend.capabilities()
    }

    pub fn backend_name(&self) -> &str {
        self.backend.backend_name()
    }

    // ── Resource creation (delegate to backend) ──

    pub fn create_mesh(&mut self, desc: MeshDescriptor) -> Result<GpuMesh, BackendError> {
        self.backend.create_mesh(desc)
    }

    pub fn create_texture(&mut self, desc: TextureDescriptor) -> Result<GpuTexture, BackendError> {
        self.backend.create_texture(desc)
    }

    pub fn create_buffer(&mut self, desc: BufferDescriptor) -> Result<GpuBuffer, BackendError> {
        self.backend.create_buffer(desc)
    }

    pub fn destroy_mesh(&mut self, id: MeshId) {
        self.backend.destroy_mesh(id);
    }

    pub fn destroy_texture(&mut self, id: TextureId) {
        self.backend.destroy_texture(id);
    }

    // ── Material management ──

    pub fn add_material(&mut self, mut material: Material) -> MaterialId {
        let id = MaterialId(self.next_material_id);
        self.next_material_id += 1;
        material.id = id;
        self.materials.insert(id, material);
        id
    }

    pub fn get_material(&self, id: MaterialId) -> Option<&Material> {
        self.materials.get(&id)
    }

    pub fn get_material_mut(&mut self, id: MaterialId) -> Option<&mut Material> {
        self.materials.get_mut(&id)
    }

    pub fn remove_material(&mut self, id: MaterialId) -> Option<Material> {
        self.materials.remove(&id)
    }

    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.backend.surface_format()
    }

    pub fn backend_mut(&mut self) -> &mut dyn RenderBackend {
        &mut *self.backend
    }

    pub fn backend(&self) -> &dyn RenderBackend {
        &*self.backend
    }
}

// ── Default shaders ──

const DEFAULT_VERTEX_SHADER: &str = r#"
struct CameraUniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_projection: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
    camera_position: vec3<f32>,
    _pad0: f32,
    near: f32,
    far: f32,
    _pad1: vec2<f32>,
};

struct ModelUniforms {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(1) @binding(0) var<uniform> model: ModelUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let world_pos = (model.model * vec4(in.position, 1.0)).xyz;
    let world_normal = normalize((model.normal_matrix * vec4(in.normal, 0.0)).xyz);

    var out: VertexOutput;
    out.clip_position = camera.view_projection * vec4(world_pos, 1.0);
    out.world_position = world_pos;
    out.world_normal = world_normal;
    out.uv = in.uv;
    return out;
}
"#;

const DEFAULT_FRAGMENT_SHADER: &str = r#"
struct CameraUniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_projection: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
    camera_position: vec3<f32>,
    _pad0: f32,
    near: f32,
    far: f32,
    _pad1: vec2<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.world_normal);
    let v = normalize(camera.camera_position - in.world_position);
    let l = normalize(vec3(-0.4, -0.8, -0.3));

    let ndl = max(dot(n, -l), 0.0);
    let h = normalize(v - l);
    let spec = pow(max(dot(n, h), 0.0), 32.0) * 0.3;

    let albedo = vec3(0.7, 0.7, 0.7);
    let ambient = vec3(0.05, 0.05, 0.08);
    let color = ambient + albedo * ndl * 0.9 + vec3(1.0) * spec;

    // Gamma
    let corrected = pow(color, vec3(1.0 / 2.2));
    return vec4(corrected, 1.0);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_config_default() {
        let config = RendererConfig::default();
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
        assert_eq!(config.msaa_samples, 1);
        assert!(!config.hdr);
        assert!(config.vsync);
        assert_eq!(config.max_instances, 65536);
    }

    #[test]
    fn frame_stats_default() {
        let stats = FrameStats::default();
        assert_eq!(stats.fps, 0.0);
        assert_eq!(stats.draw_calls, 0);
    }
}
