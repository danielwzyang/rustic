pub mod lighting;
pub mod scan_line;
pub mod edge_list;
pub mod polygon_list;
pub mod texture;

pub use crate::picture::Picture;
pub use lighting::{LightingConfig, ReflectionConstants, get_illumination};
