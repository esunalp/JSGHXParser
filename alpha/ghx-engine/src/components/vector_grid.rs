//! Implementaties van Grasshopper "Vector → Grid" componenten.

use std::collections::BTreeMap;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

const EPSILON: f64 = 1e-9;

const PIN_OUTPUT_CELLS: &str = "C";
const PIN_OUTPUT_POINTS: &str = "P";
const PIN_OUTPUT_POPULATION: &str = "P";
const PIN_OUTPUT_CLOUD: &str = "C";
const PIN_OUTPUT_NORMALS: &str = "N";

/// Beschikbare componenten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Hexagonal,
    Rectangular,
    Square,
    Radial,
    Triangular,
    PopulateGeometry,
    Populate3d,
    Populate2d,
    FreeformCloud,
    SphericalCloud,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst van componentregistraties voor de vector-grid componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{125dc122-8544-4617-945e-bb9a0c101c50}"],
        names: &["Hexagonal Grid", "HexGrid"],
        kind: ComponentKind::Hexagonal,
    },
    Registration {
        guids: &[
            "{1a25aae0-0b56-497a-85b2-cc5bf7e4b96b}",
            "{fdedcd0a-ad40-4307-959d-d2891e2f533e}",
        ],
        names: &["Rectangular Grid", "RecGrid"],
        kind: ComponentKind::Rectangular,
    },
    Registration {
        guids: &[
            "{40efea60-1902-4c28-8020-27abbb7a1449}",
            "{717a1e25-a075-4530-bc80-d43ecc2500d9}",
        ],
        names: &["Square Grid", "SqGrid"],
        kind: ComponentKind::Square,
    },
    Registration {
        guids: &[
            "{66eedc35-187d-4dab-b49b-408491b1255f}",
            "{773183d0-8c00-4fe4-a38c-f8d2408b7415}",
        ],
        names: &["Radial Grid", "RadGrid"],
        kind: ComponentKind::Radial,
    },
    Registration {
        guids: &["{86a9944b-dea5-4126-9433-9e95ff07927a}"],
        names: &["Triangular Grid", "TriGrid"],
        kind: ComponentKind::Triangular,
    },
    Registration {
        guids: &["{c8cb6a5c-2ffd-4095-ba2a-5c35015e09e4}"],
        names: &["Populate Geometry", "PopGeo"],
        kind: ComponentKind::PopulateGeometry,
    },
    Registration {
        guids: &["{e202025b-dc8e-4c51-ae19-4415b172886f}"],
        names: &["Populate 3D", "Pop3D"],
        kind: ComponentKind::Populate3d,
    },
    Registration {
        guids: &["{e2d958e8-9f08-44f7-bf47-a684882d0b2a}"],
        names: &["Populate 2D", "Pop2D"],
        kind: ComponentKind::Populate2d,
    },
    Registration {
        guids: &["{f08233f1-9772-4514-8965-bde4948503df}"],
        names: &["Freeform Cloud", "FFCloud"],
        kind: ComponentKind::FreeformCloud,
    },
    Registration {
        guids: &["{fd68754e-6c60-44b2-9927-0a58146e0250}"],
        names: &["Spherical Cloud", "SphCloud"],
        kind: ComponentKind::SphericalCloud,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::Hexagonal => evaluate_hexagonal_grid(inputs),
            Self::Rectangular => evaluate_rectangular_grid(inputs),
            Self::Square => evaluate_square_grid(inputs),
            Self::Radial => evaluate_radial_grid(inputs),
            Self::Triangular => evaluate_triangular_grid(inputs),
            Self::PopulateGeometry => evaluate_populate_geometry(inputs),
            Self::Populate3d => evaluate_populate_3d(inputs),
            Self::Populate2d => evaluate_populate_2d(inputs),
            Self::FreeformCloud => evaluate_freeform_cloud(inputs),
            Self::SphericalCloud => evaluate_spherical_cloud(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Hexagonal => "Hexagonal Grid",
            Self::Rectangular => "Rectangular Grid",
            Self::Square => "Square Grid",
            Self::Radial => "Radial Grid",
            Self::Triangular => "Triangular Grid",
            Self::PopulateGeometry => "Populate Geometry",
            Self::Populate3d => "Populate 3D",
            Self::Populate2d => "Populate 2D",
            Self::FreeformCloud => "Freeform Cloud",
            Self::SphericalCloud => "Spherical Cloud",
        }
    }
}
fn evaluate_hexagonal_grid(inputs: &[Value]) -> ComponentResult {
    let plane = parse_plane(inputs.get(0), "Hexagonal Grid")?;
    let size = coerce_positive_number(inputs.get(1), 1.0, "Hexagonal Grid")?;
    let extent_x = coerce_usize(inputs.get(2), 6, 1, "Hexagonal Grid extent X")?;
    let extent_y = coerce_usize(inputs.get(3), 6, 1, "Hexagonal Grid extent Y")?;

    let grid = build_hex_grid_by_extents(&plane, size, extent_x, extent_y);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CELLS.to_owned(), polygons_to_value(grid.cells));
    outputs.insert(
        PIN_OUTPUT_POINTS.to_owned(),
        nested_points_to_value(grid.rows),
    );
    Ok(outputs)
}

fn evaluate_rectangular_grid(inputs: &[Value]) -> ComponentResult {
    let plane = parse_plane(inputs.get(0), "Rectangular Grid")?;
    let size_x = coerce_positive_number(inputs.get(1), 1.0, "Rectangular Grid size X")?;
    let size_y = coerce_positive_number(inputs.get(2), size_x, "Rectangular Grid size Y")?;
    let cells_x = coerce_usize(inputs.get(3), 4, 1, "Rectangular Grid extent X")?;
    let cells_y = coerce_usize(inputs.get(4), 4, 1, "Rectangular Grid extent Y")?;

    let point_count_x = cells_x + 1;
    let point_count_y = cells_y + 1;
    let offset = Offset::centered(point_count_x, point_count_y, size_x, size_y);
    let grid = build_rectangular_grid(&plane, point_count_x, point_count_y, size_x, size_y, offset);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CELLS.to_owned(), polygons_to_value(grid.cells));
    outputs.insert(
        PIN_OUTPUT_POINTS.to_owned(),
        nested_points_to_value(grid.points),
    );
    Ok(outputs)
}

