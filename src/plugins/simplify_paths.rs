// Plugin
//
// Name: Simplify Paths
// Author: Nikola Lazarov
//
// Description:
// Optimizes SVG path data by performing structural simplifications:
// - Convert curves to lines when control points are collinear (straight curves)
// - Convert L to H/V line shorthands when appropriate
// - Remove zero-length segments (l 0 0, h 0, v 0, etc.)
// - Convert final line-to-start into closepath (Z)
// - Collapse consecutive same-type commands (h 10 h 20 -> h 30)
// - Convert cubic curves to smooth shorthands (C -> S) when applicable
// - Convert quadratic curves to smooth shorthands (Q -> T) when applicable

use crate::path::{Path, PathCommand, PathCommandType};
use crate::plugin::SingleElementPluginTrait;
use std::error::Error;
use xmltree::Element;

/// Tolerance for floating-point comparisons
const EPSILON: f32 = 1e-4;

pub struct SimplifyPathsPlugin {}

/// Checks if a value is effectively zero
fn is_zero(v: f32) -> bool {
    v.abs() < EPSILON
}

/// Checks if two values are approximately equal
fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < EPSILON
}

/// Computes the perpendicular distance from point (px, py) to the line from (x1, y1) to (x2, y2).
/// Used to detect if a curve's control points lie on the straight line from start to end.
fn point_to_line_distance(px: f32, py: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let line_len_sq = dx * dx + dy * dy;

    if line_len_sq < EPSILON * EPSILON {
        // Start and end are the same point — distance is just point-to-point
        let dpx = px - x1;
        let dpy = py - y1;
        return (dpx * dpx + dpy * dpy).sqrt();
    }

    // Perpendicular distance = |cross product| / |line length|
    ((dy * px - dx * py + x2 * y1 - y2 * x1).abs()) / line_len_sq.sqrt()
}

/// Resolves the absolute endpoint of a command given the current cursor position.
/// Returns (end_x, end_y) after this command executes.
fn get_absolute_endpoint(cmd: &PathCommand, cursor_x: f32, cursor_y: f32) -> (f32, f32) {
    let v = &cmd.values;
    match cmd.command_type {
        PathCommandType::MoveTo | PathCommandType::LineTo => {
            if v.len() >= 2 {
                (v[0], v[1])
            } else {
                (cursor_x, cursor_y)
            }
        }
        PathCommandType::MoveToRelative | PathCommandType::LineToRelative => {
            if v.len() >= 2 {
                (cursor_x + v[0], cursor_y + v[1])
            } else {
                (cursor_x, cursor_y)
            }
        }
        PathCommandType::HorizontalLine => {
            if v.len() >= 1 {
                (v[0], cursor_y)
            } else {
                (cursor_x, cursor_y)
            }
        }
        PathCommandType::HorizontalLineRelative => {
            if v.len() >= 1 {
                (cursor_x + v[0], cursor_y)
            } else {
                (cursor_x, cursor_y)
            }
        }
        PathCommandType::VerticalLine => {
            if v.len() >= 1 {
                (cursor_x, v[0])
            } else {
                (cursor_x, cursor_y)
            }
        }
        PathCommandType::VerticalLineRelative => {
            if v.len() >= 1 {
                (cursor_x, cursor_y + v[0])
            } else {
                (cursor_x, cursor_y)
            }
        }
        PathCommandType::CubicBezierCurve => {
            // C x1 y1 x2 y2 x y
            if v.len() >= 6 {
                (v[4], v[5])
            } else {
                (cursor_x, cursor_y)
            }
        }
        PathCommandType::CubicBezierCurveRelative => {
            if v.len() >= 6 {
                (cursor_x + v[4], cursor_y + v[5])
            } else {
                (cursor_x, cursor_y)
            }
        }
        PathCommandType::AdditionalBezierCurve => {
            // S x2 y2 x y
            if v.len() >= 4 {
                (v[2], v[3])
            } else {
                (cursor_x, cursor_y)
            }
        }
        PathCommandType::AdditionalBezierCurveRelative => {
            if v.len() >= 4 {
                (cursor_x + v[2], cursor_y + v[3])
            } else {
                (cursor_x, cursor_y)
            }
        }
        PathCommandType::QuadraticBezierCurve => {
            // Q x1 y1 x y
            if v.len() >= 4 {
                (v[2], v[3])
            } else {
                (cursor_x, cursor_y)
            }
        }
        PathCommandType::QuadraticBezierCurveRelative => {
            if v.len() >= 4 {
                (cursor_x + v[2], cursor_y + v[3])
            } else {
                (cursor_x, cursor_y)
            }
        }
        PathCommandType::AdditionalQuadraticBezierCurve => {
            // T x y
            if v.len() >= 2 {
                (v[0], v[1])
            } else {
                (cursor_x, cursor_y)
            }
        }
        PathCommandType::AdditionalQuadraticBezierCurveRelative => {
            if v.len() >= 2 {
                (cursor_x + v[0], cursor_y + v[1])
            } else {
                (cursor_x, cursor_y)
            }
        }
        PathCommandType::Arc => {
            // A rx ry x-rotation large-arc sweep x y
            if v.len() >= 7 {
                (v[5], v[6])
            } else {
                (cursor_x, cursor_y)
            }
        }
        PathCommandType::ArcRelative => {
            if v.len() >= 7 {
                (cursor_x + v[5], cursor_y + v[6])
            } else {
                (cursor_x, cursor_y)
            }
        }
        PathCommandType::Close | PathCommandType::CloseAlternate => {
            // Close returns to subpath start — handled externally
            (cursor_x, cursor_y)
        }
    }
}

