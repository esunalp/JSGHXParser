//! Implementaties van Grasshopper "Surface → Freeform" componenten.

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};

use crate::graph::node::MetaMap;
use crate::graph::value::{Domain, Value};

use super::{Component, ComponentError, ComponentResult, coerce};

const PIN_OUTPUT_SURFACE: &str = "S";
const PIN_OUTPUT_EXTRUSION: &str = "E";
const PIN_OUTPUT_OPTIONS: &str = "O";
const PIN_OUTPUT_PATCH: &str = "P";
const PIN_OUTPUT_PIPE: &str = "P";
const PIN_OUTPUT_LOFT: &str = "L";
const PIN_OUTPUT_SHAPE: &str = "S";

const EPSILON: f64 = 1e-9;

/// Beschikbare componentvarianten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    FitLoft,
    EdgeSurface,
    ExtrudeAlong,
    LoftOptions,
    SurfaceFromPoints,
    Patch,
    ControlPointLoft,
    SumSurface,
    RuledSurface,
    NetworkSurface,
    Sweep2,
    PipeVariable,
    ExtrudeLinear,
    Loft,
    ExtrudeAngled,
    Sweep1,
    ExtrudePoint,
    Pipe,
    FourPointSurface,
    FragmentPatch,
    Revolution,
    BoundarySurfaces,
    RailRevolution,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst van componentregistraties voor de surface-freeform componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{342aa574-1327-4bc2-8daf-203da2a45676}"],
        names: &["Fit Loft", "FitLoft"],
        kind: ComponentKind::FitLoft,
    },
    Registration {
        guids: &["{36132830-e2ef-4476-8ea1-6a43922344f0}"],
        names: &["Edge Surface", "EdgeSrf"],
        kind: ComponentKind::EdgeSurface,
    },
    Registration {
        guids: &["{38a5638b-6d01-4417-bf11-976d925f8a71}"],
        names: &["Extrude Along", "ExtrCrv"],
        kind: ComponentKind::ExtrudeAlong,
    },
    Registration {
        guids: &["{45f19d16-1c9f-4b0f-a9a6-45a77f3d206c}"],
        names: &["Loft Options", "Loft Opt"],
        kind: ComponentKind::LoftOptions,
    },
    Registration {
        guids: &["{4b04a1e1-cddf-405d-a7db-335aaa940541}"],
        names: &["Surface From Points", "SrfGrid"],
        kind: ComponentKind::SurfaceFromPoints,
    },
    Registration {
        guids: &["{57b2184c-8931-4e70-9220-612ec5b3809a}"],
        names: &["Patch"],
        kind: ComponentKind::Patch,
    },
    Registration {
        guids: &["{5c270622-ee80-45a4-b07a-bd8ffede92a2}"],
        names: &["Control Point Loft", "CPLoft"],
        kind: ComponentKind::ControlPointLoft,
    },
    Registration {
        guids: &["{5e33c760-adcd-4235-b1dd-05cf72eb7a38}"],
        names: &["Sum Surface", "SumSrf"],
        kind: ComponentKind::SumSurface,
    },
    Registration {
        guids: &["{6e5de495-ba76-42d0-9985-a5c265e9aeca}"],
        names: &["Ruled Surface", "RuleSrf"],
        kind: ComponentKind::RuledSurface,
    },
    Registration {
        guids: &["{71506fa8-9bf0-432d-b897-b2e0c5ac316c}"],
        names: &["Network Surface", "NetSurf"],
        kind: ComponentKind::NetworkSurface,
    },
    Registration {
        guids: &["{75164624-395a-4d24-b60b-6bf91cab0194}"],
        names: &["Sweep2", "Swp2"],
        kind: ComponentKind::Sweep2,
    },
    Registration {
        guids: &["{888f9c3c-f1e1-4344-94b0-5ee6a45aee11}"],
        names: &["Pipe Variable", "VPipe"],
        kind: ComponentKind::PipeVariable,
    },
    Registration {
        guids: &["{8efd5eb9-a896-486e-9f98-d8d1a07a49f3}"],
        names: &["Extrude Linear"],
        kind: ComponentKind::ExtrudeLinear,
    },
    Registration {
        guids: &["{a7a41d0a-2188-4f7a-82cc-1a2c4e4ec850}"],
        names: &["Loft"],
        kind: ComponentKind::Loft,
    },
    Registration {
        guids: &["{ae57e09b-a1e4-4d05-8491-abd232213bc9}"],
        names: &["Extrude Angled", "ExtrAng"],
        kind: ComponentKind::ExtrudeAngled,
    },
    Registration {
        guids: &["{bb6666e7-d0f4-41ec-a257-df2371619f13}"],
        names: &["Sweep1", "Swp1"],
        kind: ComponentKind::Sweep1,
    },
    Registration {
        guids: &["{be6636b2-2f1a-4d42-897b-fdef429b6f17}"],
        names: &["Extrude Point"],
        kind: ComponentKind::ExtrudePoint,
    },
    Registration {
        guids: &["{c277f778-6fdf-4890-8f78-347efb23c406}"],
        names: &["Pipe"],
        kind: ComponentKind::Pipe,
    },
    Registration {
        guids: &["{cdee962f-4202-456b-a1b4-f3ed9aa0dc29}"],
        names: &["Revolution", "RevSrf"],
        kind: ComponentKind::Revolution,
    },
    Registration {
        guids: &["{d51e9b65-aa4e-4fd6-976c-cef35d421d05}"],
        names: &["Boundary Surfaces", "Boundary"],
        kind: ComponentKind::BoundarySurfaces,
    },
    Registration {
        guids: &["{d8d68c35-f869-486d-adf3-69ee3cc2d501}"],
        names: &["Rail Revolution", "RailRev"],
        kind: ComponentKind::RailRevolution,
    },
    Registration {
        guids: &["{cb56b26c-2595-4d03-bdb2-eb2e6aeba82d}"],
        names: &["Fragment Patch"],
        kind: ComponentKind::FragmentPatch,
    },
    Registration {
        guids: &["{c77a8b3b-c569-4d81-9b59-1c27299a1c45}"],
        names: &["4Point Surface", "Srf4Pt"],
        kind: ComponentKind::FourPointSurface,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::FitLoft => evaluate_loft(inputs, "Fit Loft", PIN_OUTPUT_SURFACE),
            Self::EdgeSurface => evaluate_edge_surface(inputs),
            Self::ExtrudeAlong => evaluate_extrude_along(inputs),
            Self::LoftOptions => evaluate_loft_options(inputs),
            Self::SurfaceFromPoints => evaluate_surface_from_points(inputs, "Surface From Points"),
            Self::Patch => evaluate_patch(inputs),
            Self::ControlPointLoft => {
                evaluate_loft(inputs, "Control Point Loft", PIN_OUTPUT_SURFACE)
            }
            Self::SumSurface => evaluate_sum_surface(inputs),
            Self::RuledSurface => evaluate_ruled_surface(inputs),
            Self::NetworkSurface => evaluate_network_surface(inputs),
            Self::Sweep2 => evaluate_sweep_two(inputs),
            Self::PipeVariable => evaluate_pipe_variable(inputs),
            Self::ExtrudeLinear => evaluate_extrude_linear(inputs),
            Self::Loft => evaluate_loft(inputs, "Loft", PIN_OUTPUT_LOFT),
            Self::ExtrudeAngled => evaluate_extrude_angled(inputs),
            Self::Sweep1 => evaluate_sweep_one(inputs),
            Self::ExtrudePoint => evaluate_extrude_point(inputs),
            Self::Pipe => evaluate_pipe(inputs),
            Self::FourPointSurface => evaluate_four_point_surface(inputs),
            Self::FragmentPatch => evaluate_fragment_patch(inputs),
            Self::Revolution => evaluate_revolution(inputs),
            Self::BoundarySurfaces => evaluate_boundary_surfaces(inputs),
            Self::RailRevolution => evaluate_rail_revolution(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::FitLoft => "Fit Loft",
            Self::EdgeSurface => "Edge Surface",
            Self::ExtrudeAlong => "Extrude Along",
            Self::LoftOptions => "Loft Options",
            Self::SurfaceFromPoints => "Surface From Points",
            Self::Patch => "Patch",
            Self::ControlPointLoft => "Control Point Loft",
            Self::SumSurface => "Sum Surface",
            Self::RuledSurface => "Ruled Surface",
            Self::NetworkSurface => "Network Surface",
            Self::Sweep2 => "Sweep2",
            Self::PipeVariable => "Pipe Variable",
            Self::ExtrudeLinear => "Extrude Linear",
            Self::Loft => "Loft",
            Self::ExtrudeAngled => "Extrude Angled",
            Self::Sweep1 => "Sweep1",
            Self::ExtrudePoint => "Extrude Point",
            Self::Pipe => "Pipe",
            Self::FourPointSurface => "4Point Surface",
            Self::FragmentPatch => "Fragment Patch",
            Self::Revolution => "Revolution",
            Self::BoundarySurfaces => "Boundary Surfaces",
            Self::RailRevolution => "Rail Revolution",
        }
    }
}

fn unify_curve_directions(polylines: &mut [Vec<[f64; 3]>]) {
    if polylines.len() < 2 {
        return;
    }

    // Stap 1: Classificeer curves en vind gesloten curves
    let closed_indices: Vec<usize> = polylines
        .iter()
        .enumerate()
        .filter(|(_, p)| is_closed(p))
        .map(|(i, _)| i)
        .collect();

    // Stap 2: Als er gesloten curves zijn, standaardiseer hun richting
    if !closed_indices.is_empty() {
        // Neem de eerste gesloten curve als referentie.
        let first_closed_idx = closed_indices[0];
        let reference_normal = polyline_normal(&polylines[first_closed_idx]);
        let reference_winding =
            polyline_winding_direction(&polylines[first_closed_idx], reference_normal);

        // Streef naar een positieve winding (CCW). Als de referentie zelf CW is, keer de normaal om.
        let target_normal = if reference_winding < 0.0 {
            [
                -reference_normal[0],
                -reference_normal[1],
                -reference_normal[2],
            ]
        } else {
            reference_normal
        };

        // Keer elke gesloten curve om die niet overeenkomt met de doelrichting.
        for &i in &closed_indices {
            let winding = polyline_winding_direction(&polylines[i], target_normal);
            if winding < 0.0 {
                polylines[i].reverse();
            }
        }
    }

    // Stap 3: Oriënteer open curves ten opzichte van hun voorganger voor een vloeiende overgang
    for i in 1..polylines.len() {
        let (prev_curves, current_slice) = polylines.split_at_mut(i);
        let prev = &prev_curves[i - 1];
        let current = &mut current_slice[0];

        if prev.is_empty() || current.is_empty() {
            continue;
        }

        // Alleen open curves uitlijnen ten opzichte van een vorige open curve.
        // Gesloten curves hebben al een vaste oriëntatie en een arbitrair start-/eindpunt,
        // dus een op afstand gebaseerde uitlijning is onbetrouwbaar.
        if !is_closed(current) && !is_closed(prev) {
            let p_end = prev.last().unwrap();
            let c_start = current.first().unwrap();
            let c_end = current.last().unwrap();

            let dist_as_is = distance(*p_end, *c_start);
            let dist_reversed = distance(*p_end, *c_end);

            if dist_reversed < dist_as_is {
                current.reverse();
            }
        }
    }
}

