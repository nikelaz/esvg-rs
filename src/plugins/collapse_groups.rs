use crate::plugin::WholeSVGPluginTrait;
use crate::Svg;
/**
 * Plugin
 *
 * Name: Collapse Groups
 * Author: Nikola Lazarov
 *
 * Description:
 * Collapses unnecessary SVG group elements:
 * - Removes empty groups (no children)
 * - Unwraps transparent groups (no meaningful attributes) replacing them with their children
 * - Collapses single-child groups by merging group attributes onto the child element
 *
 */
use std::error::Error;
use xmltree::Element;

/// Attributes that are meaningful on a group element — if any of these are present,
/// the group is NOT transparent and cannot simply be unwrapped without consideration.
const MEANINGFUL_ATTRIBUTES: &[&str] = &[
    "id",
    "class",
    "transform",
    "clip-path",
    "clip-rule",
    "mask",
    "filter",
    "opacity",
    "display",
    "fill",
    "fill-rule",
    "fill-opacity",
    "stroke",
    "stroke-width",
    "stroke-dasharray",
    "stroke-dashoffset",
    "stroke-linecap",
    "stroke-linejoin",
    "stroke-miterlimit",
    "stroke-opacity",
    "font-size",
    "font-style",
    "font-weight",
    "font-family",
    "text-decoration",
    "style",
    "overflow",
    "transform-origin",
];

pub struct CollapseGroupsPlugin;

impl CollapseGroupsPlugin {
    fn process_element(element: &mut Element) {
        // First, recurse into children (bottom-up processing)
        for child in element.children.iter_mut() {
            if let Some(child_element) = child.as_mut_element() {
                Self::process_element(child_element);
            }
        }

        // Now process this element's children to collapse groups
        Self::collapse_child_groups(element);
    }

    fn collapse_child_groups(element: &mut Element) {
        let mut new_children = Vec::new();

        for child in element.children.drain(..) {
            match child {
                xmltree::XMLNode::Element(group) if group.name == "g" => {
                    let child_element_count = group
                        .children
                        .iter()
                        .filter(|c| c.as_element().is_some())
                        .count();

                    // Case 1: Empty group — remove entirely
                    if child_element_count == 0 && group.children.is_empty() {
                        continue;
                    }

                    // Case 2: Transparent group (no meaningful attributes) — unwrap children
                    if Self::is_transparent_group(&group) {
                        for grandchild in group.children {
                            new_children.push(grandchild);
                        }
                        continue;
                    }

                    // Case 3: Single-child group — merge attributes onto child
                    if child_element_count == 1 && group.children.len() == 1 {
                        let merged = Self::merge_single_child(group);
                        new_children.push(xmltree::XMLNode::Element(merged));
                        continue;
                    }

                    // Group not collapsible — keep as-is
                    new_children.push(xmltree::XMLNode::Element(group));
                }
                other => {
                    // Not a group element — keep as-is
                    new_children.push(other);
                }
            }
        }

        element.children = new_children;
    }

    /// A group is "transparent" if it has no meaningful attributes —
    /// i.e., it contributes nothing visually and can be removed leaving only its children.
    fn is_transparent_group(group: &Element) -> bool {
        for attr_name in group.attributes.keys() {
            if MEANINGFUL_ATTRIBUTES.contains(&attr_name.as_str()) {
                return false;
            }
        }
        true
    }

    /// Merge a single-child group's attributes onto its child element.
    /// Takes ownership of the group, returns the merged child element.
    /// If both group and child have `id`, the merge is skipped and the group is returned as-is.
    fn merge_single_child(mut group: Element) -> Element {
        let child_node = match group.children.pop() {
            Some(node) => node,
            None => return group,
        };

        let mut child = match child_node {
            xmltree::XMLNode::Element(el) => el,
            _ => return group,
        };

        // Don't merge if both group and child have `id` — both are meaningful
        if group.attributes.contains_key("id") && child.attributes.contains_key("id") {
            group.children.push(xmltree::XMLNode::Element(child));
            return group;
        }

        // Merge each group attribute onto the child
        for (attr_name, attr_value) in &group.attributes {
            if attr_name == "transform" {
                // Concatenate transforms: group transform first, then child transform
                if let Some(child_transform) = child.attributes.get("transform") {
                    let merged = format!("{} {}", attr_value, child_transform);
                    child.attributes.insert("transform".to_string(), merged);
                } else {
                    child
                        .attributes
                        .insert("transform".to_string(), attr_value.clone());
                }
            } else if child.attributes.contains_key(attr_name) {
                // Child already has this attribute — child wins (skip group's value)
            } else {
                // Child doesn't have it — inherit from group
                child
                    .attributes
                    .insert(attr_name.clone(), attr_value.clone());
            }
        }

        child
    }
}

impl WholeSVGPluginTrait for CollapseGroupsPlugin {
    fn process(&self, svg: &mut Svg) -> Result<Svg, Box<dyn Error>> {
        Self::process_element(&mut svg.root);

        Ok(svg.clone())
    }
}
