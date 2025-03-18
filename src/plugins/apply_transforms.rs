/**
 * Plugin
 * Name: Apply Transforms
 * Author: Nikola Lazarov
 * Description:
 * Apply transforms on vector elements
 *
 * Fix:
 * [ ] - currently in broken state as it erases the transform attribute (should only remove translate)
 *
 * Todo:
 * [x] - implement translate for paths
 * [ ] - more efficient memory management
 * [ ] - refactor
 * [ ] - support other shapes besides paths, prioritizing the "efficient" shapes - circle, ellipse
 * [ ] - support other transform types
 * [ ] - 
 */

use std::error::Error;
use crate::plugin::SingleElementPluginTrait;
use xmltree::Element;
use regex::Regex;
use crate::path::Path;
use crate::path::PathCommandType;

fn extract_translate_values(transform: &str) -> Option<(f64, f64)> {
  let re = Regex::new(r"translate\(\s*(-?\d*\.?\d+)\s*,?\s*(-?\d*\.?\d+)?\s*\)").unwrap();
    
  if let Some(captures) = re.captures(transform) {
    let x = captures.get(1)?.as_str().parse::<f64>().ok()?;
    let y = captures.get(2).map_or(0.0, |m| m.as_str().parse::<f64>().unwrap_or(0.0));
    return Some((x, y));
  }
  
  None
}

fn apply_translation(path: &mut Path, dx: f64, dy: f64) {
    for ref mut command in &mut path.commands {
        if  command.command_type == PathCommandType::MoveTo 
            || command.command_type == PathCommandType::LineTo
        { 

            command.values[0] += dx;
            command.values[1] += dy;
        }

        if  command.command_type == PathCommandType::HorizontalLine
        {
           command.values[0] += dx; 
        }

        if  command.command_type == PathCommandType::VerticalLine
        {
           command.values[0] += dy; 
        }

        if command.command_type == PathCommandType::CubicBezierCurve
        {
           command.values[0] += dx;
           command.values[1] += dy;
           command.values[2] += dx;
           command.values[3] += dy;
           command.values[4] += dx;
           command.values[5] += dy;
        }

        if command.command_type == PathCommandType::AdditionalBezierCurve
        {
            command.values[0] += dx;
            command.values[1] += dy;
            command.values[2] += dx;
            command.values[3] += dy;
        }

        if command.command_type == PathCommandType::QuadraticBezierCurve
        {
            command.values[0] += dx;
            command.values[1] += dy;
            command.values[2] += dx;
            command.values[3] += dy;
        }

        if command.command_type == PathCommandType::AdditionalQuadraticBezierCurve
        {
            command.values[0] += dx;
            command.values[1] += dy;
        }

        if command.command_type == PathCommandType::Arc
        {
            command.values[5] += dx;
            command.values[6] += dy;
        }
    }
}

pub struct ApplyTransformsPlugin {}

impl SingleElementPluginTrait for ApplyTransformsPlugin {
  fn process(&self, element: &Element) -> Result<Element, Box<dyn Error>> {
    let mut element_clone = element.clone();

    let transform = element.attributes.get("transform");

    if transform == None {
      return Ok(element_clone);
    }

    let translate_values = extract_translate_values(transform.unwrap());

    let (x, y) = translate_values.unwrap_or((0.0, 0.0));
 
    if translate_values == None {
      return Ok(element_clone);
    }

    if element_clone.name == "path" {
      let path_data = element_clone.attributes.get("d");

      if path_data != None {
        let mut path = Path::new(path_data.unwrap());
        
        // Apply translation
        apply_translation(&mut path, x, y);

        let transformed_path = path.to_string();

        element_clone.attributes.remove("d");
        element_clone.attributes.insert("d".to_string(), transformed_path);
        element_clone.attributes.remove("transform");
      }
    }

    Ok(element_clone)
  }
}

