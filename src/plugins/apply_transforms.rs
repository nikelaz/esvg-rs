use crate::path::Path;
use crate::path::PathCommandType;
use crate::plugin::WholeSVGPluginTrait;
use crate::transform::TransformList;
use crate::transform::TransformType;
use crate::Svg;
use regex::Regex;
/**
 * Plugin
 *
 * Name: Apply Transforms
 * Author: Nikola Lazarov
 *
 * Description:
 * Apply transforms on vector elements. Supports path, circle, ellipse and rect.
 * Applies translate and scale transformations.
 * For group elements, pushes transforms down to children and then applies
 * them recursively to leaf elements.
 *
 * Todo:
 * [x] - Support circle and ellipse (they are more efficient than path)
 * [x] - Support groups
 * [ ] - Support other transform types - matrix, skew etc
 *
 */
use std::error::Error;
use xmltree::Element;

pub struct ApplyTransformsPlugin;

impl ApplyTransformsPlugin {
    fn translate_alternating_coords(coords: &mut Vec<f32>, dx: f32, dy: f32) {
        let mut axis = 'x';

        for coord in coords.iter_mut() {
            if axis == 'x' {
                *coord += dx;
                axis = 'y';
                continue;
            }

            *coord += dy;
            axis = 'x';
        }
    }

    fn scale_alternating_coords(coords: &mut Vec<f32>, scale_dx: f32, scale_dy: f32) {
        let mut axis = 'x';

        for coord in coords.iter_mut() {
            if axis == 'x' {
                *coord *= scale_dx;
                axis = 'y';
                continue;
            }

            *coord *= scale_dy;
            axis = 'x';
        }
    }

    fn apply_translation(path: &mut Path, dx: f32, dy: f32) {
        for command in path.commands.iter_mut() {
            match command.command_type {
                // Absolute arc: translate only the endpoint of each arc segment.
                // Each arc segment is 7 values: rx ry x-rotation large-arc sweep x y
                // Only indices 5 and 6 (within each 7-value group) are the endpoint.
                // rx, ry, x-rotation, large-arc-flag, sweep-flag are unaffected.
                PathCommandType::Arc => {
                    let mut i = 0;
                    while i + 6 < command.values.len() {
                        command.values[i + 5] += dx;
                        command.values[i + 6] += dy;
                        i += 7;
                    }
                }

                // Relative arc: all values are offsets — endpoint is already relative,
                // so translation has no effect.
                PathCommandType::ArcRelative => {}

                // Absolute horizontal line: single x-coordinate.
                PathCommandType::HorizontalLine => {
                    for v in command.values.iter_mut() {
                        *v += dx;
                    }
                }

                // Absolute vertical line: single y-coordinate.
                PathCommandType::VerticalLine => {
                    for v in command.values.iter_mut() {
                        *v += dy;
                    }
                }

                // Relative commands encode offsets from the current position.
                // Translation of the entire shape does not change these offsets.
                PathCommandType::MoveToRelative
                | PathCommandType::LineToRelative
                | PathCommandType::HorizontalLineRelative
                | PathCommandType::VerticalLineRelative
                | PathCommandType::CubicBezierCurveRelative
                | PathCommandType::AdditionalBezierCurveRelative
                | PathCommandType::QuadraticBezierCurveRelative
                | PathCommandType::AdditionalQuadraticBezierCurveRelative => {}

                // Commands with no values.
                PathCommandType::Close | PathCommandType::CloseAlternate => {}

                // All remaining absolute commands use alternating x/y coordinates.
                _ => {
                    Self::translate_alternating_coords(&mut command.values, dx, dy);
                }
            }
        }
    }

