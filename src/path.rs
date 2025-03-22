/*
 * Path Data Representation
 * Author: Nikola Lazarov
 * 
 * Todo:
 * [ ] - memory management needs to be reworked, it can definitely be more efficient
 * [ ] - change PathCommandCurve data structure
 */

use std::collections::HashMap;
use std::collections::HashSet;

#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Debug)]
pub enum PathCommandType {
    MoveTo,
    MoveToRelative,
    LineTo,
    LineToRelative,
    HorizontalLine,
    HorizontalLineRelative,
    VerticalLine,
    VerticalLineRelative,
    CubicBezierCurve,
    CubicBezierCurveRelative,
    AdditionalBezierCurve,
    AdditionalBezierCurveRelative,
    QuadraticBezierCurve,
    QuadraticBezierCurveRelative,
    AdditionalQuadraticBezierCurve,
    AdditionalQuadraticBezierCurveRelative,
    Arc,
    ArcRelative,
    Close,
    CloseAlternate
}

impl PathCommandType {
    fn from_string(command: &str) -> Option<Self> {
        let mut map = HashMap::new();
        map.insert("M", PathCommandType::MoveTo);
        map.insert("m", PathCommandType::MoveToRelative);
        map.insert("L", PathCommandType::LineTo);
        map.insert("l", PathCommandType::LineToRelative);
        map.insert("H", PathCommandType::HorizontalLine);
        map.insert("h", PathCommandType::HorizontalLineRelative);
        map.insert("V", PathCommandType::VerticalLine);
        map.insert("v", PathCommandType::VerticalLineRelative);
        map.insert("C", PathCommandType::CubicBezierCurve);
        map.insert("c", PathCommandType::CubicBezierCurveRelative);
        map.insert("S", PathCommandType::AdditionalBezierCurve);
        map.insert("s", PathCommandType::AdditionalBezierCurveRelative);
        map.insert("Q", PathCommandType::QuadraticBezierCurve);
        map.insert("q", PathCommandType::QuadraticBezierCurveRelative);
        map.insert("T", PathCommandType::AdditionalQuadraticBezierCurve);
        map.insert("t", PathCommandType::AdditionalQuadraticBezierCurveRelative);
        map.insert("A", PathCommandType::Arc);
        map.insert("a", PathCommandType::ArcRelative);
        map.insert("Z", PathCommandType::Close);
        map.insert("z", PathCommandType::CloseAlternate);

        map.get(command).cloned()                               
    }

    fn to_string(&self) -> &'static str {
        match self {
            PathCommandType::MoveTo => "M",
            PathCommandType::MoveToRelative => "m",
            PathCommandType::LineTo => "L",
            PathCommandType::LineToRelative => "l",
            PathCommandType::HorizontalLine => "H",
            PathCommandType::HorizontalLineRelative => "h",
            PathCommandType::VerticalLine => "V",
            PathCommandType::VerticalLineRelative => "v",
            PathCommandType::CubicBezierCurve => "C",
            PathCommandType::CubicBezierCurveRelative => "c",
            PathCommandType::AdditionalBezierCurve => "S",
            PathCommandType::AdditionalBezierCurveRelative => "s",
            PathCommandType::QuadraticBezierCurve => "Q",
            PathCommandType::QuadraticBezierCurveRelative => "q",
            PathCommandType::AdditionalQuadraticBezierCurve => "T",
            PathCommandType::AdditionalQuadraticBezierCurveRelative => "t",
            PathCommandType::Arc => "A",
            PathCommandType::ArcRelative => "a",
            PathCommandType::Close => "Z",
            PathCommandType::CloseAlternate => "z",
        }
    }
}

fn is_command_char (c: char) -> bool {
    let command_chars: HashSet<char> = ['M', 'm', 'L', 'l', 'H', 'h', 'V', 'v', 
                                       'C', 'c', 'S', 's', 'Q', 'q', 'T', 't', 
                                       'A', 'a', 'Z', 'z'].iter().cloned().collect();
    command_chars.contains(&c)
}

#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Debug)]
pub struct PathCommand {
    pub command_type: PathCommandType,
    pub values: Vec<f32>
}

impl PathCommand {
    pub fn new(command_type: PathCommandType) -> Self {
        PathCommand {
            command_type: command_type.clone(),
            values: Vec::new()
        }
    }

    pub fn add_values(&mut self, value_string: &str) {
        self.values = value_string
            .trim()
            .replace(",", "")
            .split_whitespace()
            .filter_map(|x| x.parse::<f32>().ok())
            .collect();
    }

    pub fn to_string(&self) -> String {
        format!(
            "{} {}",
            self.command_type.to_string(),
            self.values.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(" ")
        )
    }
}

#[derive(Debug)]
pub struct Path {
    pub commands: Vec<PathCommand>
}

impl Path {
    pub fn new(raw_path: &str) -> Self {
        let mut new_instance = Path {
            commands: Vec::new()
        };
        
        let mut last_command: Option<PathCommand> = None;
        let mut last_values: String = String::new();

        let mut iter = raw_path.chars().peekable();

        while let Some(ch) = iter.next() {
            if is_command_char(ch) {
                if let Some(ref mut command) = last_command {
                    if !last_values.is_empty() {
                        command.add_values(last_values.as_str());
                    }
                    new_instance.commands.push(command.clone());
                }

                last_command = Some(PathCommand::new(
                    PathCommandType::from_string(ch.to_string().as_str()).unwrap(),
                ));
                last_values.clear();
            }
            else if last_command.is_some() {
                last_values.push(ch);
            }

            if iter.peek().is_none() {
                if let Some(ref mut command) = last_command {
                    if !last_values.is_empty() {
                        command.add_values(last_values.as_str());
                    }
                    new_instance.commands.push(command.clone());
                }
            }
        }

        new_instance
    }

    pub fn to_string(&self) -> String {
        self.commands.iter().map(|comm| comm.to_string()).collect::<Vec<String>>().join(" ")
    }
}
