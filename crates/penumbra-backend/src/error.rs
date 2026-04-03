use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("GPU device lost")]
    DeviceLost,

    #[error("Surface lost or outdated")]
    SurfaceLost,

    #[error("Out of GPU memory")]
    OutOfMemory,

    #[error("Resource creation failed: {0}")]
    ResourceCreation(String),

    #[error("Pipeline creation failed: {0}")]
    PipelineCreation(String),

    #[error("Shader compilation failed: {0}")]
    ShaderCompilation(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Backend not initialized")]
    NotInitialized,

    #[error("Feature not supported: {0}")]
    Unsupported(String),

    #[error("{0}")]
    Other(String),
}
