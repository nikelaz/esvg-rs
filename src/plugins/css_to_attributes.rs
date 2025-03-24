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
use crate::css::CSSParser;

pub struct CssToAttributesPlugin {}

const PRESENTATION_ATTRIBUTES: &[&str] = &[
    "fill",
    "stroke",
    "stroke-width"
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
                style_elements.push(element); // Move happens here
            }

            if element.attributes.contains_key("style") {
                elements_with_style_attr.push(element); // Error: element already moved
            }
        }
    }

    (style_elements, elements_with_style_attr)
}

  fn apply_inline_style_as_attr(element: &mut Element) {
    let style = element.attributes.get("style").unwrap();
    let css_props = CSSParser::parse_props(style).unwrap();
    for prop in css_props {
        if PRESENTATION_ATTRIBUTES.contains(&prop.name.as_str()) {
            println!("{} is included in attributes", prop.name);
            element.attributes.insert(prop.name.to_string(), prop.value);
        }
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