    fn apply_scale(path: &mut Path, scale_dx: f32, scale_dy: f32) {
        for command in path.commands.iter_mut() {
            match command.command_type {
                // Absolute arc: scale rx (index 0), ry (index 1), and the endpoint
                // (indices 5 and 6) within each 7-value arc segment.
                // x-rotation (2) and flags (3, 4) are untouched.
                PathCommandType::Arc => {
                    let mut i = 0;
                    while i + 6 < command.values.len() {
                        command.values[i + 0] *= scale_dx; // rx
                        command.values[i + 1] *= scale_dy; // ry
                        command.values[i + 5] *= scale_dx; // endpoint x
                        command.values[i + 6] *= scale_dy; // endpoint y
                        i += 7;
                    }
                }

                // Relative arc: rx, ry, and the relative endpoint dx/dy all scale.
                PathCommandType::ArcRelative => {
                    let mut i = 0;
                    while i + 6 < command.values.len() {
                        command.values[i + 0] *= scale_dx; // rx
                        command.values[i + 1] *= scale_dy; // ry
                        command.values[i + 5] *= scale_dx; // relative endpoint dx
                        command.values[i + 6] *= scale_dy; // relative endpoint dy
                        i += 7;
                    }
                }

                // Absolute horizontal line: single x-coordinate.
                PathCommandType::HorizontalLine | PathCommandType::HorizontalLineRelative => {
                    for v in command.values.iter_mut() {
                        *v *= scale_dx;
                    }
                }

                // Absolute/relative vertical line: single y-coordinate.
                PathCommandType::VerticalLine | PathCommandType::VerticalLineRelative => {
                    for v in command.values.iter_mut() {
                        *v *= scale_dy;
                    }
                }

                // Commands with no values.
                PathCommandType::Close | PathCommandType::CloseAlternate => {}

                // All other commands (absolute and relative) use alternating x/y values.
                _ => {
                    Self::scale_alternating_coords(&mut command.values, scale_dx, scale_dy);
                }
            }
        }
    }

    fn remove_translate_from_transform(element: &mut Element) {
        if let Some(transform_value) = element.attributes.get_mut("transform") {
            // Regex pattern to match "translate(...)"
            let re = Regex::new(r"\s*translate\([-?\d\.]+(?:\s*,?\s*[-?\d\.]+)?\)\s*").unwrap();

            // Replace the "translate(...)" part with an empty string
            let new_transform_value = re.replace_all(transform_value, "").trim().to_string();

            // If the transform is now empty (i.e., there were only translate transforms), remove the attribute
            if new_transform_value.is_empty() {
                element.attributes.remove("transform");
            } else {
                // Otherwise, update the transform attribute with the modified value
                *transform_value = new_transform_value;
            }
        }
    }

    fn remove_scale_from_transform(element: &mut Element) {
        if let Some(transform_value) = element.attributes.get_mut("transform") {
            // Regex pattern to match "scale(...)"
            let re = Regex::new(r"\s*scale\([-?\d\.]+(?:\s*,?\s*[-?\d\.]+)?\)\s*").unwrap();

            // Replace the "scale(...)" part with an empty string
            let new_transform_value = re.replace_all(transform_value, "").trim().to_string();

            // If the transform is now empty (i.e., there were only scale transforms), remove the attribute
            if new_transform_value.is_empty() {
                element.attributes.remove("transform");
            } else {
                // Otherwise, update the transform attribute with the modified value
                *transform_value = new_transform_value;
            }
        }
    }

    fn apply_circle_translation(element: &mut Element, dx: f32, dy: f32) {
        let cx = element.attributes.get("cx");
        let cy = element.attributes.get("cy");

        if cx.is_none() || cy.is_none() {
            return;
        }

        let cx = cx.unwrap().parse::<f32>().unwrap();
        let cy = cy.unwrap().parse::<f32>().unwrap();

        element
            .attributes
            .insert("cx".to_string(), (cx + dx).to_string());
        element
            .attributes
            .insert("cy".to_string(), (cy + dy).to_string());
    }

    fn apply_circle_scale(element: &mut Element, scale_x: f32, scale_y: f32) {
        let cx = element.attributes.get("cx");
        let cy = element.attributes.get("cy");
        let r = element.attributes.get("r");

        if cx.is_none() || cy.is_none() || r.is_none() {
            return;
        }

        let cx = cx.unwrap().parse::<f32>().unwrap();
        let cy = cy.unwrap().parse::<f32>().unwrap();
        let r = r.unwrap().parse::<f32>().unwrap();

        element
            .attributes
            .insert("cx".to_string(), (cx * scale_x).to_string());
        element
            .attributes
            .insert("cy".to_string(), (cy * scale_y).to_string());

        if scale_x != scale_y {
            element.name = "ellipse".to_string();
            element
                .attributes
                .insert("rx".to_string(), (r * scale_x).to_string());
            element
                .attributes
                .insert("ry".to_string(), (r * scale_y).to_string());
            element.attributes.remove("r");
        } else {
            element
                .attributes
                .insert("r".to_string(), (r * scale_x).to_string());
        }
    }

