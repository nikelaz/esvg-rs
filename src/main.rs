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
  let svg_str = "<svg version=\"1.2\"><path d=\"32423e1asfwe8192\"></path><path d=\"12e8fh92fh02dj3209\"/><rect width=\"10\"></rect></svg>";
  let svg_el = Svg::from_str(svg_str).unwrap();
  println!("Log: svg parsed successfully.");

  for node in &svg_el.root.children {
    println!("Node: {}", node.as_element().unwrap().name);
  }

  println!("Log: svg output:");
  let svg_output = svg_el.to_string();
  println!("{:?}", svg_output);
}

