use crate::plugin::WholeSVGPluginTrait;
use crate::Svg;
use std::error::Error;
use xmltree::Element;

pub struct RemoveEmptyTextPlugin;

impl RemoveEmptyTextPlugin {
    fn process_element(element: &mut Element) {
        // Recursively process children first (bottom-up)
        for child in element.children.iter_mut() {
            if let Some(child_element) = child.as_mut_element() {
                Self::process_element(child_element);
            }
        }

        // Retain non-empty text/tspan elements
        element.children.retain(|node| {
            if let Some(el) = node.as_element() {
                if el.name == "text" || el.name == "tspan" {
                    // Respect xml:space="preserve"
                    if let Some(space) = el.attributes.get("xml:space") {
                        if space == "preserve" {
                            return true;
                        }
                    }

                    return !Self::is_empty(el);
                }
            }
            true
        });
    }

    fn is_empty(element: &Element) -> bool {
        // If it has any child elements (that survived recursion), it's not empty
        if element.children.iter().any(|c| c.as_element().is_some()) {
            return false;
        }

        // Check text content
        match element.get_text() {
            Some(text) => text.trim().is_empty(),
            None => true, // No text content and no children elements -> empty
        }
    }
}

impl WholeSVGPluginTrait for RemoveEmptyTextPlugin {
    fn process(&self, svg: &mut Svg) -> Result<Svg, Box<dyn Error>> {
        Self::process_element(&mut svg.root);
        Ok(svg.clone())
    }
}