    fn apply_rect_translation(element: &mut Element, dx: f32, dy: f32) {
        let x: f32 = element
            .attributes
            .get("x")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);
        let y: f32 = element
            .attributes
            .get("y")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);

        element
            .attributes
            .insert("x".to_string(), (x + dx).to_string());
        element
            .attributes
            .insert("y".to_string(), (y + dy).to_string());
    }

    fn apply_rect_scale(element: &mut Element, scale_x: f32, scale_y: f32) {
        let x: f32 = element
            .attributes
            .get("x")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);
        let y: f32 = element
            .attributes
            .get("y")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);

        let width = element
            .attributes
            .get("width")
            .and_then(|v| v.parse::<f32>().ok());
        let height = element
            .attributes
            .get("height")
            .and_then(|v| v.parse::<f32>().ok());

        if width.is_none() || height.is_none() {
            return;
        }

        let width = width.unwrap();
        let height = height.unwrap();

        element
            .attributes
            .insert("x".to_string(), (x * scale_x).to_string());
        element
            .attributes
            .insert("y".to_string(), (y * scale_y).to_string());
        element
            .attributes
            .insert("width".to_string(), (width * scale_x).to_string());
        element
            .attributes
            .insert("height".to_string(), (height * scale_y).to_string());
    }

    /// Check if an element is a container/group element whose transform
    /// can be pushed down to its children.
    fn is_group_element(element: &Element) -> bool {
        matches!(
            element.name.as_str(),
            "g" | "a" | "clipPath" | "marker" | "mask" | "pattern" | "switch" | "symbol"
        )
    }

    /// Apply transforms to a leaf element (path, circle, ellipse) by modifying
    /// its coordinates directly. Only translate and scale are applied; any
    /// unsupported transform types remain in the transform attribute.
    fn apply_leaf_transforms(element: &mut Element) {
        let transform_str = match element.attributes.get("transform") {
            Some(t) => t.clone(),
            None => return,
        };

        // Skip if element has transform-origin
        if element.attributes.contains_key("transform-origin") {
            return;
        }

        let transforms_list = TransformList::new(&transform_str);

        if element.name == "path" {
            if let Some(path_data) = element.attributes.get("d").cloned() {
                let mut path = Path::new(&path_data);

                for transform in transforms_list.transforms {
                    if transform.transform_type == TransformType::Translate {
                        if let Some(dx) = transform.get_x() {
                            Self::apply_translation(
                                &mut path,
                                dx,
                                transform.get_y().unwrap_or(0.0),
                            );
                            Self::remove_translate_from_transform(element);
                        }
                        continue;
                    }

                    if transform.transform_type == TransformType::Scale {
                        if let Some(dx) = transform.get_x() {
                            Self::apply_scale(&mut path, dx, transform.get_y().unwrap());
                            Self::remove_scale_from_transform(element);
                        }
                    }
                }

                let transformed_path = path.to_string();
                element.attributes.insert("d".to_string(), transformed_path);
            }
        } else if element.name == "circle" {
            for transform in transforms_list.transforms {
                if transform.transform_type == TransformType::Translate {
                    if let Some(dx) = transform.get_x() {
                        Self::apply_circle_translation(
                            element,
                            dx,
                            transform.get_y().unwrap_or(0.0),
                        );
                        Self::remove_translate_from_transform(element);
                    }
                    continue;
                }

                if transform.transform_type == TransformType::Scale {
                    if let Some(dx) = transform.get_x() {
                        Self::apply_circle_scale(element, dx, transform.get_y().unwrap());
                        Self::remove_scale_from_transform(element);
                    }
                }
            }
        } else if element.name == "rect" {
            for transform in transforms_list.transforms {
                if transform.transform_type == TransformType::Translate {
                    if let Some(dx) = transform.get_x() {
                        Self::apply_rect_translation(element, dx, transform.get_y().unwrap_or(0.0));
                        Self::remove_translate_from_transform(element);
                    }
                    continue;
                }

                if transform.transform_type == TransformType::Scale {
                    if let Some(dx) = transform.get_x() {
                        Self::apply_rect_scale(element, dx, transform.get_y().unwrap());
                        Self::remove_scale_from_transform(element);
                    }
                }
            }
        }
    }

    /// Push a group's transform down to each of its child elements, then
    /// remove the transform from the group. Each child gets the group's
    /// transform prepended to its own existing transform (if any).
    fn push_transform_to_children(element: &mut Element) {
        let group_transform = match element.attributes.get("transform") {
            Some(t) => t.clone(),
            None => return,
        };

        // Skip if the group has transform-origin — cannot safely decompose
        if element.attributes.contains_key("transform-origin") {
            return;
        }

        // Check that the group has at least one child element
        let has_child_elements = element.children.iter().any(|c| c.as_element().is_some());
        if !has_child_elements {
            return;
        }

        // Prepend the group's transform to each child element's transform.
        // SVG applies the group transform first (outermost), so it goes first
        // in the string: "groupTransform childTransform"
        for child in element.children.iter_mut() {
            if let Some(child_el) = child.as_mut_element() {
                let new_transform =
                    if let Some(child_transform) = child_el.attributes.get("transform") {
                        format!("{} {}", group_transform, child_transform)
                    } else {
                        group_transform.clone()
                    };

                child_el
                    .attributes
                    .insert("transform".to_string(), new_transform);
            }
        }

        // Remove the transform from the group
        element.attributes.remove("transform");
    }

    /// Recursively process the element tree: push group transforms down to
    /// children, then apply transforms to leaf elements.
    fn apply_transforms_recursive(element: &mut Element) {
        // If this is a group element with a transform, push it down to children
        if Self::is_group_element(element) {
            Self::push_transform_to_children(element);
        }

        // Recurse into children first (depth-first), so nested groups get
        // their transforms pushed down before leaf application
        for child in element.children.iter_mut() {
            if let Some(child_el) = child.as_mut_element() {
                Self::apply_transforms_recursive(child_el);
            }
        }

        // Apply transforms on leaf elements (path, circle)
        if !Self::is_group_element(element) {
            Self::apply_leaf_transforms(element);
        }
    }
}

