mod svg;
mod plugin;
mod plugins;
mod arbiter;
mod helpers;
mod path;
mod transform;

use std::{env,  fs, process};
use std::io::Write;
use crate::svg::Svg;
use crate::arbiter::Arbiter;
use crate::plugins::RemoveUnnecessaryAttrsPlugin;
use crate::plugins::ShapeToPathPlugin;
use crate::plugins::ApplyTransformsPlugin;
use crate::plugins::CssToAttributesPlugin;
use crate::plugins::OptimizeColorsPlugin;
use crate::plugins::CombinePathsPlugin;

fn main() {
  let args: Vec<String> = env::args().collect();

  if args.len() < 3 {
    eprintln!("Usage: {} <input_file> <output_file>", args[0]);
    process::exit(1);
  }

  let file_path = &args[1];
  let out_path = &args[2];

  let svg_string = fs::read_to_string(file_path).expect("Filed to read input SVG");
  let svg_input = Svg::from_string(&svg_string).expect("Failed to parse the SVG");

  println!("Input Size: {}", svg_input.to_string().len());

  let mut arbiter = Arbiter::new();

  // Run plugins
  arbiter.add_single_element_plugin(Box::new(RemoveUnnecessaryAttrsPlugin {}));
  arbiter.add_single_element_plugin(Box::new(ShapeToPathPlugin {}));

  arbiter.add_single_element_plugin(Box::new(OptimizeColorsPlugin {}));
  arbiter.add_whole_svg_plugin(Box::new(ApplyTransformsPlugin {}));
  arbiter.add_whole_svg_plugin(Box::new(CssToAttributesPlugin {}));
  arbiter.add_whole_svg_plugin(Box::new(CombinePathsPlugin {}));

  let svg_output = arbiter.process(&svg_input).expect("Failed to process the SVG");

  println!("Output Size: {}", svg_output.to_string().len());

  let mut output_file = fs::File::create(out_path).unwrap();
  let _ = output_file.write_all(svg_output.to_string().as_bytes());
}

