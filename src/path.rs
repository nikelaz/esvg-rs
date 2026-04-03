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

#[derive(PartialEq, Clone, Debug)]
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
    CloseAlternate,
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

fn is_command_char(c: char) -> bool {
    let command_chars: HashSet<char> = [
        'M', 'm', 'L', 'l', 'H', 'h', 'V', 'v', 'C', 'c', 'S', 's', 'Q', 'q', 'T', 't', 'A', 'a',
        'Z', 'z',
    ]
    .iter()
    .cloned()
    .collect();
    command_chars.contains(&c)
}

#[derive(PartialEq, Clone, Debug)]
pub struct PathCommand {
    pub command_type: PathCommandType,
    pub values: Vec<f32>,
}

/// Tokenize SVG path value strings according to the SVG path data grammar.
///
/// SVG path data allows implicit separators beyond whitespace and commas:
/// - A minus sign `-` starts a new token (unless it follows `e` or `E` for exponents)
/// - A second decimal point `.` in a token starts a new token (e.g. `0.5.3` → `0.5`, `.3`)
///
/// Examples:
///   "-3.943-83.327"  → [-3.943, -83.327]
///   "1-2"            → [1.0, -2.0]
///   "0.5.3"          → [0.5, 0.3]
///   "1e-2"           → [0.01]  (exponent, not a split)
fn tokenize_path_values(s: &str) -> Vec<f32> {
    let mut tokens: Vec<f32> = Vec::new();
    let mut current = String::new();
    let mut has_dot = false;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        match c {
            // Whitespace or comma: flush current token
            ' ' | '\t' | '\n' | '\r' | ',' => {
                if !current.is_empty() {
                    if let Ok(v) = current.parse::<f32>() {
                        tokens.push(v);
                    }
                    current.clear();
                    has_dot = false;
                }
            }
            // Minus sign: starts a new token unless it follows 'e'/'E' (exponent)
            '-' => {
                let prev = current.chars().last();
                let after_exponent = matches!(prev, Some('e') | Some('E'));
                if !current.is_empty() && !after_exponent {
                    if let Ok(v) = current.parse::<f32>() {
                        tokens.push(v);
                    }
                    current.clear();
                    has_dot = false;
                }
                current.push(c);
            }
            // Decimal point: if the current token already has one, start a new token
            '.' => {
                if has_dot {
                    if let Ok(v) = current.parse::<f32>() {
                        tokens.push(v);
                    }
                    current.clear();
                }
                has_dot = true;
                current.push(c);
            }
            // Plus sign: starts a new token unless it follows 'e'/'E'
            '+' => {
                let prev = current.chars().last();
                let after_exponent = matches!(prev, Some('e') | Some('E'));
                if !current.is_empty() && !after_exponent {
                    if let Ok(v) = current.parse::<f32>() {
                        tokens.push(v);
                    }
                    current.clear();
                    has_dot = false;
                }
                // Don't push '+' into the token; it's an implicit positive sign
            }
            // Any other character (digit, 'e', 'E'): append to current token
            _ => {
                // 'e'/'E' in a number resets the dot context (exponent part can't have a dot)
                if c == 'e' || c == 'E' {
                    has_dot = false;
                }
                current.push(c);
            }
        }

        i += 1;
    }

    // Flush the last token
    if !current.is_empty() {
        if let Ok(v) = current.parse::<f32>() {
            tokens.push(v);
        }
    }

    tokens
}

impl PathCommand {
    pub fn new(command_type: PathCommandType) -> Self {
        PathCommand {
            command_type: command_type.clone(),
            values: Vec::new(),
        }
    }

    pub fn add_values(&mut self, value_string: &str) {
        self.values = tokenize_path_values(value_string);
    }

    pub fn to_string(&self) -> String {
        format!(
            "{} {}",
            self.command_type.to_string(),
            self.values
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

#[derive(Debug)]
pub struct Path {
    pub commands: Vec<PathCommand>,
}

impl Path {
    pub fn new(raw_path: &str) -> Self {
        let mut new_instance = Path {
            commands: Vec::new(),
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
            } else if last_command.is_some() {
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
        self.commands
            .iter()
            .map(|comm| comm.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_simple_values() {
        assert_eq!(tokenize_path_values("10 20"), vec![10.0, 20.0]);
        assert_eq!(tokenize_path_values("10,20"), vec![10.0, 20.0]);
        assert_eq!(tokenize_path_values("10, 20"), vec![10.0, 20.0]);
    }

    #[test]
    fn tokenize_implicit_minus_separator() {
        // SVG path data: minus sign acts as separator
        assert_eq!(tokenize_path_values("-3.943-83.327"), vec![-3.943, -83.327]);
        assert_eq!(tokenize_path_values("1-2"), vec![1.0, -2.0]);
        assert_eq!(tokenize_path_values("10.5-20.3"), vec![10.5, -20.3]);
    }

    #[test]
    fn tokenize_implicit_dot_separator() {
        // Second decimal point starts a new token
        assert_eq!(tokenize_path_values("0.5.3"), vec![0.5, 0.3]);
        assert_eq!(tokenize_path_values(".5.3"), vec![0.5, 0.3]);
    }

    #[test]
    fn tokenize_exponent_notation() {
        // 'e'/'E' in numbers must not be split on the minus that follows
        let result = tokenize_path_values("1e-2");
        assert_eq!(result.len(), 1);
        assert!((result[0] - 0.01).abs() < 1e-6);

        let result = tokenize_path_values("1.5E+3");
        assert_eq!(result.len(), 1);
        assert!((result[0] - 1500.0).abs() < 0.1);
    }

    #[test]
    fn parse_path_with_implicit_separators() {
        // The failing case: l-3.943-83.327 should produce a command with 2 values
        let path = Path::new("Ml-3.943-83.327");
        assert_eq!(path.commands.len(), 2);
        assert_eq!(path.commands[1].values.len(), 2);
        assert!((path.commands[1].values[0] - (-3.943)).abs() < 1e-4);
        assert!((path.commands[1].values[1] - (-83.327)).abs() < 1e-4);
    }

    #[test]
    fn parse_real_world_path() {
        // From the SVG that was breaking: M632.948,630.458,610.365,628.5l-3.943-83.327h32.2Z
        // Note: SVG M with 4 values means M to first point then implicit L to second
        // For our parser purposes, we just care that the l command gets its 2 values
        let path = Path::new("M632.948,630.458l-3.943-83.327h32.2Z");
        // M: 2 values, l: 2 values, h: 1 value, Z: 0 values
        assert_eq!(path.commands.len(), 4);
        let l_cmd = &path.commands[1];
        assert_eq!(l_cmd.command_type, PathCommandType::LineToRelative);
        assert_eq!(l_cmd.values.len(), 2);
        assert!((l_cmd.values[0] - (-3.943)).abs() < 1e-4);
        assert!((l_cmd.values[1] - (-83.327)).abs() < 1e-4);
    }

    #[test]
    fn parse_arc_with_flags() {
        // Arc: a rx ry x-rotation large-arc-flag sweep-flag x y
        // Flags are 0 or 1, often written without separators: a7.592,7.592,0,0,0,...
        let path = Path::new("Ma7.592,7.592,0,0,0,3.185,-5.335");
        assert_eq!(path.commands.len(), 2);
        let a_cmd = &path.commands[1];
        assert_eq!(a_cmd.command_type, PathCommandType::ArcRelative);
        assert_eq!(a_cmd.values.len(), 7);
    }
}
