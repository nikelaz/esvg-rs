use xmltree::{Element, ParseError};
use std::error::Error;

#[derive(Debug)]
pub struct Svg {
  pub root: Element,
}

impl Svg {
  pub fn from_str(svg_str: &str) -> Result<Self, ParseError> {
    let root = Element::parse(svg_str.as_bytes())?;
    Ok(Svg { root })
  }

  pub fn to_string(&self) -> Result<String, Box<dyn Error>> {
    let mut buffer = Vec::new();
    self.root.write(&mut buffer)?;
    let res = String::from_utf8(buffer)?;
    Ok(res)
  }

  pub fn clone(&self) -> Self {
    Svg {
      root: self.root.clone(),
    }
  }
}