fn evaluate_loft(inputs: &[Value], component: &str, output: &str) -> ComponentResult {
    let curves_value = expect_input(inputs, 0, component, "curveverzameling")?;
    let mut polylines = collect_ruled_surface_curves(curves_value)?;
    if polylines.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} vereist minimaal twee sectiecurves"
        )));
    }

    unify_curve_directions(&mut polylines);

    // Bepaal het doelaantal punten door het maximum van alle polylines te nemen.
    let target_count = polylines.iter().map(|p| p.len()).max().unwrap_or(0);

    if target_count < 2 {
        return Err(ComponentError::new(format!(
            "{component} kon geen curves met voldoende punten vinden"
        )));
    }

    // Hersample alle polylines naar het doelaantal.
    let resampled_polylines: Vec<Vec<[f64; 3]>> = polylines
        .iter()
        .map(|p| {
            // Maak een dummy polyline met het doelaantal punten om de resample-functie te gebruiken.
            let dummy_target = vec![[0.0; 3]; target_count];
            super::curve_sampler::resample_polylines(p, &dummy_target).0
        })
        .collect();

    let mut vertices = Vec::new();
    let mut faces: Vec<Vec<u32>> = Vec::new();

    for polyline in &resampled_polylines {
        vertices.extend_from_slice(polyline);
    }

    let num_curves = resampled_polylines.len();
    let num_points_per_curve = target_count;

    for i in 0..(num_curves - 1) {
        for j in 0..(num_points_per_curve - 1) {
            let base_idx = (i * num_points_per_curve + j) as u32;
            let next_in_row_idx = base_idx + 1;
            let base_in_next_curve_idx = ((i + 1) * num_points_per_curve + j) as u32;
            let next_in_next_curve_idx = base_in_next_curve_idx + 1;

            faces.push(vec![base_idx, next_in_next_curve_idx, next_in_row_idx]);
            faces.push(vec![
                base_idx,
                base_in_next_curve_idx,
                next_in_next_curve_idx,
            ]);
        }
    }

    into_output(output, Value::Surface { vertices, faces })
}

fn evaluate_edge_surface(inputs: &[Value]) -> ComponentResult {
    let component = "Edge Surface";
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Edge Surface vereist minimaal twee randcurves",
        ));
    }

    let mut points = Vec::new();
    for value in inputs.iter().take(4) {
        let segments = coerce::coerce_curve_segments(value)?;
        for (start, end) in segments {
            points.push(start);
            points.push(end);
        }
    }

    if points.len() < 4 {
        return Err(ComponentError::new(
            "Edge Surface kon onvoldoende punten uit de randen halen",
        ));
    }

    let surface = create_surface_from_points(&points, component)?;
    into_output(PIN_OUTPUT_SURFACE, surface)
}

fn evaluate_extrude_along(inputs: &[Value]) -> ComponentResult {
    let component = "Extrude Along";
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Extrude Along vereist een basis en een railcurve",
        ));
    }
    let base_segments = coerce::coerce_curve_segments(&inputs[0])?;
    if base_segments.is_empty() {
        return Err(ComponentError::new(
            "Extrude Along kon geen basiscurve herkennen",
        ));
    }
    let rail_segments = coerce::coerce_curve_segments(&inputs[1])?;
    let Some((rail_start, rail_end)) = rail_segments.first() else {
        return Err(ComponentError::new(
            "Extrude Along kon geen railcurve herkennen",
        ));
    };
    let direction = subtract_points(*rail_end, *rail_start);
    if is_zero_vector(direction) {
        return Err(ComponentError::new(
            "Extrude Along vereist een rail met lengte",
        ));
    }

    let mut points = Vec::new();
    for (start, end) in base_segments {
        points.push(start);
        points.push(end);
        points.push(add_vector(start, direction));
        points.push(add_vector(end, direction));
    }

    let surface = create_surface_from_points(&points, component)?;
    into_output(PIN_OUTPUT_EXTRUSION, surface)
}

fn evaluate_loft_options(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 5 {
        return Err(ComponentError::new(
            "Loft Options vereist geslotenheid, seam-aanpassing, rebuild, refit en type",
        ));
    }

    let closed = coerce_bool(&inputs[0], "Loft Options", "Closed")?;
    let adjust = coerce_bool(&inputs[1], "Loft Options", "Adjust")?;
    let rebuild = coerce_number(&inputs[2], "Loft Options", "Rebuild")?;
    let refit = coerce_number(&inputs[3], "Loft Options", "Refit")?;
    let loft_type = coerce_number(&inputs[4], "Loft Options", "Type")?;

    let summary = format!(
        "{{\"closed\":{closed},\"adjust\":{adjust},\"rebuild\":{rebuild},\"refit\":{refit},\"type\":{loft_type}}}"
    );

    into_output(PIN_OUTPUT_OPTIONS, Value::Text(summary))
}

fn evaluate_surface_from_points(inputs: &[Value], component: &str) -> ComponentResult {
    let points_value = expect_input(inputs, 0, component, "puntverzameling")?;
    let points = collect_points(points_value, component)?;
    if points.len() < 3 {
        return Err(ComponentError::new(format!(
            "{component} vereist minimaal drie punten"
        )));
    }

    if let Some(value) = inputs.get(1) {
        coerce_number(value, component, "U Count")?;
    }
    if let Some(value) = inputs.get(2) {
        coerce_bool(value, component, "Interpolate")?;
    }

    let surface = create_surface_from_points(&points, component)?;
    into_output(PIN_OUTPUT_SURFACE, surface)
}

fn evaluate_patch(inputs: &[Value]) -> ComponentResult {
    let component = "Patch";
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Patch vereist minimaal een lijst met curves",
        ));
    }

    let mut points = Vec::new();
    points.extend(
        coerce::coerce_curve_segments(&inputs[0])?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );

    if let Some(points_value) = inputs.get(1) {
        points.extend(collect_points(points_value, component)?);
    }
    if points.len() < 3 {
        return Err(ComponentError::new(
            "Patch kon onvoldoende inputpunten verzamelen",
        ));
    }

    if let Some(value) = inputs.get(2) {
        coerce_number(value, component, "Spans")?;
    }
    if let Some(value) = inputs.get(3) {
        coerce_number(value, component, "Flexibility")?;
    }
    if let Some(value) = inputs.get(4) {
        coerce_bool(value, component, "Trim")?;
    }

    let surface = create_surface_from_points(&points, component)?;
    into_output(PIN_OUTPUT_PATCH, surface)
}

fn evaluate_sum_surface(inputs: &[Value]) -> ComponentResult {
    let component = "Sum Surface";
    if inputs.len() < 2 {
        return Err(ComponentError::new("Sum Surface vereist twee invoercurves"));
    }

    let mut points = Vec::new();
    points.extend(
        coerce::coerce_curve_segments(&inputs[0])?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );
    points.extend(
        coerce::coerce_curve_segments(&inputs[1])?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );

    if points.len() < 4 {
        return Err(ComponentError::new(
            "Sum Surface kon onvoldoende punten verzamelen",
        ));
    }

    let surface = create_surface_from_points(&points, component)?;
    into_output(PIN_OUTPUT_SURFACE, surface)
}

fn collect_ruled_surface_curves(value: &Value) -> Result<Vec<Vec<[f64; 3]>>, ComponentError> {
    match value {
        Value::Null => Ok(Vec::new()),
        Value::CurveLine { p1, p2 } => Ok(vec![vec![*p1, *p2]]),
        Value::List(values) => {
            if values.is_empty() {
                return Ok(Vec::new());
            }

            if values.iter().all(|entry| matches!(entry, Value::Point(_))) {
                let polyline = values
                    .iter()
                    .filter_map(|entry| match entry {
                        Value::Point(point) => Some(*point),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                if polyline.len() < 2 {
                    return Ok(Vec::new());
                }
                return Ok(vec![polyline]);
            }

            if values
                .iter()
                .all(|entry| matches!(entry, Value::List(_) | Value::Null))
            {
                let mut curves = Vec::new();
                for entry in values {
                    curves.extend(collect_ruled_surface_curves(entry)?);
                }
                return Ok(curves);
            }

            let segments = coerce::coerce_curve_segments(value)?;
            Ok(group_segments_into_polylines(segments))
        }
        Value::Surface { .. } => {
            let segments = coerce::coerce_curve_segments(value)?;
            Ok(group_segments_into_polylines(segments))
        }
        other => Err(ComponentError::new(format!(
            "Ruled Surface kon invoer van type {} niet interpreteren als curve",
            other.kind()
        ))),
    }
}

fn group_segments_into_polylines(segments: Vec<([f64; 3], [f64; 3])>) -> Vec<Vec<[f64; 3]>> {
    if segments.is_empty() {
        return Vec::new();
    }

    // Maak een graaf van verbonden segmenten zodat we een consistente volgorde krijgen
    // ongeacht de volgorde van de inputsegmenten.
    let mut nodes: Vec<[f64; 3]> = Vec::new();
    let mut adjacency: Vec<Vec<usize>> = Vec::new(); // edge indices per node
    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut edge_used: Vec<bool> = Vec::new();

    fn find_or_insert_node(
        nodes: &mut Vec<[f64; 3]>,
        adjacency: &mut Vec<Vec<usize>>,
        p: [f64; 3],
    ) -> usize {
        if let Some((idx, _)) = nodes
            .iter()
            .enumerate()
            .find(|(_, existing)| points_equal(**existing, p))
        {
            idx
        } else {
            let idx = nodes.len();
            nodes.push(p);
            adjacency.push(Vec::new());
            idx
        }
    }

    for (start, end) in segments {
        let a = find_or_insert_node(&mut nodes, &mut adjacency, start);
        let b = find_or_insert_node(&mut nodes, &mut adjacency, end);
        let edge_idx = edges.len();
        edges.push((a, b));
        edge_used.push(false);
        adjacency[a].push(edge_idx);
        adjacency[b].push(edge_idx);
    }

    let mut polylines = Vec::new();

    // Greedy traversal to build each polyline from unvisited edges.
    while let Some((edge_idx, &(start, end))) = edge_used
        .iter()
        .enumerate()
        .find(|(_, used)| !**used)
        .and_then(|(i, _)| edges.get(i).map(|edge| (i, edge)))
    {
        edge_used[edge_idx] = true;

        // Kies een startnode die een open einde heeft indien beschikbaar.
        let start_node = if adjacency[start].len() == 1 {
            start
        } else if adjacency[end].len() == 1 {
            end
        } else {
            start
        };
        let mut current_node = if start_node == start { end } else { start };
        let mut prev_node = start_node;

        let mut polyline = vec![nodes[start_node], nodes[current_node]];

        loop {
            // Zoek een onbenutte edge vanaf current_node.
            let next_edge_idx = adjacency[current_node]
                .iter()
                .copied()
                .find(|&idx| !edge_used[idx] && {
                    let (a, b) = edges[idx];
                    // Vermijd direct teruggaan over dezelfde edge; kies andere richting indien mogelijk.
                    let other = if a == current_node { b } else { a };
                    !points_equal(nodes[other], nodes[prev_node])
                })
                .or_else(|| {
                    adjacency[current_node]
                        .iter()
                        .copied()
                        .find(|&idx| !edge_used[idx])
                });

            let Some(next_idx) = next_edge_idx else {
                break;
            };

            edge_used[next_idx] = true;
            let (a, b) = edges[next_idx];
            let next_node = if a == current_node { b } else { a };
            prev_node = current_node;
            current_node = next_node;

            if !points_equal(*polyline.last().unwrap(), nodes[current_node]) {
                polyline.push(nodes[current_node]);
            }
        }

        // Sluit de polyline als het een echte gesloten lus is.
        if polyline.len() > 2 && !points_equal(polyline[0], *polyline.last().unwrap()) {
            let all_degree_two = polyline.iter().all(|point| {
                nodes
                    .iter()
                    .position(|p| points_equal(*p, *point))
                    .map(|idx| adjacency[idx].len() == 2)
                    .unwrap_or(false)
            });
            if all_degree_two {
                polyline.push(polyline[0]);
            }
        }

        if polyline.len() >= 2 {
            polylines.push(polyline);
        }
    }

    polylines
}

fn points_equal(a: [f64; 3], b: [f64; 3]) -> bool {
    a.iter()
        .zip(b.iter())
        .all(|(ax, bx)| (ax - bx).abs() <= EPSILON)
}

fn evaluate_ruled_surface(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Ruled Surface vereist twee invoercurves",
        ));
    }
    let curves_a = collect_ruled_surface_curves(&inputs[0])?;
    let curves_b = collect_ruled_surface_curves(&inputs[1])?;

    if curves_a.is_empty() || curves_b.is_empty() {
        return Err(ComponentError::new(
            "Ruled Surface kon geen volledige curves interpreteren",
        ));
    }

    let target_count = match (curves_a.len(), curves_b.len()) {
        (1, b) => b,
        (a, 1) => a,
        (a, b) => a.min(b),
    };

    let mut surfaces = Vec::new();

    for idx in 0..target_count {
        let polyline_a = if curves_a.len() == 1 {
            &curves_a[0]
        } else {
            &curves_a[idx]
        };
        let polyline_b = if curves_b.len() == 1 {
            &curves_b[0]
        } else {
            &curves_b[idx]
        };

        if polyline_a.len() < 2 || polyline_b.len() < 2 {
            continue;
        }

        let (resampled_a, resampled_b) =
            super::curve_sampler::resample_polylines(polyline_a, polyline_b);

        let n = resampled_a.len();
        if n < 2 {
            continue;
        }

        let mut vertices = resampled_a;
        vertices.extend(resampled_b);

        let mut faces = Vec::with_capacity((n - 1) * 2);
        for i in 0..(n - 1) {
            let i0 = i as u32;
            let i1 = (i + 1) as u32;
            let j0 = (n + i) as u32;
            let j1 = (n + i + 1) as u32;

            faces.push(vec![i0, i1, j1]);
            faces.push(vec![i0, j1, j0]);
        }

        surfaces.push(Value::Surface { vertices, faces });
    }

    let output = match surfaces.len() {
        0 => Value::Null,
        1 => surfaces.pop().unwrap(),
        _ => Value::List(surfaces),
    };

    into_output(PIN_OUTPUT_SURFACE, output)
}

