//! Map-related functionality for OP2MapViewer

pub mod types;
pub mod loader;

// Re-export commonly used items
pub use types::{Map, MapInfo, Cell, CellType, Position};
pub use loader::{load_map, load_map_preview, MapLoadError};
