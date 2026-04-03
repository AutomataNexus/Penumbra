use std::ops::Range;
use tracing::info;

use penumbra_backend::traits::{BindGroupEntry, RenderBackend};
use penumbra_backend::*;

use crate::convert::*;
use crate::resources::{MeshData, Resources, TextureData, bytes_per_pixel};

/// Configuration for the wgpu backend.
#[derive(Debug, Clone)]
pub struct WgpuConfig {
    pub power_preference: wgpu::PowerPreference,
    pub present_mode: wgpu::PresentMode,
    pub features: wgpu::Features,
    pub limits: wgpu::Limits,
}

impl Default for WgpuConfig {
    fn default() -> Self {
        Self {
            power_preference: wgpu::PowerPreference::HighPerformance,
            present_mode: wgpu::PresentMode::AutoVsync,
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::downlevel_webgl2_defaults(),
        }
    }
}

/// Recorded render pass commands for deferred execution.
#[allow(dead_code)]
enum RecordedCommand {
    SetPipeline(PipelineId),
    SetBindGroup {
        index: u32,
        group: BindGroupId,
    },
    SetVertexBuffer {
        slot: u32,
        buffer: BufferSlice,
    },
    SetIndexBuffer(BufferSlice),
    Draw {
        vertices: Range<u32>,
        instances: Range<u32>,
    },
    DrawIndexed {
        indices: Range<u32>,
        base_vertex: i32,
        instances: Range<u32>,
    },
}

struct RecordedRenderPass {
    descriptor: RenderPassDescriptor,
    commands: Vec<RecordedCommand>,
}

struct RecordedComputePass {
    commands: Vec<RecordedComputeCommand>,
}

#[allow(dead_code)]
enum RecordedComputeCommand {
    SetPipeline(ComputePipelineId),
    SetBindGroup { index: u32, group: BindGroupId },
    Dispatch { x: u32, y: u32, z: u32 },
}

/// The default wgpu-based [`RenderBackend`] implementation.
pub struct WgpuBackend {
    device: wgpu::Device,
    queue: wgpu::Queue,
    adapter: wgpu::Adapter,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    current_surface_texture: Option<wgpu::SurfaceTexture>,
    surface_format: TextureFormat,
    resources: Resources,
    next_id: u64,
    render_passes: Vec<Option<RecordedRenderPass>>,
    compute_passes: Vec<Option<RecordedComputePass>>,
    width: u32,
    height: u32,
}

