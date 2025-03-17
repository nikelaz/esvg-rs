use std::error::Error;
use crate::plugin::SingleElementPluginTrait;
use xmltree::Element;

pub struct ShapeToPathPlugin {}

fn convertRectToPath(element: &Element) {
  let x = element.attributes.get("x");
  let y = element.attributes.get("y");
  let width = element.attributes.get("width");
  let height = element.attributes.get("height");
  let rx = element.attributes.get("rx");
  let ry = element.attributes.get("ry");

  if x == None || y == None || width == None || height == None || rx != None || ry != None {
    return;
  }
  else {
    println!("x is {}", x.unwrap());
  }
}

impl SingleElementPluginTrait for ShapeToPathPlugin {
  fn process(&self, element: &Element) -> Result<Element, Box<dyn Error>> {
    let mut element_clone = element.clone();

    if element_clone.name == "rect" {
      convertRectToPath(&element_clone);
    }
    // if let Some(name) = element_clone.name.as_str() {
    //   match name {
    //     "rect" => {
    //       if let (Some(x), Some(y), Some(width), Some(height)) = (
    //         element_clone.attributes.get("x"),
    //         element_clone.attributes.get("y"),
    //         element_clone.attributes.get("width"),
    //         element_clone.attributes.get("height"),
    //       ) {
    //         let path_data = format!(
    //           "M{} {} H{} V{} H{} Z",
    //           x,
    //           y,
    //           x.parse::<f32>().unwrap() + width.parse::<f32>().unwrap(),
    //           y.parse::<f32>().unwrap() + height.parse::<f32>().unwrap(),
    //           x
    //         );
    //         element_clone.name = "path".to_string();
    //         element_clone.attributes.clear();
    //         element_clone.attributes.insert("d".to_string(), path_data);
    //       }
    //     }
    //     "circle" => {
    //       if let (Some(cx), Some(cy), Some(r)) = (
    //         element_clone.attributes.get("cx"),
    //         element_clone.attributes.get("cy"),
    //         element_clone.attributes.get("r"),
    //       ) {
    //         let path_data = format!(
    //           "M{} {} m-{},0 a{},{} 0 1,0 {},0 a{},{} 0 1,0 -{},0",
    //           cx,
    //           cy,
    //           r,
    //           r,
    //           r,
    //           2.0 * r.parse::<f32>().unwrap(),
    //           r,
    //           r,
    //           2.0 * r.parse::<f32>().unwrap()
    //         );
    //         element_clone.name = "path".to_string();
    //         element_clone.attributes.clear();
    //         element_clone.attributes.insert("d".to_string(), path_data);
    //       }
    //     }
    //     "ellipse" => {
    //       if let (Some(cx), Some(cy), Some(rx), Some(ry)) = (
    //         element_clone.attributes.get("cx"),
    //         element_clone.attributes.get("cy"),
    //         element_clone.attributes.get("rx"),
    //         element_clone.attributes.get("ry"),
    //       ) {
    //         let path_data = format!(
    //           "M{} {} m-{},0 a{},{} 0 1,0 {},0 a{},{} 0 1,0 -{},0",
    //           cx,
    //           cy,
    //           rx,
    //           rx,
    //           ry,
    //           2.0 * rx.parse::<f32>().unwrap(),
    //           rx,
    //           ry,
    //           2.0 * rx.parse::<f32>().unwrap()
    //         );
    //         element_clone.name = "path".to_string();
    //         element_clone.attributes.clear();
    //         element_clone.attributes.insert("d".to_string(), path_data);
    //       }
    //     }
    //     "line" => {
    //       if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
    //         element_clone.attributes.get("x1"),
    //         element_clone.attributes.get("y1"),
    //         element_clone.attributes.get("x2"),
    //         element_clone.attributes.get("y2"),
    //       ) {
    //         let path_data = format!("M{} {} L{} {}", x1, y1, x2, y2);
    //         element_clone.name = "path".to_string();
    //         element_clone.attributes.clear();
    //         element_clone.attributes.insert("d".to_string(), path_data);
    //       }
    //     }
    //     "polyline" | "polygon" => {
    //       if let Some(points) = element_clone.attributes.get("points") {
    //         let mut path_data = String::new();
    //         let points: Vec<&str> = points.split_whitespace().collect();
    //         if let Some(first_point) = points.first() {
    //           path_data.push_str(&format!("M{}", first_point));
    //           for point in points.iter().skip(1) {
    //             path_data.push_str(&format!(" L{}", point));
    //           }
    //           if name == "polygon" {
    //             path_data.push_str(" Z");
    //           }
    //         }
    //         element_clone.name = "path".to_string();
    //         element_clone.attributes.clear();
    //         element_clone.attributes.insert("d".to_string(), path_data);
    //       }
    //     }
    //     _ => {}
    //   }
    // }

    Ok(element_clone)
  }
}