fn evaluate_square_grid(inputs: &[Value]) -> ComponentResult {
    let plane = parse_plane(inputs.get(0), "Square Grid")?;
    let size = coerce_positive_number(inputs.get(1), 1.0, "Square Grid size")?;
    let cells_x = coerce_usize(inputs.get(2), 4, 1, "Square Grid extent X")?;
    let cells_y = coerce_usize(inputs.get(3), 4, 1, "Square Grid extent Y")?;

    let point_count_x = cells_x + 1;
    let point_count_y = cells_y + 1;
    let offset = Offset::centered(point_count_x, point_count_y, size, size);
    let grid = build_rectangular_grid(&plane, point_count_x, point_count_y, size, size, offset);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CELLS.to_owned(), polygons_to_value(grid.cells));
    outputs.insert(
        PIN_OUTPUT_POINTS.to_owned(),
        nested_points_to_value(grid.points),
    );
    Ok(outputs)
}

fn evaluate_radial_grid(inputs: &[Value]) -> ComponentResult {
    let plane = parse_plane(inputs.get(0), "Radial Grid")?;
    let radius_step = coerce_positive_number(inputs.get(1), 1.0, "Radial Grid size")?;
    let radial_count = coerce_usize(inputs.get(2), 4, 1, "Radial Grid extent R")?;
    let polar_count = coerce_usize(inputs.get(3), 12, 3, "Radial Grid extent P")?;

    let grid = build_radial_grid(&plane, radius_step, radial_count, polar_count);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CELLS.to_owned(), polygons_to_value(grid.cells));
    outputs.insert(
        PIN_OUTPUT_POINTS.to_owned(),
        nested_points_to_value(grid.rings),
    );
    Ok(outputs)
}

fn evaluate_triangular_grid(inputs: &[Value]) -> ComponentResult {
    let plane = parse_plane(inputs.get(0), "Triangular Grid")?;
    let edge_length = coerce_positive_number(inputs.get(1), 1.0, "Triangular Grid size")?;
    let cells_x = coerce_usize(inputs.get(2), 4, 1, "Triangular Grid extent X")?;
    let cells_y = coerce_usize(inputs.get(3), 4, 1, "Triangular Grid extent Y")?;

    let grid = build_triangular_grid(&plane, edge_length, cells_x, cells_y);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CELLS.to_owned(), polygons_to_value(grid.cells));
    outputs.insert(
        PIN_OUTPUT_POINTS.to_owned(),
        nested_points_to_value(grid.points),
    );
    Ok(outputs)
}

fn evaluate_populate_geometry(inputs: &[Value]) -> ComponentResult {
    let count = resolve_count(inputs.get(1), 100, "Populate Geometry count")?;
    let mut rng = create_seeded_rng(inputs.get(2));
    let existing = collect_points(inputs.get(3), "Populate Geometry bestaande populatie")?;
    let mut population: Vec<[f64; 3]> = existing.into_iter().take(count).collect();

    let geometry = gather_geometry(inputs.get(0), "Populate Geometry")?;
    let fallback = geometry
        .fallback_points
        .first()
        .copied()
        .unwrap_or([0.0, 0.0, 0.0]);
    let mut attempts = 0usize;
    while population.len() < count && attempts < count.saturating_mul(10).max(10) {
        attempts += 1;
        if let Some(sample) = geometry.sample(&mut rng) {
            population.push(sample);
        } else {
            population.push(fallback);
            break;
        }
    }
    while population.len() < count {
        population.push(fallback);
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_POPULATION.to_owned(),
        points_to_value(population.into_iter().take(count).collect()),
    );
    Ok(outputs)
}

fn evaluate_populate_2d(inputs: &[Value]) -> ComponentResult {
    let section = extract_rectangle_section(inputs.get(0), "Populate 2D")?;
    let count = resolve_count(inputs.get(1), 100, "Populate 2D count")?;
    let mut rng = create_seeded_rng(inputs.get(2));
    let existing = collect_points(inputs.get(3), "Populate 2D bestaande populatie")?;
    let mut population: Vec<[f64; 3]> = existing.into_iter().take(count).collect();
    let mut attempts = 0usize;
    while population.len() < count && attempts < count.saturating_mul(10).max(10) {
        attempts += 1;
        let point = random_point_in_rectangle(&section, &mut rng);
        population.push(point);
        if (section.max_x - section.min_x).abs() < EPSILON
            && (section.max_y - section.min_y).abs() < EPSILON
        {
            break;
        }
    }
    while population.len() < count {
        population.push(section.plane.apply(section.min_x, section.min_y, 0.0));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_POPULATION.to_owned(),
        points_to_value(population.into_iter().take(count).collect()),
    );
    Ok(outputs)
}

fn evaluate_populate_3d(inputs: &[Value]) -> ComponentResult {
    let region = extract_box_region(inputs.get(0), "Populate 3D")?;
    let count = resolve_count(inputs.get(1), 100, "Populate 3D count")?;
    let mut rng = create_seeded_rng(inputs.get(2));
    let existing = collect_points(inputs.get(3), "Populate 3D bestaande populatie")?;
    let mut population: Vec<[f64; 3]> = existing.into_iter().take(count).collect();
    let mut attempts = 0usize;
    while population.len() < count && attempts < count.saturating_mul(10).max(10) {
        attempts += 1;
        let point = random_point_in_box(&region, &mut rng);
        population.push(point);
        if region.is_degenerate() {
            break;
        }
    }
    while population.len() < count {
        population.push(region.min);
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_POPULATION.to_owned(),
        points_to_value(population.into_iter().take(count).collect()),
    );
    Ok(outputs)
}

