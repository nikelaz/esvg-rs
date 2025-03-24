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
use crate::css::CSSPropsList;

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
    fn find_styles(svg: &mut Svg) {
        for node in svg.root.children.iter_mut() {
            if let Some(element) = node.as_mut_element() {
                if element.name == "style" {
                    println!("element.name {}", element.name);
                }

                if element.attributes.contains_key("style") {
                    CssToAttributesPlugin::apply_inline_style_as_attr(element); 
                }
            }
        }
    }

  fn apply_inline_style_as_attr(element: &mut Element) {
    let style = element.attributes.get("style").unwrap();
    let mut css_props_list = CSSPropsList::new(style);
    //let mut props_to_remove = Vec::new(); // Collect props to remove

    println!("css_props {:?}", css_props_list);

    for prop in css_props_list.list.clone() {
        if PRESENTATION_ATTRIBUTES.contains(&prop.name.as_str()) {
            println!("{} is included in attributes", prop.name);
            element.attributes.insert(prop.name.to_string(), prop.value.clone());
            css_props_list.remove(&prop.name.as_str());
        }
    }
 
    println!("css_props {:?}", css_props_list);
    if (css_props_list.list.len() == 0) {
        element.attributes.remove("style");
    }
    else {
        element.attributes.insert("style".to_string(), css_props_list.to_string());
    }
  }
}

impl WholeSVGPluginTrait for CssToAttributesPlugin {
  fn process(&self, svg: &mut Svg) -> Result<Svg, Box<dyn Error>> {
    if !CssToAttributesPlugin::has_styles(svg.to_string()?.as_str()) {
      return Ok(svg.clone());
    }

    CssToAttributesPlugin::find_styles(svg);

    Ok(svg.clone())
  }
}

