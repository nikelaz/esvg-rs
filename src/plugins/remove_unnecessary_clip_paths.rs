use crate::plugin::WholeSVGPluginTrait;
use crate::Svg;
/**
* Plugin
*
* Name: Remove Unnecessary Clip Paths
* Author: Nikola Lazarov
*
* Description:
* Removes clip-path definitions and references that have no visual effect.
* Detects the following cases:
* - Empty clipPaths (no child elements)
* - ClipPaths containing a rect that fully covers the SVG viewBox
* - Orphaned clipPath definitions that are not referenced by any element
*
*/
use std::collections::HashSet;
use std::error::Error;
use xmltree::Element;

pub struct RemoveUnnecessaryClipPathsPlugin;

/// Parsed viewBox: min-x, min-y, width, height
struct ViewBox {
    min_x: f64,
    min_y: f64,
    width: f64,
    height: f64,
}

impl ViewBox {
    fn from_str(s: &str) -> Option<Self> {
        let parts: Vec<f64> = s
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse().ok())
            .collect();

        if parts.len() == 4 {
            Some(ViewBox {
                min_x: parts[0],
                min_y: parts[1],
                width: parts[2],
                height: parts[3],
            })
        } else {
            None
        }
    }
}

impl RemoveUnnecessaryClipPathsPlugin {
    /// Parse a clip-path attribute value like `url(#someId)` and extract the ID.
    fn parse_clip_path_url(value: &str) -> Option<String> {
        let trimmed = value.trim();
        if trimmed.starts_with("url(") && trimmed.ends_with(')') {
            let inner = &trimmed[4..trimmed.len() - 1].trim();
            // Remove surrounding quotes if present
            let inner = inner.trim_matches(|c| c == '\'' || c == '"');
            if inner.starts_with('#') {
                return Some(inner[1..].to_string());
            }
        }
        None
    }

    /// Check if a clipPath element is unnecessary because it contains a single
    /// rect that fully covers the viewBox.
    fn is_clip_path_covering_viewbox(clip_path: &Element, viewbox: &ViewBox) -> bool {
        // Get child elements (skip text nodes, etc.)
        let child_elements: Vec<&Element> = clip_path
            .children
            .iter()
            .filter_map(|c| c.as_element())
            .collect();

        // Must have exactly one child element and it must be a rect
        if child_elements.len() != 1 || child_elements[0].name != "rect" {
            return false;
        }

        let rect = child_elements[0];

        let x: f64 = rect
            .attributes
            .get("x")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);
        let y: f64 = rect
            .attributes
            .get("y")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);
        let width: f64 = match rect.attributes.get("width").and_then(|v| v.parse().ok()) {
            Some(w) => w,
            None => return false, // width is required for rect
        };
        let height: f64 = match rect.attributes.get("height").and_then(|v| v.parse().ok()) {
            Some(h) => h,
            None => return false, // height is required for rect
        };

        // The rect covers the viewBox if it starts at or before the viewBox origin
        // and extends to or beyond the viewBox extent
        x <= viewbox.min_x
            && y <= viewbox.min_y
            && (x + width) >= (viewbox.min_x + viewbox.width)
            && (y + height) >= (viewbox.min_y + viewbox.height)
    }

    /// Check if a clipPath element is empty (no child elements).
    fn is_clip_path_empty(clip_path: &Element) -> bool {
        !clip_path.children.iter().any(|c| c.as_element().is_some())
    }

    /// Collect all clipPath IDs that are referenced via clip-path attributes
    /// anywhere in the tree.
    fn collect_referenced_ids(element: &Element, referenced: &mut HashSet<String>) {
        if let Some(clip_attr) = element.attributes.get("clip-path") {
            if let Some(id) = Self::parse_clip_path_url(clip_attr) {
                referenced.insert(id);
            }
        }

        for child in &element.children {
            if let Some(child_element) = child.as_element() {
                Self::collect_referenced_ids(child_element, referenced);
            }
        }
    }

    /// Remove clip-path attributes that reference IDs in the removal set.
    fn remove_clip_path_references(element: &mut Element, ids_to_remove: &HashSet<String>) {
        if let Some(clip_attr) = element.attributes.get("clip-path").cloned() {
            if let Some(id) = Self::parse_clip_path_url(&clip_attr) {
                if ids_to_remove.contains(&id) {
                    element.attributes.remove("clip-path");
                }
            }
        }

        for child in element.children.iter_mut() {
            if let Some(child_element) = child.as_mut_element() {
                Self::remove_clip_path_references(child_element, ids_to_remove);
            }
        }
    }

    /// Remove clipPath definitions from defs whose IDs are in the removal set.
    fn remove_clip_path_defs(element: &mut Element, ids_to_remove: &HashSet<String>) {
        for child in element.children.iter_mut() {
            if let Some(child_element) = child.as_mut_element() {
                if child_element.name == "defs" {
                    let new_children: Vec<xmltree::XMLNode> = child_element
                        .children
                        .iter()
                        .filter(|node| {
                            if let Some(el) = node.as_element() {
                                if el.name == "clipPath" {
                                    if let Some(id) = el.attributes.get("id") {
                                        return !ids_to_remove.contains(id);
                                    }
                                }
                            }
                            true
                        })
                        .cloned()
                        .collect();
                    child_element.children = new_children;
                }
            }
        }
    }
}

impl WholeSVGPluginTrait for RemoveUnnecessaryClipPathsPlugin {
    fn process(&self, svg: &mut Svg) -> Result<Svg, Box<dyn Error>> {
        let viewbox = svg
            .root
            .attributes
            .get("viewBox")
            .and_then(|s| ViewBox::from_str(s));

        let mut ids_to_remove: HashSet<String> = HashSet::new();

        // Phase 1: Find unnecessary clipPaths in defs
        for child in svg.root.children.iter() {
            if let Some(defs) = child.as_element() {
                if defs.name == "defs" {
                    for def_child in defs.children.iter() {
                        if let Some(clip_path) = def_child.as_element() {
                            if clip_path.name == "clipPath" {
                                if let Some(id) = clip_path.attributes.get("id") {
                                    let is_empty = Self::is_clip_path_empty(clip_path);
                                    let covers_viewbox = viewbox.as_ref().map_or(false, |vb| {
                                        Self::is_clip_path_covering_viewbox(clip_path, vb)
                                    });

                                    if is_empty || covers_viewbox {
                                        ids_to_remove.insert(id.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Phase 2: Find orphaned clipPaths (defined but never referenced)
        let mut referenced_ids: HashSet<String> = HashSet::new();
        Self::collect_referenced_ids(&svg.root, &mut referenced_ids);

        for child in svg.root.children.iter() {
            if let Some(defs) = child.as_element() {
                if defs.name == "defs" {
                    for def_child in defs.children.iter() {
                        if let Some(clip_path) = def_child.as_element() {
                            if clip_path.name == "clipPath" {
                                if let Some(id) = clip_path.attributes.get("id") {
                                    if !referenced_ids.contains(id) {
                                        ids_to_remove.insert(id.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // If nothing to remove, return early
        if ids_to_remove.is_empty() {
            return Ok(svg.clone());
        }

        // Phase 3: Remove clip-path references and clipPath definitions
        Self::remove_clip_path_references(&mut svg.root, &ids_to_remove);
        Self::remove_clip_path_defs(&mut svg.root, &ids_to_remove);

        Ok(svg.clone())
    }
}