fn evaluate_freeform_cloud(inputs: &[Value]) -> ComponentResult {
    let count = resolve_count(inputs.get(1), 100, "Freeform Cloud count")?;
    let mut rng = create_seeded_rng(inputs.get(2));
    let geometry = gather_geometry(inputs.get(0), "Freeform Cloud")?;
    let fallback = geometry
        .fallback_points
        .first()
        .copied()
        .unwrap_or([0.0, 0.0, 0.0]);

    let mut cloud = Vec::new();
    let mut attempts = 0usize;
    while cloud.len() < count && attempts < count.saturating_mul(10).max(10) {
        attempts += 1;
        if let Some(point) = geometry.sample(&mut rng) {
            cloud.push(point);
        } else {
            cloud.push(fallback);
            break;
        }
    }
    while cloud.len() < count {
        cloud.push(fallback);
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CLOUD.to_owned(), points_to_value(cloud));
    Ok(outputs)
}

fn evaluate_spherical_cloud(inputs: &[Value]) -> ComponentResult {
    let center = coerce_point(inputs.get(0), "Spherical Cloud center")?;
    let radius = coerce_positive_number(inputs.get(1), 1.0, "Spherical Cloud radius")?;
    let count = resolve_count(inputs.get(2), 100, "Spherical Cloud count")?;
    let mut rng = create_seeded_rng(inputs.get(3));

    let mut cloud = Vec::with_capacity(count);
    let mut normals = Vec::with_capacity(count);
    for _ in 0..count {
        let u: f64 = rng.random();
        let v: f64 = rng.random();
        let theta = 2.0 * std::f64::consts::PI * u;
        let phi = (2.0 * v - 1.0).acos();
        let dir = [phi.sin() * theta.cos(), phi.sin() * theta.sin(), phi.cos()];
        let point = add(center, scale(dir, radius));
        cloud.push(point);
        normals.push(dir);
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CLOUD.to_owned(), points_to_value(cloud));
    outputs.insert(PIN_OUTPUT_NORMALS.to_owned(), vectors_to_value(normals));
    Ok(outputs)
}
fn build_hex_grid_by_extents(plane: &Plane, size: f64, count_x: usize, count_y: usize) -> HexGrid {
    let mut rows = Vec::with_capacity(count_y);
    let mut cells = Vec::new();
    let mut local_rows = Vec::with_capacity(count_y);

    let step_x = size * 3.0_f64.sqrt();
    let step_y = 1.5 * size;

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for row in 0..count_y {
        let mut local_row = Vec::with_capacity(count_x);
        let row_offset = if row % 2 == 0 { 0.0 } else { step_x / 2.0 };
        for col in 0..count_x {
            let x = col as f64 * step_x + row_offset;
            let y = row as f64 * step_y;
            local_row.push((x, y));
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
        local_rows.push(local_row);
    }

    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return HexGrid {
            rows: Vec::new(),
            cells: Vec::new(),
        };
    }

    let offset_x = (min_x + max_x) * 0.5;
    let offset_y = (min_y + max_y) * 0.5;

    for local_row in &local_rows {
        let mut point_row = Vec::with_capacity(local_row.len());
        for &(x, y) in local_row {
            let center = plane.apply(x - offset_x, y - offset_y, 0.0);
            point_row.push(center);

            let mut corners = Vec::with_capacity(7);
            for i in 0..6 {
                let angle = std::f64::consts::FRAC_PI_3 * i as f64 + std::f64::consts::FRAC_PI_6;
                let corner = plane.apply(
                    x - offset_x + size * angle.cos(),
                    y - offset_y + size * angle.sin(),
                    0.0,
                );
                corners.push(corner);
            }
            if let Some(first) = corners.first().copied() {
                corners.push(first);
            }
            cells.push(corners);
        }
        rows.push(point_row);
    }

    HexGrid { rows, cells }
}

fn build_rectangular_grid(
    plane: &Plane,
    count_x: usize,
    count_y: usize,
    size_x: f64,
    size_y: f64,
    offset: Offset,
) -> RectangularGrid {
    let mut rows = Vec::with_capacity(count_y);
    for iy in 0..count_y {
        let mut row = Vec::with_capacity(count_x);
        for ix in 0..count_x {
            let x = offset.x + ix as f64 * size_x;
            let y = offset.y + iy as f64 * size_y;
            row.push(plane.apply(x, y, 0.0));
        }
        rows.push(row);
    }

    let mut cells = Vec::new();
    if count_x > 1 && count_y > 1 {
        for ix in 0..count_x - 1 {
            for iy in 0..count_y - 1 {
                let bottom_left = rows[iy][ix];
                let bottom_right = rows[iy][ix + 1];
                let top_left = rows[iy + 1][ix];
                let top_right = rows[iy + 1][ix + 1];
                cells.push(vec![
                    bottom_left,
                    bottom_right,
                    top_right,
                    top_left,
                    bottom_left,
                ]);
            }
        }
    }

    RectangularGrid {
        points: rows,
        cells,
    }
}

fn build_radial_grid(
    plane: &Plane,
    radius_step: f64,
    radial_count: usize,
    polar_count: usize,
) -> RadialGrid {
    let mut rings = Vec::with_capacity(radial_count + 1);
    let mut cells = Vec::new();

    rings.push(vec![plane.apply(0.0, 0.0, 0.0)]);

    let normalized_polar = polar_count.max(3);
    let angle_step = 2.0 * std::f64::consts::PI / normalized_polar as f64;

    for ring in 1..=radial_count {
        let radius = radius_step * ring as f64;
        let mut ring_points = Vec::with_capacity(normalized_polar);
        for segment in 0..normalized_polar {
            let angle = segment as f64 * angle_step;
            ring_points.push(plane.apply(radius * angle.cos(), radius * angle.sin(), 0.0));
        }
        rings.push(ring_points);
    }

    for ring in 0..radial_count {
        let inner_radius = radius_step * ring as f64;
        let outer_radius = radius_step * (ring as f64 + 1.0);
        for segment in 0..normalized_polar {
            let angle_a = segment as f64 * angle_step;
            let angle_b = (segment as f64 + 1.0) * angle_step;
            let mut corners = vec![
                plane.apply(
                    inner_radius * angle_a.cos(),
                    inner_radius * angle_a.sin(),
                    0.0,
                ),
                plane.apply(
                    outer_radius * angle_a.cos(),
                    outer_radius * angle_a.sin(),
                    0.0,
                ),
                plane.apply(
                    outer_radius * angle_b.cos(),
                    outer_radius * angle_b.sin(),
                    0.0,
                ),
                plane.apply(
                    inner_radius * angle_b.cos(),
                    inner_radius * angle_b.sin(),
                    0.0,
                ),
            ];
            if let Some(first) = corners.first().copied() {
                corners.push(first);
            }
            cells.push(corners);
        }
    }

    RadialGrid { rings, cells }
}

