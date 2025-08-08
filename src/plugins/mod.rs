pub mod remove_unnecessary_attrs;
pub mod shape_to_path;
pub mod apply_transforms;
pub mod css_to_attributes;
pub mod optimize_colors;
pub mod combine_paths;

// Re-export commonly used items
pub use remove_unnecessary_attrs::*;
pub use shape_to_path::*;
pub use apply_transforms::*;
pub use css_to_attributes::*;
pub use optimize_colors::*;
pub use combine_paths::*;
