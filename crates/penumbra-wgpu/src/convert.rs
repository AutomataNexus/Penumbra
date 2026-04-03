use penumbra_backend::*;

pub fn to_wgpu_texture_format(format: TextureFormat) -> wgpu::TextureFormat {
    match format {
        TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        TextureFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
        TextureFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8UnormSrgb,
        TextureFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
        TextureFormat::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
        TextureFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
        TextureFormat::Rg8Unorm => wgpu::TextureFormat::Rg8Unorm,
        TextureFormat::Depth32Float => wgpu::TextureFormat::Depth32Float,
        TextureFormat::Depth24PlusStencil8 => wgpu::TextureFormat::Depth24PlusStencil8,
    }
}

#[allow(dead_code)]
pub fn from_wgpu_texture_format(format: wgpu::TextureFormat) -> TextureFormat {
    match format {
        wgpu::TextureFormat::Rgba8Unorm => TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Rgba8UnormSrgb => TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureFormat::Bgra8Unorm => TextureFormat::Bgra8Unorm,
        wgpu::TextureFormat::Bgra8UnormSrgb => TextureFormat::Bgra8UnormSrgb,
        wgpu::TextureFormat::Rgba16Float => TextureFormat::Rgba16Float,
        wgpu::TextureFormat::Rgba32Float => TextureFormat::Rgba32Float,
        wgpu::TextureFormat::R8Unorm => TextureFormat::R8Unorm,
        wgpu::TextureFormat::Rg8Unorm => TextureFormat::Rg8Unorm,
        wgpu::TextureFormat::Depth32Float => TextureFormat::Depth32Float,
        wgpu::TextureFormat::Depth24PlusStencil8 => TextureFormat::Depth24PlusStencil8,
        other => {
            tracing::warn!("Unsupported wgpu texture format {:?}, falling back to Rgba8Unorm", other);
            TextureFormat::Rgba8Unorm
        }
    }
}

pub fn to_wgpu_texture_usage(usage: TextureUsage) -> wgpu::TextureUsages {
    let mut result = wgpu::TextureUsages::empty();
    if usage.contains(TextureUsage::COPY_SRC) {
        result |= wgpu::TextureUsages::COPY_SRC;
    }
    if usage.contains(TextureUsage::COPY_DST) {
        result |= wgpu::TextureUsages::COPY_DST;
    }
    if usage.contains(TextureUsage::TEXTURE_BINDING) {
        result |= wgpu::TextureUsages::TEXTURE_BINDING;
    }
    if usage.contains(TextureUsage::STORAGE_BINDING) {
        result |= wgpu::TextureUsages::STORAGE_BINDING;
    }
    if usage.contains(TextureUsage::RENDER_ATTACHMENT) {
        result |= wgpu::TextureUsages::RENDER_ATTACHMENT;
    }
    result
}

pub fn to_wgpu_buffer_usage(usage: BufferUsage) -> wgpu::BufferUsages {
    let mut result = wgpu::BufferUsages::empty();
    if usage.contains(BufferUsage::VERTEX) {
        result |= wgpu::BufferUsages::VERTEX;
    }
    if usage.contains(BufferUsage::INDEX) {
        result |= wgpu::BufferUsages::INDEX;
    }
    if usage.contains(BufferUsage::UNIFORM) {
        result |= wgpu::BufferUsages::UNIFORM;
    }
    if usage.contains(BufferUsage::STORAGE) {
        result |= wgpu::BufferUsages::STORAGE;
    }
    if usage.contains(BufferUsage::COPY_SRC) {
        result |= wgpu::BufferUsages::COPY_SRC;
    }
    if usage.contains(BufferUsage::COPY_DST) {
        result |= wgpu::BufferUsages::COPY_DST;
    }
    if usage.contains(BufferUsage::INDIRECT) {
        result |= wgpu::BufferUsages::INDIRECT;
    }
    result
}

pub fn to_wgpu_primitive_topology(topology: PrimitiveTopology) -> wgpu::PrimitiveTopology {
    match topology {
        PrimitiveTopology::PointList => wgpu::PrimitiveTopology::PointList,
        PrimitiveTopology::LineList => wgpu::PrimitiveTopology::LineList,
        PrimitiveTopology::LineStrip => wgpu::PrimitiveTopology::LineStrip,
        PrimitiveTopology::TriangleList => wgpu::PrimitiveTopology::TriangleList,
        PrimitiveTopology::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
    }
}

pub fn to_wgpu_front_face(face: FrontFace) -> wgpu::FrontFace {
    match face {
        FrontFace::Ccw => wgpu::FrontFace::Ccw,
        FrontFace::Cw => wgpu::FrontFace::Cw,
    }
}

pub fn to_wgpu_cull_mode(face: CullFace) -> wgpu::Face {
    match face {
        CullFace::Front => wgpu::Face::Front,
        CullFace::Back => wgpu::Face::Back,
    }
}

pub fn to_wgpu_compare(func: CompareFunction) -> wgpu::CompareFunction {
    match func {
        CompareFunction::Never => wgpu::CompareFunction::Never,
        CompareFunction::Less => wgpu::CompareFunction::Less,
        CompareFunction::Equal => wgpu::CompareFunction::Equal,
        CompareFunction::LessEqual => wgpu::CompareFunction::LessEqual,
        CompareFunction::Greater => wgpu::CompareFunction::Greater,
        CompareFunction::NotEqual => wgpu::CompareFunction::NotEqual,
        CompareFunction::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
        CompareFunction::Always => wgpu::CompareFunction::Always,
    }
}

pub fn to_wgpu_blend_factor(factor: BlendFactor) -> wgpu::BlendFactor {
    match factor {
        BlendFactor::Zero => wgpu::BlendFactor::Zero,
        BlendFactor::One => wgpu::BlendFactor::One,
        BlendFactor::SrcAlpha => wgpu::BlendFactor::SrcAlpha,
        BlendFactor::OneMinusSrcAlpha => wgpu::BlendFactor::OneMinusSrcAlpha,
        BlendFactor::DstAlpha => wgpu::BlendFactor::DstAlpha,
        BlendFactor::OneMinusDstAlpha => wgpu::BlendFactor::OneMinusDstAlpha,
        BlendFactor::SrcColor => wgpu::BlendFactor::Src,
        BlendFactor::OneMinusSrcColor => wgpu::BlendFactor::OneMinusSrc,
        BlendFactor::DstColor => wgpu::BlendFactor::Dst,
        BlendFactor::OneMinusDstColor => wgpu::BlendFactor::OneMinusDst,
    }
}

pub fn to_wgpu_blend_operation(op: BlendOperation) -> wgpu::BlendOperation {
    match op {
        BlendOperation::Add => wgpu::BlendOperation::Add,
        BlendOperation::Subtract => wgpu::BlendOperation::Subtract,
        BlendOperation::ReverseSubtract => wgpu::BlendOperation::ReverseSubtract,
        BlendOperation::Min => wgpu::BlendOperation::Min,
        BlendOperation::Max => wgpu::BlendOperation::Max,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texture_format_roundtrip() {
        let formats = [
            TextureFormat::Rgba8Unorm,
            TextureFormat::Depth32Float,
            TextureFormat::Rgba16Float,
        ];
        for fmt in formats {
            assert_eq!(from_wgpu_texture_format(to_wgpu_texture_format(fmt)), fmt);
        }
    }
}