fn evaluate_network_surface(inputs: &[Value]) -> ComponentResult {
    let component = "Network Surface";
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Network Surface vereist lijsten met U- en V-curves",
        ));
    }

    let mut points = Vec::new();
    points.extend(
        coerce::coerce_curve_segments(&inputs[0])?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );
    points.extend(
        coerce::coerce_curve_segments(&inputs[1])?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );

    if points.len() < 6 {
        return Err(ComponentError::new(
            "Network Surface vereist meerdere snijdende curves",
        ));
    }

    if let Some(value) = inputs.get(2) {
        coerce_number(value, component, "Continuity")?;
    }

    let surface = create_surface_from_points(&points, component)?;
    into_output(PIN_OUTPUT_SURFACE, surface)
}

fn evaluate_sweep_two(inputs: &[Value]) -> ComponentResult {
    let component = "Sweep2";
    if inputs.len() < 3 {
        return Err(ComponentError::new("Sweep2 vereist twee rails en secties"));
    }

    let mut points = Vec::new();
    points.extend(
        coerce::coerce_curve_segments(&inputs[0])?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );
    points.extend(
        coerce::coerce_curve_segments(&inputs[1])?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );
    points.extend(
        coerce::coerce_curve_segments(&inputs[2])?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );

    if let Some(value) = inputs.get(3) {
        coerce_bool(value, component, "Same Height")?;
    }

    let surface = create_surface_from_points(&points, component)?;
    into_output(PIN_OUTPUT_SURFACE, Value::List(vec![surface]))
}

fn evaluate_pipe_variable(inputs: &[Value]) -> ComponentResult {
    let component = "Pipe Variable";
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Pipe Variable vereist een curve, parameters en radii",
        ));
    }
    let segments = coerce::coerce_curve_segments(&inputs[0])?;
    if segments.is_empty() {
        return Err(ComponentError::new(
            "Pipe Variable kon de railcurve niet lezen",
        ));
    }
    let _parameters = coerce_number_list(&inputs[1], component, "Parameters")?;
    let radii = coerce_number_list(&inputs[2], component, "Radii")?;
    if radii.is_empty() {
        return Err(ComponentError::new(
            "Pipe Variable vereist minstens één straal",
        ));
    }
    if let Some(value) = inputs.get(3) {
        coerce_number(value, component, "Caps")?;
    }

    let average_radius = radii.iter().map(|value| value.abs()).sum::<f64>() / radii.len() as f64;

    let mut points = Vec::new();
    for (start, end) in segments {
        points.push(start);
        points.push(end);
    }

    let surface = create_surface_from_points_with_padding(&points, average_radius, component)?;
    into_output(PIN_OUTPUT_PIPE, Value::List(vec![surface]))
}

fn evaluate_extrude_linear(inputs: &[Value]) -> ComponentResult {
    let component = "Extrude Linear";
    let profile_segments = coerce::coerce_curve_segments(inputs.get(0).unwrap_or(&Value::Null))?;
    if profile_segments.is_empty() {
        return Err(ComponentError::new(
            "Extrude Linear kon geen profielcurve herkennen",
        ));
    }
    let axis_direction = coerce::coerce_vector_with_default(inputs.get(2));
    if is_zero_vector(axis_direction) {
        return Err(ComponentError::new(
            "Extrude Linear vereist een as met lengte",
        ));
    }

    let mut points = Vec::new();
    for (start, end) in profile_segments {
        points.push(start);
        points.push(end);
        points.push(add_vector(start, axis_direction));
        points.push(add_vector(end, axis_direction));
    }

    let surface = create_surface_from_points(&points, component)?;
    into_output(PIN_OUTPUT_EXTRUSION, surface)
}

fn evaluate_extrude_angled(inputs: &[Value]) -> ComponentResult {
    let component = "Extrude Angled";
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Extrude Angled vereist een polyline en twee hoogtes",
        ));
    }

    let base_points = collect_points(&inputs[0], component)?;
    if base_points.len() < 2 {
        return Err(ComponentError::new(
            "Extrude Angled verwacht minstens twee punten voor de polyline",
        ));
    }
    let base_height = coerce_number(&inputs[1], component, "Base height")?;
    let top_height = coerce_number(&inputs[2], component, "Top height")?;
    if let Some(value) = inputs.get(3) {
        coerce_number_list(value, component, "Angles")?;
    }

    let mut points = base_points.clone();
    for point in &base_points {
        points.push([point[0], point[1], point[2] + base_height]);
        points.push([point[0], point[1], point[2] + top_height]);
    }

    let surface = create_surface_from_points(&points, component)?;
    into_output(PIN_OUTPUT_SHAPE, surface)
}

fn evaluate_sweep_one(inputs: &[Value]) -> ComponentResult {
    let component = "Sweep1";
    if inputs.len() < 2 {
        return Err(ComponentError::new("Sweep1 vereist een rail en secties"));
    }

    let rail_segments = coerce::coerce_curve_segments(&inputs[0])?;
    let rail_polyline = pick_longest_polyline(group_segments_into_polylines(rail_segments))
        .ok_or_else(|| ComponentError::new("Sweep1 kon de railcurve niet lezen"))?;
    if rail_polyline.len() < 2 {
        return Err(ComponentError::new("Sweep1 vereist een rail met lengte"));
    }

    let mut section_surfaces = Vec::new();
    collect_surfaces_recursive(&inputs[1], &mut section_surfaces)?;

    if !section_surfaces.is_empty() {
        if let Some(value) = inputs.get(2) {
            coerce_number(value, component, "Miter")?;
        }

        let mut sweeps = Vec::new();
        for surface in section_surfaces {
            let solid = sweep_surface_along_polyline(surface, &rail_polyline, component)?;
            sweeps.push(solid);
        }
        return into_output(PIN_OUTPUT_SURFACE, Value::List(sweeps));
    }

    let sections = collect_ruled_surface_curves(&inputs[1])?;
    if sections.is_empty() {
        return Err(ComponentError::new(
            "Sweep1 verwacht minstens één sectiepolyline",
        ));
    }

    if let Some(value) = inputs.get(2) {
        coerce_number(value, component, "Miter")?;
    }

    let mut sections = sections;
    unify_curve_directions(&mut sections);

    let mut sweeps = Vec::new();
    for profile in sections {
        let surface = sweep_polyline_along_rail(&profile, &rail_polyline, component)?;
        sweeps.push(surface);
    }

    into_output(PIN_OUTPUT_SURFACE, Value::List(sweeps))
}

fn evaluate_extrude_point(inputs: &[Value]) -> ComponentResult {
    let component = "Extrude Point";
    let base_segments = coerce::coerce_curve_segments(inputs.get(0).unwrap_or(&Value::Null))?;
    if base_segments.is_empty() {
        return Err(ComponentError::new(
            "Extrude Point kon de basiscurve niet lezen",
        ));
    }
    let tip = coerce::coerce_point_with_default(inputs.get(1));

    let mut points = vec![tip];
    for (start, end) in base_segments {
        points.push(start);
        points.push(end);
    }

    let surface = create_surface_from_points(&points, component)?;
    into_output(PIN_OUTPUT_EXTRUSION, surface)
}

fn evaluate_pipe(inputs: &[Value]) -> ComponentResult {
    let component = "Pipe";
    let segments = coerce::coerce_curve_segments(inputs.get(0).unwrap_or(&Value::Null))?;
    if segments.is_empty() {
        return Err(ComponentError::new("Pipe kon de railcurve niet lezen"));
    }
    let radius = coerce::coerce_number_with_default(inputs.get(1)).abs();
    if let Some(value) = inputs.get(2) {
        coerce_number(value, component, "Caps")?;
    }

    let mut points = Vec::new();
    for (start, end) in segments {
        points.push(start);
        points.push(end);
    }

    let surface = create_surface_from_points_with_padding(&points, radius, component)?;
    into_output(PIN_OUTPUT_PIPE, Value::List(vec![surface]))
}

fn evaluate_four_point_surface(inputs: &[Value]) -> ComponentResult {
    let component = "4Point Surface";
    let mut points = Vec::new();

    for index in 0..4 {
        points.push(coerce::coerce_point_with_default(inputs.get(index)));
    }

    if points.len() < 3 {
        return Err(ComponentError::new(
            "4Point Surface vereist minimaal drie hoekpunten",
        ));
    }

    let surface = create_surface_from_points(&points, component)?;
    into_output(PIN_OUTPUT_SURFACE, surface)
}

fn evaluate_fragment_patch(inputs: &[Value]) -> ComponentResult {
    let component = "Fragment Patch";
    let boundary = expect_input(inputs, 0, component, "boundary")?;
    let points = collect_points(boundary, component)?;
    if points.len() < 3 {
        return Err(ComponentError::new(
            "Fragment Patch vereist minimaal drie punten",
        ));
    }

    let surface = create_surface_from_points(&points, component)?;
    into_output(PIN_OUTPUT_PATCH, surface)
}

fn evaluate_revolution(inputs: &[Value]) -> ComponentResult {
    let component = "Revolution";
    let profile_segments = coerce::coerce_curve_segments(inputs.get(0).unwrap_or(&Value::Null))?;
    let axis_segments = coerce::coerce_curve_segments(inputs.get(1).unwrap_or(&Value::Null))?;
    if profile_segments.is_empty() || axis_segments.is_empty() {
        return Err(ComponentError::new(
            "Revolution kon profiel of as niet lezen",
        ));
    }
    let angle = match inputs.get(2) {
        Some(value) => coerce_angle_domain(value, component)?,
        None => 2.0 * std::f64::consts::PI,
    };

    let mut points = Vec::new();
    for (start, end) in profile_segments.into_iter().chain(axis_segments) {
        points.push(start);
        points.push(end);
    }

    let surface = create_surface_from_points_with_padding(&points, angle.abs(), component)?;
    into_output(PIN_OUTPUT_SURFACE, surface)
}

