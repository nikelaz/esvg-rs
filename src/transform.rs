use regex::Regex;
use std::collections::HashMap;

#[derive(PartialEq, Clone, Debug)]
pub enum TransformType {
    Translate,
    Scale,
}

impl TransformType {
    fn from_string(command: &str) -> Option<Self> {
        let mut map = HashMap::new();
        map.insert("translate", TransformType::Translate);
        map.insert("scale", TransformType::Scale);

        map.get(command).cloned()
    }

    fn to_string(&self) -> &'static str {
        match self {
            TransformType::Translate => "translate",
            TransformType::Scale => "scale",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Transform {
    pub transform_type: TransformType,
    pub params: Vec<f32>,
}

impl Transform {
    pub fn get_x(&self) -> Option<f32> {
        if let Some(x) = self.params.get(0) {
            Some(*x)
        } else {
            None
        }
    }

    pub fn get_y(&self) -> Option<f32> {
        if let Some(y) = self.params.get(1) {
            Some(*y)
        } else if self.transform_type == TransformType::Scale {
            self.get_x()
        } else if self.transform_type == TransformType::Translate {
            Some(0.0)
        } else {
            None
        }
    }
}

fn parse_transforms(raw_list: &str) -> Vec<Transform> {
    let re = Regex::new(r"(\w+)\(([^)]+)\)").unwrap();
    let mut transforms = Vec::new();

    for cap in re.captures_iter(raw_list) {
        let raw_transform_type = &cap[1];
        let transform_type = TransformType::from_string(raw_transform_type);

        if transform_type.is_none() {
            continue;
        }

        let params = cap[2]
            .split_whitespace() // Split by spaces
            .flat_map(|s| s.split(',')) // Also split by commas
            .filter_map(|s| s.parse::<f32>().ok()) // Parse to f32
            .collect();

        transforms.push(Transform {
            transform_type: transform_type.unwrap(),
            params,
        });
    }

    // The vector needs to be reversed becasuse that is the order in which the SVG applies
    // transformations
    transforms.reverse();

    transforms
}

#[derive(Clone, Debug)]
pub struct TransformList {
    pub raw_list: String,
    pub transforms: Vec<Transform>,
}

impl TransformList {
    pub fn new(raw_list: &str) -> Self {
        TransformList {
            raw_list: raw_list.to_string().clone(),
            transforms: parse_transforms(raw_list),
        }
    }
}
