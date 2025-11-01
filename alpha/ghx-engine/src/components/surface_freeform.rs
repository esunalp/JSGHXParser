//! Implementaties van Grasshopper "Surface → Freeform" componenten.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::{Domain, Value};

use super::{Component, ComponentError, ComponentResult};

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
            Self::ControlPointLoft => evaluate_loft(inputs, "Control Point Loft", PIN_OUTPUT_SURFACE),
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

fn evaluate_loft(inputs: &[Value], component: &str, output: &str) -> ComponentResult {
    let curves_value = expect_input(inputs, 0, component, "curveverzameling")?;
    let segments = collect_curve_segments(curves_value, component)?;
    if segments.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} vereist minimaal twee sectiecurves"
        )));
    }

    let mut points = Vec::new();
    for (start, end) in segments {
        points.push(start);
        points.push(end);
    }

    let surface = create_surface_from_points(&points, component)?;
    into_output(output, surface)
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
        let segments = collect_curve_segments(value, component)?;
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
    let base_segments = collect_curve_segments(&inputs[0], component)?;
    if base_segments.is_empty() {
        return Err(ComponentError::new(
            "Extrude Along kon geen basiscurve herkennen",
        ));
    }
    let rail_segments = collect_curve_segments(&inputs[1], component)?;
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
        collect_curve_segments(&inputs[0], component)?
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
        return Err(ComponentError::new(
            "Sum Surface vereist twee invoercurves",
        ));
    }

    let mut points = Vec::new();
    points.extend(
        collect_curve_segments(&inputs[0], component)?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );
    points.extend(
        collect_curve_segments(&inputs[1], component)?
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

fn evaluate_ruled_surface(inputs: &[Value]) -> ComponentResult {
    let component = "Ruled Surface";
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Ruled Surface vereist twee invoercurves",
        ));
    }
    let curve_a = collect_curve_segments(&inputs[0], component)?;
    let curve_b = collect_curve_segments(&inputs[1], component)?;
    if curve_a.is_empty() || curve_b.is_empty() {
        return Err(ComponentError::new(
            "Ruled Surface kon geen volledige curves interpreteren",
        ));
    }

    let mut points = Vec::new();
    for (a, b) in curve_a.into_iter().zip(curve_b.into_iter()) {
        points.push(a.0);
        points.push(a.1);
        points.push(b.0);
        points.push(b.1);
    }

    let surface = create_surface_from_points(&points, component)?;
    into_output(PIN_OUTPUT_SURFACE, surface)
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
        collect_curve_segments(&inputs[0], component)?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );
    points.extend(
        collect_curve_segments(&inputs[1], component)?
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
        return Err(ComponentError::new(
            "Sweep2 vereist twee rails en secties",
        ));
    }

    let mut points = Vec::new();
    points.extend(
        collect_curve_segments(&inputs[0], component)?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );
    points.extend(
        collect_curve_segments(&inputs[1], component)?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );
    points.extend(
        collect_curve_segments(&inputs[2], component)?
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
    let segments = collect_curve_segments(&inputs[0], component)?;
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

    let average_radius = radii.iter().map(|value| value.abs()).sum::<f64>()
        / radii.len() as f64;

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
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Extrude Linear vereist een profiel en een as",
        ));
    }

    let profile_segments = collect_curve_segments(&inputs[0], component)?;
    if profile_segments.is_empty() {
        return Err(ComponentError::new(
            "Extrude Linear kon geen profielcurve herkennen",
        ));
    }
    let axis_direction = coerce_direction(&inputs[2], component, "Axis")?;
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
        return Err(ComponentError::new(
            "Sweep1 vereist een rail en secties",
        ));
    }

    let mut points = Vec::new();
    points.extend(
        collect_curve_segments(&inputs[0], component)?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );
    points.extend(
        collect_curve_segments(&inputs[1], component)?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );

    if let Some(value) = inputs.get(2) {
        coerce_number(value, component, "Miter")?;
    }

    let surface = create_surface_from_points(&points, component)?;
    into_output(PIN_OUTPUT_SURFACE, Value::List(vec![surface]))
}

