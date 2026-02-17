pub mod apply_transforms;
pub mod combine_paths;
pub mod css_to_attributes;
pub mod optimize_colors;
pub mod remove_unnecessary_attrs;
pub mod remove_unnecessary_clip_paths;
pub mod shape_to_path;

// Re-export commonly used items
pub use apply_transforms::*;
pub use combine_paths::*;
pub use css_to_attributes::*;
pub use optimize_colors::*;
pub use remove_unnecessary_attrs::*;
pub use remove_unnecessary_clip_paths::*;
pub use shape_to_path::*;