impl WholeSVGPluginTrait for ApplyTransformsPlugin {
    fn process(&self, svg: &mut Svg) -> Result<Svg, Box<dyn Error>> {
        let mut svg_clone = svg.clone();

        // Process all children of the root SVG element
        for child in svg_clone.root.children.iter_mut() {
            if let Some(child_el) = child.as_mut_element() {
                Self::apply_transforms_recursive(child_el);
            }
        }

        Ok(svg_clone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::Path;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    /// Translating an Arc command with two chained arc segments must move
    /// the endpoint of BOTH segments, not just the first.
    #[test]
    fn translate_multi_arc_moves_all_endpoints() {
        // Two chained arcs: A rx ry rot flag flag x1 y1 rx ry rot flag flag x2 y2
        let mut path = Path::new("M 0 0 A 10 10 0 0 1 20 30 10 10 0 0 0 40 50 Z");
        ApplyTransformsPlugin::apply_translation(&mut path, 100.0, 200.0);

        let arc = &path.commands[1];
        // First arc endpoint: (20+100, 30+200) = (120, 230)
        assert!(
            approx_eq(arc.values[5], 120.0),
            "first arc x: {}",
            arc.values[5]
        );
        assert!(
            approx_eq(arc.values[6], 230.0),
            "first arc y: {}",
            arc.values[6]
        );
        // Second arc endpoint: (40+100, 50+200) = (140, 250)
        assert!(
            approx_eq(arc.values[12], 140.0),
            "second arc x: {}",
            arc.values[12]
        );
        assert!(
            approx_eq(arc.values[13], 250.0),
            "second arc y: {}",
            arc.values[13]
        );
        // Radii and flags must be unchanged
        assert!(approx_eq(arc.values[0], 10.0));
        assert!(approx_eq(arc.values[3], 0.0)); // large-arc flag
        assert!(approx_eq(arc.values[4], 1.0)); // sweep flag
    }

    /// Scaling an Arc command with two chained arc segments must scale
    /// rx, ry, and the endpoint of BOTH segments.
    #[test]
    fn scale_multi_arc_scales_all_segments() {
        let mut path = Path::new("M 0 0 A 10 20 0 0 1 30 40 5 6 0 1 0 70 80 Z");
        ApplyTransformsPlugin::apply_scale(&mut path, 2.0, 3.0);

        let arc = &path.commands[1];
        // First arc: rx*2=20, ry*3=60, x*2=60, y*3=120
        assert!(
            approx_eq(arc.values[0], 20.0),
            "first rx: {}",
            arc.values[0]
        );
        assert!(
            approx_eq(arc.values[1], 60.0),
            "first ry: {}",
            arc.values[1]
        );
        assert!(approx_eq(arc.values[5], 60.0), "first x: {}", arc.values[5]);
        assert!(
            approx_eq(arc.values[6], 120.0),
            "first y: {}",
            arc.values[6]
        );
        // Flags must be unchanged
        assert!(approx_eq(arc.values[3], 0.0)); // large-arc flag
        assert!(approx_eq(arc.values[4], 1.0)); // sweep flag
                                                // Second arc: rx*2=10, ry*3=18, x*2=140, y*3=240
        assert!(
            approx_eq(arc.values[7], 10.0),
            "second rx: {}",
            arc.values[7]
        );
        assert!(
            approx_eq(arc.values[8], 18.0),
            "second ry: {}",
            arc.values[8]
        );
        assert!(
            approx_eq(arc.values[12], 140.0),
            "second x: {}",
            arc.values[12]
        );
        assert!(
            approx_eq(arc.values[13], 240.0),
            "second y: {}",
            arc.values[13]
        );
        // Second arc flags unchanged
        assert!(approx_eq(arc.values[10], 1.0)); // large-arc flag
        assert!(approx_eq(arc.values[11], 0.0)); // sweep flag
    }

    /// Regression test for the specific path from undraw_file-analysis_nbtc.svg.
    /// The path has a sub-path starting with 'm' (relative move) followed by two
    /// chained absolute 'A' arcs. Translation must be applied to both arc endpoints.
    #[test]
    fn translate_real_world_multi_arc_path() {
        // Simplified version of the problematic sub-path:
        // m-.07,36.631 A17.4 17.4 0 1 0 245.5,608.594 17.4 17.4 0 0 0 262.863,626.027 Z
        // with translate(368.984, 161.693)
        let d = "M 262.933 589.4 a 19.232 19.232 0 1 1 -19.268 19.195 \
                 A 19.232 19.232 0 0 1 262.933 589.4 Z \
                 m -0.07 36.631 \
                 A 17.4 17.4 0 1 0 245.5 608.594 17.4 17.4 0 0 0 262.863 626.027 Z";
        let mut path = Path::new(d);
        ApplyTransformsPlugin::apply_translation(&mut path, 368.984, 161.693);

        // Find the second A command (the one after 'm')
        // Commands: M, a, A, m, A, Z
        let a_cmd = path
            .commands
            .iter()
            .find(|c| c.command_type == crate::path::PathCommandType::Arc && c.values.len() >= 14)
            .expect("should find multi-arc A command");

        // First endpoint: 245.5 + 368.984 = 614.484, 608.594 + 161.693 = 770.287
        assert!(
            approx_eq(a_cmd.values[5], 614.484),
            "first x: {}",
            a_cmd.values[5]
        );
        assert!(
            approx_eq(a_cmd.values[6], 770.287),
            "first y: {}",
            a_cmd.values[6]
        );
        // Second endpoint: 262.863 + 368.984 = 631.847, 626.027 + 161.693 = 787.72
        assert!(
            approx_eq(a_cmd.values[12], 631.847),
            "second x: {}",
            a_cmd.values[12]
        );
        assert!(
            approx_eq(a_cmd.values[13], 787.72),
            "second y: {}",
            a_cmd.values[13]
        );
    }
}
