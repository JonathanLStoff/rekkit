//! Model definition, textured vertices, and OBJ-style quad mesh parsing.
//!
//! Re-exports and builds upon [`crate::core::model_loader`]:
//! - [`RenderModel`]
//! - [`TexVertex`]
//! - [`ParseError`]

pub use crate::core::model_loader::{ParseError, RenderModel, TexVertex};