/// Checks if a cubic bezier curve is actually a straight line.
/// A cubic C x1 y1 x2 y2 x y is straight when both control points
/// (x1,y1) and (x2,y2) lie on the line from (start) to (x,y).
fn is_cubic_straight(start_x: f32, start_y: f32, values: &[f32], is_relative: bool) -> bool {
    if values.len() < 6 {
        return false;
    }

    let (cp1x, cp1y, cp2x, cp2y, end_x, end_y) = if is_relative {
        (
            start_x + values[0],
            start_y + values[1],
            start_x + values[2],
            start_y + values[3],
            start_x + values[4],
            start_y + values[5],
        )
    } else {
        (
            values[0], values[1], values[2], values[3], values[4], values[5],
        )
    };

    let d1 = point_to_line_distance(cp1x, cp1y, start_x, start_y, end_x, end_y);
    let d2 = point_to_line_distance(cp2x, cp2y, start_x, start_y, end_x, end_y);

    d1 < EPSILON && d2 < EPSILON
}

/// Checks if a quadratic bezier curve is actually a straight line.
/// Q x1 y1 x y is straight when the control point (x1,y1) lies on
/// the line from (start) to (x,y).
fn is_quadratic_straight(start_x: f32, start_y: f32, values: &[f32], is_relative: bool) -> bool {
    if values.len() < 4 {
        return false;
    }

    let (cp_x, cp_y, end_x, end_y) = if is_relative {
        (
            start_x + values[0],
            start_y + values[1],
            start_x + values[2],
            start_y + values[3],
        )
    } else {
        (values[0], values[1], values[2], values[3])
    };

    let d = point_to_line_distance(cp_x, cp_y, start_x, start_y, end_x, end_y);
    d < EPSILON
}

/// Checks if an arc is effectively a straight line (zero radii or zero-length).
fn is_arc_straight(values: &[f32], is_relative: bool) -> bool {
    if values.len() < 7 {
        return false;
    }

    let rx = values[0];
    let ry = values[1];
    let dx = if is_relative { values[5] } else { values[5] }; // endpoint x
    let dy = if is_relative { values[6] } else { values[6] }; // endpoint y

    // Zero radii means it's a straight line
    if is_zero(rx) || is_zero(ry) {
        return true;
    }

    // Zero-length arc (endpoint == start in relative terms)
    if is_relative && is_zero(dx) && is_zero(dy) {
        return true;
    }

    false
}

