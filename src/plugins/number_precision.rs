// Plugin
//
// Name: Number Precision
// Author: Nikola Lazarov
//
// Description:
// Rounds floating-point numbers in SVG attributes to a configurable precision.
// Reduces file size by trimming unnecessary decimal places from coordinates,
// dimensions, and other numeric values.

use crate::path::Path;
use crate::plugin::SingleElementPluginTrait;
use std::error::Error;
use xmltree::Element;

const NUMERIC_ATTRIBUTES: [&str; 26] = [
    "x",
    "y",
    "cx",
    "cy",
    "r",
    "rx",
    "ry",
    "width",
    "height",
    "x1",
    "y1",
    "x2",
    "y2",
    "dx",
    "dy",
    "opacity",
    "fill-opacity",
    "stroke-opacity",
    "stroke-width",
    "stroke-miterlimit",
    "stroke-dashoffset",
    "font-size",
    "letter-spacing",
    "word-spacing",
    "baseline-shift",
    "offset",
];

pub struct NumberPrecisionPlugin {
    pub precision: u32,
}

impl NumberPrecisionPlugin {
    fn round_value(value: f32, precision: u32) -> String {
        let formatted = format!("{:.prec$}", value, prec = precision as usize);
        // Trim trailing zeros after decimal point
        if formatted.contains('.') {
            let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
            trimmed.to_string()
        } else {
            formatted
        }
    }

    fn round_string_value(value_str: &str, precision: u32) -> String {
        match value_str.parse::<f32>() {
            Ok(v) => Self::round_value(v, precision),
            Err(_) => value_str.to_string(),
        }
    }

    fn round_f32(value: f32, precision: u32) -> f32 {
        let factor = 10f32.powi(precision as i32);
        (value * factor).round() / factor
    }

    fn round_path_data(&self, d: &str) -> String {
        let mut path = Path::new(d);
        for command in path.commands.iter_mut() {
            for val in command.values.iter_mut() {
                *val = Self::round_f32(*val, self.precision);
            }
        }
        path.to_string()
    }

    fn round_multi_value_attr(&self, value_str: &str) -> String {
        // Handle comma and/or space separated values (e.g. viewBox, stroke-dasharray)
        let parts: Vec<&str> = value_str
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .collect();

        let mut result = Vec::new();
        for part in &parts {
            result.push(Self::round_string_value(part, self.precision));
        }

        result.join(" ")
    }
}

impl SingleElementPluginTrait for NumberPrecisionPlugin {
    fn process(&self, element: &mut Element) -> Result<Element, Box<dyn Error>> {
        let mut element_clone = element.clone();

        // Round path data
        if let Some(d) = element_clone.attributes.get("d").cloned() {
            let rounded = self.round_path_data(&d);
            element_clone.attributes.insert("d".to_string(), rounded);
        }

        // Round numeric attributes
        for attr_name in NUMERIC_ATTRIBUTES.iter() {
            if let Some(value) = element_clone.attributes.get(*attr_name).cloned() {
                let rounded = Self::round_string_value(&value, self.precision);
                element_clone
                    .attributes
                    .insert(attr_name.to_string(), rounded);
            }
        }

        // Round viewBox values
        if let Some(viewbox) = element_clone.attributes.get("viewBox").cloned() {
            let rounded = self.round_multi_value_attr(&viewbox);
            element_clone
                .attributes
                .insert("viewBox".to_string(), rounded);
        }

        // Round stroke-dasharray values
        if let Some(dasharray) = element_clone.attributes.get("stroke-dasharray").cloned() {
            let rounded = self.round_multi_value_attr(&dasharray);
            element_clone
                .attributes
                .insert("stroke-dasharray".to_string(), rounded);
        }

        Ok(element_clone)
    }
}
