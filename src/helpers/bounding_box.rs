use crate::path::{Path, PathCommandType};
use xmltree::Element;

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy)]
pub struct BBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BBox {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        BBox {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    /// Returns true if both axes have non-negative extent.
    pub fn is_valid(&self) -> bool {
        self.max_x >= self.min_x && self.max_y >= self.min_y
    }

    /// Returns true if `self` fully contains `other`.
    pub fn contains(&self, other: &BBox) -> bool {
        self.min_x <= other.min_x
            && self.min_y <= other.min_y
            && self.max_x >= other.max_x
            && self.max_y >= other.max_y
    }

    /// Returns the smallest bbox that contains both `self` and `other`.
    pub fn union(&self, other: &BBox) -> BBox {
        BBox {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }

    /// Expand this bbox to include the point (x, y).
    fn expand_to(&mut self, x: f64, y: f64) {
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
        self.max_x = self.max_x.max(x);
        self.max_y = self.max_y.max(y);
    }
}

/// Parse an attribute as f64, returning a default if missing or unparseable.
fn attr_f64(element: &Element, name: &str, default: f64) -> f64 {
    element
        .attributes
        .get(name)
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(default)
}

/// Parse a required attribute as f64, returning None if missing or unparseable.
fn attr_f64_required(element: &Element, name: &str) -> Option<f64> {
    element
        .attributes
        .get(name)
        .and_then(|v| v.parse::<f64>().ok())
}

/// Compute the bounding box of a `<rect>` element.
fn bbox_rect(element: &Element) -> Option<BBox> {
    let x = attr_f64(element, "x", 0.0);
    let y = attr_f64(element, "y", 0.0);
    let width = attr_f64_required(element, "width")?;
    let height = attr_f64_required(element, "height")?;

    if width < 0.0 || height < 0.0 {
        return None;
    }

    Some(BBox::new(x, y, x + width, y + height))
}

/// Compute the bounding box of a `<circle>` element.
fn bbox_circle(element: &Element) -> Option<BBox> {
    let cx = attr_f64(element, "cx", 0.0);
    let cy = attr_f64(element, "cy", 0.0);
    let r = attr_f64_required(element, "r")?;

    if r < 0.0 {
        return None;
    }

    Some(BBox::new(cx - r, cy - r, cx + r, cy + r))
}

/// Compute the bounding box of an `<ellipse>` element.
fn bbox_ellipse(element: &Element) -> Option<BBox> {
    let cx = attr_f64(element, "cx", 0.0);
    let cy = attr_f64(element, "cy", 0.0);
    let rx = attr_f64_required(element, "rx")?;
    let ry = attr_f64_required(element, "ry")?;

    if rx < 0.0 || ry < 0.0 {
        return None;
    }

    Some(BBox::new(cx - rx, cy - ry, cx + rx, cy + ry))
}

/// Compute the bounding box of a `<line>` element.
fn bbox_line(element: &Element) -> Option<BBox> {
    let x1 = attr_f64(element, "x1", 0.0);
    let y1 = attr_f64(element, "y1", 0.0);
    let x2 = attr_f64(element, "x2", 0.0);
    let y2 = attr_f64(element, "y2", 0.0);

    Some(BBox::new(x1.min(x2), y1.min(y2), x1.max(x2), y1.max(y2)))
}

/// Parse a `points` attribute (space and/or comma separated coordinate pairs)
/// and compute the bounding box.
fn bbox_points(element: &Element) -> Option<BBox> {
    let points_str = element.attributes.get("points")?;

    let nums: Vec<f64> = points_str
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();

    // Need at least one coordinate pair
    if nums.len() < 2 || nums.len() % 2 != 0 {
        return None;
    }

    let mut bbox = BBox::new(nums[0], nums[1], nums[0], nums[1]);
    for pair in nums.chunks(2) {
        bbox.expand_to(pair[0], pair[1]);
    }

    Some(bbox)
}

/// Compute the bounding box of a `<path>` element using the conservative
/// control-point hull approach. For curves, all control points are included
/// which overestimates the bbox — this is intentionally safe for our use case.
fn bbox_path(element: &Element) -> Option<BBox> {
    let d = element.attributes.get("d")?;
    let path = Path::new(d);

    if path.commands.is_empty() {
        return None;
    }

    let mut bbox: Option<BBox> = None;
    let mut cur_x: f64 = 0.0;
    let mut cur_y: f64 = 0.0;
    // Track the start of the current sub-path for Z commands
    let mut sub_start_x: f64 = 0.0;
    let mut sub_start_y: f64 = 0.0;

    for cmd in &path.commands {
        let vals: Vec<f64> = cmd.values.iter().map(|v| *v as f64).collect();

        match cmd.command_type {
            // ── Move ──────────────────────────────────────────────
            PathCommandType::MoveTo => {
                if vals.len() >= 2 {
                    cur_x = vals[0];
                    cur_y = vals[1];
                    sub_start_x = cur_x;
                    sub_start_y = cur_y;
                    expand_bbox(&mut bbox, cur_x, cur_y);
                }
            }
            PathCommandType::MoveToRelative => {
                if vals.len() >= 2 {
                    cur_x += vals[0];
                    cur_y += vals[1];
                    sub_start_x = cur_x;
                    sub_start_y = cur_y;
                    expand_bbox(&mut bbox, cur_x, cur_y);
                }
            }

            // ── Line ──────────────────────────────────────────────
            PathCommandType::LineTo => {
                if vals.len() >= 2 {
                    cur_x = vals[0];
                    cur_y = vals[1];
                    expand_bbox(&mut bbox, cur_x, cur_y);
                }
            }
            PathCommandType::LineToRelative => {
                if vals.len() >= 2 {
                    cur_x += vals[0];
                    cur_y += vals[1];
                    expand_bbox(&mut bbox, cur_x, cur_y);
                }
            }
            PathCommandType::HorizontalLine => {
                if vals.len() >= 1 {
                    cur_x = vals[0];
                    expand_bbox(&mut bbox, cur_x, cur_y);
                }
            }
            PathCommandType::HorizontalLineRelative => {
                if vals.len() >= 1 {
                    cur_x += vals[0];
                    expand_bbox(&mut bbox, cur_x, cur_y);
                }
            }
            PathCommandType::VerticalLine => {
                if vals.len() >= 1 {
                    cur_y = vals[0];
                    expand_bbox(&mut bbox, cur_x, cur_y);
                }
            }
            PathCommandType::VerticalLineRelative => {
                if vals.len() >= 1 {
                    cur_y += vals[0];
                    expand_bbox(&mut bbox, cur_x, cur_y);
                }
            }

            // ── Cubic Bézier ──────────────────────────────────────
            // C: x1 y1 x2 y2 x y (6 values)
            PathCommandType::CubicBezierCurve => {
                if vals.len() >= 6 {
                    // Include control points (conservative)
                    expand_bbox(&mut bbox, vals[0], vals[1]);
                    expand_bbox(&mut bbox, vals[2], vals[3]);
                    cur_x = vals[4];
                    cur_y = vals[5];
                    expand_bbox(&mut bbox, cur_x, cur_y);
                }
            }
            PathCommandType::CubicBezierCurveRelative => {
                if vals.len() >= 6 {
                    expand_bbox(&mut bbox, cur_x + vals[0], cur_y + vals[1]);
                    expand_bbox(&mut bbox, cur_x + vals[2], cur_y + vals[3]);
                    cur_x += vals[4];
                    cur_y += vals[5];
                    expand_bbox(&mut bbox, cur_x, cur_y);
                }
            }

            // ── Smooth Cubic Bézier ───────────────────────────────
            // S: x2 y2 x y (4 values)
            PathCommandType::AdditionalBezierCurve => {
                if vals.len() >= 4 {
                    expand_bbox(&mut bbox, vals[0], vals[1]);
                    cur_x = vals[2];
                    cur_y = vals[3];
                    expand_bbox(&mut bbox, cur_x, cur_y);
                }
            }
            PathCommandType::AdditionalBezierCurveRelative => {
                if vals.len() >= 4 {
                    expand_bbox(&mut bbox, cur_x + vals[0], cur_y + vals[1]);
                    cur_x += vals[2];
                    cur_y += vals[3];
                    expand_bbox(&mut bbox, cur_x, cur_y);
                }
            }

            // ── Quadratic Bézier ──────────────────────────────────
            // Q: x1 y1 x y (4 values)
            PathCommandType::QuadraticBezierCurve => {
                if vals.len() >= 4 {
                    expand_bbox(&mut bbox, vals[0], vals[1]);
                    cur_x = vals[2];
                    cur_y = vals[3];
                    expand_bbox(&mut bbox, cur_x, cur_y);
                }
            }
            PathCommandType::QuadraticBezierCurveRelative => {
                if vals.len() >= 4 {
                    expand_bbox(&mut bbox, cur_x + vals[0], cur_y + vals[1]);
                    cur_x += vals[2];
                    cur_y += vals[3];
                    expand_bbox(&mut bbox, cur_x, cur_y);
                }
            }

            // ── Smooth Quadratic Bézier ───────────────────────────
            // T: x y (2 values)
            PathCommandType::AdditionalQuadraticBezierCurve => {
                if vals.len() >= 2 {
                    cur_x = vals[0];
                    cur_y = vals[1];
                    expand_bbox(&mut bbox, cur_x, cur_y);
                }
            }
            PathCommandType::AdditionalQuadraticBezierCurveRelative => {
                if vals.len() >= 2 {
                    cur_x += vals[0];
                    cur_y += vals[1];
                    expand_bbox(&mut bbox, cur_x, cur_y);
                }
            }

            // ── Arc ───────────────────────────────────────────────
            // A: rx ry x-rotation large-arc sweep x y (7 values)
            // Conservative: include the endpoint only.
            PathCommandType::Arc => {
                if vals.len() >= 7 {
                    cur_x = vals[5];
                    cur_y = vals[6];
                    expand_bbox(&mut bbox, cur_x, cur_y);
                }
            }
            PathCommandType::ArcRelative => {
                if vals.len() >= 7 {
                    cur_x += vals[5];
                    cur_y += vals[6];
                    expand_bbox(&mut bbox, cur_x, cur_y);
                }
            }

            // ── Close ─────────────────────────────────────────────
            PathCommandType::Close | PathCommandType::CloseAlternate => {
                cur_x = sub_start_x;
                cur_y = sub_start_y;
            }
        }
    }

    bbox.filter(|b| b.is_valid())
}

/// Helper to expand an Option<BBox> to include a point.
fn expand_bbox(bbox: &mut Option<BBox>, x: f64, y: f64) {
    match bbox {
        Some(b) => b.expand_to(x, y),
        None => *bbox = Some(BBox::new(x, y, x, y)),
    }
}

/// Compute the bounding box of a group element by taking the union of
/// all children's bounding boxes. Returns None if any child's bbox
/// cannot be computed (conservative).
fn bbox_group(element: &Element) -> Option<BBox> {
    let mut result: Option<BBox> = None;
    let mut has_children = false;

    for child in &element.children {
        if let Some(child_el) = child.as_element() {
            has_children = true;
            let child_bbox = compute_element_bbox(child_el)?;
            result = Some(match result {
                Some(existing) => existing.union(&child_bbox),
                None => child_bbox,
            });
        }
    }

    if !has_children {
        return None;
    }

    result
}

/// Compute the axis-aligned bounding box of an SVG element.
///
/// Returns `None` when the bbox cannot be reliably determined:
/// - Element has a `transform` attribute
/// - Element type is unsupported (text, image, use, foreignObject, etc.)
/// - Required attributes are missing or unparseable
///
/// For path curves (C, S, Q, T), the control-point bounding box is used,
/// which conservatively overestimates the true bbox. For arcs, only the
/// endpoint is included.
pub fn compute_element_bbox(element: &Element) -> Option<BBox> {
    // Bail out if element has a transform — we cannot reliably compute
    // the bbox without applying it.
    if element.attributes.contains_key("transform") {
        return None;
    }

    match element.name.as_str() {
        "rect" => bbox_rect(element),
        "circle" => bbox_circle(element),
        "ellipse" => bbox_ellipse(element),
        "line" => bbox_line(element),
        "polyline" | "polygon" => bbox_points(element),
        "path" => bbox_path(element),
        "g" | "a" | "clipPath" | "marker" | "symbol" => bbox_group(element),
        // Elements whose geometry we cannot determine
        _ => None,
    }
}
