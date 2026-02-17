// Plugin
//
// Name: Shape to Path
// Author: Nikola Lazarov
//
// Description:
// Converts simple shapes like <rect>, <line> to <path>
// Does not try to convert <circle> and <ellipse> as that is never more efficient

use crate::plugin::SingleElementPluginTrait;
use std::error::Error;
use xmltree::Element;

pub struct ShapeToPathPlugin;

impl ShapeToPathPlugin {
    fn rect_to_path(element: &mut Element) {
        let width = element.attributes.get("width");
        let height = element.attributes.get("height");
        let rx = element.attributes.get("rx");
        let ry = element.attributes.get("ry");

        if width == None || height == None || rx != None || ry != None {
            return;
        }

        let default_zero = "0".to_string();
        let x_val = element.attributes.get("x").unwrap_or(&default_zero);
        let y_val = element.attributes.get("y").unwrap_or(&default_zero);
        let width_val = width.unwrap();
        let height_val = height.unwrap();

        let path_data = format!(
            "M{} {} H{} V{} H{} Z",
            x_val,
            y_val,
            x_val.parse::<f32>().unwrap() + width_val.parse::<f32>().unwrap(),
            y_val.parse::<f32>().unwrap() + height_val.parse::<f32>().unwrap(),
            x_val
        );

        element.name = "path".to_string();
        element.attributes.remove("x");
        element.attributes.remove("y");
        element.attributes.remove("width");
        element.attributes.remove("height");
        element.attributes.insert("d".to_string(), path_data);
    }

    fn line_to_path(element: &mut Element) {
        let x1 = element.attributes.get("x1");
        let y1 = element.attributes.get("y1");
        let x2 = element.attributes.get("x2");
        let y2 = element.attributes.get("y2");

        if x1 == None || y1 == None || x2 == None || y2 == None {
            return;
        }

        let path_data = format!(
            "M{} {} L{} {}",
            x1.unwrap(),
            y1.unwrap(),
            x2.unwrap(),
            y2.unwrap()
        );

        element.name = "path".to_string();
        element.attributes.remove("x1");
        element.attributes.remove("y1");
        element.attributes.remove("x2");
        element.attributes.remove("y2");
        element.attributes.insert("d".to_string(), path_data);
    }

    fn polygon_to_path(element: &mut Element, close: bool) {
        let points = element.attributes.get("points");

        if points == None {
            return;
        }

        let coords = points
            .unwrap()
            .split_whitespace()
            .map(|p| p.replace(',', " "))
            .collect::<Vec<_>>();

        if coords.is_empty() {
            return;
        }

        let mut path_data = format!("M {}", coords[0]); // Move to first point

        for point in &coords[1..] {
            path_data.push_str(&format!(" L {}", point)); // Line to subsequent points
        }

        if close {
            path_data.push_str(" Z");
        }

        element.name = "path".to_string();
        element.attributes.remove("points");
        element.attributes.insert("d".to_string(), path_data);
    }
}

impl SingleElementPluginTrait for ShapeToPathPlugin {
    fn process(&self, element: &mut Element) -> Result<Element, Box<dyn Error>> {
        let mut element_clone = element.clone();

        match element_clone.name.as_str() {
            "rect" => Self::rect_to_path(&mut element_clone),
            "line" => Self::line_to_path(&mut element_clone),
            "polyline" => Self::polygon_to_path(&mut element_clone, false),
            "polygon" => Self::polygon_to_path(&mut element_clone, true),
            _ => {}
        }

        Ok(element_clone)
    }
}