fn evaluate_boundary_surfaces(inputs: &[Value]) -> ComponentResult {
    let component = "Boundary Surfaces";
    let edges = expect_input(inputs, 0, component, "edges")?;
    let segments = coerce::coerce_curve_segments(edges)?;
    if segments.is_empty() {
        return Err(ComponentError::new(
            "Boundary Surfaces vereist minstens één gesloten rand",
        ));
    }

    let mut points = Vec::new();
    for (start, end) in segments {
        points.push(start);
        points.push(end);
    }

    let surface = create_surface_from_points(&points, component)?;
    into_output(PIN_OUTPUT_SURFACE, Value::List(vec![surface]))
}

fn evaluate_rail_revolution(inputs: &[Value]) -> ComponentResult {
    let component = "Rail Revolution";
    if inputs.len() < 4 {
        return Err(ComponentError::new(
            "Rail Revolution vereist profiel, rail, as en schaal",
        ));
    }

    let mut points = Vec::new();
    points.extend(
        coerce::coerce_curve_segments(&inputs[0])?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );
    points.extend(
        coerce::coerce_curve_segments(&inputs[1])?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );
    points.extend(
        coerce::coerce_curve_segments(&inputs[2])?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );
    let scale = coerce_number(&inputs[3], component, "Scale")?.abs();

    let surface = create_surface_from_points_with_padding(&points, scale, component)?;
    into_output(PIN_OUTPUT_SURFACE, surface)
}

fn collect_points(value: &Value, component: &str) -> Result<Vec<[f64; 3]>, ComponentError> {
    match value {
        Value::Point(point) => Ok(vec![*point]),
        Value::Vector(vector) => Ok(vec![*vector]),
        Value::CurveLine { p1, p2 } => Ok(vec![*p1, *p2]),
        Value::Surface { vertices, .. } => Ok(vertices.clone()),
        Value::List(values) => {
            let mut points = Vec::new();
            for entry in values {
                points.extend(collect_points(entry, component)?);
            }
            Ok(points)
        }
        other => Err(ComponentError::new(format!(
            "{component} verwacht punt-achtige invoer, kreeg {}",
            other.kind()
        ))),
    }
}

fn coerce_direction(
    value: &Value,
    component: &str,
    name: &str,
) -> Result<[f64; 3], ComponentError> {
    match value {
        Value::Vector(vector) => Ok(*vector),
        Value::CurveLine { p1, p2 } => Ok(subtract_points(*p2, *p1)),
        Value::Number(height) => Ok([0.0, 0.0, *height]),
        Value::List(values) if values.len() == 1 => coerce_direction(&values[0], component, name),
        other => Err(ComponentError::new(format!(
            "{component} verwacht een richting voor {name}, kreeg {}",
            other.kind()
        ))),
    }
}

fn coerce_point(value: &Value, component: &str, name: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Value::Point(point) => Ok(*point),
        Value::List(values) if values.len() == 1 => coerce_point(&values[0], component, name),
        other => Err(ComponentError::new(format!(
            "{component} verwacht een punt voor {name}, kreeg {}",
            other.kind()
        ))),
    }
}

fn coerce_number(value: &Value, component: &str, name: &str) -> Result<f64, ComponentError> {
    match value {
        Value::Number(number) => Ok(*number),
        Value::List(values) if values.len() == 1 => coerce_number(&values[0], component, name),
        other => Err(ComponentError::new(format!(
            "{component} verwacht een getal voor {name}, kreeg {}",
            other.kind()
        ))),
    }
}

fn coerce_bool(value: &Value, component: &str, name: &str) -> Result<bool, ComponentError> {
    match value {
        Value::Boolean(flag) => Ok(*flag),
        Value::Number(number) => Ok(*number != 0.0),
        Value::List(values) if values.len() == 1 => coerce_bool(&values[0], component, name),
        other => Err(ComponentError::new(format!(
            "{component} verwacht een booleaanse waarde voor {name}, kreeg {}",
            other.kind()
        ))),
    }
}

fn coerce_number_list(
    value: &Value,
    component: &str,
    name: &str,
) -> Result<Vec<f64>, ComponentError> {
    match value {
        Value::Number(number) => Ok(vec![*number]),
        Value::List(values) => {
            let mut result = Vec::new();
            for entry in values {
                result.extend(coerce_number_list(entry, component, name)?);
            }
            Ok(result)
        }
        other => Err(ComponentError::new(format!(
            "{component} verwacht een (lijst) getallen voor {name}, kreeg {}",
            other.kind()
        ))),
    }
}

fn coerce_angle_domain(value: &Value, component: &str) -> Result<f64, ComponentError> {
    match value {
        Value::Number(number) => Ok(*number),
        Value::Domain(Domain::One(domain)) => Ok(domain.length.abs()),
        Value::Domain(Domain::Two(domain)) => Ok(domain.u.length.abs().max(domain.v.length.abs())),
        Value::List(values) if values.len() == 1 => coerce_angle_domain(&values[0], component),
        other => Err(ComponentError::new(format!(
            "{component} verwacht een hoek of domein, kreeg {}",
            other.kind()
        ))),
    }
}

fn create_surface_from_points(
    points: &[[f64; 3]],
    component: &str,
) -> Result<Value, ComponentError> {
    create_surface_from_points_with_padding(points, 0.0, component)
}

fn create_surface_from_points_with_padding(
    points: &[[f64; 3]],
    padding: f64,
    component: &str,
) -> Result<Value, ComponentError> {
    if points.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} vereist minstens twee unieke punten"
        )));
    }

    let mut min = points[0];
    let mut max = points[0];
    for point in points.iter().skip(1) {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }

    let padding = padding.max(0.0);
    for axis in 0..3 {
        min[axis] -= padding;
        max[axis] += padding;
    }

    let spans = [
        (max[0] - min[0], 0usize),
        (max[1] - min[1], 1usize),
        (max[2] - min[2], 2usize),
    ];

    let mut sorted = spans;
    sorted.sort_by(
        |a, b| match (a.0.partial_cmp(&b.0), b.0.partial_cmp(&a.0)) {
            (Some(order), _) => order.reverse(),
            (None, Some(order)) => order,
            _ => Ordering::Equal,
        },
    );

    let (primary_span, primary_axis) = sorted[0];
    if primary_span.abs() <= EPSILON {
        return Err(ComponentError::new(format!(
            "{component} kon geen oppervlak vormen uit samenvallende punten"
        )));
    }

    let secondary_axis = sorted
        .iter()
        .skip(1)
        .find(|(span, axis)| *axis != primary_axis && span.abs() > EPSILON)
        .map(|(_, axis)| *axis)
        .unwrap_or_else(|| if primary_axis != 0 { 0 } else { 1 });

    let mut min_secondary = min[secondary_axis];
    let mut max_secondary = max[secondary_axis];
    if (max_secondary - min_secondary).abs() <= EPSILON {
        min_secondary -= 0.5;
        max_secondary += 0.5;
    }

    let third_axis = (0..3)
        .find(|axis| *axis != primary_axis && *axis != secondary_axis)
        .unwrap_or(primary_axis);
    let mid_third = (min[third_axis] + max[third_axis]) * 0.5;

    let mut vertices = Vec::with_capacity(4);
    for &a in &[min[primary_axis], max[primary_axis]] {
        for &b in &[min_secondary, max_secondary] {
            let mut vertex = [0.0; 3];
            vertex[primary_axis] = a;
            vertex[secondary_axis] = b;
            vertex[third_axis] = mid_third;
            vertices.push(vertex);
        }
    }

    let faces = vec![vec![0, 1, 2], vec![0, 2, 3]];
    Ok(Value::Surface { vertices, faces })
}

fn expect_input<'a>(
    inputs: &'a [Value],
    index: usize,
    component: &str,
    description: &str,
) -> Result<&'a Value, ComponentError> {
    inputs.get(index).ok_or_else(|| {
        ComponentError::new(format!("{component} vereist een invoer voor {description}"))
    })
}

fn add_vector(point: [f64; 3], direction: [f64; 3]) -> [f64; 3] {
    [
        point[0] + direction[0],
        point[1] + direction[1],
        point[2] + direction[2],
    ]
}

