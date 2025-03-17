use std::error::Error;
use crate::Svg;
use crate::plugin::WholeSVGPluginTrait;

pub struct RemoveAttrsWholePlugin {}

impl WholeSVGPluginTrait for RemoveAttrsWholePlugin {
  fn process(&self, svg: &Svg) -> Result<Svg, Box<dyn Error>> {
    let mut svg_clone = svg.clone();

    for node in &mut svg_clone.root.children {
      if let Some(element) = node.as_mut_element() {
      element.attributes.clear();
      }
    }

    Ok(svg_clone)
  }
}