fn build_triangular_grid(
    plane: &Plane,
    edge_length: f64,
    count_x: usize,
    count_y: usize,
) -> TriangularGrid {
    let height = edge_length * (3.0_f64).sqrt() / 2.0;
    let mut local_rows = Vec::with_capacity(count_y + 1);
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for row in 0..=count_y {
        let mut local_row = Vec::with_capacity(count_x + 1);
        for col in 0..=count_x {
            let x = (col as f64 + row as f64 / 2.0) * edge_length;
            let y = row as f64 * height;
            local_row.push((x, y));
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
        local_rows.push(local_row);
    }

    if !min_x.is_finite() || !max_x.is_finite() || !min_y.is_finite() || !max_y.is_finite() {
        return TriangularGrid {
            points: Vec::new(),
            cells: Vec::new(),
        };
    }

    let offset_x = (min_x + max_x) * 0.5;
    let offset_y = (min_y + max_y) * 0.5;

    let mut rows = Vec::with_capacity(local_rows.len());
    for local_row in &local_rows {
        let mut row = Vec::with_capacity(local_row.len());
        for &(x, y) in local_row {
            row.push(plane.apply(x - offset_x, y - offset_y, 0.0));
        }
        rows.push(row);
    }

    let mut cells = Vec::new();
    for row in 0..count_y {
        for col in 0..count_x {
            let p00 = rows[row][col];
            let p10 = rows[row][col + 1];
            let p01 = rows[row + 1][col];
            let p11 = rows[row + 1][col + 1];
            if (row + col) % 2 == 0 {
                cells.push(vec![p00, p10, p11, p00]);
                cells.push(vec![p00, p11, p01, p00]);
            } else {
                cells.push(vec![p00, p10, p01, p00]);
                cells.push(vec![p10, p11, p01, p10]);
            }
        }
    }

    TriangularGrid {
        points: rows,
        cells,
    }
}

fn collect_points(value: Option<&Value>, context: &str) -> Result<Vec<[f64; 3]>, ComponentError> {
    let mut points = Vec::new();
    if let Some(value) = value {
        collect_points_into(value, context, &mut points)?;
    }
    Ok(points)
}

fn collect_points_into(
    value: &Value,
    context: &str,
    output: &mut Vec<[f64; 3]>,
) -> Result<(), ComponentError> {
    match value {
        Value::Point(point) | Value::Vector(point) => {
            output.push(*point);
            Ok(())
        }
        Value::CurveLine { p1, p2 } => {
            output.push(*p1);
            output.push(*p2);
            Ok(())
        }
        Value::Surface { vertices, .. } => {
            output.extend(vertices.iter().copied());
            Ok(())
        }
        Value::List(values) => {
            if values.is_empty() {
                return Ok(());
            }
            for entry in values {
                collect_points_into(entry, context, output)?;
            }
            Ok(())
        }
        Value::Number(number) => {
            output.push([*number, 0.0, 0.0]);
            Ok(())
        }
        Value::Boolean(boolean) => {
            output.push([if *boolean { 1.0 } else { 0.0 }, 0.0, 0.0]);
            Ok(())
        }
        Value::Text(text) => {
            if let Ok(parsed) = text.trim().parse::<f64>() {
                output.push([parsed, 0.0, 0.0]);
                Ok(())
            } else {
                Err(ComponentError::new(format!(
                    "{} kon tekst '{}' niet als punt interpreteren",
                    context, text
                )))
            }
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht puntwaarden, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_point(value: Option<&Value>, context: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Some(Value::Point(point) | Value::Vector(point)) => Ok(*point),
        Some(Value::List(values)) if values.len() == 1 => coerce_point(values.get(0), context),
        Some(Value::List(values)) if values.len() >= 3 => {
            let x = coerce_number(values.get(0), 0.0, context)?;
            let y = coerce_number(values.get(1), 0.0, context)?;
            let z = coerce_number(values.get(2), 0.0, context)?;
            Ok([x, y, z])
        }
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een punt, kreeg {}",
            context,
            other.kind()
        ))),
        None => Ok([0.0, 0.0, 0.0]),
    }
}