fn evaluate_extrude_point(inputs: &[Value]) -> ComponentResult {
    let component = "Extrude Point";
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Extrude Point vereist een basis en een doelpunt",
        ));
    }

    let base_segments = collect_curve_segments(&inputs[0], component)?;
    if base_segments.is_empty() {
        return Err(ComponentError::new(
            "Extrude Point kon de basiscurve niet lezen",
        ));
    }
    let tip = coerce_point(&inputs[1], component, "Point")?;

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
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Pipe vereist minimaal een curve en een straal",
        ));
    }

    let segments = collect_curve_segments(&inputs[0], component)?;
    if segments.is_empty() {
        return Err(ComponentError::new(
            "Pipe kon de railcurve niet lezen",
        ));
    }
    let radius = coerce_number(&inputs[1], component, "Radius")?.abs();
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

    for index in 0..inputs.len().min(4) {
        points.extend(collect_points(&inputs[index], component)?);
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
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Revolution vereist profiel, as en domein",
        ));
    }
    let profile_segments = collect_curve_segments(&inputs[0], component)?;
    let axis_segments = collect_curve_segments(&inputs[1], component)?;
    if profile_segments.is_empty() || axis_segments.is_empty() {
        return Err(ComponentError::new(
            "Revolution kon profiel of as niet lezen",
        ));
    }
    let angle = coerce_angle_domain(&inputs[2], component)?;

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
    let segments = collect_curve_segments(edges, component)?;
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
        collect_curve_segments(&inputs[0], component)?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );
    points.extend(
        collect_curve_segments(&inputs[1], component)?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );
    points.extend(
        collect_curve_segments(&inputs[2], component)?
            .into_iter()
            .flat_map(|(a, b)| [a, b]),
    );
    let scale = coerce_number(&inputs[3], component, "Scale")?.abs();

    let surface = create_surface_from_points_with_padding(&points, scale, component)?;
    into_output(PIN_OUTPUT_SURFACE, surface)
}

fn collect_curve_segments(
    value: &Value,
    component: &str,
) -> Result<Vec<([f64; 3], [f64; 3])>, ComponentError> {
    match value {
        Value::CurveLine { p1, p2 } => Ok(vec![(*p1, *p2)]),
        Value::List(values) => {
            let mut segments = Vec::new();
            for entry in values {
                segments.extend(collect_curve_segments(entry, component)?);
            }
            Ok(segments)
        }
        Value::Surface { vertices, .. } => {
            if vertices.len() < 2 {
                return Err(ComponentError::new(format!(
                    "{component} verwachtte ten minste twee punten in het oppervlak"
                )));
            }
            let mut segments = Vec::new();
            for pair in vertices.windows(2) {
                segments.push((pair[0], pair[1]));
            }
            Ok(segments)
        }
        other => Err(ComponentError::new(format!(
            "{component} verwacht een curve-achtige invoer, kreeg {}",
            other.kind()
        ))),
    }
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

fn coerce_direction(value: &Value, component: &str, name: &str) -> Result<[f64; 3], ComponentError> {
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
        Value::Domain(Domain::Two(domain)) => {
            Ok(domain.u.length.abs().max(domain.v.length.abs()))
        }
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
    sorted.sort_by(|a, b| match (a.0.partial_cmp(&b.0), b.0.partial_cmp(&a.0)) {
        (Some(order), _) => order.reverse(),
        (None, Some(order)) => order,
        _ => Ordering::Equal,
    });

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
        ComponentError::new(format!(
            "{component} vereist een invoer voor {description}"
        ))
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

fn is_zero_vector(vector: [f64; 3]) -> bool {
    vector.iter().all(|component| component.abs() < EPSILON)
}

fn into_output(pin: &str, value: Value) -> ComponentResult {
    let mut outputs = BTreeMap::new();
    outputs.insert(pin.to_owned(), value);
    Ok(outputs)
}

#[cfg(test)]
mod tests {
    use super::{
        Component, ComponentKind, PIN_OUTPUT_EXTRUSION, PIN_OUTPUT_LOFT, PIN_OUTPUT_OPTIONS,
        PIN_OUTPUT_PIPE, PIN_OUTPUT_SURFACE,
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

        assert!(matches!(outputs.get(PIN_OUTPUT_LOFT), Some(Value::Surface { .. })));
    }
}
