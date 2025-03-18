use crate::svg::Svg;
use std::error::Error;
use crate::plugin::SingleElementPluginTrait;
use crate::plugin::WholeSVGPluginTrait;
use crate::helpers::element_to_string;

pub struct Arbiter {
  single_element_plugins: Vec<Box<dyn SingleElementPluginTrait>>,
  whole_svg_plugins: Vec<Box<dyn WholeSVGPluginTrait>>,
}

impl Arbiter {
  pub fn new() -> Self {
    Arbiter {
      single_element_plugins: Vec::new(),
      whole_svg_plugins: Vec::new(),
    }
  }

  pub fn add_single_element_plugin(&mut self, plugin: Box<dyn SingleElementPluginTrait>) {
    self.single_element_plugins.push(plugin);
  }

  pub fn add_whole_svg_plugin(&mut self, plugin: Box<dyn WholeSVGPluginTrait>) {
    self.whole_svg_plugins.push(plugin);
  }

  pub fn process(&self, svg: &Svg) -> Result<Svg, Box<dyn Error>> {
    let mut svg_clone = svg.clone();

    for whole_svg_plugin in &self.whole_svg_plugins {
      let svg_output = whole_svg_plugin.process(&svg_clone)?;
      if svg_output.to_string().unwrap().len() < svg_clone.to_string().unwrap().len() {
        svg_clone = svg_output;
      }
    }

    for single_plugin in &self.single_element_plugins {
      let root_plugin_result = single_plugin.process(&svg_clone.root)?;

      if element_to_string(&root_plugin_result).unwrap().len() < element_to_string(&svg_clone.root).unwrap().len() {
        svg_clone.root = root_plugin_result; 
      }

      for node in &mut svg_clone.root.children {
        let element = node.as_mut_element().unwrap();
        let element_plugin_result = single_plugin.process(&element)?;
        if element_to_string(&element_plugin_result).unwrap().len() < element_to_string(&element).unwrap().len() {      
          *element = element_plugin_result;
        }
      }
    }

    Ok(svg_clone)
  }
}
