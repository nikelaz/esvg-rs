use std::error::Error;
use xmltree::Element;
use crate::svg::Svg;

pub trait WholeSVGPluginTrait {
  fn process(&self, svg: &Svg) -> Result<Svg, Box<dyn Error>>;
}

pub trait SingleElementPluginTrait {
  fn process(&self, element: &Element) -> Result<Element, Box<dyn Error>>;
}
