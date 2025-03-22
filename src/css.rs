use std::vec;
use regex::Regex;

#[derive(Debug)]
struct CssProperty {
  name: String,
  value: String
}

#[derive(Debug)]
struct CssRule {
  selector: String,
  properties: Vec<CssProperty>
}

pub struct CSSParser {}

impl CSSParser {
  pub fn from_string(input_str: &str) -> Vec<(String, String)> {
    let re = Regex::new(r"(?s)([^{}]+)\s*\{([^}]*)\}").unwrap();
    let mut results = Vec::new();

    for cap in re.captures_iter(input_str) {
      let selector = cap[1].trim().to_string();
      let body = cap[2].trim().to_string();
      results.push((selector, body));
    }

    results
  } 
}