impl SimplifyPathsPlugin {
    /// 1. Straight curves: Convert C/c, Q/q to L/l when control points are collinear.
    ///    Also convert arcs to lines when radii are zero.
    fn straight_curves(&self, commands: &mut Vec<PathCommand>, cursor_positions: &[(f32, f32)]) {
        let len = commands.len();
        for i in 0..len {
            let (sx, sy) = if i > 0 {
                cursor_positions[i - 1]
            } else {
                (0.0, 0.0)
            };

            // Never convert a curve to a line when the next command is a smooth
            // continuation (S/s or T/t): those commands derive their implicit first
            // control point from the preceding curve, so removing the curve changes
            // their shape.
            let next_continues_smooth = i + 1 < len
                && matches!(
                    commands[i + 1].command_type,
                    PathCommandType::AdditionalBezierCurve
                        | PathCommandType::AdditionalBezierCurveRelative
                        | PathCommandType::AdditionalQuadraticBezierCurve
                        | PathCommandType::AdditionalQuadraticBezierCurveRelative
                );
            if next_continues_smooth {
                continue;
            }

            match commands[i].command_type {
                PathCommandType::CubicBezierCurve => {
                    if is_cubic_straight(sx, sy, &commands[i].values, false) {
                        let end = [commands[i].values[4], commands[i].values[5]];
                        commands[i].command_type = PathCommandType::LineTo;
                        commands[i].values = end.to_vec();
                    }
                }
                PathCommandType::CubicBezierCurveRelative => {
                    if is_cubic_straight(sx, sy, &commands[i].values, true) {
                        let end = [commands[i].values[4], commands[i].values[5]];
                        commands[i].command_type = PathCommandType::LineToRelative;
                        commands[i].values = end.to_vec();
                    }
                }
                PathCommandType::QuadraticBezierCurve => {
                    if is_quadratic_straight(sx, sy, &commands[i].values, false) {
                        let end = [commands[i].values[2], commands[i].values[3]];
                        commands[i].command_type = PathCommandType::LineTo;
                        commands[i].values = end.to_vec();
                    }
                }
                PathCommandType::QuadraticBezierCurveRelative => {
                    if is_quadratic_straight(sx, sy, &commands[i].values, true) {
                        let end = [commands[i].values[2], commands[i].values[3]];
                        commands[i].command_type = PathCommandType::LineToRelative;
                        commands[i].values = end.to_vec();
                    }
                }
                PathCommandType::Arc => {
                    if is_arc_straight(&commands[i].values, false) {
                        let end = [commands[i].values[5], commands[i].values[6]];
                        commands[i].command_type = PathCommandType::LineTo;
                        commands[i].values = end.to_vec();
                    }
                }
                PathCommandType::ArcRelative => {
                    if is_arc_straight(&commands[i].values, true) {
                        let end = [commands[i].values[5], commands[i].values[6]];
                        commands[i].command_type = PathCommandType::LineToRelative;
                        commands[i].values = end.to_vec();
                    }
                }
                _ => {}
            }
        }
    }

    /// 2. Line shorthands: Convert L/l to H/h or V/v when one axis doesn't change.
    fn line_shorthands(&self, commands: &mut Vec<PathCommand>, cursor_positions: &[(f32, f32)]) {
        for (i, cmd) in commands.iter_mut().enumerate() {
            let (sx, sy) = if i > 0 {
                cursor_positions[i - 1]
            } else {
                (0.0, 0.0)
            };

            match cmd.command_type {
                PathCommandType::LineTo => {
                    let end_x = cmd.values[0];
                    let end_y = cmd.values[1];
                    if approx_eq(end_y, sy) {
                        // Horizontal line
                        cmd.command_type = PathCommandType::HorizontalLine;
                        cmd.values = vec![end_x];
                    } else if approx_eq(end_x, sx) {
                        // Vertical line
                        cmd.command_type = PathCommandType::VerticalLine;
                        cmd.values = vec![end_y];
                    }
                }
                PathCommandType::LineToRelative => {
                    let dx = cmd.values[0];
                    let dy = cmd.values[1];
                    if is_zero(dy) {
                        cmd.command_type = PathCommandType::HorizontalLineRelative;
                        cmd.values = vec![dx];
                    } else if is_zero(dx) {
                        cmd.command_type = PathCommandType::VerticalLineRelative;
                        cmd.values = vec![dy];
                    }
                }
                _ => {}
            }
        }
    }

