use crate::plugin::WholeSVGPluginTrait;
use crate::Svg;
use std::collections::HashMap;
/**
* Plugin
*
* Name: Combine Paths
* Author: Nikola Lazarov
*
* Description:
* Combines paths to a single element if they have the same attributes
* and order context
*
* TODO:
* [ ] - Test and refactor
*/
use std::error::Error;
use xmltree::Element;

pub struct CombinePathsPlugin;

impl CombinePathsPlugin {
    fn traverse_paths(element: &mut Element) {
        for child in element.children.iter_mut() {
            if let Some(child_element) = child.as_mut_element() {
                Self::traverse_paths(child_element);
            }
        }

        Self::combine_adjacent_paths(element);
    }

    // TODO: This NEEDS to be refactored and implemented in a more ellegant manner
    fn combine_adjacent_paths(element: &mut Element) {
        let mut new_children = Vec::new();
        let mut i = 0;

        while i < element.children.len() {
            let current_child = &element.children[i];

            if let Some(current_element) = current_child.as_element() {
                if current_element.name == "path" {
                    // Found a path, check if we can combine with following paths
                    let mut combined_d = String::new();
                    let mut last_combined_index = i;

                    // Get the d attribute of the first path
                    if let Some(d_attr) = current_element.attributes.get("d") {
                        combined_d.push_str(d_attr);
                    }

                    // Look for consecutive paths with same attributes
                    for j in (i + 1)..element.children.len() {
                        if let Some(next_element) = element.children[j].as_element() {
                            if next_element.name == "path"
                                && Self::have_same_non_d_attributes(current_element, next_element)
                                && !Self::paths_bboxes_overlap(current_element, next_element)
                            {
                                // Combine the d attributes
                                if let Some(next_d) = next_element.attributes.get("d") {
                                    if !combined_d.is_empty() && !next_d.is_empty() {
                                        combined_d.push(' ');
                                    }
                                    combined_d
                                        .push_str(&Self::make_leading_moveto_absolute(next_d));
                                }
                                last_combined_index = j;
                            } else {
                                // Stop at first non-matching path or non-path element
                                break;
                            }
                        } else {
                            // Stop at non-element (text, etc.)
                            break;
                        }
                    }

                    if last_combined_index > i {
                        // We found paths to combine
                        let mut combined_element = current_element.clone();
                        combined_element
                            .attributes
                            .insert("d".to_string(), combined_d);
                        new_children.push(xmltree::XMLNode::Element(combined_element));
                        i = last_combined_index + 1;
                    } else {
                        // No combination, keep original
                        new_children.push(current_child.clone());
                        i += 1;
                    }
                } else {
                    // Not a path element, keep as is
                    new_children.push(current_child.clone());
                    i += 1;
                }
            } else {
                // Not an element (text node, etc.), keep as is
                new_children.push(current_child.clone());
                i += 1;
            }
        }

        element.children = new_children;
    }

