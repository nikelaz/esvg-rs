use std::fmt;
use xmltree::{Element, EmitterConfig};

#[derive(Debug, Clone)]
pub struct Svg {
    pub root: Element,
}

impl Svg {
    pub fn from_string(svg_string: &String) -> Result<Self, String> {
        let root = Element::parse(svg_string.as_bytes()).map_err(|e| e.to_string())?;

        Ok(Svg { root })
    }

    pub fn new(root: &Element) -> Self {
        Self { root: root.clone() }
    }

    pub fn to_pretty_string(&self) -> Result<String, String> {
        let mut buffer = Vec::new();
        let config = EmitterConfig::new()
            .perform_indent(true)
            .write_document_declaration(false);

        self.root
            .write_with_config(&mut buffer, config)
            .map_err(|e| e.to_string())?;

        String::from_utf8(buffer).map_err(|e| e.to_string())
    }
}

impl fmt::Display for Svg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buffer = Vec::new();

        if let Err(e) = self.root.write(&mut buffer) {
            return write!(f, "<invalid SVG: {}>", e);
        }

        match String::from_utf8(buffer) {
            Ok(s) => f.write_str(&s),
            Err(e) => write!(f, "<invalid UTF-8: {}>", e),
        }
    }
}
