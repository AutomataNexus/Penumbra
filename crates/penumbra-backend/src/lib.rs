//! penumbra-backend — GPU abstraction layer for Penumbra.
//!
//! Defines the [`RenderBackend`] trait that all Penumbra feature crates depend on.
//! The default implementation is provided by `penumbra-wgpu`.

mod error;
pub mod traits;
mod types;

pub use error::BackendError;
pub use traits::RenderBackend;
pub use types::*;
