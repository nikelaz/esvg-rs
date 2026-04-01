pub mod arbiter;
pub mod helpers;
pub mod path;
pub mod plugin;
pub mod plugins;
pub mod svg;
pub mod transform;

pub use arbiter::Arbiter;
pub use plugin::{SingleElementPluginTrait, WholeSVGPluginTrait};
pub use plugins::{
    ApplyTransformsPlugin, CollapseGroupsPlugin, CombinePathsPlugin, CssToAttributesPlugin,
    MangleIdsPlugin, NumberPrecisionPlugin, OptimizeColorsPlugin, RemoveEmptyTextPlugin,
    RemoveUnnecessaryAttrsPlugin, RemoveUnnecessaryClipPathsPlugin, ShapeToPathPlugin,
    SimplifyPathsPlugin, SortAttrsPlugin,
};
pub use svg::Svg;

#[cfg(feature = "ffi")]
pub mod ffi;