fn subtract_points(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross_product(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[allow(dead_code)]
fn dot_product(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let mag = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if mag > EPSILON {
        [v[0] / mag, v[1] / mag, v[2] / mag]
    } else {
        [0.0, 0.0, 0.0]
    }
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let delta = subtract_points(a, b);
    (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt()
}

fn is_zero_vector(vector: [f64; 3]) -> bool {
    vector.iter().all(|component| component.abs() < EPSILON)
}

fn offset_rail_polyline(
    rail_polyline: &[[f64; 3]],
    section_origin: [f64; 3],
) -> Vec<[f64; 3]> {
    if rail_polyline.is_empty() {
        return Vec::new();
    }

    let translation = subtract_points(section_origin, rail_polyline[0]);
    rail_polyline
        .iter()
        .map(|point| add_vector(*point, translation))
        .collect()
}

fn dedup_consecutive_points(mut points: Vec<[f64; 3]>, closed: bool) -> Vec<[f64; 3]> {
    let mut deduped = Vec::with_capacity(points.len());
    for point in points.drain(..) {
        if deduped
            .last()
            .map_or(true, |last| !points_equal(*last, point))
        {
            deduped.push(point);
        }
    }

    if closed && deduped.len() > 2 && points_equal(deduped[0], *deduped.last().unwrap()) {
        deduped.pop();
    }

    deduped
}

fn project_point_on_polyline(point: [f64; 3], polyline: &[[f64; 3]]) -> (f64, f64) {
    if polyline.len() < 2 {
        return (0.0, distance(point, polyline.get(0).copied().unwrap_or([0.0; 3])));
    }

    let mut best_t = 0.0;
    let mut best_dist = f64::MAX;
    let mut accumulated = 0.0;
    let total_length = polyline_length(polyline);

    for window in polyline.windows(2) {
        let a = window[0];
        let b = window[1];
        let ab = subtract_points(b, a);
        let ab_len_sq = dot_product(ab, ab);
        if ab_len_sq < EPSILON {
            continue;
        }
        let ap = subtract_points(point, a);
        let t_seg = (dot_product(ap, ab) / ab_len_sq).clamp(0.0, 1.0);
        let closest = add_vector(a, [
            ab[0] * t_seg,
            ab[1] * t_seg,
            ab[2] * t_seg,
        ]);
        let dist = distance(point, closest);
        if dist < best_dist {
            best_dist = dist;
            let seg_length = ab_len_sq.sqrt();
            let seg_t = accumulated + seg_length * t_seg;
            best_t = if total_length > 0.0 {
                seg_t / total_length
            } else {
                0.0
            };
        }
        accumulated += ab_len_sq.sqrt();
    }

    (best_t, best_dist)
}

fn point_on_polyline_fraction(polyline: &[[f64; 3]], t: f64) -> [f64; 3] {
    if polyline.is_empty() {
        return [0.0; 3];
    }
    if polyline.len() == 1 {
        return polyline[0];
    }
    let total_length = polyline_length(polyline);
    if total_length <= EPSILON {
        return polyline[0];
    }

    let target_len = (t.clamp(0.0, 1.0)) * total_length;
    let mut accumulated = 0.0;

    for window in polyline.windows(2) {
        let a = window[0];
        let b = window[1];
        let seg_len = distance(a, b);
        if seg_len < EPSILON {
            continue;
        }
        if accumulated + seg_len >= target_len {
            let local_t = (target_len - accumulated) / seg_len;
            return add_vector(a, [
                (b[0] - a[0]) * local_t,
                (b[1] - a[1]) * local_t,
                (b[2] - a[2]) * local_t,
            ]);
        }
        accumulated += seg_len;
    }

    *polyline.last().unwrap()
}

fn plane_basis(normal: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let n = {
        let n = normalize(normal);
        if is_zero_vector(n) {
            [0.0, 0.0, 1.0]
        } else {
            n
        }
    };

    let mut tangent = cross_product(n, [1.0, 0.0, 0.0]);
    if is_zero_vector(tangent) {
        tangent = cross_product(n, [0.0, 1.0, 0.0]);
    }
    if is_zero_vector(tangent) {
        tangent = [1.0, 0.0, 0.0];
    }
    tangent = normalize(tangent);
    let bitangent = normalize(cross_product(n, tangent));
    (tangent, bitangent)
}

fn signed_area_in_plane(polyline: &[[f64; 3]], normal: [f64; 3]) -> f64 {
    if polyline.len() < 3 {
        return 0.0;
    }
    let (x_axis, y_axis) = plane_basis(normal);
    let origin = polyline[0];

    let mut area = 0.0;
    for i in 0..polyline.len() {
        let j = (i + 1) % polyline.len();
        let vi = subtract_points(polyline[i], origin);
        let vj = subtract_points(polyline[j], origin);
        let ui = dot_product(vi, x_axis);
        let wi = dot_product(vi, y_axis);
        let uj = dot_product(vj, x_axis);
        let wj = dot_product(vj, y_axis);
        area += ui * wj - uj * wi;
    }

    area * 0.5
}

fn into_output(pin: &str, value: Value) -> ComponentResult {
    let mut outputs = BTreeMap::new();
    outputs.insert(pin.to_owned(), value);
    Ok(outputs)
}

fn collect_surfaces_recursive<'a>(
    value: &'a Value,
    surfaces: &mut Vec<coerce::Surface<'a>>,
) -> Result<(), ComponentError> {
    match value {
        Value::Surface { .. } => surfaces.push(coerce::coerce_surface(value)?),
        Value::List(values) => {
            for entry in values {
                collect_surfaces_recursive(entry, surfaces)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn pick_longest_polyline(polylines: Vec<Vec<[f64; 3]>>) -> Option<Vec<[f64; 3]>> {
    polylines
        .into_iter()
        .max_by(|a, b| match (polyline_length(&a), polyline_length(&b)) {
            (x, y) if x.is_finite() && y.is_finite() => {
                x.partial_cmp(&y).unwrap_or(Ordering::Equal)
            }
            _ => Ordering::Equal,
        })
}

fn polyline_length(polyline: &[[f64; 3]]) -> f64 {
    polyline
        .windows(2)
        .map(|pair| distance(pair[0], pair[1]))
        .sum()
}

fn find_boundary_polylines(surface: &coerce::Surface<'_>) -> Vec<Vec<u32>> {
    let mut edge_counts = HashMap::new();
    for face in surface.faces {
        if face.len() < 2 {
            continue;
        }
        for i in 0..face.len() {
            let p1_idx = face[i];
            let p2_idx = face[(i + 1) % face.len()];

            // Normaliseer de edge door de kleinste index eerst te plaatsen
            let edge = if p1_idx < p2_idx {
                (p1_idx, p2_idx)
            } else {
                (p2_idx, p1_idx)
            };
            *edge_counts.entry(edge).or_insert(0) += 1;
        }
    }

    let boundary_edges: Vec<_> = edge_counts
        .into_iter()
        .filter(|(_, count)| *count == 1)
        .map(|(edge, _)| edge)
        .collect();

    if boundary_edges.is_empty() {
        return Vec::new();
    }

    let mut adj_list: HashMap<u32, Vec<u32>> = HashMap::new();
    for (p1, p2) in boundary_edges {
        adj_list.entry(p1).or_default().push(p2);
        adj_list.entry(p2).or_default().push(p1);
    }

    let mut polylines = Vec::new();
    let mut visited = std::collections::HashSet::new();

    for start_node in adj_list.keys() {
        if visited.contains(start_node) {
            continue;
        }

        let mut current_polyline_indices = Vec::new();
        let mut current_node = *start_node;

        while !visited.contains(&current_node) {
            visited.insert(current_node);
            current_polyline_indices.push(current_node);

            let next_node = adj_list
                .get(&current_node)
                .unwrap()
                .iter()
                .find(|&node| !visited.contains(node));

            if let Some(node) = next_node {
                current_node = *node;
            } else {
                // Einde van een open polyline
                break;
            }
        }
        if current_polyline_indices.len() > 1 {
            polylines.push(current_polyline_indices);
        }
    }

    polylines
}

fn calculate_surface_normal(surface: &coerce::Surface<'_>) -> [f64; 3] {
    if surface.faces.is_empty() || surface.faces[0].len() < 3 {
        return [0.0, 0.0, 1.0]; // Standaard normaal als het oppervlak niet goed gedefinieerd is
    }

    let first_face_indices = &surface.faces[0];
    let p1 = surface.vertices[first_face_indices[0] as usize];
    let p2 = surface.vertices[first_face_indices[1] as usize];
    let p3 = surface.vertices[first_face_indices[2] as usize];

    let v1 = subtract_points(p2, p1);
    let v2 = subtract_points(p3, p1);

    normalize(cross_product(v1, v2))
}

fn sweep_surface_along_polyline(
    surface: coerce::Surface<'_>,
    rail_polyline: &[[f64; 3]],
    component: &str,
) -> Result<Value, ComponentError> {
    if surface.vertices.is_empty() {
        return Err(ComponentError::new(format!(
            "{component} verwacht een surface met minstens één vertex",
        )));
    }
    if surface.faces.is_empty() {
        return Err(ComponentError::new(format!(
            "{component} verwacht een surface met minstens één face",
        )));
    }
    if rail_polyline.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} vereist een rail met minstens twee punten",
        )));
    }

    let rail_polyline: Vec<[f64; 3]> = dedup_consecutive_points(rail_polyline.to_vec(), false);
    if rail_polyline.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} vereist een rail met minstens twee unieke punten",
        )));
    }

    let surface_normal = calculate_surface_normal(&surface);
    let boundary_polylines_indices = find_boundary_polylines(&surface);
    let section_origin = surface.vertices[0];
    let offset_rail = offset_rail_polyline(&rail_polyline, section_origin);

    let mut vertices: Vec<[f64; 3]> = surface.vertices.to_vec();
    let mut faces = surface.faces.clone();

    let mut last_layer_start = 0u32;
    let mut last_layer_vertices = vertices.clone();
    let base_faces = surface.faces.clone();

    for window in offset_rail.windows(2) {
        let direction = subtract_points(window[1], window[0]);
        if is_zero_vector(direction) {
            continue;
        }

        let new_layer_start = vertices.len() as u32;
        let new_layer_vertices: Vec<[f64; 3]> = last_layer_vertices
            .iter()
            .map(|vertex| add_vector(*vertex, direction))
            .collect();
        vertices.extend(new_layer_vertices.iter());

        for polyline_indices in &boundary_polylines_indices {
            let polyline_vertices: Vec<[f64; 3]> = polyline_indices
                .iter()
                .map(|&i| vertices[i as usize])
                .collect();

            // Bereken de normaal van de polyline
            let p1 = polyline_vertices[0];
            let p2 = polyline_vertices[1];
            let p3 = *polyline_vertices.get(2).unwrap_or(&p1);
            let v1 = subtract_points(p2, p1);
            let v2 = subtract_points(p3, p1);
            let polyline_normal = normalize(cross_product(v1, v2));

            let mut corrected_indices = polyline_indices.clone();
            // Keer de polyline om als de normaal in de tegenovergestelde richting van de oppervlaknormaal wijst
            if dot_product(polyline_normal, surface_normal) < 0.0 {
                corrected_indices.reverse();
            }

            let n = corrected_indices.len();
            if n < 2 {
                continue;
            }

            for i in 0..n {
                let current_idx = corrected_indices[i];
                let next_idx = corrected_indices[(i + 1) % n];

                let v1 = last_layer_start + current_idx;
                let v2 = last_layer_start + next_idx;
                let v3 = new_layer_start + next_idx;
                let v4 = new_layer_start + current_idx;

                // Gebruik een consistente winding order voor de vlakken
                faces.push(vec![v1, v4, v2]);
                faces.push(vec![v2, v4, v3]);
            }
        }

        last_layer_start = new_layer_start;
        last_layer_vertices = new_layer_vertices;
    }

    for face in &base_faces {
        if face.len() < 2 {
            continue;
        }
        let mut top_face = Vec::with_capacity(face.len());
        for &index in face.iter().rev() {
            top_face.push(last_layer_start + index);
        }
        faces.push(top_face);
    }

    Ok(Value::Surface { vertices, faces })
}

fn sweep_polyline_along_rail(
    profile: &[[f64; 3]],
    rail_polyline: &[[f64; 3]],
    component: &str,
) -> Result<Value, ComponentError> {
    let mut profile = profile.to_vec();
    let mut profile_closed = false;
    if profile.len() >= 3 && points_equal(profile[0], *profile.last().unwrap()) {
        profile.pop(); // remove duplicate closing point, keep closed flag
        profile_closed = true;
    } else if profile.len() >= 2 && points_equal(profile[0], *profile.last().unwrap()) {
        profile.pop(); // degenerate "closed" with only two equal points -> treat as open
    }

    // Verwijder opeenvolgende dubbele punten om degeneratie te voorkomen.
    profile = dedup_consecutive_points(profile, profile_closed);

    // Zorg voor een consistente CCW-winding zoals in BoxRectangle zodat front-faces correct zijn.
    if profile_closed && profile.len() >= 3 {
        let normal = {
            let n = polyline_normal(&profile);
            if is_zero_vector(n) {
                [0.0, 0.0, 1.0]
            } else {
                n
            }
        };
        let signed_area = signed_area_in_plane(&profile, normal);
        if signed_area < 0.0 {
            profile.reverse();
        }
    }

    if profile.is_empty() {
        return Err(ComponentError::new(format!(
            "{component} verwacht een sectiepolyline",
        )));
    }
    let section_origin = profile[0];

    if profile.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} verwacht een sectiepolyline met minstens twee punten",
        )));
    }
    if rail_polyline.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} vereist een rail met minstens twee punten",
        )));
    }

    let rail_polyline: Vec<[f64; 3]> = dedup_consecutive_points(rail_polyline.to_vec(), false);
    if rail_polyline.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} vereist een rail met minstens twee unieke punten",
        )));
    }

    let offset_rail = offset_rail_polyline(&rail_polyline, section_origin);

    let layer_size = profile.len();
    let profile_indices: Vec<u32> = (0..layer_size as u32).collect();
    let ordered_profile = if profile_closed && layer_size >= 3 {
        let normal = polyline_normal(&profile);
        let winding = polyline_winding_direction(&profile, normal);
        if winding < 0.0 {
            let mut reversed = profile_indices.clone();
            reversed.reverse();
            reversed
        } else {
            profile_indices.clone()
        }
    } else {
        profile_indices.clone()
    };
    let mut vertices = profile.clone();
    let mut faces: Vec<Vec<u32>> = Vec::new();

    if profile_closed && layer_size >= 3 {
        faces.push(ordered_profile.clone());
    }

    let mut last_layer_start = 0u32;
    let mut last_layer_vertices = profile;

    for window in offset_rail.windows(2) {
        let direction = subtract_points(window[1], window[0]);
        if is_zero_vector(direction) {
            continue;
        }

        let new_layer_start = vertices.len() as u32;
        let new_layer_vertices: Vec<[f64; 3]> = last_layer_vertices
            .iter()
            .map(|vertex| add_vector(*vertex, direction))
            .collect();

        vertices.extend(new_layer_vertices.iter());

        let edge_count = if profile_closed { layer_size } else { layer_size.saturating_sub(1) };
        for i in 0..edge_count {
            let current_idx = ordered_profile[i];
            let next_idx = ordered_profile[(i + 1) % layer_size];
            let v1 = last_layer_start + current_idx;
            let v2 = last_layer_start + next_idx;
            let v3 = new_layer_start + next_idx;
            let v4 = new_layer_start + current_idx;
            faces.push(vec![v1, v4, v2]);
            faces.push(vec![v2, v4, v3]);
        }

        last_layer_start = new_layer_start;
        last_layer_vertices = new_layer_vertices;
    }

    if profile_closed && layer_size >= 3 {
        let mut top_face = Vec::with_capacity(layer_size);
        for &index in ordered_profile.iter().rev() {
            top_face.push(last_layer_start + index);
        }
        faces.push(top_face);
    }

    Ok(Value::Surface { vertices, faces })
}

