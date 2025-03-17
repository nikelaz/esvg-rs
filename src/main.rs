use xmltree::{Element, ParseError};

#[derive(Debug)]
pub struct Svg {
  pub root: Element,
}

impl Svg {
  pub fn from_str(svg_str: &str) -> Result<Self, ParseError> {
    let root = Element::parse(svg_str.as_bytes())?;
    Ok(Svg { root })
  }

  pub fn to_string(&self) -> String {
    let mut buffer = Vec::new();
    self.root.write(&mut buffer).unwrap();
    String::from_utf8(buffer).unwrap() 
  }
}

fn main() {
  let svg_str = "<svg version=\"1.2\"><path d=\"test\"></path><path d=\"test\"/><rect width=\"10\"></rect></svg>";
  let svg_el = Svg::from_str(svg_str).expect("Failed to parse the SVG");
  println!("Svg parsed successfully.");

  for node in &svg_el.root.children {
    let node_name = &node.as_element().unwrap().name;

    println!("Node: {}", node_name);
  }

  println!("Svg output:");
  println!("{:?}", svg_el.to_string());
}
