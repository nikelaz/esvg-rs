use crate::plugin::WholeSVGPluginTrait;
use crate::Svg;
use std::collections::HashMap;
/**
* Plugin
*
* Name: Combine Paths
* Author: Nikola Lazarov
*
* Description:
* Combines paths to a single element if they have the same attributes
* and order context
*
* TODO:
* [ ] - Test and refactor
*/
use std::error::Error;
use xmltree::Element;

pub struct CombinePathsPlugin;

impl CombinePathsPlugin {
    fn traverse_paths(element: &mut Element) {
        for child in element.children.iter_mut() {
            if let Some(child_element) = child.as_mut_element() {
                Self::traverse_paths(child_element);
            }
        }

        Self::combine_adjacent_paths(element);
    }

    // TODO: This NEEDS to be refactored and implemented in a more ellegant manner
    fn combine_adjacent_paths(element: &mut Element) {
        let mut new_children = Vec::new();
        let mut i = 0;

        while i < element.children.len() {
            let current_child = &element.children[i];

            if let Some(current_element) = current_child.as_element() {
                if current_element.name == "path" {
                    // Found a path, check if we can combine with following paths
                    let mut combined_d = String::new();
                    let mut last_combined_index = i;

                    // Get the d attribute of the first path
                    if let Some(d_attr) = current_element.attributes.get("d") {
                        combined_d.push_str(d_attr);
                    }

                    // Look for consecutive paths with same attributes
                    for j in (i + 1)..element.children.len() {
                        if let Some(next_element) = element.children[j].as_element() {
                            if next_element.name == "path"
                                && Self::have_same_non_d_attributes(current_element, next_element)
                            {
                                // Combine the d attributes
                                if let Some(next_d) = next_element.attributes.get("d") {
                                    if !combined_d.is_empty() && !next_d.is_empty() {
                                        combined_d.push(' ');
                                    }
                                    combined_d.push_str(next_d);
                                }
                                last_combined_index = j;
                            } else {
                                // Stop at first non-matching path or non-path element
                                break;
                            }
                        } else {
                            // Stop at non-element (text, etc.)
                            break;
                        }
                    }

                    if last_combined_index > i {
                        // We found paths to combine
                        let mut combined_element = current_element.clone();
                        combined_element
                            .attributes
                            .insert("d".to_string(), combined_d);
                        new_children.push(xmltree::XMLNode::Element(combined_element));
                        i = last_combined_index + 1;
                    } else {
                        // No combination, keep original
                        new_children.push(current_child.clone());
                        i += 1;
                    }
                } else {
                    // Not a path element, keep as is
                    new_children.push(current_child.clone());
                    i += 1;
                }
            } else {
                // Not an element (text node, etc.), keep as is
                new_children.push(current_child.clone());
                i += 1;
            }
        }

        element.children = new_children;
    }

    fn have_same_non_d_attributes(path1: &Element, path2: &Element) -> bool {
        // Create copies of attributes without the 'd' attribute
        let mut attrs1: HashMap<String, String> = path1.attributes.clone();
        let mut attrs2: HashMap<String, String> = path2.attributes.clone();

        attrs1.remove("d");
        attrs2.remove("d");

        attrs1 == attrs2
    }
}

impl WholeSVGPluginTrait for CombinePathsPlugin {
    fn process(&self, svg: &mut Svg) -> Result<Svg, Box<dyn Error>> {
        Self::traverse_paths(&mut svg.root);

        Ok(svg.clone())
    }
}
