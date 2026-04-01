use std::ffi::CString;
use std::os::raw::c_char;

use crate::plugins::{
    ApplyTransformsPlugin, CollapseGroupsPlugin, CombinePathsPlugin, CssToAttributesPlugin,
    MangleIdsPlugin, NumberPrecisionPlugin, OptimizeColorsPlugin, RemoveEmptyTextPlugin,
    RemoveUnnecessaryAttrsPlugin, RemoveUnnecessaryClipPathsPlugin, ShapeToPathPlugin,
    SimplifyPathsPlugin, SortAttrsPlugin,
};
use crate::Arbiter;
use crate::Svg;

pub const ESVG_PLUGIN_REMOVE_UNNECESSARY_ATTRS: u64 = 1 << 0;
pub const ESVG_PLUGIN_SHAPE_TO_PATH: u64 = 1 << 1;
pub const ESVG_PLUGIN_OPTIMIZE_COLORS: u64 = 1 << 2;
pub const ESVG_PLUGIN_COLLAPSE_GROUPS: u64 = 1 << 3;
pub const ESVG_PLUGIN_NUMBER_PRECISION: u64 = 1 << 4;
pub const ESVG_PLUGIN_REMOVE_EMPTY_TEXT: u64 = 1 << 5;
pub const ESVG_PLUGIN_REMOVE_UNNECESSARY_CLIPPATH: u64 = 1 << 6;
pub const ESVG_PLUGIN_SORT_ATTRS: u64 = 1 << 7;
pub const ESVG_PLUGIN_APPLY_TRANSFORMS: u64 = 1 << 8;
pub const ESVG_PLUGIN_CSS_TO_ATTRIBUTES: u64 = 1 << 9;
pub const ESVG_PLUGIN_COMBINE_PATHS: u64 = 1 << 10;
pub const ESVG_PLUGIN_MANGLE_IDS: u64 = 1 << 11;
pub const ESVG_PLUGIN_SIMPLIFY_PATHS: u64 = 1 << 12;

fn parse_svg(input: *const c_char, len: usize) -> Option<Svg> {
    let bytes = unsafe { std::slice::from_raw_parts(input as *const u8, len) };
    let svg_str = std::str::from_utf8(bytes).ok()?.to_owned();
    Svg::from_string(&svg_str).ok()
}

