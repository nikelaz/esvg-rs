mod svg;
mod plugin;
mod plugins;
mod arbiter;

use crate::svg::Svg;
use crate::plugins::RemoveUnnecessaryAttrsPlugin;
use crate::plugins::ShapeToPathPlugin;
use crate::arbiter::Arbiter;

fn main() {
  let svg_str = "<svg version=\"1.2\"><path d=\"test\" data-test=\"name\"></path><path d=\"test\"/><rect x=\"150\" width=\"10\"></rect></svg>";
  let svg_input = Svg::from_str(svg_str).expect("Failed to parse the SVG");

  // println!("SVG Input: {:?}", svg_input);
  println!("Input Size: {}", svg_input.to_string().unwrap().len());

  println!("Creating a new arbiter");

  let mut arbiter = Arbiter::new();
  arbiter.add_single_element_plugin(Box::new(RemoveUnnecessaryAttrsPlugin {}));
  arbiter.add_single_element_plugin(Box::new(ShapeToPathPlugin {}));
  let svg_output = arbiter.process(&svg_input).expect("Failed to process the SVG");

  // println!("SVG Output: {:?}", svg_output);
  println!("Output Size: {}", svg_output.to_string().unwrap().len());

  println!("Svg output:");
  println!("{:?}", svg_output.to_string());
}
