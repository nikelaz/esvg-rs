/**
 * Plugin
 * 
 * Name: CSS to Attributes
 * Author: Nikola Lazarov
 *
 * Description:
 * Converts CSS to svg attributes
 *
 */

use std::error::Error;
use xmltree::Element;
use crate::plugin::WholeSVGPluginTrait;
use regex::Regex;
use crate::Svg;
use crate::css::InlineStyle;

pub struct CssToAttributesPlugin {}

const PRESENTATION_ATTRIBUTES: &[&str] = &[
    "fill",
    "fill-rule",
    "fill-opacity",
    "font-size",
    "font-style",
    "font-weight",
    "font-family",
    "filter",
    "clip-path",
    "clip-rule",
    "display",
    "opacity",
    "overflow",
    "stroke",
    "stroke-width",
    "stroke-dasharray",
    "stroke-dashoffset",
    "stroke-linecap",
    "stroke-linejoin",
    "stroke-miterlimit",
    "stroke-opacity",
    "text-decoration",
    "transform",
    "transform-origin",
];

impl CssToAttributesPlugin {
    fn has_styles(svg_content: &str) -> bool {
        let style_tag_regex = Regex::new(r"<style[\s>]").unwrap();
        let style_attr_regex = Regex::new(r#"style=".*?""#).unwrap();
        style_tag_regex.is_match(svg_content) || style_attr_regex.is_match(svg_content)
    }

    /*
     * TODO: This doesn't work for nested elements, for example a path in <g> or <style> in <defs>
     */
    fn find_styles(svg: &mut Svg) -> (Vec<&mut xmltree::Element>, Vec<&mut xmltree::Element>) {
        let mut style_elements = Vec::new();
        let mut elements_with_style_attr = Vec::new();

        for node in svg.root.children.iter_mut() {
            if let Some(element) = node.as_mut_element() {
                if element.name == "style" {
                    style_elements.push(element);
                    continue;
                }

                if element.attributes.contains_key("style") {
                    elements_with_style_attr.push(element);
                }
            }
        }

        (style_elements, elements_with_style_attr)
    }

    fn apply_inline_style_as_attr(element: &mut Element) {
        let style = element.attributes.get("style").unwrap();
        let mut inline_style = InlineStyle::from_string(style).unwrap();
        let mut props_to_remove = Vec::new();

        for prop in inline_style.props.iter() {
            if PRESENTATION_ATTRIBUTES.contains(&prop.name.as_str()) {
                element.attributes.insert(prop.name.to_string(), prop.value.clone());
                props_to_remove.push(prop.name.clone());
            }
        }

        // Remove properties **after** the iteration
        for prop_name in props_to_remove {
            inline_style.remove_prop(&prop_name);
        }

        // Update the "style" attribute if needed
        let inline_style_string = inline_style.to_string();

        if inline_style_string.trim().is_empty() {
            element.attributes.remove("style");    
        } else {
            element.attributes.insert("style".to_string(), inline_style_string);
        }
    }
}

impl WholeSVGPluginTrait for CssToAttributesPlugin {
    fn process(&self, svg: &mut Svg) -> Result<Svg, Box<dyn Error>> {
        if !CssToAttributesPlugin::has_styles(svg.to_string()?.as_str()) {
            return Ok(svg.clone());
        }

        let (mut style_elements, mut elements_with_style_attr) = CssToAttributesPlugin::find_styles(svg);

        for element in &mut elements_with_style_attr {
            CssToAttributesPlugin::apply_inline_style_as_attr(element);
        }

        Ok(svg.clone())
    }
}

