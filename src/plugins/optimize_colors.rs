// Plugin
//
// Name: Optimize Colors
// Author: Nikola Lazarov
// 
// Description:
// Optimizes color values to their most efficient representation
//
// Todo:
// [x] - Optimize hex colors
// [ ] - Support other color systems/representations

use std::error::Error;
use crate::plugin::SingleElementPluginTrait;
use xmltree::Element;

const COLOR_ATTRIBUTES: [&str; 4] = [
  "fill",
  "stroke",
  "stop-color",
  "color",
];

static STATIC_REPLACEMENTS: &[(&str, &str)] = &[
  ("black"  , "#000"),
  ("white"  , "#fff"),
  ("green"  , "#0f0"),
  ("blue"   , "#00f"),
  ("cyan"   , "#0ff"),
  ("magenta", "#f0f"),
  ("yellow" , "#ff0"),
  ("#ff0000", "red" ),
];

pub struct OptimizeColorsPlugin;

impl OptimizeColorsPlugin {
  fn normalize_hex_color(color: &str) -> Option<String> {
    if !color.starts_with('#') {
      return None;
    }
 
    match color.len() {
      4 => {
        // #abc -> #aabbcc
        let chars: Vec<char> = color.chars().collect();
        Some(format!(
          "#{}{}{}{}{}{}",
          chars[1], chars[1],
          chars[2], chars[2],
          chars[3], chars[3]
        ))
      }
      7 => Some(color.to_string()), // already full hex
      _ => None, // unsupported format
    }
  }

  fn optimize_hex_color(color: &str) -> Option<String> {
    if !color.starts_with('#') || color.len() != 7 {
      return None;
    }
    
    let chars: Vec<char> = color.chars().collect();
    
    if chars[1] == chars[2] &&
    chars[3] == chars[4] &&
    chars[5] == chars[6] {
      Some(format!("#{}{}{}", chars[1], chars[3], chars[5]))
    } else {
      Some(color.to_string())
    }
  }

  fn optimize_color_value(color_value: &str) -> String {
    let color_value = color_value.to_ascii_lowercase();
    let color_normalized = Self::normalize_hex_color(&color_value).unwrap_or(color_value.to_string());
    
    for &(key, value) in STATIC_REPLACEMENTS {
      if key == color_normalized {
        return value.to_string();
      }
    }

    Self::optimize_hex_color(&color_normalized).unwrap_or(color_value).to_string()
  }

  fn optimize_color_attributes(element: &mut Element) {
    for attr_name in COLOR_ATTRIBUTES.iter() {
      if let Some(color_value) = element.attributes.get_mut(*attr_name) {
        let simplified = Self::optimize_color_value(color_value);
        *color_value = simplified.to_string();
      }
    }
  }
}

impl SingleElementPluginTrait for OptimizeColorsPlugin {
  fn process(&self, element: &mut Element) -> Result<Element, Box<dyn Error>> {
    let mut element_clone = element.clone();

    Self::optimize_color_attributes(&mut element_clone);

    Ok(element_clone)
  }
}