    /// 3. Remove useless: Remove zero-length segments that don't move the cursor.
    fn remove_useless(&self, commands: &mut Vec<PathCommand>) {
        commands.retain(|cmd| {
            match cmd.command_type {
                PathCommandType::LineToRelative => {
                    // l 0 0 — does nothing
                    !(cmd.values.len() >= 2 && is_zero(cmd.values[0]) && is_zero(cmd.values[1]))
                }
                PathCommandType::HorizontalLineRelative => {
                    // h 0 — does nothing
                    !(cmd.values.len() >= 1 && is_zero(cmd.values[0]))
                }
                PathCommandType::VerticalLineRelative => {
                    // v 0 — does nothing
                    !(cmd.values.len() >= 1 && is_zero(cmd.values[0]))
                }
                PathCommandType::CubicBezierCurveRelative => {
                    // c 0 0 0 0 0 0 — does nothing
                    cmd.values.len() < 6 || !cmd.values.iter().all(|v| is_zero(*v))
                }
                PathCommandType::QuadraticBezierCurveRelative => {
                    // q 0 0 0 0 — does nothing
                    cmd.values.len() < 4 || !cmd.values.iter().all(|v| is_zero(*v))
                }
                PathCommandType::AdditionalBezierCurveRelative => {
                    // s 0 0 0 0 — does nothing
                    cmd.values.len() < 4 || !cmd.values.iter().all(|v| is_zero(*v))
                }
                PathCommandType::AdditionalQuadraticBezierCurveRelative => {
                    // t 0 0 — does nothing
                    cmd.values.len() < 2 || !cmd.values.iter().all(|v| is_zero(*v))
                }
                PathCommandType::ArcRelative => {
                    // a with zero endpoint delta — does nothing
                    if cmd.values.len() >= 7 {
                        !(is_zero(cmd.values[5]) && is_zero(cmd.values[6]))
                    } else {
                        true
                    }
                }
                // Also remove absolute zero-length segments (L to same point is handled
                // differently — we'd need cursor tracking, so we skip absolute for safety)
                _ => true,
            }
        });
    }

    /// 4. Convert to Z: If the last drawing command before a Z (or at end of path)
    ///    is a line back to the subpath start, replace it with Z.
    fn convert_to_z(&self, commands: &mut Vec<PathCommand>) {
        if commands.len() < 2 {
            return;
        }

        // Track subpath starts
        let mut subpath_start_x: f32 = 0.0;
        let mut subpath_start_y: f32 = 0.0;
        let mut cursor_x: f32 = 0.0;
        let mut cursor_y: f32 = 0.0;
        let mut indices_to_replace: Vec<usize> = Vec::new();

        for i in 0..commands.len() {
            let cmd = &commands[i];

            match cmd.command_type {
                PathCommandType::MoveTo => {
                    subpath_start_x = cmd.values[0];
                    subpath_start_y = cmd.values[1];
                    cursor_x = subpath_start_x;
                    cursor_y = subpath_start_y;
                }
                PathCommandType::MoveToRelative => {
                    subpath_start_x = cursor_x + cmd.values[0];
                    subpath_start_y = cursor_y + cmd.values[1];
                    cursor_x = subpath_start_x;
                    cursor_y = subpath_start_y;
                }
                PathCommandType::Close | PathCommandType::CloseAlternate => {
                    cursor_x = subpath_start_x;
                    cursor_y = subpath_start_y;
                }
                _ => {
                    let (end_x, end_y) = get_absolute_endpoint(cmd, cursor_x, cursor_y);

                    // Check if this line goes back to subpath start and is followed by Z or is the last command
                    let is_line = matches!(
                        cmd.command_type,
                        PathCommandType::LineTo
                            | PathCommandType::LineToRelative
                            | PathCommandType::HorizontalLine
                            | PathCommandType::HorizontalLineRelative
                            | PathCommandType::VerticalLine
                            | PathCommandType::VerticalLineRelative
                    );

                    if is_line
                        && approx_eq(end_x, subpath_start_x)
                        && approx_eq(end_y, subpath_start_y)
                    {
                        // Check if next command is Z or this is the last command
                        let next_is_z = if i + 1 < commands.len() {
                            matches!(
                                commands[i + 1].command_type,
                                PathCommandType::Close | PathCommandType::CloseAlternate
                            )
                        } else {
                            false
                        };

                        if next_is_z {
                            // Mark this line for removal (Z already follows)
                            indices_to_replace.push(i);
                        }
                    }

                    cursor_x = end_x;
                    cursor_y = end_y;
                }
            }
        }

        // Remove the redundant line commands (iterate in reverse to preserve indices)
        for &idx in indices_to_replace.iter().rev() {
            commands.remove(idx);
        }
    }