impl WgpuBackend {
    /// Create a headless backend (no surface) for testing and offscreen rendering.
    pub fn headless(width: u32, height: u32, config: WgpuConfig) -> Result<Self, BackendError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: config.power_preference,
            compatible_surface: None,
            force_fallback_adapter: true,
        }))
        .ok_or(BackendError::NotInitialized)?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("penumbra-wgpu"),
                required_features: config.features,
                required_limits: config.limits,
                ..Default::default()
            },
            None,
        ))
        .map_err(|e| BackendError::ResourceCreation(e.to_string()))?;

        info!(
            adapter = adapter.get_info().name,
            backend = ?adapter.get_info().backend,
            "WgpuBackend headless initialized"
        );

        Ok(Self {
            device,
            queue,
            adapter,
            surface: None,
            surface_config: None,
            current_surface_texture: None,
            surface_format: TextureFormat::Rgba8Unorm,
            resources: Resources::new(),
            next_id: 1,
            render_passes: Vec::new(),
            compute_passes: Vec::new(),
            width,
            height,
        })
    }

    /// Create a backend from a winit window with a real GPU surface.
    pub fn from_window(
        window: std::sync::Arc<winit::window::Window>,
        config: WgpuConfig,
    ) -> Result<Self, BackendError> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::DX12 | wgpu::Backends::METAL,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window)
            .map_err(|e| BackendError::ResourceCreation(e.to_string()))?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: config.power_preference,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or(BackendError::NotInitialized)?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("penumbra-wgpu"),
                required_features: config.features,
                required_limits: config.limits,
                ..Default::default()
            },
            None,
        ))
        .map_err(|e| BackendError::ResourceCreation(e.to_string()))?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: config.present_mode,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let penumbra_format = from_wgpu_texture_format(format);

        info!(
            adapter = adapter.get_info().name,
            backend = ?adapter.get_info().backend,
            format = ?format,
            "WgpuBackend windowed initialized"
        );

        Ok(Self {
            device,
            queue,
            adapter,
            surface: Some(surface),
            surface_config: Some(surface_config),
            current_surface_texture: None,
            surface_format: penumbra_format,
            resources: Resources::new(),
            next_id: 1,
            render_passes: Vec::new(),
            compute_passes: Vec::new(),
            width: size.width,
            height: size.height,
        })
    }

    /// Get a reference to the wgpu device (for advanced usage).
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get a reference to the wgpu queue (for advanced usage).
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl RenderBackend for WgpuBackend {
    fn create_mesh(&mut self, desc: MeshDescriptor) -> Result<GpuMesh, BackendError> {
        let id = self.alloc_id();

        let aabb = if desc.vertices.is_empty() {
            Aabb::new(glam::Vec3::ZERO, glam::Vec3::ZERO)
        } else {
            let points: Vec<glam::Vec3> = desc
                .vertices
                .iter()
                .map(|v| glam::Vec3::from_array(v.position))
                .collect();
            Aabb::from_points(&points)
        };

        let vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: desc.label.as_deref(),
            size: (desc.vertices.len() * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue
            .write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&desc.vertices));

        let index_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: desc.label.as_deref(),
            size: (desc.indices.len() * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue
            .write_buffer(&index_buffer, 0, bytemuck::cast_slice(&desc.indices));

        let vertex_count = desc.vertices.len() as u32;
        let index_count = desc.indices.len() as u32;

        self.resources.meshes.insert(
            id,
            MeshData {
                vertex_buffer,
                index_buffer,
                vertex_count,
                index_count,
                aabb,
            },
        );

        Ok(GpuMesh {
            id: MeshId(id),
            vertex_count,
            index_count,
            aabb,
        })
    }

    fn create_texture(&mut self, desc: TextureDescriptor) -> Result<GpuTexture, BackendError> {
        let id = self.alloc_id();
        let wgpu_format = to_wgpu_texture_format(desc.format);
        let wgpu_usage = to_wgpu_texture_usage(desc.usage);

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: desc.label.as_deref(),
            size: wgpu::Extent3d {
                width: desc.width,
                height: desc.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: desc.mip_levels.max(1),
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu_format,
            usage: wgpu_usage,
            view_formats: &[],
        });

        if let Some(ref data) = desc.data {
            let bpp = bytes_per_pixel(desc.format);
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(desc.width * bpp),
                    rows_per_image: Some(desc.height),
                },
                wgpu::Extent3d {
                    width: desc.width,
                    height: desc.height,
                    depth_or_array_layers: 1,
                },
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.resources.textures.insert(
            id,
            TextureData {
                texture,
                view,
                format: desc.format,
            },
        );

        Ok(GpuTexture {
            id: TextureId(id),
            width: desc.width,
            height: desc.height,
            format: desc.format,
        })
    }

    fn create_buffer(&mut self, desc: BufferDescriptor) -> Result<GpuBuffer, BackendError> {
        let id = self.alloc_id();
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: desc.label.as_deref(),
            size: desc.size,
            usage: to_wgpu_buffer_usage(desc.usage),
            mapped_at_creation: desc.mapped_at_creation,
        });
        self.resources.buffers.insert(id, buffer);
        Ok(GpuBuffer {
            id: BufferId(id),
            size: desc.size,
            usage: desc.usage,
        })
    }

    fn create_pipeline(&mut self, desc: PipelineDescriptor) -> Result<PipelineId, BackendError> {
        let id = self.alloc_id();
        let ShaderSource::Wgsl(ref vertex_src) = desc.vertex_shader;
        let ShaderSource::Wgsl(ref fragment_src) = desc.fragment_shader;

        let vs_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("vertex"),
                source: wgpu::ShaderSource::Wgsl(vertex_src.into()),
            });
        let fs_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("fragment"),
                source: wgpu::ShaderSource::Wgsl(fragment_src.into()),
            });

        // Create bind group layouts
        let mut bgl_refs = Vec::new();
        let mut bgls = Vec::new();
        for layout_desc in &desc.bind_group_layouts {
            let entries: Vec<wgpu::BindGroupLayoutEntry> = layout_desc
                .entries
                .iter()
                .map(|e| wgpu::BindGroupLayoutEntry {
                    binding: e.binding,
                    visibility: wgpu::ShaderStages::from_bits_truncate(e.visibility.0),
                    ty: match &e.ty {
                        BindingType::UniformBuffer => wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        BindingType::StorageBuffer { read_only } => wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage {
                                read_only: *read_only,
                            },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        BindingType::Texture => wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        BindingType::StorageTexture { format } => {
                            wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: to_wgpu_texture_format(*format),
                                view_dimension: wgpu::TextureViewDimension::D2,
                            }
                        }
                        BindingType::Sampler => {
                            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
                        }
                    },
                    count: None,
                })
                .collect();
            let bgl = self
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: layout_desc.label.as_deref(),
                    entries: &entries,
                });
            bgls.push(bgl);
        }
        for bgl in &bgls {
            bgl_refs.push(bgl);
        }

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: desc.label.as_deref(),
                bind_group_layouts: &bgl_refs,
                push_constant_ranges: &[],
            });

        let vertex_attributes: Vec<wgpu::VertexAttribute> = desc
            .vertex_layout
            .attributes
            .iter()
            .map(|attr| wgpu::VertexAttribute {
                format: match attr.format {
                    VertexFormat::Float32 => wgpu::VertexFormat::Float32,
                    VertexFormat::Float32x2 => wgpu::VertexFormat::Float32x2,
                    VertexFormat::Float32x3 => wgpu::VertexFormat::Float32x3,
                    VertexFormat::Float32x4 => wgpu::VertexFormat::Float32x4,
                    VertexFormat::Uint32 => wgpu::VertexFormat::Uint32,
                    VertexFormat::Sint32 => wgpu::VertexFormat::Sint32,
                },
                offset: attr.offset,
                shader_location: attr.shader_location,
            })
            .collect();

        let color_targets: Vec<Option<wgpu::ColorTargetState>> = desc
            .color_targets
            .iter()
            .map(|ct| {
                Some(wgpu::ColorTargetState {
                    format: to_wgpu_texture_format(ct.format),
                    blend: ct.blend.map(|b| wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: to_wgpu_blend_factor(b.color.src_factor),
                            dst_factor: to_wgpu_blend_factor(b.color.dst_factor),
                            operation: to_wgpu_blend_operation(b.color.operation),
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: to_wgpu_blend_factor(b.alpha.src_factor),
                            dst_factor: to_wgpu_blend_factor(b.alpha.dst_factor),
                            operation: to_wgpu_blend_operation(b.alpha.operation),
                        },
                    }),
                    write_mask: wgpu::ColorWrites::from_bits_truncate(ct.write_mask.0),
                })
            })
            .collect();

        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: desc.label.as_deref(),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vs_module,
                    entry_point: Some("vs_main"),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: desc.vertex_layout.stride,
                        step_mode: match desc.vertex_layout.step_mode {
                            VertexStepMode::Vertex => wgpu::VertexStepMode::Vertex,
                            VertexStepMode::Instance => wgpu::VertexStepMode::Instance,
                        },
                        attributes: &vertex_attributes,
                    }],
                    compilation_options: Default::default(),
                },
                primitive: wgpu::PrimitiveState {
                    topology: to_wgpu_primitive_topology(desc.primitive.topology),
                    strip_index_format: None,
                    front_face: to_wgpu_front_face(desc.primitive.front_face),
                    cull_mode: desc.primitive.cull_mode.map(to_wgpu_cull_mode),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: desc.depth_stencil.map(|ds| wgpu::DepthStencilState {
                    format: to_wgpu_texture_format(ds.format),
                    depth_write_enabled: ds.depth_write,
                    depth_compare: to_wgpu_compare(ds.depth_compare),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: desc.multisample.count,
                    mask: desc.multisample.mask,
                    alpha_to_coverage_enabled: desc.multisample.alpha_to_coverage,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fs_module,
                    entry_point: Some("fs_main"),
                    targets: &color_targets,
                    compilation_options: Default::default(),
                }),
                multiview: None,
                cache: None,
            });

        self.resources.render_pipelines.insert(id, pipeline);
        Ok(PipelineId(id))
    }

    fn create_compute_pipeline(
        &mut self,
        desc: ComputePipelineDescriptor,
    ) -> Result<ComputePipelineId, BackendError> {
        let id = self.alloc_id();
        let ShaderSource::Wgsl(ref src) = desc.shader;
        let module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: desc.label.as_deref(),
                source: wgpu::ShaderSource::Wgsl(src.into()),
            });

        let pipeline = self
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: desc.label.as_deref(),
                layout: None,
                module: &module,
                entry_point: Some(&desc.entry_point),
                compilation_options: Default::default(),
                cache: None,
            });

        self.resources.compute_pipelines.insert(id, pipeline);
        Ok(ComputePipelineId(id))
    }

    fn create_bind_group(
        &mut self,
        layout_desc: &BindGroupLayoutDescriptor,
        entries: &[BindGroupEntry],
    ) -> Result<BindGroupId, BackendError> {
        let id = self.alloc_id();

        // Create the bind group layout
        let layout_entries: Vec<wgpu::BindGroupLayoutEntry> = layout_desc
            .entries
            .iter()
            .map(|e| wgpu::BindGroupLayoutEntry {
                binding: e.binding,
                visibility: wgpu::ShaderStages::from_bits_truncate(e.visibility.0),
                ty: match &e.ty {
                    BindingType::UniformBuffer => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    BindingType::StorageBuffer { read_only } => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage {
                            read_only: *read_only,
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    BindingType::Texture => wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    BindingType::StorageTexture { format } => wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: to_wgpu_texture_format(*format),
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    BindingType::Sampler => {
                        wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
                    }
                },
                count: None,
            })
            .collect();

        let layout = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: layout_desc.label.as_deref(),
                entries: &layout_entries,
            });

        // Ensure default sampler exists before building entries
        let default_sampler_id = 0_u64;
        if !self.resources.samplers.contains_key(&default_sampler_id) {
            let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("default_sampler"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });
            self.resources.samplers.insert(default_sampler_id, sampler);
        }

        // Build wgpu bind group entries
        let wgpu_entries: Vec<wgpu::BindGroupEntry<'_>> = entries
            .iter()
            .filter_map(|entry| match entry {
                BindGroupEntry::Buffer {
                    binding,
                    buffer,
                    offset,
                    size,
                } => self
                    .resources
                    .buffers
                    .get(&buffer.0)
                    .map(|buf| wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: buf,
                            offset: *offset,
                            size: std::num::NonZeroU64::new(*size),
                        }),
                    }),
                BindGroupEntry::Texture { binding, texture } => self
                    .resources
                    .textures
                    .get(&texture.0)
                    .map(|td| &td.view)
                    .map(|view| wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::TextureView(view),
                    }),
                BindGroupEntry::Sampler { binding } => self
                    .resources
                    .samplers
                    .get(&default_sampler_id)
                    .map(|sampler| wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    }),
            })
            .collect();

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: layout_desc.label.as_deref(),
            layout: &layout,
            entries: &wgpu_entries,
        });

        self.resources.bind_groups.insert(id, bind_group);
        self.resources.bind_group_layouts.insert(id, layout);
        Ok(BindGroupId(id))
    }

    fn destroy_mesh(&mut self, id: MeshId) {
        self.resources.meshes.remove(&id.0);
    }

    fn destroy_texture(&mut self, id: TextureId) {
        self.resources.textures.remove(&id.0);
    }

    fn destroy_buffer(&mut self, id: BufferId) {
        self.resources.buffers.remove(&id.0);
    }

    fn write_buffer(&mut self, id: BufferId, offset: u64, data: &[u8]) {
        if let Some(buffer) = self.resources.buffers.get(&id.0) {
            self.queue.write_buffer(buffer, offset, data);
        }
    }

    fn read_buffer(&mut self, id: BufferId, offset: u64, len: u64) -> Vec<u8> {
        let src_buffer = match self.resources.buffers.get(&id.0) {
            Some(b) => b,
            None => return Vec::new(),
        };

        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("read_buffer_staging"),
            size: len,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("read_buffer_copy"),
            });
        encoder.copy_buffer_to_buffer(src_buffer, offset, &staging, 0, len);
        self.queue.submit(std::iter::once(encoder.finish()));

        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).ok();
        });
        self.device.poll(wgpu::Maintain::Wait);

        match rx.recv() {
            Ok(Ok(())) => {
                let data = slice.get_mapped_range();
                let result = data.to_vec();
                drop(data);
                staging.unmap();
                result
            }
            _ => Vec::new(),
        }
    }

    fn write_texture(&mut self, id: TextureId, region: TextureRegion, data: &[u8]) {
        if let Some(tex_data) = self.resources.textures.get(&id.0) {
            let bpp = bytes_per_pixel(tex_data.format);
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &tex_data.texture,
                    mip_level: region.mip_level,
                    origin: wgpu::Origin3d {
                        x: region.x,
                        y: region.y,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(region.width * bpp),
                    rows_per_image: Some(region.height),
                },
                wgpu::Extent3d {
                    width: region.width,
                    height: region.height,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    fn begin_frame(&mut self) -> Result<(), BackendError> {
        self.render_passes.clear();
        self.compute_passes.clear();

        // Acquire surface texture if we have a surface
        if let Some(surface) = &self.surface {
            let texture = surface
                .get_current_texture()
                .map_err(|_| BackendError::SurfaceLost)?;

            // Store a view of the surface texture as a resource so render passes can target it
            let view = texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let surface_tex_id = 0u64; // reserved ID for the surface texture
            self.resources.textures.insert(
                surface_tex_id,
                TextureData {
                    texture: texture.texture.clone(),
                    view,
                    format: self.surface_format,
                },
            );
            self.current_surface_texture = Some(texture);
        }
        Ok(())
    }

    fn end_frame(&mut self) -> Result<(), BackendError> {
        self.device.poll(wgpu::Maintain::Wait);
        Ok(())
    }

    fn present(&mut self) -> Result<(), BackendError> {
        // Remove the surface texture view from resources before presenting
        self.resources.textures.remove(&0u64);
        if let Some(texture) = self.current_surface_texture.take() {
            texture.present();
        }
        Ok(())
    }

    fn begin_render_pass(&mut self, desc: RenderPassDescriptor) -> RenderPassHandle {
        let handle = self.render_passes.len() as u64;
        self.render_passes.push(Some(RecordedRenderPass {
            descriptor: desc,
            commands: Vec::new(),
        }));
        RenderPassHandle(handle)
    }

    fn end_render_pass(&mut self, handle: RenderPassHandle) {
        let pass_data = match self
            .render_passes
            .get_mut(handle.0 as usize)
            .and_then(|p| p.take())
        {
            Some(d) => d,
            None => return,
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: pass_data.descriptor.label.as_deref(),
            });

        // Resolve color attachments to wgpu texture views
        let color_views: Vec<Option<&wgpu::TextureView>> = pass_data
            .descriptor
            .color_attachments
            .iter()
            .map(|att| {
                self.resources
                    .textures
                    .get(&att.texture.0)
                    .map(|td| &td.view)
            })
            .collect();

        let depth_view: Option<&wgpu::TextureView> = pass_data
            .descriptor
            .depth_attachment
            .as_ref()
            .and_then(|att| {
                self.resources
                    .textures
                    .get(&att.texture.0)
                    .map(|td| &td.view)
            });

        // Build wgpu color attachments
        let wgpu_color_attachments: Vec<Option<wgpu::RenderPassColorAttachment<'_>>> = pass_data
            .descriptor
            .color_attachments
            .iter()
            .zip(color_views.iter())
            .map(|(att, view)| {
                view.map(|v| wgpu::RenderPassColorAttachment {
                    view: v,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match att.load_op {
                            LoadOp::Clear(c) => wgpu::LoadOp::Clear(wgpu::Color {
                                r: c.r,
                                g: c.g,
                                b: c.b,
                                a: c.a,
                            }),
                            LoadOp::Load => wgpu::LoadOp::Load,
                        },
                        store: match att.store_op {
                            StoreOp::Store => wgpu::StoreOp::Store,
                            StoreOp::Discard => wgpu::StoreOp::Discard,
                        },
                    },
                })
            })
            .collect();

        let wgpu_depth_attachment =
            pass_data
                .descriptor
                .depth_attachment
                .as_ref()
                .and_then(|att| {
                    depth_view.map(|v| wgpu::RenderPassDepthStencilAttachment {
                        view: v,
                        depth_ops: Some(wgpu::Operations {
                            load: match att.depth_load_op {
                                LoadOp::Clear(c) => wgpu::LoadOp::Clear(c.r as f32),
                                LoadOp::Load => wgpu::LoadOp::Load,
                            },
                            store: match att.depth_store_op {
                                StoreOp::Store => wgpu::StoreOp::Store,
                                StoreOp::Discard => wgpu::StoreOp::Discard,
                            },
                        }),
                        stencil_ops: None,
                    })
                });

        // Create the actual wgpu render pass and replay commands
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: pass_data.descriptor.label.as_deref(),
                color_attachments: &wgpu_color_attachments,
                depth_stencil_attachment: wgpu_depth_attachment,
                ..Default::default()
            });

            for cmd in &pass_data.commands {
                match cmd {
                    RecordedCommand::SetPipeline(id) => {
                        if let Some(pipeline) = self.resources.render_pipelines.get(&id.0) {
                            rpass.set_pipeline(pipeline);
                        }
                    }
                    RecordedCommand::SetBindGroup { index, group } => {
                        if let Some(bg) = self.resources.bind_groups.get(&group.0) {
                            rpass.set_bind_group(*index, bg, &[]);
                        }
                    }
                    RecordedCommand::SetVertexBuffer { slot, buffer } => {
                        if let Some(buf) = self.resources.buffers.get(&buffer.buffer.0) {
                            rpass.set_vertex_buffer(
                                *slot,
                                buf.slice(buffer.offset..buffer.offset + buffer.size),
                            );
                        }
                    }
                    RecordedCommand::SetIndexBuffer(buffer) => {
                        if let Some(buf) = self.resources.buffers.get(&buffer.buffer.0) {
                            rpass.set_index_buffer(
                                buf.slice(buffer.offset..buffer.offset + buffer.size),
                                wgpu::IndexFormat::Uint32,
                            );
                        }
                    }
                    RecordedCommand::Draw {
                        vertices,
                        instances,
                    } => {
                        rpass.draw(vertices.clone(), instances.clone());
                    }
                    RecordedCommand::DrawIndexed {
                        indices,
                        base_vertex,
                        instances,
                    } => {
                        rpass.draw_indexed(indices.clone(), *base_vertex, instances.clone());
                    }
                }
            }
        } // rpass dropped here, ending the render pass

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn set_pipeline(&mut self, handle: RenderPassHandle, pipeline: PipelineId) {
        if let Some(Some(pass)) = self.render_passes.get_mut(handle.0 as usize) {
            pass.commands.push(RecordedCommand::SetPipeline(pipeline));
        }
    }

    fn set_bind_group(&mut self, handle: RenderPassHandle, index: u32, group: BindGroupId) {
        if let Some(Some(pass)) = self.render_passes.get_mut(handle.0 as usize) {
            pass.commands
                .push(RecordedCommand::SetBindGroup { index, group });
        }
    }

    fn set_vertex_buffer(&mut self, handle: RenderPassHandle, slot: u32, buffer: BufferSlice) {
        if let Some(Some(pass)) = self.render_passes.get_mut(handle.0 as usize) {
            pass.commands
                .push(RecordedCommand::SetVertexBuffer { slot, buffer });
        }
    }

    fn set_index_buffer(&mut self, handle: RenderPassHandle, buffer: BufferSlice) {
        if let Some(Some(pass)) = self.render_passes.get_mut(handle.0 as usize) {
            pass.commands.push(RecordedCommand::SetIndexBuffer(buffer));
        }
    }

    fn draw(&mut self, handle: RenderPassHandle, vertices: Range<u32>, instances: Range<u32>) {
        if let Some(Some(pass)) = self.render_passes.get_mut(handle.0 as usize) {
            pass.commands.push(RecordedCommand::Draw {
                vertices,
                instances,
            });
        }
    }

    fn draw_indexed(
        &mut self,
        handle: RenderPassHandle,
        indices: Range<u32>,
        base_vertex: i32,
        instances: Range<u32>,
    ) {
        if let Some(Some(pass)) = self.render_passes.get_mut(handle.0 as usize) {
            pass.commands.push(RecordedCommand::DrawIndexed {
                indices,
                base_vertex,
                instances,
            });
        }
    }

    fn begin_compute_pass(&mut self) -> ComputePassHandle {
        let handle = self.compute_passes.len() as u64;
        self.compute_passes.push(Some(RecordedComputePass {
            commands: Vec::new(),
        }));
        ComputePassHandle(handle)
    }

    fn end_compute_pass(&mut self, handle: ComputePassHandle) {
        let pass_data = match self
            .compute_passes
            .get_mut(handle.0 as usize)
            .and_then(|p| p.take())
        {
            Some(d) => d,
            None => return,
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("compute"),
            });

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("compute_pass"),
                ..Default::default()
            });

            for cmd in &pass_data.commands {
                match cmd {
                    RecordedComputeCommand::SetPipeline(id) => {
                        if let Some(pipeline) = self.resources.compute_pipelines.get(&id.0) {
                            cpass.set_pipeline(pipeline);
                        }
                    }
                    RecordedComputeCommand::SetBindGroup { index, group } => {
                        if let Some(bg) = self.resources.bind_groups.get(&group.0) {
                            cpass.set_bind_group(*index, bg, &[]);
                        }
                    }
                    RecordedComputeCommand::Dispatch { x, y, z } => {
                        cpass.dispatch_workgroups(*x, *y, *z);
                    }
                }
            }
        } // cpass dropped here

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn set_compute_pipeline(&mut self, handle: ComputePassHandle, pipeline: ComputePipelineId) {
        if let Some(Some(pass)) = self.compute_passes.get_mut(handle.0 as usize) {
            pass.commands
                .push(RecordedComputeCommand::SetPipeline(pipeline));
        }
    }

    fn set_compute_bind_group(
        &mut self,
        handle: ComputePassHandle,
        index: u32,
        group: BindGroupId,
    ) {
        if let Some(Some(pass)) = self.compute_passes.get_mut(handle.0 as usize) {
            pass.commands
                .push(RecordedComputeCommand::SetBindGroup { index, group });
        }
    }

    fn dispatch(&mut self, handle: ComputePassHandle, x: u32, y: u32, z: u32) {
        if let Some(Some(pass)) = self.compute_passes.get_mut(handle.0 as usize) {
            pass.commands
                .push(RecordedComputeCommand::Dispatch { x, y, z });
        }
    }

    fn capabilities(&self) -> BackendCapabilities {
        let limits = self.adapter.limits();
        BackendCapabilities {
            max_texture_size: limits.max_texture_dimension_2d,
            max_buffer_size: limits.max_buffer_size,
            max_instances: 65536,
            supports_compute: self.adapter.features().contains(wgpu::Features::empty()),
            supports_indirect: true,
            supports_timestamp_queries: self
                .adapter
                .features()
                .contains(wgpu::Features::TIMESTAMP_QUERY),
            supports_hdr: true,
        }
    }

    fn backend_name(&self) -> &str {
        "wgpu"
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        if let (Some(surface), Some(config)) = (&self.surface, &mut self.surface_config) {
            config.width = width.max(1);
            config.height = height.max(1);
            surface.configure(&self.device, config);
        }
    }

    fn surface_format(&self) -> TextureFormat {
        self.surface_format
    }

    fn surface_texture(&self) -> Result<TextureId, BackendError> {
        if self.surface.is_some() {
            // ID 0 is reserved for the current surface texture, set in begin_frame
            Ok(TextureId(0))
        } else {
            Err(BackendError::InvalidOperation(
                "Headless backend has no surface texture".to_string(),
            ))
        }
    }
}