    /// Convert a leading lowercase `m` to `M` (absolute moveto) so that when
    /// this path is appended to another path as a subpath, the moveto is not
    /// interpreted as relative to the preceding subpath's current point.
    ///
    /// In SVG, extra coordinate pairs after the first in an `m` command are
    /// implicit **relative** `l` (lineto) commands.  Simply swapping `m`→`M`
    /// changes those implicit linetos from relative to absolute, corrupting the
    /// shape.  This function therefore inserts an explicit `l` separator after
    /// the first coordinate pair so the remaining pairs retain their original
    /// relative-lineto semantics.
    ///
    /// Example:  `m5346 1983-192 132s…`
    ///   → `M5346 1983 l-192 132s…`
    fn make_leading_moveto_absolute(d: &str) -> String {
        let mut chars = d.chars().peekable();
        match chars.next() {
            Some('m') => {
                // Collect everything after the leading 'm' up to (but not
                // including) the next command letter. That span contains the
                // coordinate pairs of the `m` command. We must keep the first
                // pair as part of `M` and turn any subsequent pairs into an
                // explicit `l` command.
                let rest: String = chars.collect();
                let rest = rest.trim_start();

                // Walk through `rest` to find where the first coordinate pair
                // ends. A coordinate pair is two numbers separated by
                // whitespace or a comma. Numbers may start with '-' or '+'.
                // We scan until we've seen two numbers, then check whether
                // what follows is another number (= implicit lineto pair) or a
                // command letter / end-of-string.
                if let Some(after_first_pair) = Self::after_first_coord_pair(rest) {
                    let first_pair = &rest[..after_first_pair];
                    let tail = rest[after_first_pair..].trim_start();

                    if tail.is_empty() || tail.starts_with(|c: char| c.is_alphabetic()) {
                        // No implicit lineto pairs – a simple `m`→`M` swap is safe.
                        format!("M{}{}", first_pair, tail)
                    } else {
                        // There are implicit lineto pairs after the moveto.
                        // Insert an explicit `l` to preserve relative semantics.
                        format!("M{} l{}", first_pair, tail)
                    }
                } else {
                    // Couldn't parse – fall back to a plain letter swap.
                    format!("M{}", rest)
                }
            }
            Some(first) => {
                // Not a lowercase `m` – return unchanged.
                let rest: String = chars.collect();
                format!("{}{}", first, rest)
            }
            None => String::new(),
        }
    }

    /// Return the byte offset in `s` immediately after the first SVG coordinate
    /// pair (two numbers). Returns `None` if fewer than two numbers are found.
    fn after_first_coord_pair(s: &str) -> Option<usize> {
        let bytes = s.as_bytes();
        let len = bytes.len();
        let mut pos = 0;

        for _ in 0..2 {
            // Skip optional whitespace and comma separators.
            while pos < len
                && (bytes[pos] == b' '
                    || bytes[pos] == b'\t'
                    || bytes[pos] == b'\n'
                    || bytes[pos] == b'\r'
                    || bytes[pos] == b',')
            {
                pos += 1;
            }
            if pos >= len {
                return None;
            }
            // Skip optional sign.
            if pos < len && (bytes[pos] == b'-' || bytes[pos] == b'+') {
                pos += 1;
            }
            if pos >= len {
                return None;
            }
            // Must have at least one digit (or a dot for numbers like `.5`).
            if !bytes[pos].is_ascii_digit() && bytes[pos] != b'.' {
                return None;
            }
            // Consume digits.
            while pos < len && bytes[pos].is_ascii_digit() {
                pos += 1;
            }
            // Consume optional fractional part.
            if pos < len && bytes[pos] == b'.' {
                pos += 1;
                while pos < len && bytes[pos].is_ascii_digit() {
                    pos += 1;
                }
            }
            // Consume optional exponent.
            if pos < len && (bytes[pos] == b'e' || bytes[pos] == b'E') {
                pos += 1;
                if pos < len && (bytes[pos] == b'-' || bytes[pos] == b'+') {
                    pos += 1;
                }
                while pos < len && bytes[pos].is_ascii_digit() {
                    pos += 1;
                }
            }
        }

        Some(pos)
    }

    /// Returns true if the axis-aligned bounding boxes of the two paths overlap.
    /// Uses a conservative estimate based only on the coordinate values found in
    /// the `d` attribute — it does not account for curve control points bulging
    /// outside the endpoint bounding box, so it may miss some overlaps, but it
    /// will never incorrectly report a non-overlapping pair as overlapping.
    fn paths_bboxes_overlap(path1: &Element, path2: &Element) -> bool {
        let d1 = match path1.attributes.get("d") {
            Some(d) => d,
            None => return false,
        };
        let d2 = match path2.attributes.get("d") {
            Some(d) => d,
            None => return false,
        };
        let (min1, max1) = match Self::path_bbox(d1) {
            Some(b) => b,
            None => return false,
        };
        let (min2, max2) = match Self::path_bbox(d2) {
            Some(b) => b,
            None => return false,
        };
        // Two AABBs overlap iff they overlap on both axes.
        min1.0 <= max2.0 && max1.0 >= min2.0 && min1.1 <= max2.1 && max1.1 >= min2.1
    }

