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
    NumberPrecisionPlugin, OptimizeColorsPlugin, RemoveUnnecessaryAttrsPlugin,
    RemoveUnnecessaryClipPathsPlugin, ShapeToPathPlugin, SimplifyPathsPlugin,
};
pub use svg::Svg;