fn coerce_number(
    value: Option<&Value>,
    default: f64,
    context: &str,
) -> Result<f64, ComponentError> {
    match value {
        None => Ok(default),
        Some(Value::Number(number)) => Ok(*number),
        Some(Value::Boolean(boolean)) => Ok(if *boolean { 1.0 } else { 0.0 }),
        Some(Value::Text(text)) => text.trim().parse::<f64>().map_err(|_| {
            ComponentError::new(format!(
                "{} kon tekst '{}' niet als getal interpreteren",
                context, text
            ))
        }),
        Some(Value::List(values)) if values.len() == 1 => {
            coerce_number(values.get(0), default, context)
        }
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een numerieke waarde, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_positive_number(
    value: Option<&Value>,
    default: f64,
    context: &str,
) -> Result<f64, ComponentError> {
    let number = coerce_number(value, default, context)?;
    Ok(number.max(EPSILON))
}

fn coerce_usize(
    value: Option<&Value>,
    default: usize,
    min: usize,
    context: &str,
) -> Result<usize, ComponentError> {
    let number = coerce_number(value, default as f64, context)?;
    if !number.is_finite() {
        return Ok(min.max(default));
    }
    let rounded = number.round();
    if rounded < min as f64 {
        Ok(min)
    } else {
        Ok(rounded as usize)
    }
}

fn resolve_count(
    value: Option<&Value>,
    default: usize,
    context: &str,
) -> Result<usize, ComponentError> {
    coerce_usize(value, default, 1, context)
}

fn parse_plane(value: Option<&Value>, context: &str) -> Result<Plane, ComponentError> {
    match value {
        None => Ok(Plane::default()),
        Some(value) => coerce_plane(value, context),
    }
}

fn coerce_plane(value: &Value, context: &str) -> Result<Plane, ComponentError> {
    match value {
        Value::List(values) if values.len() >= 3 => {
            let a = coerce_point(values.get(0), context)?;
            let b = coerce_point(values.get(1), context)?;
            let c = coerce_point(values.get(2), context)?;
            Ok(Plane::from_points(a, b, c))
        }
        Value::List(values) if values.len() == 2 => {
            let origin = coerce_point(values.get(0), context)?;
            let direction = coerce_point(values.get(1), context)?;
            let mut dir = subtract(direction, origin);
            if vector_length_squared(dir) < EPSILON {
                dir = [1.0, 0.0, 0.0];
            }
            let x_axis = normalize(dir);
            let z_axis = orthogonal_vector(x_axis);
            let y_axis = normalize(cross(z_axis, x_axis));
            Ok(Plane::normalize_axes(origin, x_axis, y_axis, z_axis))
        }
        Value::List(values) if values.len() == 1 => coerce_plane(&values[0], context),
        Value::Point(point) => {
            let mut plane = Plane::default();
            plane.origin = *point;
            Ok(plane)
        }
        Value::Vector(vector) => {
            let normal = if vector_length_squared(*vector) < EPSILON {
                [0.0, 0.0, 1.0]
            } else {
                normalize(*vector)
            };
            let x_axis = orthogonal_vector(normal);
            let y_axis = normalize(cross(normal, x_axis));
            Ok(Plane::normalize_axes(
                [0.0, 0.0, 0.0],
                x_axis,
                y_axis,
                normal,
            ))
        }
        Value::CurveLine { p1, p2 } => {
            let line = Line {
                start: *p1,
                end: *p2,
            };
            Ok(Plane::from(line))
        }
        Value::Surface { vertices, .. } if vertices.len() >= 3 => {
            Ok(Plane::from_points(vertices[0], vertices[1], vertices[2]))
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht een vlak, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn create_seeded_rng(seed: Option<&Value>) -> StdRng {
    let seed_value = seed
        .and_then(|value| coerce_number(Some(value), 0.0, "seed").ok())
        .unwrap_or(0.0);
    StdRng::seed_from_u64(seed_value.to_bits())
}

fn points_to_value(points: Vec<[f64; 3]>) -> Value {
    Value::List(points.into_iter().map(Value::Point).collect())
}

fn vectors_to_value(vectors: Vec<[f64; 3]>) -> Value {
    Value::List(vectors.into_iter().map(Value::Vector).collect())
}

fn polygons_to_value(polygons: Vec<Vec<[f64; 3]>>) -> Value {
    Value::List(
        polygons
            .into_iter()
            .map(|polygon| Value::List(polygon.into_iter().map(Value::Point).collect()))
            .collect(),
    )
}

fn nested_points_to_value(points: Vec<Vec<[f64; 3]>>) -> Value {
    Value::List(
        points
            .into_iter()
            .map(|row| Value::List(row.into_iter().map(Value::Point).collect()))
            .collect(),
    )
}

fn random_point_in_rectangle(section: &RectangleSection, rng: &mut impl Rng) -> [f64; 3] {
    let u: f64 = rng.random();
    let v: f64 = rng.random();
    let x = lerp(section.min_x, section.max_x, u);
    let y = lerp(section.min_y, section.max_y, v);
    section.plane.apply(x, y, 0.0)
}

fn random_point_in_box(region: &BoxRegion, rng: &mut impl Rng) -> [f64; 3] {
    [
        lerp(region.min[0], region.max[0], rng.random()),
        lerp(region.min[1], region.max[1], rng.random()),
        lerp(region.min[2], region.max[2], rng.random()),
    ]
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
fn extract_rectangle_section(
    value: Option<&Value>,
    context: &str,
) -> Result<RectangleSection, ComponentError> {
    let mut plane = value
        .and_then(|v| coerce_plane(v, context).ok())
        .unwrap_or_default();
    let points = value
        .map(|v| collect_points(Some(v), context))
        .transpose()?
        .unwrap_or_default();
    if points.len() >= 3 {
        plane = Plane::from_points(points[0], points[1], points[2]);
    }
    if points.is_empty() {
        return Ok(RectangleSection::default());
    }
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for point in points {
        let coords = plane.coordinates(point);
        min_x = min_x.min(coords[0]);
        max_x = max_x.max(coords[0]);
        min_y = min_y.min(coords[1]);
        max_y = max_y.max(coords[1]);
    }
    if !min_x.is_finite() || !max_x.is_finite() || !min_y.is_finite() || !max_y.is_finite() {
        return Ok(RectangleSection::default());
    }
    Ok(RectangleSection {
        plane,
        min_x,
        max_x,
        min_y,
        max_y,
    })
}

fn extract_box_region(value: Option<&Value>, context: &str) -> Result<BoxRegion, ComponentError> {
    let points = value
        .map(|v| collect_points(Some(v), context))
        .transpose()?
        .unwrap_or_default();
    if points.is_empty() {
        return Ok(BoxRegion::default());
    }
    let mut min = points[0];
    let mut max = points[0];
    for point in points.iter().skip(1) {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    Ok(BoxRegion { min, max })
}

fn gather_geometry(
    value: Option<&Value>,
    context: &str,
) -> Result<GeometryCollection, ComponentError> {
    let mut collection = GeometryCollection::default();
    if let Some(value) = value {
        gather_geometry_into(value, context, &mut collection)?;
    }
    Ok(collection)
}

fn gather_geometry_into(
    value: &Value,
    context: &str,
    collection: &mut GeometryCollection,
) -> Result<(), ComponentError> {
    match value {
        Value::Point(point) | Value::Vector(point) => {
            collection.push_point(*point);
            Ok(())
        }
        Value::CurveLine { p1, p2 } => {
            collection.push_line(*p1, *p2);
            Ok(())
        }
        Value::Surface { vertices, faces } => {
            for vertex in vertices {
                collection.push_point(*vertex);
            }
            for face in faces {
                if face.len() < 3 {
                    continue;
                }
                let a = vertices[face[0] as usize];
                for window in face.windows(2) {
                    let b = vertices[window[0] as usize];
                    let c = vertices[window[1] as usize];
                    collection.push_triangle(a, b, c);
                }
                let last = vertices[*face.last().unwrap() as usize];
                let second = vertices[face[1] as usize];
                collection.push_triangle(a, last, second);
            }
            Ok(())
        }
        Value::List(values) => {
            for entry in values {
                gather_geometry_into(entry, context, collection)?;
            }
            Ok(())
        }
        Value::Number(number) => {
            collection.push_point([*number, 0.0, 0.0]);
            Ok(())
        }
        Value::Boolean(boolean) => {
            collection.push_point([if *boolean { 1.0 } else { 0.0 }, 0.0, 0.0]);
            Ok(())
        }
        Value::Text(text) => {
            if let Ok(parsed) = text.trim().parse::<f64>() {
                collection.push_point([parsed, 0.0, 0.0]);
                Ok(())
            } else {
                Err(ComponentError::new(format!(
                    "{} kon tekst '{}' niet als geometrie interpreteren",
                    context, text
                )))
            }
        }
        other => Err(ComponentError::new(format!(
            "{} bevat een niet-ondersteund type: {}",
            context,
            other.kind()
        ))),
    }
}

#[derive(Debug, Default)]
struct GeometryCollection {
    samplers: Vec<Sampler>,
    fallback_points: Vec<[f64; 3]>,
    bounds: BoundingBox,
}

impl GeometryCollection {
    fn push_point(&mut self, point: [f64; 3]) {
        self.samplers.push(Sampler::Point(point));
        self.fallback_points.push(point);
        self.bounds.include(point);
    }

    fn push_line(&mut self, start: [f64; 3], end: [f64; 3]) {
        self.samplers.push(Sampler::Line { start, end });
        self.fallback_points.push(start);
        self.fallback_points.push(end);
        self.bounds.include(start);
        self.bounds.include(end);
    }

    fn push_triangle(&mut self, a: [f64; 3], b: [f64; 3], c: [f64; 3]) {
        self.samplers.push(Sampler::Triangle { a, b, c });
        self.fallback_points.push(a);
        self.fallback_points.push(b);
        self.fallback_points.push(c);
        self.bounds.include(a);
        self.bounds.include(b);
        self.bounds.include(c);
    }

    fn sample(&self, rng: &mut impl Rng) -> Option<[f64; 3]> {
        if self.samplers.is_empty() {
            return if self.bounds.valid {
                Some(self.bounds.random_point(rng))
            } else {
                None
            };
        }
        let index = rng.random_range(0..self.samplers.len());
        Some(self.samplers[index].sample(rng))
    }
}

#[derive(Debug, Clone, Copy)]
struct BoxRegion {
    min: [f64; 3],
    max: [f64; 3],
}

impl Default for BoxRegion {
    fn default() -> Self {
        Self {
            min: [-0.5, -0.5, -0.5],
            max: [0.5, 0.5, 0.5],
        }
    }
}

impl BoxRegion {
    fn is_degenerate(&self) -> bool {
        (self.max[0] - self.min[0]).abs() < EPSILON
            && (self.max[1] - self.min[1]).abs() < EPSILON
            && (self.max[2] - self.min[2]).abs() < EPSILON
    }
}

#[derive(Debug, Clone, Copy)]
struct RectangleSection {
    plane: Plane,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
}

impl Default for RectangleSection {
    fn default() -> Self {
        Self {
            plane: Plane::default(),
            min_x: -0.5,
            max_x: 0.5,
            min_y: -0.5,
            max_y: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Offset {
    x: f64,
    y: f64,
}

impl Offset {
    fn centered(count_x: usize, count_y: usize, size_x: f64, size_y: f64) -> Self {
        Self {
            x: -((count_x - 1) as f64 * size_x) / 2.0,
            y: -((count_y - 1) as f64 * size_y) / 2.0,
        }
    }
}

#[derive(Debug)]
struct HexGrid {
    rows: Vec<Vec<[f64; 3]>>,
    cells: Vec<Vec<[f64; 3]>>,
}

#[derive(Debug)]
struct RectangularGrid {
    points: Vec<Vec<[f64; 3]>>,
    cells: Vec<Vec<[f64; 3]>>,
}

#[derive(Debug)]
struct RadialGrid {
    rings: Vec<Vec<[f64; 3]>>,
    cells: Vec<Vec<[f64; 3]>>,
}

#[derive(Debug)]
struct TriangularGrid {
    points: Vec<Vec<[f64; 3]>>,
    cells: Vec<Vec<[f64; 3]>>,
}

#[derive(Debug, Clone, Copy)]
struct Plane {
    origin: [f64; 3],
    x_axis: [f64; 3],
    y_axis: [f64; 3],
    z_axis: [f64; 3],
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            origin: [0.0, 0.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
            z_axis: [0.0, 0.0, 1.0],
        }
    }
}

impl Plane {
    fn normalize_axes(
        origin: [f64; 3],
        x_axis: [f64; 3],
        y_axis: [f64; 3],
        z_axis: [f64; 3],
    ) -> Self {
        let z = safe_normalized(z_axis)
            .map(|(vector, _)| vector)
            .unwrap_or([0.0, 0.0, 1.0]);

        let mut x = safe_normalized(x_axis)
            .map(|(vector, _)| vector)
            .unwrap_or_else(|| orthogonal_vector(z));

        let mut y = safe_normalized(y_axis)
            .map(|(vector, _)| vector)
            .unwrap_or_else(|| normalize(cross(z, x)));

        x = normalize(cross(y, z));
        y = normalize(cross(z, x));

        Self {
            origin,
            x_axis: x,
            y_axis: y,
            z_axis: z,
        }
    }

    fn from_points(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Self {
        let ab = subtract(b, a);
        let ac = subtract(c, a);
        let normal = cross(ab, ac);
        if vector_length_squared(normal) < EPSILON {
            return Self::default();
        }
        let x_axis = if vector_length_squared(ab) < EPSILON {
            orthogonal_vector(normal)
        } else {
            normalize(ab)
        };
        let z_axis = normalize(normal);
        let y_axis = normalize(cross(z_axis, x_axis));
        Self::normalize_axes(a, x_axis, y_axis, z_axis)
    }

    fn apply(&self, u: f64, v: f64, w: f64) -> [f64; 3] {
        add(
            add(
                add(self.origin, scale(self.x_axis, u)),
                scale(self.y_axis, v),
            ),
            scale(self.z_axis, w),
        )
    }

    fn coordinates(&self, point: [f64; 3]) -> [f64; 3] {
        let relative = subtract(point, self.origin);
        [
            dot(relative, self.x_axis),
            dot(relative, self.y_axis),
            dot(relative, self.z_axis),
        ]
    }
}

impl From<Line> for Plane {
    fn from(line: Line) -> Self {
        let direction = line.direction();
        if vector_length_squared(direction) < EPSILON {
            return Self::default();
        }
        let x_axis = normalize(direction);
        let y_axis = orthogonal_vector(x_axis);
        let z_axis = normalize(cross(x_axis, y_axis));
        Self::normalize_axes(line.start, x_axis, y_axis, z_axis)
    }
}

#[derive(Debug, Clone, Copy)]
struct Line {
    start: [f64; 3],
    end: [f64; 3],
}

impl Line {
    fn direction(self) -> [f64; 3] {
        subtract(self.end, self.start)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct BoundingBox {
    min: [f64; 3],
    max: [f64; 3],
    valid: bool,
}

impl BoundingBox {
    fn include(&mut self, point: [f64; 3]) {
        if !self.valid {
            self.min = point;
            self.max = point;
            self.valid = true;
            return;
        }
        for axis in 0..3 {
            self.min[axis] = self.min[axis].min(point[axis]);
            self.max[axis] = self.max[axis].max(point[axis]);
        }
    }

    fn random_point(&self, rng: &mut impl Rng) -> [f64; 3] {
        [
            lerp(self.min[0], self.max[0], rng.random()),
            lerp(self.min[1], self.max[1], rng.random()),
            lerp(self.min[2], self.max[2], rng.random()),
        ]
    }
}

#[derive(Debug, Clone, Copy)]
enum Sampler {
    Point([f64; 3]),
    Line {
        start: [f64; 3],
        end: [f64; 3],
    },
    Triangle {
        a: [f64; 3],
        b: [f64; 3],
        c: [f64; 3],
    },
}

impl Sampler {
    fn sample(self, rng: &mut impl Rng) -> [f64; 3] {
        match self {
            Self::Point(point) => point,
            Self::Line { start, end } => {
                let t: f64 = rng.random();
                add(start, scale(subtract(end, start), t))
            }
            Self::Triangle { a, b, c } => {
                let r1: f64 = rng.random();
                let r2: f64 = rng.random();
                let sqrt_r1 = r1.sqrt();
                let u = 1.0 - sqrt_r1;
                let v = r2 * sqrt_r1;
                let w = 1.0 - u - v;
                add(add(scale(a, u), scale(b, v)), scale(c, w))
            }
        }
    }
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn subtract(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(vector: [f64; 3], factor: f64) -> [f64; 3] {
    [vector[0] * factor, vector[1] * factor, vector[2] * factor]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn vector_length_squared(vector: [f64; 3]) -> f64 {
    dot(vector, vector)
}

fn normalize(vector: [f64; 3]) -> [f64; 3] {
    if let Some((normalized, _)) = safe_normalized(vector) {
        normalized
    } else {
        [0.0, 0.0, 0.0]
    }
}

fn safe_normalized(vector: [f64; 3]) -> Option<([f64; 3], f64)> {
    let length = vector_length_squared(vector).sqrt();
    if length < EPSILON {
        None
    } else {
        Some((scale(vector, 1.0 / length), length))
    }
}

fn orthogonal_vector(vector: [f64; 3]) -> [f64; 3] {
    let abs_x = vector[0].abs();
    let abs_y = vector[1].abs();
    let abs_z = vector[2].abs();
    if abs_x <= abs_y && abs_x <= abs_z {
        normalize([0.0, -vector[2], vector[1]])
    } else if abs_y <= abs_x && abs_y <= abs_z {
        normalize([-vector[2], 0.0, vector[0]])
    } else {
        normalize([vector[1], -vector[0], 0.0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xy_plane_value() -> Value {
        Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
        ])
    }

    #[test]
    fn hexagonal_grid_generates_expected_cells() {
        let inputs = vec![
            xy_plane_value(),
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(2.0),
        ];
        let outputs = evaluate_hexagonal_grid(&inputs).expect("hex grid");
        match outputs.get(PIN_OUTPUT_POINTS) {
            Some(Value::List(rows)) => {
                assert_eq!(rows.len(), 2);
                for row in rows {
                    match row {
                        Value::List(points) => assert_eq!(points.len(), 2),
                        _ => panic!("expected row list"),
                    }
                }
            }
            _ => panic!("expected rows"),
        }
        match outputs.get(PIN_OUTPUT_CELLS) {
            Some(Value::List(cells)) => assert_eq!(cells.len(), 4),
            _ => panic!("expected cells"),
        }
    }

    #[test]
    fn rectangular_grid_builds_cells_and_points() {
        let inputs = vec![
            xy_plane_value(),
            Value::Number(1.0),
            Value::Number(1.5),
            Value::Number(2.0),
            Value::Number(1.0),
        ];
        let outputs = evaluate_rectangular_grid(&inputs).expect("rect grid");
        match outputs.get(PIN_OUTPUT_POINTS) {
            Some(Value::List(rows)) => {
                assert_eq!(rows.len(), 2);
                for row in rows {
                    match row {
                        Value::List(points) => assert_eq!(points.len(), 3),
                        _ => panic!("expected row list"),
                    }
                }
            }
            _ => panic!("expected points"),
        }
        match outputs.get(PIN_OUTPUT_CELLS) {
            Some(Value::List(cells)) => {
                assert_eq!(cells.len(), 2);
                for cell in cells {
                    match cell {
                        Value::List(vertices) => assert_eq!(vertices.len(), 5),
                        _ => panic!("expected polyline"),
                    }
                }
            }
            _ => panic!("expected cells"),
        }
    }

    #[test]
    fn radial_grid_contains_center_ring() {
        let inputs = vec![
            xy_plane_value(),
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(6.0),
        ];
        let outputs = evaluate_radial_grid(&inputs).expect("radial grid");
        match outputs.get(PIN_OUTPUT_POINTS) {
            Some(Value::List(rings)) => {
                assert_eq!(rings.len(), 3);
                match &rings[0] {
                    Value::List(center) => assert_eq!(center.len(), 1),
                    _ => panic!("expected center ring"),
                }
            }
            _ => panic!("expected rings"),
        }
        match outputs.get(PIN_OUTPUT_CELLS) {
            Some(Value::List(cells)) => assert_eq!(cells.len(), 12),
            _ => panic!("expected cells"),
        }
    }

    #[test]
    fn triangular_grid_outputs_twice_cell_count() {
        let inputs = vec![
            xy_plane_value(),
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(1.0),
        ];
        let outputs = evaluate_triangular_grid(&inputs).expect("tri grid");
        match outputs.get(PIN_OUTPUT_CELLS) {
            Some(Value::List(cells)) => assert_eq!(cells.len(), 4),
            _ => panic!("expected triangles"),
        }
    }

    #[test]
    fn populate_geometry_samples_surface() {
        let geometry = Value::Surface {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            faces: vec![vec![0, 1, 2]],
        };
        let inputs = vec![
            geometry,
            Value::Number(5.0),
            Value::Number(42.0),
            Value::List(vec![]),
        ];
        let outputs = evaluate_populate_geometry(&inputs).expect("populate geom");
        let Value::List(points) = outputs.get(PIN_OUTPUT_POPULATION).expect("population") else {
            panic!("expected point list");
        };
        assert_eq!(points.len(), 5);
        for point in points {
            let Value::Point(coords) = point else {
                panic!("expected point value");
            };
            assert!(coords[0] >= -1e-6 && coords[0] <= 1.0 + 1e-6);
            assert!(coords[1] >= -1e-6 && coords[1] <= 1.0 + 1e-6);
            assert!(coords[2].abs() < 1e-6);
        }
    }

    #[test]
    fn populate_2d_respects_existing_points() {
        let rectangle = Value::List(vec![
            Value::Point([-1.0, -1.0, 0.0]),
            Value::Point([1.0, -1.0, 0.0]),
            Value::Point([-1.0, 1.0, 0.0]),
            Value::Point([1.0, 1.0, 0.0]),
        ]);
        let inputs = vec![
            rectangle,
            Value::Number(3.0),
            Value::Number(0.0),
            Value::List(vec![Value::Point([2.0, 2.0, 0.0])]),
        ];
        let outputs = evaluate_populate_2d(&inputs).expect("populate 2d");
        let Value::List(points) = outputs.get(PIN_OUTPUT_POPULATION).expect("population") else {
            panic!("expected point list");
        };
        assert_eq!(points.len(), 3);
        let Value::Point(first) = &points[0] else {
            panic!("expected point");
        };
        assert_eq!(*first, [2.0, 2.0, 0.0]);
        for point in points.iter().skip(1) {
            let Value::Point(coords) = point else {
                panic!("expected point");
            };
            assert!(coords[2].abs() < 1e-6);
        }
    }

    #[test]
    fn populate_3d_generates_points_in_box() {
        let region = Value::List(vec![
            Value::Point([-1.0, -1.0, -1.0]),
            Value::Point([1.0, 1.0, 1.0]),
        ]);
        let inputs = vec![
            region,
            Value::Number(4.0),
            Value::Number(0.0),
            Value::List(vec![]),
        ];
        let outputs = evaluate_populate_3d(&inputs).expect("populate 3d");
        let Value::List(points) = outputs.get(PIN_OUTPUT_POPULATION).expect("population") else {
            panic!("expected point list");
        };
        assert_eq!(points.len(), 4);
        for point in points {
            let Value::Point(coords) = point else {
                panic!("expected point");
            };
            for component in coords {
                assert!(*component >= -1.0 - 1e-6 && *component <= 1.0 + 1e-6);
            }
        }
    }

    #[test]
    fn freeform_cloud_falls_back_to_geometry() {
        let geometry = Value::List(vec![Value::Point([0.0, 0.0, 0.0])]);
        let inputs = vec![geometry, Value::Number(3.0), Value::Number(5.0)];
        let outputs = evaluate_freeform_cloud(&inputs).expect("freeform");
        let Value::List(points) = outputs.get(PIN_OUTPUT_CLOUD).expect("cloud") else {
            panic!("expected point list");
        };
        assert_eq!(points.len(), 3);
        for point in points {
            let Value::Point(coords) = point else {
                panic!("expected point");
            };
            assert_eq!(*coords, [0.0, 0.0, 0.0]);
        }
    }

    #[test]
    fn spherical_cloud_produces_normals() {
        let inputs = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(1.0),
        ];
        let outputs = evaluate_spherical_cloud(&inputs).expect("spherical");
        let Value::List(cloud) = outputs.get(PIN_OUTPUT_CLOUD).expect("cloud") else {
            panic!("expected cloud");
        };
        let Value::List(normals) = outputs.get(PIN_OUTPUT_NORMALS).expect("normals") else {
            panic!("expected normals");
        };
        assert_eq!(cloud.len(), 3);
        assert_eq!(normals.len(), 3);
        for (point, normal) in cloud.iter().zip(normals.iter()) {
            let Value::Point(coords) = point else {
                panic!("expected point");
            };
            let Value::Vector(dir) = normal else {
                panic!("expected vector");
            };
            let radius =
                (coords[0] * coords[0] + coords[1] * coords[1] + coords[2] * coords[2]).sqrt();
            assert!((radius - 2.0).abs() < 1e-6);
            let length = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
            assert!((length - 1.0).abs() < 1e-6);
        }
    }
}
