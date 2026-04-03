use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};
use serde::{Deserialize, Serialize};

// ── Handle types ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MeshId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextureId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BufferId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PipelineId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComputePipelineId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BindGroupId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderPassHandle(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComputePassHandle(pub u64);

// ── Vertex ──

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 4],
}

// ── Resource descriptors ──

#[derive(Debug, Clone)]
pub struct MeshDescriptor {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub usage: TextureUsage,
    pub data: Option<Vec<u8>>,
    pub label: Option<String>,
    pub mip_levels: u32,
}

#[derive(Debug, Clone)]
pub struct BufferDescriptor {
    pub size: u64,
    pub usage: BufferUsage,
    pub label: Option<String>,
    pub mapped_at_creation: bool,
}

#[derive(Debug, Clone)]
pub struct PipelineDescriptor {
    pub vertex_shader: ShaderSource,
    pub fragment_shader: ShaderSource,
    pub vertex_layout: VertexLayout,
    pub primitive: PrimitiveState,
    pub depth_stencil: Option<DepthStencilState>,
    pub multisample: MultisampleState,
    pub color_targets: Vec<ColorTargetState>,
    pub bind_group_layouts: Vec<BindGroupLayoutDescriptor>,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ComputePipelineDescriptor {
    pub shader: ShaderSource,
    pub entry_point: String,
    pub bind_group_layouts: Vec<BindGroupLayoutDescriptor>,
    pub label: Option<String>,
}

// ── Shader ──

#[derive(Debug, Clone)]
pub enum ShaderSource {
    Wgsl(String),
}

// ── Texture ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFormat {
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Bgra8Unorm,
    Bgra8UnormSrgb,
    Rgba16Float,
    Rgba32Float,
    R8Unorm,
    Rg8Unorm,
    Depth32Float,
    Depth24PlusStencil8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureUsage(pub u32);

impl TextureUsage {
    pub const COPY_SRC: Self = Self(1);
    pub const COPY_DST: Self = Self(2);
    pub const TEXTURE_BINDING: Self = Self(4);
    pub const STORAGE_BINDING: Self = Self(8);
    pub const RENDER_ATTACHMENT: Self = Self(16);

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for TextureUsage {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone)]
pub struct TextureRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub mip_level: u32,
}

// ── Buffer ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferUsage(pub u32);

