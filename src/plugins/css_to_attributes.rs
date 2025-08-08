/**
* Plugin
* 
* Name: CSS to Attributes
* Author: Nikola Lazarov
*
* Description:
* Converts CSS to svg attributes
*
* Todo:
* [x] Convert inline styles to attributes
* [ ] Convert CSS blocks to attributes
*
*/

use std::error::Error;
use xmltree::Element;
use crate::plugin::WholeSVGPluginTrait;
use crate::Svg;
use css_structs::css_declaration_list::CSSDeclarationList;


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

pub struct CssToAttributesPlugin;

impl CssToAttributesPlugin {
  fn traverse_and_apply_styles(element: &mut Element) {
    // If the element has a "style" attribute, apply styles as attributes
    if element.attributes.contains_key("style") {
      Self::apply_inline_style_as_attr(element);
    }

    // Recurse for all child elements
    for child in element.children.iter_mut() {
      if let Some(child_element) = child.as_mut_element() {
        Self::traverse_and_apply_styles(child_element);
      }
    }
  }

  fn apply_inline_style_as_attr(element: &mut Element) {
    let style = element.attributes.get("style").unwrap();
    let mut inline_style = CSSDeclarationList::from_string(style).unwrap();
    let mut props_to_remove = Vec::new();

    for prop in inline_style.declarations.iter() {
      if PRESENTATION_ATTRIBUTES.contains(&prop.name.as_str()) {
        element.attributes.insert(prop.name.to_string(), prop.value.clone());
        props_to_remove.push(prop.name.clone());
      }
    }

    // Remove properties **after** the iteration
    for prop_name in props_to_remove {
      inline_style.remove_declaration(&prop_name);
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
    CssToAttributesPlugin::traverse_and_apply_styles(&mut svg.root);
    
    Ok(svg.clone())
  }
}

