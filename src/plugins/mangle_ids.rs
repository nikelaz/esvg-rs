use crate::plugin::WholeSVGPluginTrait;
use crate::svg::Svg;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use xmltree::Element;

pub struct MangleIdsPlugin {
    pub prefix: Option<String>,
}

impl MangleIdsPlugin {
    fn generate_id(index: usize, prefix: &Option<String>) -> String {
        let mut name = String::new();
        let mut n = index;
        let alphabet: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
            .chars()
            .collect();
        let base = alphabet.len();

        loop {
            name.push(alphabet[n % base]);
            n /= base;
            if n == 0 {
                break;
            }
            n -= 1; // Adjust for 0-indexed logic in subsequent iterations if needed
        }

        let generated = name.chars().rev().collect::<String>();
        match prefix {
            Some(p) => format!("{}{}", p, generated),
            None => generated,
        }
    }

    fn collect_ids_and_classes(
        element: &Element,
        ids: &mut HashSet<String>,
        classes: &mut HashSet<String>,
    ) {
        if let Some(id) = element.attributes.get("id") {
            ids.insert(id.clone());
        }

        if let Some(class_attr) = element.attributes.get("class") {
            for class_name in class_attr.split_whitespace() {
                classes.insert(class_name.to_string());
            }
        }

        for child in &element.children {
            if let Some(child_element) = child.as_element() {
                Self::collect_ids_and_classes(child_element, ids, classes);
            }
        }
    }

    fn update_element(
        element: &mut Element,
        id_map: &HashMap<String, String>,
        class_map: &HashMap<String, String>,
    ) {
        // Update ID
        if let Some(id) = element.attributes.get_mut("id") {
            if let Some(new_id) = id_map.get(id.as_str()) {
                *id = new_id.clone();
            }
        }

        // Update Class
        if let Some(class_attr) = element.attributes.get_mut("class") {
            let new_classes: Vec<String> = class_attr
                .split_whitespace()
                .map(|c| class_map.get(c).unwrap_or(&c.to_string()).clone())
                .collect();
            *class_attr = new_classes.join(" ");
        }

        // Update References in attributes (e.g. url(#old_id), href="#old_id")
        // Basic regex approach for attributes that might contain references
        // Common attributes: clip-path, mask, marker-*, fill, stroke, href, xlink:href, filter, style
        // Note: This is a simplified approach and might not cover all edge cases or complex CSS selectors in style attributes.
        // For style attributes with full CSS parsing, we would need a full CSS parser (like css-structs logic, but applied here).
        // Since we don't have a mutable CSS parser handy for just this, we'll do simple string replacement for now.

        let mut attrs_to_update: Vec<(String, String)> = Vec::new();

        for (key, value) in &element.attributes {
            let mut new_value = value.clone();
            let mut changed = false;

            // Simple reference: url(#id)
            for (old_id, new_id) in id_map {
                if new_value.contains(old_id) {
                    // Check for url(#id)
                    let url_pattern = format!("url(#{0})", old_id);
                    if new_value.contains(&url_pattern) {
                        new_value = new_value.replace(&url_pattern, &format!("url(#{0})", new_id));
                        changed = true;
                    }
                    // Check for href="#id"
                    let href_pattern = format!("#{0}", old_id);
                    if (key == "href" || key.ends_with(":href")) && new_value == href_pattern {
                        new_value = href_pattern.replace(old_id, new_id);
                        changed = true;
                    }
                }
            }
            // Class references in style? .old_class
            // This is risky with simple replace. We assume styles are handled by CSS parser plugin or we need a robust Regex.
            // For now, let's focus on XML attributes.

            if changed {
                attrs_to_update.push((key.clone(), new_value));
            }
        }

        for (key, val) in attrs_to_update {
            element.attributes.insert(key, val);
        }

        for child in &mut element.children {
            if let Some(child_element) = child.as_mut_element() {
                Self::update_element(child_element, id_map, class_map);
            }
        }
    }

    // Helper to update style content if possible (if it's a <style> tag)
    fn update_style_element(
        element: &mut Element,
        id_map: &HashMap<String, String>,
        class_map: &HashMap<String, String>,
    ) {
        if element.name == "style" {
            if let Some(text) = element.get_text() {
                let mut new_text = text.to_string();
                // Naive replacement for classes and IDs in CSS
                // To do this safely, we need a CSS tokenizer.
                // Given the constraints and dependencies, we might try a regex that matches .classname and #idname
                // But be careful not to match hex colors.

                for (old_class, new_class) in class_map {
                    // match .old_class but not if it follows something else? CSS is complex.
                    // Simple approach: replace exact word match if it starts with .
                    // This is very heuristic.
                    let re = Regex::new(&format!("\\.{}", regex::escape(old_class))).unwrap();
                    new_text = re
                        .replace_all(&new_text, format!(".{}", new_class))
                        .to_string();
                }

                for (old_id, new_id) in id_map {
                    let re = Regex::new(&format!("#{}", regex::escape(old_id))).unwrap();
                    new_text = re
                        .replace_all(&new_text, format!("#{}", new_id))
                        .to_string();
                }

                // Clear children and set new text
                element.children.clear();
                element.children.push(xmltree::XMLNode::Text(new_text));
            }
        }

        for child in &mut element.children {
            if let Some(child_element) = child.as_mut_element() {
                Self::update_style_element(child_element, id_map, class_map);
            }
        }
    }
}

impl WholeSVGPluginTrait for MangleIdsPlugin {
    fn process(&self, svg: &mut Svg) -> Result<Svg, Box<dyn Error>> {
        let mut ids = HashSet::new();
        let mut classes = HashSet::new();

        Self::collect_ids_and_classes(&svg.root, &mut ids, &mut classes);

        let mut id_map = HashMap::new();
        let mut class_map = HashMap::new();

        let mut sorted_ids: Vec<_> = ids.into_iter().collect();
        sorted_ids.sort(); // Deterministic order
        for (i, id) in sorted_ids.iter().enumerate() {
            id_map.insert(id.clone(), Self::generate_id(i, &self.prefix));
        }

        let mut sorted_classes: Vec<_> = classes.into_iter().collect();
        sorted_classes.sort();
        for (i, class) in sorted_classes.iter().enumerate() {
            class_map.insert(class.clone(), Self::generate_id(i, &self.prefix));
        }

        let mut svg_clone = svg.clone();

        // 2 passes: 1 for attributes, 1 for style text (optional/risky)
        Self::update_element(&mut svg_clone.root, &id_map, &class_map);
        Self::update_style_element(&mut svg_clone.root, &id_map, &class_map);

        Ok(svg_clone)
    }
}