    /// Compute the axis-aligned bounding box of a path by scanning all numeric
    /// tokens, resolving relative commands to absolute coordinates, and tracking
    /// min/max x and y. Returns `None` if no coordinates could be parsed.
    ///
    /// This is intentionally approximate: it uses endpoint coordinates only
    /// (not Bézier control points), which means curves that bulge outside their
    /// endpoint box are not fully captured. For the purpose of detecting
    /// obviously overlapping vs. clearly disjoint paths this is sufficient.
    fn path_bbox(d: &str) -> Option<((f64, f64), (f64, f64))> {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        // Current pen position for resolving relative commands.
        let mut cx = 0f64;
        let mut cy = 0f64;
        // Start-of-subpath position (for Z).
        let mut sx = 0f64;
        let mut sy = 0f64;
        // Whether this is the very first command in the path (first 'm' = absolute).
        let mut first_cmd = true;

        let mut update = |x: f64, y: f64| {
            if x < min_x {
                min_x = x;
            }
            if x > max_x {
                max_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if y > max_y {
                max_y = y;
            }
        };
        // Do NOT seed with (0,0) – only record actual path points.

        // Tokenise: split into (command_char, [numbers…]) groups.
        let tokens: Vec<(char, Vec<f64>)> = Self::tokenise_path(d);

        for (cmd, nums) in &tokens {
            match cmd {
                'M' => {
                    first_cmd = false;
                    let mut i = 0;
                    while i + 1 < nums.len() {
                        cx = nums[i];
                        cy = nums[i + 1];
                        if i == 0 {
                            sx = cx;
                            sy = cy;
                        }
                        update(cx, cy);
                        i += 2;
                    }
                }
                'm' => {
                    let mut i = 0;
                    while i + 1 < nums.len() {
                        if first_cmd {
                            // The very first command in a standalone path: 'm' is
                            // relative to the origin, i.e. effectively absolute.
                            cx = nums[i];
                            cy = nums[i + 1];
                        } else {
                            cx += nums[i];
                            cy += nums[i + 1];
                        }
                        if i == 0 {
                            sx = cx;
                            sy = cy;
                            first_cmd = false;
                        }
                        update(cx, cy);
                        i += 2;
                    }
                }
                'L' => {
                    let mut i = 0;
                    while i + 1 < nums.len() {
                        cx = nums[i];
                        cy = nums[i + 1];
                        update(cx, cy);
                        i += 2;
                    }
                }
                'l' => {
                    let mut i = 0;
                    while i + 1 < nums.len() {
                        cx += nums[i];
                        cy += nums[i + 1];
                        update(cx, cy);
                        i += 2;
                    }
                }
                'H' => {
                    for &x in nums {
                        cx = x;
                        update(cx, cy);
                    }
                }
                'h' => {
                    for &dx in nums {
                        cx += dx;
                        update(cx, cy);
                    }
                }
                'V' => {
                    for &y in nums {
                        cy = y;
                        update(cx, cy);
                    }
                }
                'v' => {
                    for &dy in nums {
                        cy += dy;
                        update(cx, cy);
                    }
                }
                // Cubic Bézier: 6 numbers per segment (cp1x cp1y cp2x cp2y x y)
                'C' => {
                    let mut i = 0;
                    while i + 5 < nums.len() {
                        cx = nums[i + 4];
                        cy = nums[i + 5];
                        update(cx, cy);
                        i += 6;
                    }
                }
                'c' => {
                    let mut i = 0;
                    while i + 5 < nums.len() {
                        cx += nums[i + 4];
                        cy += nums[i + 5];
                        update(cx, cy);
                        i += 6;
                    }
                }
                // Smooth cubic: 4 numbers (cp2x cp2y x y)
                'S' => {
                    let mut i = 0;
                    while i + 3 < nums.len() {
                        cx = nums[i + 2];
                        cy = nums[i + 3];
                        update(cx, cy);
                        i += 4;
                    }
                }
                's' => {
                    let mut i = 0;
                    while i + 3 < nums.len() {
                        cx += nums[i + 2];
                        cy += nums[i + 3];
                        update(cx, cy);
                        i += 4;
                    }
                }
                // Quadratic Bézier: 4 numbers (cpx cpy x y)
                'Q' => {
                    let mut i = 0;
                    while i + 3 < nums.len() {
                        cx = nums[i + 2];
                        cy = nums[i + 3];
                        update(cx, cy);
                        i += 4;
                    }
                }
                'q' => {
                    let mut i = 0;
                    while i + 3 < nums.len() {
                        cx += nums[i + 2];
                        cy += nums[i + 3];
                        update(cx, cy);
                        i += 4;
                    }
                }
                // Smooth quadratic: 2 numbers (x y)
                'T' => {
                    let mut i = 0;
                    while i + 1 < nums.len() {
                        cx = nums[i];
                        cy = nums[i + 1];
                        update(cx, cy);
                        i += 2;
                    }
                }
                't' => {
                    let mut i = 0;
                    while i + 1 < nums.len() {
                        cx += nums[i];
                        cy += nums[i + 1];
                        update(cx, cy);
                        i += 2;
                    }
                }
                // Arc: 7 numbers (rx ry x-rot large-arc sweep x y)
                'A' => {
                    let mut i = 0;
                    while i + 6 < nums.len() {
                        cx = nums[i + 5];
                        cy = nums[i + 6];
                        update(cx, cy);
                        i += 7;
                    }
                }
                'a' => {
                    let mut i = 0;
                    while i + 6 < nums.len() {
                        cx += nums[i + 5];
                        cy += nums[i + 6];
                        update(cx, cy);
                        i += 7;
                    }
                }
                'Z' | 'z' => {
                    cx = sx;
                    cy = sy;
                }
                _ => {}
            }
            first_cmd = false;
        }

        if min_x.is_infinite() {
            None
        } else {
            Some(((min_x, min_y), (max_x, max_y)))
        }
    }

    /// Split an SVG path `d` string into `(command_letter, numbers)` pairs.
    fn tokenise_path(d: &str) -> Vec<(char, Vec<f64>)> {
        let mut result: Vec<(char, Vec<f64>)> = Vec::new();
        let mut current_cmd: Option<char> = None;
        let mut current_nums: Vec<f64> = Vec::new();

        // Insert spaces before command letters so we can split cleanly.
        let mut spaced = String::with_capacity(d.len() * 2);
        for ch in d.chars() {
            if ch.is_alphabetic() {
                spaced.push(' ');
                spaced.push(ch);
                spaced.push(' ');
            } else if ch == '-' || ch == '+' {
                // A sign can start a new number even without whitespace/comma.
                spaced.push(' ');
                spaced.push(ch);
            } else {
                spaced.push(ch);
            }
        }

        for token in spaced.split_whitespace() {
            if token.len() == 1 && token.chars().next().map_or(false, |c| c.is_alphabetic()) {
                // Flush previous command.
                if let Some(cmd) = current_cmd.take() {
                    result.push((cmd, std::mem::take(&mut current_nums)));
                }
                current_cmd = token.chars().next();
            } else if let Ok(n) = token.parse::<f64>() {
                current_nums.push(n);
            }
        }
        if let Some(cmd) = current_cmd {
            result.push((cmd, current_nums));
        }
        result
    }

    fn have_same_non_d_attributes(path1: &Element, path2: &Element) -> bool {
        // Create copies of attributes without the 'd' attribute
        let mut attrs1: HashMap<String, String> = path1.attributes.clone();
        let mut attrs2: HashMap<String, String> = path2.attributes.clone();

        attrs1.remove("d");
        attrs2.remove("d");

        attrs1 == attrs2
    }
}

impl WholeSVGPluginTrait for CombinePathsPlugin {
    fn process(&self, svg: &mut Svg) -> Result<Svg, Box<dyn Error>> {
        Self::traverse_paths(&mut svg.root);

        Ok(svg.clone())
    }
}
