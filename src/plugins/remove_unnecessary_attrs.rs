use std::error::Error;
use crate::plugin::SingleElementPluginTrait;
use xmltree::Element;

pub struct RemoveUnnecessaryAttrsPlugin {}

const UNNECESSARY_ATTR_PATTERNS: [&str; 8] = [
  "lang",
  "desc",
  "title",
  "version",
  "xml:lang",
  "baseProfile",
  "contentStyleType",
  "contentScriptType",
];

impl SingleElementPluginTrait for RemoveUnnecessaryAttrsPlugin {
  fn process(&self, element: &Element) -> Result<Element, Box<dyn Error>> {
    let mut element_clone = element.clone();

    element_clone.attributes.retain(|key, _| {
      !UNNECESSARY_ATTR_PATTERNS.contains(&key.as_str()) && !key.starts_with("data-")
    });

    Ok(element_clone)
  }
}
