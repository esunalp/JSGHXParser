#[must_use]
pub fn triangulate_grid(u_count: usize, v_count: usize) -> Vec<u32> {
    triangulate_grid_wrapped(u_count, v_count, false, false)
}

#[must_use]
pub fn triangulate_grid_wrapped(
    u_count: usize,
    v_count: usize,
    wrap_u: bool,
    wrap_v: bool,
) -> Vec<u32> {
    let u_count = if wrap_u { u_count.max(3) } else { u_count.max(2) };
    let v_count = if wrap_v { v_count.max(3) } else { v_count.max(2) };

    let quad_u = if wrap_u { u_count } else { u_count - 1 };
    let quad_v = if wrap_v { v_count } else { v_count - 1 };
    let triangle_count = quad_u * quad_v * 2;
    let mut indices = Vec::with_capacity(triangle_count * 3);

    let stride = u_count;
    for v in 0..quad_v {
        let v0 = v;
        let v1 = if wrap_v { (v + 1) % v_count } else { v + 1 };

        for u in 0..quad_u {
            let u0 = u;
            let u1 = if wrap_u { (u + 1) % u_count } else { u + 1 };

            let i0 = (v0 * stride + u0) as u32;
            let i1 = (v0 * stride + u1) as u32;
            let i2 = (v1 * stride + u0) as u32;
            let i3 = (v1 * stride + u1) as u32;

            indices.extend_from_slice(&[i0, i1, i2]);
            indices.extend_from_slice(&[i2, i1, i3]);
        }
    }

    indices
}

use super::trim::TrimRegion;
use super::{Tolerance, UvPoint};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TriangulationOptions {
    pub min_triangle_area: f64,
    pub min_triangle_quality: f64,
    pub cull_skinny_triangles: bool,
}