    /// 5. Collapse repeated: Merge consecutive commands of the same type.
    ///    h 10 h 20 -> h 30, v 5 v 10 -> v 15, etc.
    fn collapse_repeated(&self, commands: &mut Vec<PathCommand>) {
        if commands.len() < 2 {
            return;
        }

        let mut result: Vec<PathCommand> = Vec::with_capacity(commands.len());

        for cmd in commands.drain(..) {
            if let Some(last) = result.last_mut() {
                let merged = match (&last.command_type, &cmd.command_type) {
                    // h + h -> h (sum)
                    (
                        PathCommandType::HorizontalLineRelative,
                        PathCommandType::HorizontalLineRelative,
                    ) => {
                        if last.values.len() >= 1 && cmd.values.len() >= 1 {
                            last.values[0] += cmd.values[0];
                            true
                        } else {
                            false
                        }
                    }
                    // v + v -> v (sum)
                    (
                        PathCommandType::VerticalLineRelative,
                        PathCommandType::VerticalLineRelative,
                    ) => {
                        if last.values.len() >= 1 && cmd.values.len() >= 1 {
                            last.values[0] += cmd.values[0];
                            true
                        } else {
                            false
                        }
                    }
                    // H + H -> H (last wins, since absolute)
                    (PathCommandType::HorizontalLine, PathCommandType::HorizontalLine) => {
                        if cmd.values.len() >= 1 {
                            last.values = cmd.values.clone();
                            true
                        } else {
                            false
                        }
                    }
                    // V + V -> V (last wins, since absolute)
                    (PathCommandType::VerticalLine, PathCommandType::VerticalLine) => {
                        if cmd.values.len() >= 1 {
                            last.values = cmd.values.clone();
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                };

                if !merged {
                    result.push(cmd);
                }
            } else {
                result.push(cmd);
            }
        }

        *commands = result;
    }

    /// 6. Curve smooth shorthands: Convert C->S and Q->T when the first control point
    ///    is the reflection of the previous curve's last control point.
    fn curve_smooth_shorthands(
        &self,
        commands: &mut Vec<PathCommand>,
        cursor_positions: &[(f32, f32)],
    ) {
        // Track cubic and quadratic smooth chains independently.
        // Mixing them (e.g. Q after C) must not carry over the cubic cp2 as quadratic
        // context: C->S is a cubic chain, Q->T is a separate quadratic chain.
        let mut prev_cubic_cp2: Option<(f32, f32)> = None;
        let mut prev_quadratic_cp1: Option<(f32, f32)> = None;

        for (i, cmd) in commands.iter_mut().enumerate() {
            let (sx, sy) = if i > 0 {
                cursor_positions[i - 1]
            } else {
                (0.0, 0.0)
            };

            match cmd.command_type {
                PathCommandType::CubicBezierCurve => {
                    // C x1 y1 x2 y2 x y
                    // Can convert to S x2 y2 x y if x1,y1 is reflection of prev cubic cp2
                    if let Some((prev_x2, prev_y2)) = prev_cubic_cp2 {
                        let reflected_x = 2.0 * sx - prev_x2;
                        let reflected_y = 2.0 * sy - prev_y2;
                        if approx_eq(cmd.values[0], reflected_x)
                            && approx_eq(cmd.values[1], reflected_y)
                        {
                            let x2 = cmd.values[2];
                            let y2 = cmd.values[3];
                            let x = cmd.values[4];
                            let y = cmd.values[5];
                            prev_cubic_cp2 = Some((x2, y2));
                            prev_quadratic_cp1 = None;
                            cmd.command_type = PathCommandType::AdditionalBezierCurve;
                            cmd.values = vec![x2, y2, x, y];
                            continue;
                        }
                    }
                    prev_cubic_cp2 = Some((cmd.values[2], cmd.values[3]));
                    prev_quadratic_cp1 = None;
                }
                PathCommandType::CubicBezierCurveRelative => {
                    // c dx1 dy1 dx2 dy2 dx dy
                    if let Some((prev_x2, prev_y2)) = prev_cubic_cp2 {
                        let reflected_x = 2.0 * sx - prev_x2;
                        let reflected_y = 2.0 * sy - prev_y2;
                        let abs_cp1_x = sx + cmd.values[0];
                        let abs_cp1_y = sy + cmd.values[1];
                        if approx_eq(abs_cp1_x, reflected_x) && approx_eq(abs_cp1_y, reflected_y) {
                            let dx2 = cmd.values[2];
                            let dy2 = cmd.values[3];
                            let dx = cmd.values[4];
                            let dy = cmd.values[5];
                            prev_cubic_cp2 = Some((sx + dx2, sy + dy2));
                            prev_quadratic_cp1 = None;
                            cmd.command_type = PathCommandType::AdditionalBezierCurveRelative;
                            cmd.values = vec![dx2, dy2, dx, dy];
                            continue;
                        }
                    }
                    prev_cubic_cp2 = Some((sx + cmd.values[2], sy + cmd.values[3]));
                    prev_quadratic_cp1 = None;
                }
                PathCommandType::AdditionalBezierCurve => {
                    // S already — keep tracking cubic cp2
                    prev_cubic_cp2 = Some((cmd.values[0], cmd.values[1]));
                    prev_quadratic_cp1 = None;
                }
                PathCommandType::AdditionalBezierCurveRelative => {
                    prev_cubic_cp2 = Some((sx + cmd.values[0], sy + cmd.values[1]));
                    prev_quadratic_cp1 = None;
                }
                PathCommandType::QuadraticBezierCurve => {
                    // Q x1 y1 x y -> T x y if x1,y1 is reflection of prev quadratic cp1
                    if let Some((prev_cpx, prev_cpy)) = prev_quadratic_cp1 {
                        let reflected_x = 2.0 * sx - prev_cpx;
                        let reflected_y = 2.0 * sy - prev_cpy;
                        if approx_eq(cmd.values[0], reflected_x)
                            && approx_eq(cmd.values[1], reflected_y)
                        {
                            let x = cmd.values[2];
                            let y = cmd.values[3];
                            prev_quadratic_cp1 = Some((cmd.values[0], cmd.values[1]));
                            prev_cubic_cp2 = None;
                            cmd.command_type = PathCommandType::AdditionalQuadraticBezierCurve;
                            cmd.values = vec![x, y];
                            continue;
                        }
                    }
                    prev_quadratic_cp1 = Some((cmd.values[0], cmd.values[1]));
                    prev_cubic_cp2 = None;
                }
                PathCommandType::QuadraticBezierCurveRelative => {
                    if let Some((prev_cpx, prev_cpy)) = prev_quadratic_cp1 {
                        let reflected_x = 2.0 * sx - prev_cpx;
                        let reflected_y = 2.0 * sy - prev_cpy;
                        let abs_cp_x = sx + cmd.values[0];
                        let abs_cp_y = sy + cmd.values[1];
                        if approx_eq(abs_cp_x, reflected_x) && approx_eq(abs_cp_y, reflected_y) {
                            let dx = cmd.values[2];
                            let dy = cmd.values[3];
                            prev_quadratic_cp1 = Some((abs_cp_x, abs_cp_y));
                            prev_cubic_cp2 = None;
                            cmd.command_type =
                                PathCommandType::AdditionalQuadraticBezierCurveRelative;
                            cmd.values = vec![dx, dy];
                            continue;
                        }
                    }
                    prev_quadratic_cp1 = Some((sx + cmd.values[0], sy + cmd.values[1]));
                    prev_cubic_cp2 = None;
                }
                _ => {
                    // Any non-curve command resets both smooth chains
                    prev_cubic_cp2 = None;
                    prev_quadratic_cp1 = None;
                }
            }
        }
    }

    /// Compute cursor positions after each command (absolute coordinates).
    /// Returns a vector of (x, y) positions — one per command.
    fn compute_cursor_positions(&self, commands: &[PathCommand]) -> Vec<(f32, f32)> {
        let mut positions = Vec::with_capacity(commands.len());
        let mut cursor_x: f32 = 0.0;
        let mut cursor_y: f32 = 0.0;
        let mut subpath_start_x: f32 = 0.0;
        let mut subpath_start_y: f32 = 0.0;

        for cmd in commands {
            match cmd.command_type {
                PathCommandType::MoveTo => {
                    if cmd.values.len() >= 2 {
                        cursor_x = cmd.values[0];
                        cursor_y = cmd.values[1];
                        subpath_start_x = cursor_x;
                        subpath_start_y = cursor_y;
                    }
                }
                PathCommandType::MoveToRelative => {
                    if cmd.values.len() >= 2 {
                        cursor_x += cmd.values[0];
                        cursor_y += cmd.values[1];
                        subpath_start_x = cursor_x;
                        subpath_start_y = cursor_y;
                    }
                }
                PathCommandType::Close | PathCommandType::CloseAlternate => {
                    cursor_x = subpath_start_x;
                    cursor_y = subpath_start_y;
                }
                _ => {
                    let (end_x, end_y) = get_absolute_endpoint(cmd, cursor_x, cursor_y);
                    cursor_x = end_x;
                    cursor_y = end_y;
                }
            }

            positions.push((cursor_x, cursor_y));
        }

        positions
    }

    /// Returns the minimum number of values required for a command type.
    fn min_values_for(cmd_type: &PathCommandType) -> usize {
        match cmd_type {
            PathCommandType::Close | PathCommandType::CloseAlternate => 0,
            PathCommandType::HorizontalLine
            | PathCommandType::HorizontalLineRelative
            | PathCommandType::VerticalLine
            | PathCommandType::VerticalLineRelative => 1,
            PathCommandType::MoveTo
            | PathCommandType::MoveToRelative
            | PathCommandType::LineTo
            | PathCommandType::LineToRelative
            | PathCommandType::AdditionalQuadraticBezierCurve
            | PathCommandType::AdditionalQuadraticBezierCurveRelative => 2,
            PathCommandType::AdditionalBezierCurve
            | PathCommandType::AdditionalBezierCurveRelative
            | PathCommandType::QuadraticBezierCurve
            | PathCommandType::QuadraticBezierCurveRelative => 4,
            PathCommandType::CubicBezierCurve | PathCommandType::CubicBezierCurveRelative => 6,
            PathCommandType::Arc | PathCommandType::ArcRelative => 7,
        }
    }

    /// Main simplification pipeline
    fn simplify(&self, path_data: &str) -> String {
        let mut path = Path::new(path_data);

        if path.commands.is_empty() {
            return path.to_string();
        }

        // Skip simplification for any path that has commands with insufficient values
        // to avoid index-out-of-bounds panics in the pipeline below.
        for cmd in &path.commands {
            if cmd.values.len() < Self::min_values_for(&cmd.command_type) {
                return path.to_string();
            }
        }

        // Phase 1: Straight curves (needs cursor positions)
        let positions = self.compute_cursor_positions(&path.commands);
        self.straight_curves(&mut path.commands, &positions);

        // Phase 2: Line shorthands (needs updated cursor positions)
        let positions = self.compute_cursor_positions(&path.commands);
        self.line_shorthands(&mut path.commands, &positions);

        // Phase 3: Remove useless zero-length segments
        self.remove_useless(&mut path.commands);

        // Phase 4: Convert to Z (line-to-start before Z)
        self.convert_to_z(&mut path.commands);

        // Phase 5: Collapse repeated same-type commands
        self.collapse_repeated(&mut path.commands);

        // Phase 6: Curve smooth shorthands (C->S, Q->T)
        let positions = self.compute_cursor_positions(&path.commands);
        self.curve_smooth_shorthands(&mut path.commands, &positions);

        path.to_string()
    }
}

impl SingleElementPluginTrait for SimplifyPathsPlugin {
    fn process(&self, element: &mut Element) -> Result<Element, Box<dyn Error>> {
        let mut element_clone = element.clone();

        // Only process elements with a "d" attribute (path data)
        if let Some(d) = element_clone.attributes.get("d").cloned() {
            let simplified = self.simplify(&d);
            element_clone.attributes.insert("d".to_string(), simplified);
        }

        Ok(element_clone)
    }
}