fn svg_to_ptr(svg: Svg) -> *mut c_char {
    match CString::new(svg.to_string()) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Optimize an SVG string. Returns a heap-allocated null-terminated C string,
/// or null on error. Must be freed with `esvg_free`.
#[unsafe(no_mangle)]
pub extern "C" fn esvg_optimize(input: *const c_char, len: usize) -> *mut c_char {
    esvg_optimize_with_flags(
        input,
        len,
        ESVG_PLUGIN_REMOVE_UNNECESSARY_ATTRS
            | ESVG_PLUGIN_SHAPE_TO_PATH
            | ESVG_PLUGIN_OPTIMIZE_COLORS
            | ESVG_PLUGIN_APPLY_TRANSFORMS
            | ESVG_PLUGIN_CSS_TO_ATTRIBUTES
            | ESVG_PLUGIN_COMBINE_PATHS,
    )
}

/// Optimize an SVG string with a plugin selection bitmask and extended options.
/// `number_precision` sets the decimal precision for the Number Precision plugin (1–10);
/// only used when ESVG_PLUGIN_NUMBER_PRECISION is set in flags.
/// Returns a heap-allocated null-terminated C string, or null on error.
/// Must be freed with `esvg_free`.
#[unsafe(no_mangle)]
pub extern "C" fn esvg_optimize_with_flags_ex(
    input: *const c_char,
    len: usize,
    flags: u64,
    number_precision: u32,
) -> *mut c_char {
    let svg = match parse_svg(input, len) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };

    let mut arbiter = Arbiter::new();

    if flags & ESVG_PLUGIN_REMOVE_UNNECESSARY_ATTRS != 0 {
        arbiter.add_single_element_plugin(Box::new(RemoveUnnecessaryAttrsPlugin {}));
    }
    if flags & ESVG_PLUGIN_SHAPE_TO_PATH != 0 {
        arbiter.add_single_element_plugin(Box::new(ShapeToPathPlugin {}));
    }
    if flags & ESVG_PLUGIN_OPTIMIZE_COLORS != 0 {
        arbiter.add_single_element_plugin(Box::new(OptimizeColorsPlugin {}));
    }
    if flags & ESVG_PLUGIN_COLLAPSE_GROUPS != 0 {
        arbiter.add_whole_svg_plugin(Box::new(CollapseGroupsPlugin {}));
    }
    if flags & ESVG_PLUGIN_NUMBER_PRECISION != 0 {
        let precision = number_precision.clamp(1, 10);
        arbiter.add_single_element_plugin(Box::new(NumberPrecisionPlugin { precision }));
    }
    if flags & ESVG_PLUGIN_REMOVE_EMPTY_TEXT != 0 {
        arbiter.add_whole_svg_plugin(Box::new(RemoveEmptyTextPlugin {}));
    }
    if flags & ESVG_PLUGIN_REMOVE_UNNECESSARY_CLIPPATH != 0 {
        arbiter.add_whole_svg_plugin(Box::new(RemoveUnnecessaryClipPathsPlugin {}));
    }
    if flags & ESVG_PLUGIN_SORT_ATTRS != 0 {
        arbiter.add_single_element_plugin(Box::new(SortAttrsPlugin {}));
    }
    if flags & ESVG_PLUGIN_APPLY_TRANSFORMS != 0 {
        arbiter.add_whole_svg_plugin(Box::new(ApplyTransformsPlugin {}));
    }
    if flags & ESVG_PLUGIN_CSS_TO_ATTRIBUTES != 0 {
        arbiter.add_whole_svg_plugin(Box::new(CssToAttributesPlugin {}));
    }
    if flags & ESVG_PLUGIN_COMBINE_PATHS != 0 {
        arbiter.add_whole_svg_plugin(Box::new(CombinePathsPlugin {}));
    }
    if flags & ESVG_PLUGIN_MANGLE_IDS != 0 {
        arbiter.add_whole_svg_plugin(Box::new(MangleIdsPlugin { prefix: None }));
    }
    if flags & ESVG_PLUGIN_SIMPLIFY_PATHS != 0 {
        arbiter.add_single_element_plugin(Box::new(SimplifyPathsPlugin {}));
    }

    match arbiter.process(&svg) {
        Ok(result) => svg_to_ptr(result),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Optimize an SVG string with a plugin selection bitmask.
/// Returns a heap-allocated null-terminated C string, or null on error.
/// Must be freed with `esvg_free`.
#[unsafe(no_mangle)]
pub extern "C" fn esvg_optimize_with_flags(
    input: *const c_char,
    len: usize,
    flags: u64,
) -> *mut c_char {
    let svg = match parse_svg(input, len) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };

    let mut arbiter = Arbiter::new();

    if flags & ESVG_PLUGIN_REMOVE_UNNECESSARY_ATTRS != 0 {
        arbiter.add_single_element_plugin(Box::new(RemoveUnnecessaryAttrsPlugin {}));
    }
    if flags & ESVG_PLUGIN_SHAPE_TO_PATH != 0 {
        arbiter.add_single_element_plugin(Box::new(ShapeToPathPlugin {}));
    }
    if flags & ESVG_PLUGIN_OPTIMIZE_COLORS != 0 {
        arbiter.add_single_element_plugin(Box::new(OptimizeColorsPlugin {}));
    }
    if flags & ESVG_PLUGIN_COLLAPSE_GROUPS != 0 {
        arbiter.add_whole_svg_plugin(Box::new(CollapseGroupsPlugin {}));
    }
    if flags & ESVG_PLUGIN_NUMBER_PRECISION != 0 {
        arbiter.add_single_element_plugin(Box::new(NumberPrecisionPlugin { precision: 3 }));
    }
    if flags & ESVG_PLUGIN_REMOVE_EMPTY_TEXT != 0 {
        arbiter.add_whole_svg_plugin(Box::new(RemoveEmptyTextPlugin {}));
    }
    if flags & ESVG_PLUGIN_REMOVE_UNNECESSARY_CLIPPATH != 0 {
        arbiter.add_whole_svg_plugin(Box::new(RemoveUnnecessaryClipPathsPlugin {}));
    }
    if flags & ESVG_PLUGIN_SORT_ATTRS != 0 {
        arbiter.add_single_element_plugin(Box::new(SortAttrsPlugin {}));
    }
    if flags & ESVG_PLUGIN_APPLY_TRANSFORMS != 0 {
        arbiter.add_whole_svg_plugin(Box::new(ApplyTransformsPlugin {}));
    }
    if flags & ESVG_PLUGIN_CSS_TO_ATTRIBUTES != 0 {
        arbiter.add_whole_svg_plugin(Box::new(CssToAttributesPlugin {}));
    }
    if flags & ESVG_PLUGIN_COMBINE_PATHS != 0 {
        arbiter.add_whole_svg_plugin(Box::new(CombinePathsPlugin {}));
    }
    if flags & ESVG_PLUGIN_MANGLE_IDS != 0 {
        arbiter.add_whole_svg_plugin(Box::new(MangleIdsPlugin { prefix: None }));
    }
    if flags & ESVG_PLUGIN_SIMPLIFY_PATHS != 0 {
        arbiter.add_single_element_plugin(Box::new(SimplifyPathsPlugin {}));
    }

    match arbiter.process(&svg) {
        Ok(result) => svg_to_ptr(result),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a string returned by `esvg_optimize` or `esvg_optimize_with_flags`. Passing null is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn esvg_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe { drop(CString::from_raw(ptr)) };
    }
}