impl TriangulationOptions {
    #[must_use]
    pub fn for_tolerance(tol: Tolerance) -> Self {
        let eps2 = tol.eps_squared();
        Self {
            min_triangle_area: eps2,
            min_triangle_quality: 0.0,
            cull_skinny_triangles: false,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TriangulationDiagnostics {
    pub input_vertex_count: usize,
    pub output_triangle_count: usize,
    pub culled_degenerate_triangles: usize,
    pub below_min_quality_triangles: usize,
    pub culled_skinny_triangles: usize,
    pub min_kept_triangle_quality: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TriangulationResult {
    pub vertices: Vec<UvPoint>,
    pub indices: Vec<u32>,
    pub diagnostics: TriangulationDiagnostics,
}

#[derive(Debug, Clone, Copy)]
struct Node {
    idx: u32,
    point: UvPoint,
    prev: usize,
    next: usize,
    removed: bool,
}

#[must_use]
pub fn triangulate_trim_region(region: &TrimRegion, tol: Tolerance) -> Result<TriangulationResult, String> {
    triangulate_trim_region_with_options(region, tol, TriangulationOptions::for_tolerance(tol))
}

#[must_use]
pub fn triangulate_trim_region_with_options(
    region: &TrimRegion,
    tol: Tolerance,
    options: TriangulationOptions,
) -> Result<TriangulationResult, String> {
    let mut vertices = Vec::new();
    vertices.extend_from_slice(region.outer.points());
    for hole in &region.holes {
        vertices.extend_from_slice(hole.points());
    }

    if vertices.iter().any(|p| !p.u.is_finite() || !p.v.is_finite()) {
        return Err("triangulation vertices must be finite".to_string());
    }

    let mut nodes: Vec<Node> = Vec::new();

    let outer_len = region.outer.points().len();
    if outer_len < 3 {
        return Err("trim region outer loop must have at least 3 points".to_string());
    }

    let outer_start = build_ring_nodes(&mut nodes, 0, outer_len as u32, &vertices);
    let mut outer_start = match filter_ring_points(outer_start, &mut nodes, tol) {
        Some(start) => start,
        None => return Err("trim region outer loop degenerates after filtering".to_string()),
    };

    let mut hole_starts = Vec::new();
    let mut cursor = outer_len as u32;
    for hole in &region.holes {
        let len = hole.points().len();
        if len < 3 {
            continue;
        }
        let start = build_ring_nodes(&mut nodes, cursor, cursor + len as u32, &vertices);
        if let Some(filtered) = filter_ring_points(start, &mut nodes, tol) {
            hole_starts.push(filtered);
        }
        cursor += len as u32;
    }

    let mut hole_lefts: Vec<usize> = hole_starts
        .into_iter()
        .map(|start| leftmost_node(start, &nodes))
        .collect();
    hole_lefts.sort_by(|&a, &b| {
        let pa = nodes[a].point;
        let pb = nodes[b].point;
        pa.u.total_cmp(&pb.u).then_with(|| pa.v.total_cmp(&pb.v))
    });

    for hole_left in hole_lefts {
        let bridge = find_hole_bridge(hole_left, outer_start, &nodes, tol);
        let Some(bridge) = bridge else {
            return Err("failed to find a bridge from hole to outer loop".to_string());
        };
        split_polygon(bridge, hole_left, &mut nodes);
        outer_start = match filter_ring_points(outer_start, &mut nodes, tol) {
            Some(start) => start,
            None => return Err("trim region degenerates after hole merge".to_string()),
        };
    }

    let mut triangles = earclip_polygon(outer_start, &mut nodes, tol)?;
    if triangles.is_empty() {
        return Err("triangulation produced no triangles".to_string());
    }

    let mut culled_degenerate = 0usize;
    let mut below_min_quality = 0usize;
    let mut culled_skinny = 0usize;
    let mut kept_min_quality = f64::INFINITY;
    let mut indices: Vec<u32> = Vec::with_capacity(triangles.len() * 3);

    for tri in triangles.drain(..) {
        let i0 = tri[0];
        let i1 = tri[1];
        let i2 = tri[2];

        if i0 == i1 || i1 == i2 || i0 == i2 {
            culled_degenerate += 1;
            continue;
        }

        let a = vertices[i0 as usize];
        let b = vertices[i1 as usize];
        let c = vertices[i2 as usize];

        let area2 = orient2d(a, b, c).abs();
        let area = 0.5 * area2;
        if !area.is_finite() || area <= options.min_triangle_area {
            culled_degenerate += 1;
            continue;
        }

        let quality = triangle_quality(a, b, c);
        if options.min_triangle_quality > 0.0 && quality < options.min_triangle_quality {
            below_min_quality += 1;
            if options.cull_skinny_triangles {
                culled_skinny += 1;
                continue;
            }
        }

        kept_min_quality = kept_min_quality.min(quality);
        indices.extend_from_slice(&[i0, i1, i2]);
    }

    if kept_min_quality == f64::INFINITY {
        kept_min_quality = 0.0;
    }

    let input_vertex_count = vertices.len();
    let output_triangle_count = indices.len() / 3;

    Ok(TriangulationResult {
        vertices,
        indices,
        diagnostics: TriangulationDiagnostics {
            input_vertex_count,
            output_triangle_count,
            culled_degenerate_triangles: culled_degenerate,
            below_min_quality_triangles: below_min_quality,
            culled_skinny_triangles: culled_skinny,
            min_kept_triangle_quality: kept_min_quality,
        },
    })
}

/// Triangulates a trim region with additional Steiner (interior) points.
///
/// This function extends the basic constrained triangulation by incorporating
/// additional interior points into the mesh. These "Steiner points" are useful for:
/// - Fitting a surface through specific target points
/// - Controlling mesh density and quality in specific areas
/// - Implementing patch flexibility behavior
///
/// The algorithm:
/// 1. First triangulates the region using constrained earclip triangulation.
/// 2. Then incrementally inserts each Steiner point into the triangulation
///    by finding the containing triangle and re-triangulating locally.
///
/// # Arguments
/// * `region` - The trim region (outer boundary + optional holes).
/// * `steiner_points` - Additional interior points to incorporate.
/// * `tol` - Tolerance for geometric operations.
///
/// # Returns
/// A `TriangulationResult` containing the vertices (including Steiner points) and triangle indices.
#[must_use]
pub fn triangulate_trim_region_with_steiner_points(
    region: &TrimRegion,
    steiner_points: &[UvPoint],
    tol: Tolerance,
) -> Result<TriangulationResult, String> {
    // If no steiner points, delegate to the standard triangulation
    if steiner_points.is_empty() {
        return triangulate_trim_region(region, tol);
    }

    // Filter out duplicate or invalid steiner points
    let valid_steiner: Vec<UvPoint> = steiner_points
        .iter()
        .copied()
        .filter(|p| p.u.is_finite() && p.v.is_finite())
        .collect();

    if valid_steiner.is_empty() {
        return triangulate_trim_region(region, tol);
    }

    // Collect all boundary points (outer + holes)
    let mut boundary_points: Vec<UvPoint> = Vec::new();
    boundary_points.extend_from_slice(region.outer.points());
    for hole in &region.holes {
        boundary_points.extend_from_slice(hole.points());
    }

    // Combine boundary points with steiner points for a unified point set
    let mut all_points: Vec<(f64, f64)> = boundary_points
        .iter()
        .map(|p| (p.u, p.v))
        .collect();

    // Filter steiner points that are too close to existing boundary points
    let boundary_count = all_points.len();
    for sp in valid_steiner.iter() {
        let too_close = all_points.iter().any(|(u, v)| {
            let du = sp.u - u;
            let dv = sp.v - v;
            (du * du + dv * dv).sqrt() < tol.eps * 10.0
        });
        if !too_close {
            all_points.push((sp.u, sp.v));
        }
    }

    // If we only have boundary points (all steiner were too close), use standard triangulation
    if all_points.len() == boundary_count {
        return triangulate_trim_region(region, tol);
    }

    // Use Delaunay triangulation on all points
    let delaunay_tris = delaunay_triangulate(&all_points)?;

    // Filter triangles: keep only those whose centroid is inside the region
    let mut kept_triangles: Vec<[u32; 3]> = Vec::new();
    for (i0, i1, i2) in delaunay_tris {
        let p0 = UvPoint::new(all_points[i0].0, all_points[i0].1);
        let p1 = UvPoint::new(all_points[i1].0, all_points[i1].1);
        let p2 = UvPoint::new(all_points[i2].0, all_points[i2].1);

        // Calculate centroid
        let cu = (p0.u + p1.u + p2.u) / 3.0;
        let cv = (p0.v + p1.v + p2.v) / 3.0;
        let centroid = UvPoint::new(cu, cv);

        // Check if centroid is inside the region (respects holes)
        if region.contains(centroid, tol) {
            // Check triangle winding - ensure CCW
            let area = orient2d(p0, p1, p2);
            if area > tol.eps {
                kept_triangles.push([i0 as u32, i1 as u32, i2 as u32]);
            } else if area < -tol.eps {
                // Reverse winding
                kept_triangles.push([i0 as u32, i2 as u32, i1 as u32]);
            }
            // Skip degenerate triangles (area ≈ 0)
        }
    }

    if kept_triangles.is_empty() {
        return Err("triangulation with steiner points produced no valid triangles".to_string());
    }

    // Build output
    let vertices: Vec<UvPoint> = all_points
        .iter()
        .map(|(u, v)| UvPoint::new(*u, *v))
        .collect();

    let mut indices: Vec<u32> = Vec::with_capacity(kept_triangles.len() * 3);
    for tri in kept_triangles.iter() {
        indices.extend_from_slice(tri);
    }

    let input_vertex_count = vertices.len();
    let output_triangle_count = kept_triangles.len();

    Ok(TriangulationResult {
        vertices,
        indices,
        diagnostics: TriangulationDiagnostics {
            input_vertex_count,
            output_triangle_count,
            culled_degenerate_triangles: 0,
            below_min_quality_triangles: 0,
            culled_skinny_triangles: 0,
            min_kept_triangle_quality: 0.0, // Could compute if needed
        },
    })
}

fn build_ring_nodes(nodes: &mut Vec<Node>, start: u32, end: u32, vertices: &[UvPoint]) -> usize {
    let start_idx = nodes.len();
    let len = (end - start) as usize;
    for i in 0..len {
        let idx = start + i as u32;
        nodes.push(Node {
            idx,
            point: vertices[idx as usize],
            prev: 0,
            next: 0,
            removed: false,
        });
    }

    for i in 0..len {
        let current = start_idx + i;
        nodes[current].prev = start_idx + ((i + len - 1) % len);
        nodes[current].next = start_idx + ((i + 1) % len);
    }

    start_idx
}

fn ring_len(start: usize, nodes: &[Node]) -> usize {
    let mut count = 0usize;
    let mut cur = start;
    loop {
        count += 1;
        cur = nodes[cur].next;
        if cur == start || count > nodes.len().saturating_add(1) {
            break;
        }
    }
    count
}

fn filter_ring_points(start: usize, nodes: &mut Vec<Node>, tol: Tolerance) -> Option<usize> {
    if ring_len(start, nodes) < 3 {
        return None;
    }

    let mut start = start;
    let mut cur = start;
    let mut guard = 0usize;

    loop {
        guard += 1;
        if guard > nodes.len().saturating_mul(4).max(16) {
            break;
        }

        let prev = nodes[cur].prev;
        let next = nodes[cur].next;
        if cur == next || cur == prev || prev == next {
            break;
        }

        let p = nodes[prev].point;
        let c = nodes[cur].point;
        let n = nodes[next].point;

        let dup = approx_eq_uv(p, c, tol) || approx_eq_uv(c, n, tol);
        let collinear = distance_point_to_line_2d(p, c, n) <= tol.eps;

        if dup || collinear {
            if cur == start {
                start = next;
            }
            remove_node(cur, nodes);
            cur = prev;
            if ring_len(start, nodes) < 3 {
                return None;
            }
        } else {
            cur = next;
        }

        if cur == start {
            break;
        }
    }

    Some(start)
}

fn leftmost_node(start: usize, nodes: &[Node]) -> usize {
    let mut left = start;
    let mut cur = nodes[start].next;
    while cur != start {
        let a = nodes[cur].point;
        let b = nodes[left].point;
        if a.u < b.u || (a.u == b.u && a.v < b.v) {
            left = cur;
        }
        cur = nodes[cur].next;
    }
    left
}

fn find_hole_bridge(hole: usize, outer_start: usize, nodes: &[Node], tol: Tolerance) -> Option<usize> {
    let hole_p = nodes[hole].point;
    let mut best_x = f64::NEG_INFINITY;
    let mut best_edge = None;

    let mut p = outer_start;
    loop {
        let q = nodes[p].next;
        let a = nodes[p].point;
        let b = nodes[q].point;

        if (a.v > hole_p.v) != (b.v > hole_p.v) {
            let denom = b.v - a.v;
            if denom != 0.0 {
                let t = (hole_p.v - a.v) / denom;
                let x = a.u + t * (b.u - a.u);
                if x <= hole_p.u + tol.eps && x > best_x {
                    best_x = x;
                    best_edge = Some((p, q));
                }
            }
        }

        p = q;
        if p == outer_start {
            break;
        }
    }

    let (e0, e1) = best_edge?;
    let candidates = if nodes[e0].point.u < nodes[e1].point.u {
        [e0, e1]
    } else {
        [e1, e0]
    };

    for cand in candidates {
        if is_visible(hole_p, nodes[cand].point, cand, outer_start, nodes, tol) {
            return Some(cand);
        }
    }

    let mut best = None;
    let mut best_dist2 = f64::INFINITY;

    let mut v = outer_start;
    loop {
        let p = nodes[v].point;
        if p.u <= hole_p.u + tol.eps
            && is_visible(hole_p, p, v, outer_start, nodes, tol)
        {
            let du = p.u - hole_p.u;
            let dv = p.v - hole_p.v;
            let d2 = du * du + dv * dv;
            if d2 < best_dist2 {
                best_dist2 = d2;
                best = Some(v);
            }
        }

        v = nodes[v].next;
        if v == outer_start {
            break;
        }
    }

    best
}

fn split_polygon(a: usize, b: usize, nodes: &mut Vec<Node>) {
    let a_next = nodes[a].next;
    let b_prev = nodes[b].prev;

    let a2 = nodes.len();
    nodes.push(Node {
        idx: nodes[a].idx,
        point: nodes[a].point,
        prev: 0,
        next: 0,
        removed: false,
    });

    let b2 = nodes.len();
    nodes.push(Node {
        idx: nodes[b].idx,
        point: nodes[b].point,
        prev: 0,
        next: 0,
        removed: false,
    });

    nodes[a].next = b;
    nodes[b].prev = a;

    nodes[b_prev].next = b2;
    nodes[b2].prev = b_prev;

    nodes[b2].next = a2;
    nodes[a2].prev = b2;

    nodes[a2].next = a_next;
    nodes[a_next].prev = a2;
}

fn earclip_polygon(start: usize, nodes: &mut Vec<Node>, tol: Tolerance) -> Result<Vec<[u32; 3]>, String> {
    let mut start = match filter_ring_points(start, nodes, tol) {
        Some(start) => start,
        None => return Err("polygon degenerates after filtering".to_string()),
    };

    let is_ccw = signed_area_ring(start, nodes) > 0.0;
    let mut remaining = ring_len(start, nodes);
    if remaining < 3 {
        return Err("polygon has fewer than 3 vertices".to_string());
    }

    let mut ear = start;
    let mut stop = start;
    let mut triangles = Vec::with_capacity(remaining.saturating_sub(2));
    let mut passes_without_clip = 0usize;

    while remaining > 2 {
        let prev = nodes[ear].prev;
        let next = nodes[ear].next;
        if is_ear(prev, ear, next, start, nodes, is_ccw, tol) {
            if is_ccw {
                triangles.push([nodes[prev].idx, nodes[ear].idx, nodes[next].idx]);
            } else {
                triangles.push([nodes[prev].idx, nodes[next].idx, nodes[ear].idx]);
            }

            if ear == start {
                start = next;
            }
            remove_node(ear, nodes);
            remaining -= 1;
            ear = next;
            stop = next;
            passes_without_clip = 0;
            continue;
        }

        ear = next;
        if ear == stop {
            passes_without_clip += 1;
            if passes_without_clip > 2 {
                return Err("failed to triangulate polygon (no ears found)".to_string());
            }
            start = match filter_ring_points(start, nodes, tol) {
                Some(start) => start,
                None => return Err("polygon degenerates during triangulation".to_string()),
            };
            remaining = ring_len(start, nodes);
            ear = start;
            stop = start;
        }
    }

    Ok(triangles)
}

fn is_ear(
    prev: usize,
    ear: usize,
    next: usize,
    _start: usize,
    nodes: &[Node],
    is_ccw: bool,
    tol: Tolerance,
) -> bool {
    let a = nodes[prev].point;
    let b = nodes[ear].point;
    let c = nodes[next].point;

    let cross = orient2d(a, b, c);
    if distance_point_to_line_2d(a, b, c) <= tol.eps {
        return false;
    }

    if is_ccw {
        if cross <= 0.0 {
            return false;
        }
    } else if cross >= 0.0 {
        return false;
    }

    let mut p = nodes[next].next;
    let mut guard = 0usize;
    while p != prev {
        guard += 1;
        if guard > nodes.len().saturating_add(1) {
            break;
        }
        let pt = nodes[p].point;
        if point_in_triangle(a, b, c, pt, is_ccw, tol) {
            let prev_p = nodes[p].prev;
            let next_p = nodes[p].next;
            let cross_p = orient2d(nodes[prev_p].point, pt, nodes[next_p].point);
            let is_reflex = if is_ccw {
                cross_p <= tol.eps
            } else {
                cross_p >= -tol.eps
            };
            if is_reflex {
                return false;
            }
        }
        p = nodes[p].next;
    }

    true
}

fn signed_area_ring(start: usize, nodes: &[Node]) -> f64 {
    let mut area = 0.0;
    let mut p = start;
    loop {
        let q = nodes[p].next;
        let a = nodes[p].point;
        let b = nodes[q].point;
        area += a.u * b.v - b.u * a.v;
        p = q;
        if p == start {
            break;
        }
    }
    0.5 * area
}

fn remove_node(node: usize, nodes: &mut [Node]) {
    let prev = nodes[node].prev;
    let next = nodes[node].next;
    nodes[prev].next = next;
    nodes[next].prev = prev;
    nodes[node].removed = true;
}

fn approx_eq_uv(a: UvPoint, b: UvPoint, tol: Tolerance) -> bool {
    (a.u - b.u).abs() <= tol.eps && (a.v - b.v).abs() <= tol.eps
}

fn orient2d(a: UvPoint, b: UvPoint, c: UvPoint) -> f64 {
    (b.u - a.u) * (c.v - a.v) - (b.v - a.v) * (c.u - a.u)
}

fn point_in_triangle(a: UvPoint, b: UvPoint, c: UvPoint, p: UvPoint, is_ccw: bool, tol: Tolerance) -> bool {
    let ab = orient2d(a, b, p);
    let bc = orient2d(b, c, p);
    let ca = orient2d(c, a, p);

    if is_ccw {
        ab >= -tol.eps && bc >= -tol.eps && ca >= -tol.eps
    } else {
        ab <= tol.eps && bc <= tol.eps && ca <= tol.eps
    }
}

fn is_visible(
    a: UvPoint,
    b: UvPoint,
    b_node: usize,
    ring_start: usize,
    nodes: &[Node],
    tol: Tolerance,
) -> bool {
    let mut e = ring_start;
    loop {
        let n = nodes[e].next;
        if e != b_node && n != b_node {
            let c = nodes[e].point;
            let d = nodes[n].point;
            if segments_intersect(a, b, c, d, tol) {
                return false;
            }
        }

        e = n;
        if e == ring_start {
            break;
        }
    }
    true
}

fn segments_intersect(a: UvPoint, b: UvPoint, c: UvPoint, d: UvPoint, tol: Tolerance) -> bool {
    let o1 = orient2d(a, b, c);
    let o2 = orient2d(a, b, d);
    let o3 = orient2d(c, d, a);
    let o4 = orient2d(c, d, b);

    if o1.abs() <= tol.eps && on_segment(a, c, b, tol) {
        return true;
    }
    if o2.abs() <= tol.eps && on_segment(a, d, b, tol) {
        return true;
    }
    if o3.abs() <= tol.eps && on_segment(c, a, d, tol) {
        return true;
    }
    if o4.abs() <= tol.eps && on_segment(c, b, d, tol) {
        return true;
    }

    let ab = (o1 > tol.eps && o2 < -tol.eps) || (o1 < -tol.eps && o2 > tol.eps);
    let cd = (o3 > tol.eps && o4 < -tol.eps) || (o3 < -tol.eps && o4 > tol.eps);
    ab && cd
}

fn on_segment(a: UvPoint, p: UvPoint, b: UvPoint, tol: Tolerance) -> bool {
    let min_x = a.u.min(b.u) - tol.eps;
    let max_x = a.u.max(b.u) + tol.eps;
    let min_y = a.v.min(b.v) - tol.eps;
    let max_y = a.v.max(b.v) + tol.eps;
    p.u >= min_x && p.u <= max_x && p.v >= min_y && p.v <= max_y
}

fn triangle_quality(a: UvPoint, b: UvPoint, c: UvPoint) -> f64 {
    let area2 = orient2d(a, b, c).abs();
    if !area2.is_finite() || area2 <= 0.0 {
        return 0.0;
    }

    let ab2 = (a.u - b.u).powi(2) + (a.v - b.v).powi(2);
    let bc2 = (b.u - c.u).powi(2) + (b.v - c.v).powi(2);
    let ca2 = (c.u - a.u).powi(2) + (c.v - a.v).powi(2);
    let sum = ab2 + bc2 + ca2;
    if !sum.is_finite() || sum <= 0.0 {
        return 0.0;
    }

    let area = 0.5 * area2;
    (4.0 * 3.0_f64.sqrt() * area / sum).clamp(0.0, 1.0)
}

fn distance_point_to_line_2d(a: UvPoint, p: UvPoint, b: UvPoint) -> f64 {
    let ab_u = b.u - a.u;
    let ab_v = b.v - a.v;
    let denom2 = ab_u * ab_u + ab_v * ab_v;
    if !denom2.is_finite() || denom2 <= 0.0 {
        return ((p.u - a.u).powi(2) + (p.v - a.v).powi(2)).sqrt();
    }

    let denom = denom2.sqrt();
    orient2d(a, b, p).abs() / denom
}

// ============================================================================
// Delaunay Triangulation
// ============================================================================

/// Perform Delaunay triangulation on 2D points.
///
/// Uses the Bowyer-Watson algorithm to compute the Delaunay triangulation
/// of the given point set. Returns a list of triangles as index triples.
///
/// This is useful for triangulating scattered point clouds where no
/// boundary constraints or holes are present. For constrained triangulation
/// with boundaries and holes, use `triangulate_trim_region` instead.
///
/// # Arguments
/// * `points` - 2D point coordinates as (u, v) tuples
///
/// # Returns
/// A list of triangles, where each triangle is represented as (i0, i1, i2)
/// indices into the original points array.
///
/// # Errors
/// Returns an error if fewer than 3 points are provided or if any point
/// has non-finite coordinates.
#[must_use]
pub fn delaunay_triangulate(points: &[(f64, f64)]) -> Result<Vec<(usize, usize, usize)>, String> {
    let n = points.len();
    if n < 3 {
        return Err("need at least 3 points for triangulation".to_string());
    }

    // Check for NaN/Inf
    for (i, (u, v)) in points.iter().enumerate() {
        if !u.is_finite() || !v.is_finite() {
            return Err(format!("point {} has non-finite coordinates", i));
        }
    }

    // Compute bounding box
    let (mut min_u, mut max_u) = (f64::MAX, f64::MIN);
    let (mut min_v, mut max_v) = (f64::MAX, f64::MIN);
    for &(u, v) in points {
        min_u = min_u.min(u);
        max_u = max_u.max(u);
        min_v = min_v.min(v);
        max_v = max_v.max(v);
    }

    let du = (max_u - min_u).max(1e-10);
    let dv = (max_v - min_v).max(1e-10);
    let d_max = du.max(dv);

    // Create super-triangle that contains all points
    // The super-triangle vertices are placed far outside the point set
    let margin = 10.0 * d_max;
    let center_u = (min_u + max_u) / 2.0;
    let center_v = (min_v + max_v) / 2.0;

    // Super-triangle vertices (indices n, n+1, n+2)
    let super_vertices = [
        (center_u - 2.0 * margin, center_v - margin),
        (center_u + 2.0 * margin, center_v - margin),
        (center_u, center_v + 2.0 * margin),
    ];

    // All vertices: original points + super-triangle
    let mut all_points: Vec<(f64, f64)> = points.to_vec();
    all_points.extend_from_slice(&super_vertices);

    // Initialize with super-triangle
    let mut triangles: Vec<DelaunayTriangle> = vec![DelaunayTriangle::new(n, n + 1, n + 2)];

    // Insert points one by one (Bowyer-Watson algorithm)
    for point_idx in 0..n {
        let p = all_points[point_idx];

        // Find all triangles whose circumcircle contains the point
        let mut bad_triangles = Vec::new();
        for (tri_idx, tri) in triangles.iter().enumerate() {
            if tri.circumcircle_contains(p, &all_points) {
                bad_triangles.push(tri_idx);
            }
        }

        // Find the boundary of the polygonal hole (edges not shared by bad triangles)
        let mut polygon_edges: Vec<(usize, usize)> = Vec::new();
        for &tri_idx in &bad_triangles {
            let tri = &triangles[tri_idx];
            let edges = tri.edges();
            for edge in edges {
                // Check if this edge is shared with another bad triangle
                let mut shared = false;
                for &other_idx in &bad_triangles {
                    if other_idx != tri_idx {
                        let other = &triangles[other_idx];
                        if other.has_edge(edge.0, edge.1) {
                            shared = true;
                            break;
                        }
                    }
                }
                if !shared {
                    polygon_edges.push(edge);
                }
            }
        }

        // Remove bad triangles (in reverse order to maintain indices)
        let mut bad_sorted = bad_triangles.clone();
        bad_sorted.sort_by(|a, b| b.cmp(a));
        for tri_idx in bad_sorted {
            triangles.swap_remove(tri_idx);
        }

        // Create new triangles from polygon edges to the new point
        for (e0, e1) in polygon_edges {
            triangles.push(DelaunayTriangle::new(e0, e1, point_idx));
        }
    }

    // Remove triangles that share vertices with super-triangle
    triangles.retain(|tri| tri.v0 < n && tri.v1 < n && tri.v2 < n);

    if triangles.is_empty() {
        return Err("Delaunay triangulation produced no triangles".to_string());
    }

    // Convert to output format
    let result: Vec<(usize, usize, usize)> = triangles
        .iter()
        .map(|tri| (tri.v0, tri.v1, tri.v2))
        .collect();

    Ok(result)
}

/// A triangle in the Delaunay triangulation.
#[derive(Debug, Clone, Copy)]
struct DelaunayTriangle {
    v0: usize,
    v1: usize,
    v2: usize,
}

impl DelaunayTriangle {
    fn new(v0: usize, v1: usize, v2: usize) -> Self {
        Self { v0, v1, v2 }
    }

    /// Get the three edges of this triangle.
    fn edges(&self) -> [(usize, usize); 3] {
        [
            (self.v0, self.v1),
            (self.v1, self.v2),
            (self.v2, self.v0),
        ]
    }

    /// Check if this triangle has the given edge (in either direction).
    fn has_edge(&self, a: usize, b: usize) -> bool {
        let edges = self.edges();
        for (e0, e1) in edges {
            if (e0 == a && e1 == b) || (e0 == b && e1 == a) {
                return true;
            }
        }
        false
    }

    /// Check if a point is inside this triangle's circumcircle.
    fn circumcircle_contains(&self, p: (f64, f64), points: &[(f64, f64)]) -> bool {
        let (ax, ay) = points[self.v0];
        let (bx, by) = points[self.v1];
        let (cx, cy) = points[self.v2];
        let (px, py) = p;

        // Use the determinant formula for circumcircle test
        // Point is inside if the determinant is positive (for CCW triangle)
        // or negative (for CW triangle)

        let ax_px = ax - px;
        let ay_py = ay - py;
        let bx_px = bx - px;
        let by_py = by - py;
        let cx_px = cx - px;
        let cy_py = cy - py;

        let det = (ax_px * ax_px + ay_py * ay_py) * (bx_px * cy_py - cx_px * by_py)
            - (bx_px * bx_px + by_py * by_py) * (ax_px * cy_py - cx_px * ay_py)
            + (cx_px * cx_px + cy_py * cy_py) * (ax_px * by_py - bx_px * ay_py);

        // The sign depends on the winding order of the triangle
        // Check triangle orientation
        let orientation = (bx - ax) * (cy - ay) - (by - ay) * (cx - ax);

        if orientation > 0.0 {
            det > 0.0
        } else {
            det < 0.0
        }
    }
}

#[cfg(test)]
mod delaunay_tests {
    use super::*;

    #[test]
    fn test_delaunay_triangulate_square() {
        // 4 points forming a square
        let points = vec![
            (0.0, 0.0),
            (1.0, 0.0),
            (1.0, 1.0),
            (0.0, 1.0),
        ];

        let result = delaunay_triangulate(&points);
        assert!(result.is_ok());

        let triangles = result.unwrap();
        // A square should produce exactly 2 triangles
        assert_eq!(triangles.len(), 2);

        // All indices should be valid
        for (i0, i1, i2) in &triangles {
            assert!(*i0 < 4);
            assert!(*i1 < 4);
            assert!(*i2 < 4);
            // No duplicate vertices in a triangle
            assert_ne!(i0, i1);
            assert_ne!(i1, i2);
            assert_ne!(i2, i0);
        }
    }

    #[test]
    fn test_delaunay_triangulate_triangle() {
        // 3 points forming a single triangle
        let points = vec![
            (0.0, 0.0),
            (1.0, 0.0),
            (0.5, 1.0),
        ];

        let result = delaunay_triangulate(&points);
        assert!(result.is_ok());

        let triangles = result.unwrap();
        assert_eq!(triangles.len(), 1);
    }

    #[test]
    fn test_delaunay_triangulate_pentagon() {
        // 5 points forming a convex pentagon-ish shape
        let points = vec![
            (0.0, 0.0),
            (1.0, 0.0),
            (1.5, 0.5),
            (0.5, 1.0),
            (-0.5, 0.5),
        ];

        let result = delaunay_triangulate(&points);
        assert!(result.is_ok());

        let triangles = result.unwrap();
        // 5 points should produce 3 triangles (n - 2 for convex hull)
        assert_eq!(triangles.len(), 3);
    }

    #[test]
    fn test_delaunay_too_few_points() {
        let points = vec![(0.0, 0.0), (1.0, 1.0)];
        let result = delaunay_triangulate(&points);
        assert!(result.is_err());
    }

    #[test]
    fn test_delaunay_with_interior_point() {
        // Square with a point in the middle
        let points = vec![
            (0.0, 0.0),
            (2.0, 0.0),
            (2.0, 2.0),
            (0.0, 2.0),
            (1.0, 1.0), // Interior point
        ];

        let result = delaunay_triangulate(&points);
        assert!(result.is_ok());

        let triangles = result.unwrap();
        // With interior point, should have 4 triangles
        assert_eq!(triangles.len(), 4);
    }

    #[test]
    fn test_delaunay_nan_rejection() {
        let points = vec![
            (0.0, 0.0),
            (f64::NAN, 0.0),
            (0.5, 1.0),
        ];
        let result = delaunay_triangulate(&points);
        assert!(result.is_err());
    }

    #[test]
    fn test_delaunay_inf_rejection() {
        let points = vec![
            (0.0, 0.0),
            (1.0, f64::INFINITY),
            (0.5, 1.0),
        ];
        let result = delaunay_triangulate(&points);
        assert!(result.is_err());
    }
}