fn sweep_sections_along_rail(
    mut sections: Vec<Vec<[f64; 3]>>,
    rail_polyline: &[[f64; 3]],
    component: &str,
) -> Result<Value, ComponentError> {
    if rail_polyline.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} vereist een rail met minstens twee punten",
        )));
    }
    if sections.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} verwacht minstens twee sectiecurves",
        )));
    }

    let rail_polyline: Vec<[f64; 3]> = dedup_consecutive_points(rail_polyline.to_vec(), false);

    // Note closed flags before deduplication.
    let mut section_closed: Vec<bool> = sections.iter().map(|p| is_closed(p)).collect();
    for (poly, closed) in sections.iter_mut().zip(section_closed.iter_mut()) {
        *poly = dedup_consecutive_points(std::mem::take(poly), *closed);
        *closed = *closed && poly.len() >= 3;
    }

    // Plaats secties langs de rail met gelijke verdeling over de rail lengte.
    let mut positioned_sections = Vec::with_capacity(sections.len());
    let mut positioned_closed = Vec::with_capacity(sections.len());
    let section_count = sections.len().max(1);
    for (idx, (poly, closed)) in sections.into_iter().zip(section_closed).enumerate() {
        let t = if section_count == 1 {
            0.0
        } else {
            idx as f64 / ((section_count - 1) as f64)
        };
        let target_point = point_on_polyline_fraction(&rail_polyline, t);
        let origin = poly.first().copied().unwrap_or([0.0; 3]);
        let translation = subtract_points(target_point, origin);
        let translated: Vec<[f64; 3]> = poly.iter().map(|p| add_vector(*p, translation)).collect();
        positioned_sections.push(translated);
        positioned_closed.push(closed);
    }

    // Zorg voor consistente oriëntatie per sectie.
    unify_curve_directions(&mut positioned_sections);

    // Bepaal doel sample-grootte.
    let target_count = positioned_sections.iter().map(|p| p.len()).max().unwrap_or(0);
    if target_count < 2 {
        return Err(ComponentError::new(format!(
            "{component} kon geen secties met voldoende punten vinden",
        )));
    }

    let resampled_sections: Vec<Vec<[f64; 3]>> = positioned_sections
        .iter()
        .map(|p| {
            let dummy_target = vec![[0.0; 3]; target_count];
            super::curve_sampler::resample_polylines(p, &dummy_target).0
        })
        .collect();

    let mut vertices = Vec::new();
    let mut faces: Vec<Vec<u32>> = Vec::new();

    for section in &resampled_sections {
        vertices.extend_from_slice(section);
    }

    let num_sections = resampled_sections.len();
    let pts_per_section = target_count as u32;

    for i in 0..(num_sections - 1) {
        let base = i as u32 * pts_per_section;
        let next_base = (i as u32 + 1) * pts_per_section;
        let closed = positioned_closed[i] && positioned_closed[i + 1];
        let edge_count = if closed {
            pts_per_section
        } else {
            pts_per_section.saturating_sub(1)
        };

        for j in 0..edge_count {
            let j_next = (j + 1) % pts_per_section;
            let v1 = base + j;
            let v2 = base + j_next;
            let v3 = next_base + j_next;
            let v4 = next_base + j;
            faces.push(vec![v1, v4, v2]);
            faces.push(vec![v2, v4, v3]);
        }
    }

    // Caps voor gesloten secties.
    if positioned_closed.first().copied().unwrap_or(false) {
        let mut first_face = Vec::with_capacity(pts_per_section as usize);
        for idx in 0..pts_per_section {
            first_face.push(idx);
        }
        faces.push(first_face);
    }
    if positioned_closed.last().copied().unwrap_or(false) {
        let mut last_face = Vec::with_capacity(pts_per_section as usize);
        let start = (num_sections as u32 - 1) * pts_per_section;
        for idx in (0..pts_per_section).rev() {
            last_face.push(start + idx);
        }
        faces.push(last_face);
    }

    Ok(Value::Surface { vertices, faces })
}

#[allow(dead_code)]
fn extrude_surface_along_vector(
    surface: coerce::Surface<'_>,
    direction: [f64; 3],
    component: &str,
) -> Result<Value, ComponentError> {
    if surface.vertices.is_empty() {
        return Err(ComponentError::new(format!(
            "{component} verwacht een surface met minstens één vertex"
        )));
    }
    if surface.faces.is_empty() {
        return Err(ComponentError::new(format!(
            "{component} verwacht een surface met minstens één face"
        )));
    }
    if is_zero_vector(direction) {
        return Err(ComponentError::new(format!(
            "{component} kan niet extruderen zonder railrichting"
        )));
    }

    let offset = surface.vertices.len() as u32;

    let mut vertices = surface.vertices.clone();
    vertices.extend(
        surface
            .vertices
            .iter()
            .map(|vertex| add_vector(*vertex, direction)),
    );

    let mut faces = Vec::new();
    for face in surface.faces.iter() {
        if face.len() < 2 {
            continue;
        }

        faces.push(face.clone());

        let mut top_face = Vec::with_capacity(face.len());
        for &index in face.iter().rev() {
            top_face.push(index + offset);
        }
        faces.push(top_face);

        for (current, next) in face
            .iter()
            .zip(face.iter().cycle().skip(1))
            .take(face.len())
        {
            faces.push(vec![*current, *next, *next + offset, *current + offset]);
        }
    }

    Ok(Value::Surface { vertices, faces })
}

/// Bepaalt of een polyline gesloten is door het eerste en laatste punt te vergelijken.
fn is_closed(polyline: &[[f64; 3]]) -> bool {
    if polyline.len() < 3 {
        return false;
    }
    points_equal(*polyline.first().unwrap(), *polyline.last().unwrap())
}

/// Berekent de gemiddelde normaal van een polyline.
/// Dit wordt gedaan door de normaal te berekenen voor elk segment ten opzichte van het centroïde
/// en deze te middelen. Dit geeft een robuuste normaal, zelfs voor niet-vlakke polylines.
fn polyline_normal(polyline: &[[f64; 3]]) -> [f64; 3] {
    if polyline.len() < 3 {
        return [0.0, 0.0, 1.0]; // Standaard Z-as voor onvoldoende punten
    }

    let centroid = polyline.iter().fold([0.0; 3], |acc, p| add_vector(acc, *p));
    let n = polyline.len() as f64;
    let centroid = [centroid[0] / n, centroid[1] / n, centroid[2] / n];

    let mut normal = [0.0; 3];
    for i in 0..polyline.len() {
        let p1 = polyline[i];
        let p2 = polyline[(i + 1) % polyline.len()];
        let v1 = subtract_points(p1, centroid);
        let v2 = subtract_points(p2, centroid);
        normal = add_vector(normal, cross_product(v1, v2));
    }

    normalize(normal)
}

/// Bepaalt de oriëntatie (winding direction) van een gesloten, vlakke polyline.
/// Retourneert een positieve waarde voor tegen de klok in (CCW), negatief voor met de klok mee (CW),
/// en nul als de oriëntatie niet bepaald kan worden.
fn polyline_winding_direction(polyline: &[[f64; 3]], normal: [f64; 3]) -> f64 {
    if polyline.len() < 3 {
        return 0.0;
    }

    let mut area_sum = 0.0;
    for i in 0..polyline.len() {
        let p1 = polyline[i];
        let p2 = polyline[(i + 1) % polyline.len()];
        let cross = cross_product(p1, p2);
        area_sum += dot_product(cross, normal);
    }

    area_sum
}

#[cfg(test)]
mod tests {
    use super::{
        Component, ComponentKind, EPSILON, PIN_OUTPUT_EXTRUSION, PIN_OUTPUT_LOFT,
        PIN_OUTPUT_OPTIONS, PIN_OUTPUT_PIPE, PIN_OUTPUT_SURFACE, add_vector, is_closed,
        polyline_normal, polyline_winding_direction, signed_area_in_plane,
    };
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn fit_loft_generates_surface() {
        let component = ComponentKind::FitLoft;
        let inputs = [Value::List(vec![
            Value::CurveLine {
                p1: [0.0, 0.0, 0.0],
                p2: [1.0, 0.0, 0.0],
            },
            Value::CurveLine {
                p1: [0.0, 1.0, 0.0],
                p2: [1.0, 1.0, 0.0],
            },
        ])];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("fit loft surface");

        let Value::Surface { vertices, faces } = outputs.get(PIN_OUTPUT_SURFACE).unwrap() else {
            panic!("expected surface output");
        };
        assert_eq!(vertices.len(), 4);
        assert_eq!(faces.len(), 2);
    }

