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
    ApplyTransformsPlugin, CombinePathsPlugin, CssToAttributesPlugin, OptimizeColorsPlugin,
    RemoveUnnecessaryAttrsPlugin, RemoveUnnecessaryClipPathsPlugin, ShapeToPathPlugin,
};
pub use svg::Svg;
