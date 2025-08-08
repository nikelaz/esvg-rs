/**
 * Plugin
 *
 * Name: Apply Transforms
 * Author: Nikola Lazarov
 * 
 * Description:
 * Apply transforms on vector elements. Supports path, circle and ellipse.
 * Applies translate and scale transformations.
 *
 * Todo:
 * [x] - Support circle and ellipse (they are more efficient than path)
 * [ ] - Support groups
 * [ ] - Support other transform types - matrix, skew etc
 *
 */

use std::error::Error;
use crate::plugin::SingleElementPluginTrait;
use xmltree::Element;
use regex::Regex;
use crate::path::Path;
use crate::path::PathCommandType;
use crate::transform::TransformList;
use crate::transform::TransformType;


pub struct ApplyTransformsPlugin;

impl ApplyTransformsPlugin {
  fn translate_alternating_coords(coords: &mut Vec<f32>, dx: f32, dy: f32) {
    let mut axis = 'x';

    for coord in coords.iter_mut() {
      if axis == 'x' {
        *coord += dx;
        axis = 'y';
        continue;
      }

      *coord += dy;
      axis = 'x';
    }
  }

  fn scale_alternating_coords(coords: &mut Vec<f32>, scale_dx: f32, scale_dy: f32) {
    let mut axis = 'x';

    for coord in coords.iter_mut() {
      if axis == 'x' {
        *coord *= scale_dx;
        axis = 'y';
        continue;
      }

      *coord *= scale_dy;
      axis = 'x';
    }
  }

  fn apply_translation(path: &mut Path, dx: f32, dy: f32) {
    for command in path.commands.iter_mut() {
      if command.command_type == PathCommandType::Arc {
        // Handle invalid arcs
        if command.values.len() < 5 {
          continue;
        }

        command.values[5] += dx;
        command.values[6] += dy;
        continue;
      }

      Self::translate_alternating_coords(&mut command.values, dx, dy);
    }
  }

  fn apply_scale(path: &mut Path, scale_dx: f32, scale_dy: f32) {
    for command in path.commands.iter_mut() {
      if command.command_type == PathCommandType::Arc {
        // Handle invalid arcs
        if command.values.len() < 5 {
          continue;
        }

        command.values[5] = command.values[5] * scale_dx;
        command.values[6] = command.values[5] * scale_dy;
        continue;
      }

      Self::scale_alternating_coords(&mut command.values, scale_dx, scale_dy);
    }
  }

  fn remove_translate_from_transform(element: &mut Element) {
    if let Some(transform_value) = element.attributes.get_mut("transform") {
      // Regex pattern to match "translate(...)"
      let re = Regex::new(r"\s*translate\([-?\d\.]+(?:\s*,?\s*[-?\d\.]+)?\)\s*").unwrap();

      // Replace the "translate(...)" part with an empty string
      let new_transform_value = re.replace_all(transform_value, "").trim().to_string();

      // If the transform is now empty (i.e., there were only translate transforms), remove the attribute
      if new_transform_value.is_empty() {
        element.attributes.remove("transform");
      } else {
        // Otherwise, update the transform attribute with the modified value
        *transform_value = new_transform_value;
      }
    }
  }

  fn remove_scale_from_transform(element: &mut Element) {
    if let Some(transform_value) = element.attributes.get_mut("transform") {
      // Regex pattern to match "translate(...)"
      let re = Regex::new(r"\s*scale\([-?\d\.]+(?:\s*,?\s*[-?\d\.]+)?\)\s*").unwrap();

      // Replace the "translate(...)" part with an empty string
      let new_transform_value = re.replace_all(transform_value, "").trim().to_string();

      // If the transform is now empty (i.e., there were only translate transforms), remove the attribute
      if new_transform_value.is_empty() {
        element.attributes.remove("transform");
      } else {
        // Otherwise, update the transform attribute with the modified value
        *transform_value = new_transform_value;
      }
    }
  }

  fn apply_circle_translation(element: &mut Element, dx: f32, dy: f32) {
    let cx = element.attributes.get("cx");
    let cy = element.attributes.get("cy");

    if cx.is_none() || cy.is_none()  {
      return;
    }

    let cx = cx.unwrap().parse::<f32>().unwrap();
    let cy = cy.unwrap().parse::<f32>().unwrap();

    element.attributes.insert("cx".to_string(), (cx + dx).to_string());
    element.attributes.insert("cy".to_string(), (cy + dy).to_string());
  }

  fn apply_circle_scale(element: &mut Element, scale_x: f32, scale_y: f32) {
    let cx = element.attributes.get("cx");
    let cy = element.attributes.get("cy");
    let r = element.attributes.get("r");

    if cx.is_none() || cy.is_none() || r.is_none() {
      return;
    }

    let cx = cx.unwrap().parse::<f32>().unwrap();
    let cy = cy.unwrap().parse::<f32>().unwrap();
    let r = r.unwrap().parse::<f32>().unwrap();

    element.attributes.insert("cx".to_string(), (cx * scale_x).to_string());
    element.attributes.insert("cy".to_string(), (cy * scale_y).to_string());

    if scale_x != scale_y {
      element.name = "ellipse".to_string();
      element.attributes.insert("rx".to_string(), (r * scale_x).to_string());
      element.attributes.insert("ry".to_string(), (r * scale_y).to_string());
      element.attributes.remove("r");
    }
    else {
      element.attributes.insert("r".to_string(), (r * scale_x).to_string());
    }
  }
}

impl SingleElementPluginTrait for ApplyTransformsPlugin {
  fn process(&self, element: &mut Element) -> Result<Element, Box<dyn Error>> {
    let transform = element.attributes.get("transform");

    if transform.is_none() {
      return Ok(element.clone());
    }

    let transform_origin = element.attributes.get("transform-origin");

    if !transform_origin.is_none() {
      return Ok(element.clone()); 
    }

    let transforms_list = TransformList::new(transform.unwrap());

    if element.name == "path" {
      if let Some(path_data) = element.attributes.get("d") {
        let mut path = Path::new(path_data);
        let mut element_clone = element.clone();

        for transform in transforms_list.transforms {
          if transform.transform_type == TransformType::Translate {
            if let Some(dx) = transform.get_x() {
              Self::apply_translation(&mut path, dx, transform.get_y().unwrap_or(0.0));
              Self::remove_translate_from_transform(&mut element_clone);
            }
            continue;
          }

          if transform.transform_type == TransformType::Scale {
            if let Some(dx) = transform.get_x() {
              Self::apply_scale(&mut path, dx, transform.get_y().unwrap());
              Self::remove_scale_from_transform(&mut element_clone);
            }
          }
        }

        let transformed_path = path.to_string();
        element_clone.attributes.insert("d".to_string(), transformed_path);

        return Ok(element_clone);
      }
    }

    if element.name == "circle" {
      let mut element_clone = element.clone();

      for transform in transforms_list.transforms {
        if transform.transform_type == TransformType::Translate {
          if let Some(dx) = transform.get_x() {
            Self::apply_circle_translation(&mut element_clone, dx, transform.get_y().unwrap_or(0.0));
            Self::remove_translate_from_transform(&mut element_clone);
          }
          continue;
        }

        if transform.transform_type == TransformType::Scale {
          if let Some(dx) = transform.get_x() {
            Self::apply_circle_scale(&mut element_clone, dx, transform.get_y().unwrap());
            Self::remove_scale_from_transform(&mut element_clone);
          }
        }
      }

      return Ok(element_clone);
    }

    Ok(element.clone())
  }
}