    #[test]
    fn sweep_one_with_curved_rail_and_surface_section() {
        let component = ComponentKind::Sweep1;
        let inputs = [
            // Rail: Polyline with 2 segments, starts at the same position as the section
            Value::List(vec![
                Value::CurveLine {
                    p1: [0.0, 0.0, 0.0],
                    p2: [0.0, 0.0, 10.0],
                },
                Value::CurveLine {
                    p1: [0.0, 0.0, 10.0],
                    p2: [5.0, 0.0, 10.0],
                },
            ]),
            // Section: A rectangular surface at the origin
            Value::Surface {
                vertices: vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 1.0, 0.0],
                    [0.0, 1.0, 0.0],
                ],
                faces: vec![vec![0, 1, 2, 3]],
            },
        ];

        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("sweep with curved rail");

        let Value::List(solids) = outputs.get(PIN_OUTPUT_SURFACE).unwrap() else {
            panic!("expected list output");
        };
        let Value::Surface { vertices, faces } = &solids[0] else {
            panic!("expected surface solid");
        };

        // Vertices: 4 (base) + 4 (middle) + 4 (end) = 12
        assert_eq!(vertices.len(), 12, "expected vertices for each rail point");

        // Faces:
        // - 1 base face (quad)
        // - 1 top face (quad)
        // - 4 side faces for the first segment (4 * 2 triangles = 8)
        // - 4 side faces for the second segment (4 * 2 triangles = 8)
        // Total = 1 + 1 + 8 + 8 = 18
        assert_eq!(faces.len(), 18, "expected caps and triangulated side faces");

        // Check if the final vertices are at the correct translated position
        let final_position = [5.0, 0.0, 10.0];
        assert_eq!(vertices[8], add_vector([0.0, 0.0, 0.0], final_position));
        assert_eq!(vertices[9], add_vector([1.0, 0.0, 0.0], final_position));
        assert_eq!(vertices[10], add_vector([1.0, 1.0, 0.0], final_position));
        assert_eq!(vertices[11], add_vector([0.0, 1.0, 0.0], final_position));
    }

    #[test]
    fn extrude_along_uses_rail_direction() {
        let component = ComponentKind::ExtrudeAlong;
        let inputs = [
            Value::CurveLine {
                p1: [0.0, 0.0, 0.0],
                p2: [1.0, 0.0, 0.0],
            },
            Value::CurveLine {
                p1: [0.0, 0.0, 0.0],
                p2: [0.0, 0.0, 2.0],
            },
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("extrude along");

        let Value::Surface { vertices, .. } = outputs.get(PIN_OUTPUT_EXTRUSION).unwrap() else {
            panic!("expected extrusion surface");
        };
        assert!(vertices.iter().any(|vertex| (vertex[2] - 2.0).abs() < 1e-9));
    }

    #[test]
    fn loft_options_formats_summary() {
        let component = ComponentKind::LoftOptions;
        let inputs = [
            Value::Boolean(true),
            Value::Boolean(false),
            Value::Number(5.0),
            Value::Number(0.25),
            Value::Number(2.0),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("loft options");

        let Value::Text(text) = outputs.get(PIN_OUTPUT_OPTIONS).unwrap() else {
            panic!("expected options text");
        };
        assert!(text.contains("\"closed\":true"));
        assert!(text.contains("\"type\":2"));
    }

    #[test]
    fn boundary_surfaces_returns_list() {
        let component = ComponentKind::BoundarySurfaces;
        let inputs = [Value::List(vec![
            Value::CurveLine {
                p1: [0.0, 0.0, 0.0],
                p2: [1.0, 0.0, 0.0],
            },
            Value::CurveLine {
                p1: [1.0, 0.0, 0.0],
                p2: [1.0, 1.0, 0.0],
            },
        ])];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("boundary surfaces");

        let Value::List(values) = outputs.get(PIN_OUTPUT_SURFACE).unwrap() else {
            panic!("expected list of surfaces");
        };
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn group_segments_orders_unsorted_segments() {
        // Unsorted triangle edges should produce a closed, ordered polyline.
        let segments = vec![
            ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
            ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
            ([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
        ];
        let polylines = group_segments_into_polylines(segments);
        assert_eq!(polylines.len(), 1);
        let polyline = &polylines[0];
        assert!(
            polyline.len() == 4 && points_equal(polyline[0], *polyline.last().unwrap()),
            "polyline should close and use all edges"
        );
        // Every edge from the polyline should correspond to one of the original segments.
        for w in polyline.windows(2) {
            let found = [
                ([w[0][0], w[0][1], w[0][2]], [w[1][0], w[1][1], w[1][2]]),
                ([w[1][0], w[1][1], w[1][2]], [w[0][0], w[0][1], w[0][2]]),
            ]
            .iter()
            .any(|pair| {
                let original = vec![
                    ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
                    ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
                    ([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
                ];
                original
                    .iter()
                    .any(|s| points_equal(s.0, pair.0) && points_equal(s.1, pair.1))
            });
            assert!(found, "polyline edge must map to original segment");
        }
    }

    #[test]
    fn signed_area_detects_winding() {
        let ccw = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        assert!(signed_area_in_plane(&ccw, [0.0, 0.0, 1.0]) > 0.0);

        let cw = vec![
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
        ];
        assert!(signed_area_in_plane(&cw, [0.0, 0.0, 1.0]) < 0.0);
    }

    #[test]
    fn pipe_variable_accounts_for_radii() {
        let component = ComponentKind::PipeVariable;
        let inputs = [
            Value::CurveLine {
                p1: [0.0, 0.0, 0.0],
                p2: [0.0, 0.0, 1.0],
            },
            Value::List(vec![Value::Number(0.0), Value::Number(1.0)]),
            Value::List(vec![Value::Number(0.5), Value::Number(1.0)]),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("pipe variable");

        let Value::List(values) = outputs.get(PIN_OUTPUT_PIPE).unwrap() else {
            panic!("expected list of pipes");
        };
        let Value::Surface { vertices, .. } = &values[0] else {
            panic!("expected surface");
        };
        let span = vertices
            .iter()
            .map(|vertex| vertex[0])
            .fold((f64::MAX, f64::MIN), |(min, max), value| {
                (min.min(value), max.max(value))
            });
        assert!((span.1 - span.0) > 0.5);
    }

    #[test]
    fn loft_component_uses_loft_pin() {
        let component = ComponentKind::Loft;
        let inputs = [Value::List(vec![
            Value::CurveLine {
                p1: [0.0, 0.0, 0.0],
                p2: [1.0, 0.0, 0.0],
            },
            Value::CurveLine {
                p1: [0.0, 1.0, 0.0],
                p2: [1.0, 1.0, 0.0],
            },
        ])];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("loft surface");

        assert!(matches!(
            outputs.get(PIN_OUTPUT_LOFT),
            Some(Value::Surface { .. })
        ));
    }

    #[test]
    fn sweep_one_extrudes_surface_along_rail() {
        let component = ComponentKind::Sweep1;
        let inputs = [
            Value::CurveLine {
                p1: [0.0, 0.0, 0.0],
                p2: [0.0, 0.0, 2.0],
            },
            Value::Surface {
                vertices: vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 1.0, 0.0],
                    [0.0, 1.0, 0.0],
                ],
                faces: vec![vec![0, 1, 2, 3]],
            },
        ];

        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("sweep solid");

        let Value::List(solids) = outputs.get(PIN_OUTPUT_SURFACE).unwrap() else {
            panic!("expected list output");
        };
        let Value::Surface { vertices, faces } = &solids[0] else {
            panic!("expected surface solid");
        };

        assert_eq!(vertices.len(), 8, "should duplicate vertices for extrusion");
        assert!(vertices.iter().any(|vertex| (vertex[2] - 2.0).abs() < 1e-9));
        assert!(faces.len() >= 6, "solid should include caps and side faces");
    }

    #[test]
    fn sweep_one_handles_grafted_surfaces() {
        let component = ComponentKind::Sweep1;
        let rail = Value::CurveLine {
            p1: [0.0, 0.0, 0.0],
            p2: [0.0, 0.0, 1.0],
        };

        let surface_a = Value::Surface {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            faces: vec![vec![0, 1, 2, 3]],
        };
        let surface_b = Value::Surface {
            vertices: vec![
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            faces: vec![vec![0, 1, 2, 3]],
        };

        let inputs = [
            rail,
            Value::List(vec![
                Value::List(vec![surface_a]),
                Value::List(vec![surface_b]),
            ]),
        ];

        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("sweep grafted surfaces");

        let Value::List(solids) = outputs.get(PIN_OUTPUT_SURFACE).unwrap() else {
            panic!("expected list output");
        };
        assert_eq!(solids.len(), 2, "expected a sweep per grafted surface");
        for solid in solids {
            assert!(matches!(solid, Value::Surface { .. }), "expected surface output");
        }
    }

    #[test]
    fn sweep_one_follows_polyline_rail() {
        let component = ComponentKind::Sweep1;
        let inputs = [
            Value::List(vec![
                Value::CurveLine {
                    p1: [0.0, 0.0, 0.0],
                    p2: [0.0, 0.0, 1.0],
                },
                Value::CurveLine {
                    p1: [0.0, 0.0, 1.0],
                    p2: [1.0, 0.0, 1.0],
                },
            ]),
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
                Value::Point([1.0, 1.0, 0.0]),
                Value::Point([0.0, 1.0, 0.0]),
            ]),
        ];

        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("sweep polyline rail");

        let Value::List(solids) = outputs.get(PIN_OUTPUT_SURFACE).unwrap() else {
            panic!("expected list output");
        };
        let Value::Surface { vertices, faces } = &solids[0] else {
            panic!("expected surface solid");
        };

        assert_eq!(vertices.len(), 12, "expected a layer per rail segment");
        assert!(
            vertices
                .iter()
                .any(|vertex| (vertex[0] - 1.0).abs() < 1e-9 && (vertex[2] - 1.0).abs() < 1e-9),
            "sweep should reach end of second segment",
        );
        assert!(faces.len() >= 10, "caps and side faces should be present");
    }

    #[test]
    fn sweep_one_does_not_close_open_profile() {
        let component = ComponentKind::Sweep1;
        let inputs = [
            Value::CurveLine {
                p1: [0.0, 0.0, 0.0],
                p2: [0.0, 0.0, 1.0],
            },
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
                Value::Point([1.0, 1.0, 0.0]),
                Value::Point([0.0, 1.0, 0.0]),
            ]),
        ];

        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("sweep open profile");

        let Value::List(solids) = outputs.get(PIN_OUTPUT_SURFACE).unwrap() else {
            panic!("expected list output");
        };
        let Value::Surface { vertices, faces } = &solids[0] else {
            panic!("expected surface");
        };

        assert_eq!(vertices.len(), 8, "one segment should duplicate profile vertices once");
        assert_eq!(faces.len(), 6, "open profile should only create side strips without caps");
        assert!(
            faces.iter().all(|f| !(f.contains(&0) && f.contains(&3))),
            "open profile should not wrap between ends"
        );
    }

    #[test]
    fn sweep_one_merges_curve_primitive_rectangle() {
        // Four separate lines forming a rectangle (like Curve Primitives output).
        let component = ComponentKind::Sweep1;
        let inputs = [
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([0.0, 0.0, 1.0]),
            ]),
            Value::List(vec![
                Value::CurveLine {
                    p1: [0.0, 0.0, 0.0],
                    p2: [1.0, 0.0, 0.0],
                },
                Value::CurveLine {
                    p1: [1.0, 0.0, 0.0],
                    p2: [1.0, 1.0, 0.0],
                },
                Value::CurveLine {
                    p1: [1.0, 1.0, 0.0],
                    p2: [0.0, 1.0, 0.0],
                },
                Value::CurveLine {
                    p1: [0.0, 1.0, 0.0],
                    p2: [0.0, 0.0, 0.0],
                },
            ]),
        ];

        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("sweep merged rectangle");

        let Value::List(solids) = outputs.get(PIN_OUTPUT_SURFACE).unwrap() else {
            panic!("expected list output");
        };
        let Value::Surface { vertices, faces } = &solids[0] else {
            panic!("expected surface");
        };

        assert_eq!(vertices.len(), 8, "single rectangle swept along one segment");
        assert_eq!(faces.len(), 10, "caps plus four sides (triangulated)");
        // Should have one closed profile (no duplicate strips).
        assert!(faces.iter().all(|f| f.len() == 3), "triangulated faces expected");
    }

    #[test]
    fn sweep_one_ignores_duplicate_rail_points() {
        let component = ComponentKind::Sweep1;
        let inputs = [
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([0.0, 0.0, 0.0]), // duplicate
                Value::Point([0.0, 0.0, 1.0]),
            ]),
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
                Value::Point([1.0, 1.0, 0.0]),
                Value::Point([0.0, 1.0, 0.0]),
            ]),
        ];

        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("sweep with duplicate rail points");

        let Value::List(solids) = outputs.get(PIN_OUTPUT_SURFACE).unwrap() else {
            panic!("expected list output");
        };
        let Value::Surface { vertices, faces } = &solids[0] else {
            panic!("expected surface");
        };

        assert_eq!(vertices.len(), 8, "duplicate rail points should be ignored");
        assert_eq!(faces.len(), 6, "should still form one strip of faces");
    }

    #[test]
    fn ruled_surface_with_uneven_curves() {
        let component = ComponentKind::RuledSurface;
        let inputs = [
            // Curve A: 1 segment, 2 points
            Value::List(vec![Value::CurveLine {
                p1: [0.0, 0.0, 0.0],
                p2: [1.0, 0.0, 0.0],
            }]),
            // Curve B: 2 segments, 3 points
            Value::List(vec![
                Value::CurveLine {
                    p1: [0.0, 1.0, 0.0],
                    p2: [0.5, 1.0, 1.0],
                },
                Value::CurveLine {
                    p1: [0.5, 1.0, 1.0],
                    p2: [1.0, 1.0, 0.0],
                },
            ]),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("ruled surface");

        let Value::Surface { vertices, faces } = outputs.get(PIN_OUTPUT_SURFACE).unwrap() else {
            panic!("expected surface output");
        };

        // After resampling, both polylines should have 3 points. Total vertices = 6.
        assert_eq!(vertices.len(), 6, "Expected 6 vertices after resampling");
        // We should have 2 quads, which means 4 triangles.
        assert_eq!(faces.len(), 4, "Expected 4 faces for 2 quads");

        // Verify that the resampled version of curve A has a point in the middle.
        // Original was [0,0,0] -> [1,0,0]. Resampled should have a point at [0.5,0,0].
        assert_eq!(vertices[0], [0.0, 0.0, 0.0]);
        assert_eq!(vertices[1], [0.5, 0.0, 0.0]);
        assert_eq!(vertices[2], [1.0, 0.0, 0.0]);

        // Verify the original points of curve B are untouched.
        assert_eq!(vertices[3], [0.0, 1.0, 0.0]);
        assert_eq!(vertices[4], [0.5, 1.0, 1.0]);
        assert_eq!(vertices[5], [1.0, 1.0, 0.0]);
    }

    #[test]
    fn ruled_surface_with_line_and_arc() {
        let component = ComponentKind::RuledSurface;
        let inputs = [
            // Input A: A simple straight line (1 segment, 2 points)
            Value::CurveLine {
                p1: [0.0, 0.0, 0.0],
                p2: [2.0, 0.0, 0.0],
            },
            // Input B: A polyline simulating an arc (3 segments, 4 points)
            Value::List(vec![
                Value::Point([0.0, 2.0, 0.0]),
                Value::Point([1.0, 2.0, 1.0]),
                Value::Point([2.0, 2.0, 0.0]),
            ]),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("ruled surface with arc");

        let Value::Surface { vertices, faces } = outputs.get(PIN_OUTPUT_SURFACE).unwrap() else {
            panic!("expected surface output");
        };

        // Curve A has 2 points, Curve B has 3 points.
        // After resampling, both should have 3 points. Total vertices = 6.
        assert_eq!(vertices.len(), 6, "Expected 6 vertices after resampling");
        // (3 - 1) = 2 quads, which means 4 triangles.
        assert_eq!(faces.len(), 4, "Expected 4 faces");

        // Check that the line (Curve A) was resampled correctly.
        assert_eq!(vertices[0], [0.0, 0.0, 0.0]); // Start
        assert_eq!(vertices[1], [1.0, 0.0, 0.0]); // Interpolated midpoint
        assert_eq!(vertices[2], [2.0, 0.0, 0.0]); // End

        // Check that the arc points (Curve B) are preserved.
        assert_eq!(vertices[3], [0.0, 2.0, 0.0]);
        assert_eq!(vertices[4], [1.0, 2.0, 1.0]);
        assert_eq!(vertices[5], [2.0, 2.0, 0.0]);
    }

    #[test]
    fn ruled_surface_generates_multiple_pairs() {
        let component = ComponentKind::RuledSurface;
        let inputs = [
            Value::List(vec![
                Value::List(vec![
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Point([0.0, 0.0, 5.0]),
                ]),
                Value::List(vec![
                    Value::Point([2.0, 0.0, 0.0]),
                    Value::Point([2.0, 0.0, 5.0]),
                ]),
            ]),
            Value::List(vec![
                Value::List(vec![
                    Value::Point([1.0, 2.0, 0.0]),
                    Value::Point([1.0, 2.0, 5.0]),
                ]),
                Value::List(vec![
                    Value::Point([3.0, 2.0, 0.0]),
                    Value::Point([3.0, 2.0, 5.0]),
                ]),
            ]),
        ];

        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("multiple ruled surfaces");

        let Value::List(surfaces) = outputs.get(PIN_OUTPUT_SURFACE).unwrap() else {
            panic!("expected list of surfaces");
        };

        assert_eq!(surfaces.len(), 2, "expected two ruled surfaces");

        let expect_spans = [(0.0, 1.0), (2.0, 3.0)];
        for (surface, (min_x, max_x)) in surfaces.iter().zip(expect_spans) {
            let Value::Surface { vertices, faces } = surface else {
                panic!("expected surface value");
            };
            assert_eq!(faces.len(), 2, "each ruled pair should create 2 faces");

            let xs: Vec<f64> = vertices.iter().map(|v| v[0]).collect();
            let span = (
                xs.iter().fold(f64::INFINITY, |acc, value| acc.min(*value)),
                xs.iter()
                    .fold(f64::NEG_INFINITY, |acc, value| acc.max(*value)),
            );
            assert!((span.0 - min_x).abs() < 1e-9, "unexpected minimum span");
            assert!((span.1 - max_x).abs() < 1e-9, "unexpected maximum span");
        }
    }

    #[test]
    fn loft_with_three_curves() {
        let component = ComponentKind::Loft;
        let inputs = [Value::List(vec![
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
            ]),
            Value::List(vec![
                Value::Point([0.0, 1.0, 1.0]),
                Value::Point([1.0, 1.0, 1.0]),
                Value::Point([2.0, 1.0, 1.0]),
            ]),
            Value::List(vec![
                Value::Point([0.0, 0.0, 2.0]),
                Value::Point([1.0, 0.0, 2.0]),
            ]),
        ])];

        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("loft with three curves");

        let Value::Surface { vertices, faces } = outputs.get(PIN_OUTPUT_LOFT).unwrap() else {
            panic!("expected surface output");
        };

        // 3 curves, resampled to 3 points each = 9 vertices
        assert_eq!(vertices.len(), 9, "Expected 9 vertices");
        // 2 sections between 3 curves, each section is (3-1) * 2 = 4 triangles
        // Total faces = 2 * 4 = 8
        assert_eq!(faces.len(), 8, "Expected 8 faces");

        // Controleer of de middelste curve correct geïnterpoleerd is
        assert_eq!(vertices[3], [0.0, 1.0, 1.0]);
        assert_eq!(vertices[4], [1.0, 1.0, 1.0]);
        assert_eq!(vertices[5], [2.0, 1.0, 1.0]);
    }

    #[test]
    fn test_is_closed() {
        let closed_poly = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        assert!(is_closed(&closed_poly));
        let open_poly = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        assert!(!is_closed(&open_poly));
        let short_poly = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        assert!(!is_closed(&short_poly));
    }

    #[test]
    fn test_polyline_normal_and_winding() {
        // CCW square on XY plane
        let poly_ccw = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let normal_ccw = polyline_normal(&poly_ccw);
        assert!(
            (normal_ccw[2] - 1.0).abs() < EPSILON,
            "Expected normal to be Z-axis for CCW"
        );
        assert!(
            polyline_winding_direction(&poly_ccw, [0.0, 0.0, 1.0]) > 0.0,
            "Expected positive (CCW) winding"
        );

        // CW square on XY plane
        let mut poly_cw = poly_ccw.clone();
        poly_cw.reverse();
        let normal_cw = polyline_normal(&poly_cw);
        assert!(
            (normal_cw[2] + 1.0).abs() < EPSILON,
            "Expected normal to be negative Z-axis for CW"
        );
        assert!(
            polyline_winding_direction(&poly_cw, [0.0, 0.0, 1.0]) < 0.0,
            "Expected negative (CW) winding"
        );
    }

    #[test]
    fn loft_aligns_two_reversed_closed_curves() {
        let component = ComponentKind::Loft;
        let inputs = [Value::List(vec![
            // First rectangle, CCW
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
                Value::Point([1.0, 1.0, 0.0]),
                Value::Point([0.0, 1.0, 0.0]),
                Value::Point([0.0, 0.0, 0.0]),
            ]),
            // Second rectangle, CW, shifted in Z
            Value::List(vec![
                Value::Point([0.0, 0.0, 1.0]),
                Value::Point([0.0, 1.0, 1.0]),
                Value::Point([1.0, 1.0, 1.0]),
                Value::Point([1.0, 0.0, 1.0]),
                Value::Point([0.0, 0.0, 1.0]),
            ]),
        ])];

        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("loft failed");
        let Value::Surface { vertices, .. } = outputs.get(PIN_OUTPUT_LOFT).unwrap() else {
            panic!("expected surface output");
        };

        // After unify, the second curve (originally CW) should be reversed to match the first (CCW).
        let first_curve_second_pt = vertices[1];
        let second_curve_second_pt = vertices[6]; // 5 points per curve, so 5+1=6

        assert_eq!(first_curve_second_pt, [1.0, 0.0, 0.0]);
        // The original second point of the second curve was [0.0, 1.0, 1.0].
        // After reversal, the new second point should be [1.0, 0.0, 1.0].
        assert_eq!(second_curve_second_pt, [1.0, 0.0, 1.0]);
    }

    #[test]
    fn loft_aligns_mixed_open_and_closed_curves() {
        let component = ComponentKind::Loft;
        let inputs = [Value::List(vec![
            // Closed rectangle, CW
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([0.0, 1.0, 0.0]),
                Value::Point([1.0, 1.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
                Value::Point([0.0, 0.0, 0.0]),
            ]),
            // Open line that should NOT be reversed
            Value::List(vec![
                Value::Point([1.0, 0.0, 1.0]),
                Value::Point([0.0, 0.0, 1.0]),
            ]),
        ])];

        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("loft failed");
        let Value::Surface { vertices, .. } = outputs.get(PIN_OUTPUT_LOFT).unwrap() else {
            panic!("expected surface output");
        };

        // The closed polyline (CW) will be reversed. Its last non-closing point is [1,0,0].
        // The open line starts at [1,0,1] and ends at [0,0,1].
        // distance([1,0,0], start [1,0,1]) = 1
        // distance([1,0,0], end [0,0,1]) = sqrt(2)
        // Since 1 < sqrt(2), the line should NOT be reversed.
        let second_resampled_start = vertices[5];
        let second_resampled_end = vertices[9];

        assert_eq!(second_resampled_start, [1.0, 0.0, 1.0]);
        assert_eq!(second_resampled_end, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn loft_aligns_open_curves_by_distance() {
        let component = ComponentKind::Loft;
        let inputs = [Value::List(vec![
            // First line
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
            ]),
            // Second line, whose end is closer to the previous' end
            Value::List(vec![
                Value::Point([0.0, 1.0, 1.0]), // This start is far
                Value::Point([1.0, 1.0, 1.0]), // This end is near
            ]),
        ])];

        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("loft failed");
        let Value::Surface { vertices, .. } = outputs.get(PIN_OUTPUT_LOFT).unwrap() else {
            panic!("expected surface output");
        };

        // End of first line is [1,0,0].
        // Start of second is [0,1,1], distance = sqrt(3).
        // End of second is [1,1,1], distance = sqrt(2).
        // Since sqrt(2) < sqrt(3), the second line should be reversed.
        let second_curve_start = vertices[2];
        let second_curve_end = vertices[3];

        // Original was [0,1,1] -> [1,1,1]. Reversed is [1,1,1] -> [0,1,1]
        assert_eq!(second_curve_start, [1.0, 1.0, 1.0]);
        assert_eq!(second_curve_end, [0.0, 1.0, 1.0]);
    }
}
