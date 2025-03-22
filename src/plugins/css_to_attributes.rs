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

pub struct CssToAttributesPlugin {}

impl CssToAttributesPlugin {
  fn has_styles(svg_content: &str) -> bool {
    let style_tag_regex = Regex::new(r"<style[\s>]").unwrap();
    let style_attr_regex = Regex::new(r#"style=".*?""#).unwrap();
    
    style_tag_regex.is_match(svg_content) || style_attr_regex.is_match(svg_content)
  }
}

impl WholeSVGPluginTrait for CssToAttributesPlugin {
  fn process(&self, svg: &Svg) -> Result<Svg, Box<dyn Error>> {
    if !CssToAttributesPlugin::has_styles(svg.to_string()?.as_str()) {
      println!("Svg does not have styles");
      return Ok(svg.clone());
    }

    println!("Svg has styles");
    Ok(svg.clone())
  }
}