impl BufferUsage {
    pub const VERTEX: Self = Self(1);
    pub const INDEX: Self = Self(2);
    pub const UNIFORM: Self = Self(4);
    pub const STORAGE: Self = Self(8);
    pub const COPY_SRC: Self = Self(16);
    pub const COPY_DST: Self = Self(32);
    pub const INDIRECT: Self = Self(64);

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for BufferUsage {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BufferSlice {
    pub buffer: BufferId,
    pub offset: u64,
    pub size: u64,
}

// ── Pipeline state ──

#[derive(Debug, Clone)]
pub struct VertexLayout {
    pub stride: u64,
    pub attributes: Vec<VertexAttribute>,
    pub step_mode: VertexStepMode,
}

#[derive(Debug, Clone)]
pub struct VertexAttribute {
    pub format: VertexFormat,
    pub offset: u64,
    pub shader_location: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexFormat {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Uint32,
    Sint32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexStepMode {
    Vertex,
    Instance,
}

#[derive(Debug, Clone, Copy)]
pub struct PrimitiveState {
    pub topology: PrimitiveTopology,
    pub front_face: FrontFace,
    pub cull_mode: Option<CullFace>,
}

impl Default for PrimitiveState {
    fn default() -> Self {
        Self {
            topology: PrimitiveTopology::TriangleList,
            front_face: FrontFace::Ccw,
            cull_mode: Some(CullFace::Back),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveTopology {
    PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontFace {
    Ccw,
    Cw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullFace {
    Front,
    Back,
}

#[derive(Debug, Clone, Copy)]
pub struct DepthStencilState {
    pub format: TextureFormat,
    pub depth_write: bool,
    pub depth_compare: CompareFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareFunction {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

#[derive(Debug, Clone, Copy)]
pub struct MultisampleState {
    pub count: u32,
    pub mask: u64,
    pub alpha_to_coverage: bool,
}

impl Default for MultisampleState {
    fn default() -> Self {
        Self {
            count: 1,
            mask: !0,
            alpha_to_coverage: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColorTargetState {
    pub format: TextureFormat,
    pub blend: Option<BlendState>,
    pub write_mask: ColorWrites,
}

#[derive(Debug, Clone, Copy)]
pub struct BlendState {
    pub color: BlendComponent,
    pub alpha: BlendComponent,
}

impl BlendState {
    pub const ALPHA_BLENDING: Self = Self {
        color: BlendComponent {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
    };

    pub const PREMULTIPLIED_ALPHA: Self = Self {
        color: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
    };
}

#[derive(Debug, Clone, Copy)]
pub struct BlendComponent {
    pub src_factor: BlendFactor,
    pub dst_factor: BlendFactor,
    pub operation: BlendOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendFactor {
    Zero,
    One,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendOperation {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

#[derive(Debug, Clone, Copy)]
pub struct ColorWrites(pub u32);

impl ColorWrites {
    pub const RED: Self = Self(1);
    pub const GREEN: Self = Self(2);
    pub const BLUE: Self = Self(4);
    pub const ALPHA: Self = Self(8);
    pub const ALL: Self = Self(15);
}

// ── Bind group ──

#[derive(Debug, Clone)]
pub struct BindGroupLayoutDescriptor {
    pub entries: Vec<BindGroupLayoutEntry>,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BindGroupLayoutEntry {
    pub binding: u32,
    pub visibility: ShaderStages,
    pub ty: BindingType,
}

#[derive(Debug, Clone, Copy)]
pub struct ShaderStages(pub u32);

impl ShaderStages {
    pub const VERTEX: Self = Self(1);
    pub const FRAGMENT: Self = Self(2);
    pub const COMPUTE: Self = Self(4);
    pub const VERTEX_FRAGMENT: Self = Self(3);
}

#[derive(Debug, Clone)]
pub enum BindingType {
    UniformBuffer,
    StorageBuffer { read_only: bool },
    Texture,
    StorageTexture { format: TextureFormat },
    Sampler,
}

// ── Render pass ──

#[derive(Debug, Clone)]
pub struct RenderPassDescriptor {
    pub color_attachments: Vec<ColorAttachment>,
    pub depth_attachment: Option<DepthAttachment>,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ColorAttachment {
    pub texture: TextureId,
    pub resolve_target: Option<TextureId>,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
}

#[derive(Debug, Clone, Copy)]
pub enum LoadOp {
    Clear(ClearColor),
    Load,
}

#[derive(Debug, Clone, Copy)]
pub struct ClearColor {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl ClearColor {
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
}

#[derive(Debug, Clone, Copy)]
pub enum StoreOp {
    Store,
    Discard,
}

#[derive(Debug, Clone)]
pub struct DepthAttachment {
    pub texture: TextureId,
    pub depth_load_op: LoadOp,
    pub depth_store_op: StoreOp,
}

// ── GPU resource types (returned by backend) ──

#[derive(Debug, Clone)]
pub struct GpuMesh {
    pub id: MeshId,
    pub vertex_count: u32,
    pub index_count: u32,
    pub aabb: Aabb,
    pub vertex_buffer: BufferId,
    pub index_buffer: BufferId,
    pub vertex_buffer_size: u64,
    pub index_buffer_size: u64,
}

#[derive(Debug, Clone)]
pub struct GpuTexture {
    pub id: TextureId,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
}

#[derive(Debug, Clone)]
pub struct GpuBuffer {
    pub id: BufferId,
    pub size: u64,
    pub usage: BufferUsage,
}

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_points(points: &[Vec3]) -> Self {
        if points.is_empty() {
            return Self {
                min: Vec3::ZERO,
                max: Vec3::ZERO,
            };
        }
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for &p in points {
            min = min.min(p);
            max = max.max(p);
        }
        Self { min, max }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }
}

// ── Capabilities ──

#[derive(Debug, Clone)]
pub struct BackendCapabilities {
    pub max_texture_size: u32,
    pub max_buffer_size: u64,
    pub max_instances: u32,
    pub supports_compute: bool,
    pub supports_indirect: bool,
    pub supports_timestamp_queries: bool,
    pub supports_hdr: bool,
}

// ── Color types ──

#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
#[repr(C)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Vec4> for Rgba {
    fn from(v: Vec4) -> Self {
        Self::new(v.x, v.y, v.z, v.w)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
#[repr(C)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Rgb {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };

    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }
}

impl From<Vec3> for Rgb {
    fn from(v: Vec3) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aabb_from_points() {
        let points = vec![
            Vec3::new(-1.0, -2.0, -3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(0.0, 0.0, 0.0),
        ];
        let aabb = Aabb::from_points(&points);
        assert_eq!(aabb.min, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(aabb.max, Vec3::new(4.0, 5.0, 6.0));
    }

    #[test]
    fn aabb_contains_point() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        assert!(aabb.contains(Vec3::splat(0.5)));
        assert!(!aabb.contains(Vec3::splat(2.0)));
    }

    #[test]
    fn aabb_intersects() {
        let a = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let b = Aabb::new(Vec3::splat(0.5), Vec3::splat(1.5));
        let c = Aabb::new(Vec3::splat(2.0), Vec3::splat(3.0));
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn aabb_center_extents() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(aabb.center(), Vec3::ZERO);
        assert_eq!(aabb.extents(), Vec3::ONE);
    }

    #[test]
    fn texture_usage_bitops() {
        let usage = TextureUsage::COPY_DST | TextureUsage::TEXTURE_BINDING;
        assert!(usage.contains(TextureUsage::COPY_DST));
        assert!(usage.contains(TextureUsage::TEXTURE_BINDING));
        assert!(!usage.contains(TextureUsage::RENDER_ATTACHMENT));
    }

    #[test]
    fn rgba_from_vec4() {
        let color: Rgba = Vec4::new(1.0, 0.5, 0.25, 0.8).into();
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 0.25);
        assert_eq!(color.a, 0.8);
    }
}
