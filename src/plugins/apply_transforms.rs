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
            if command.command_type == PathCommandType::Arc {
                // Handle invalid arcs
                if command.values.len() < 5 {
                    continue;
                }

                command.values[5] += dx;
                command.values[6] += dy;
                continue;
            }

            Self::translate_alternating_coords(&mut command.values, dx, dy);
        }
    }

    fn apply_scale(path: &mut Path, scale_dx: f32, scale_dy: f32) {
        for command in path.commands.iter_mut() {
            if command.command_type == PathCommandType::Arc {
                // Handle invalid arcs
                if command.values.len() < 5 {
                    continue;
                }

                command.values[5] = command.values[5] * scale_dx;
                command.values[6] = command.values[5] * scale_dy;
                continue;
            }

            Self::scale_alternating_coords(&mut command.values, scale_dx, scale_dy);
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
