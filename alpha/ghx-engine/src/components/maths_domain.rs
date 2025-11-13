//! Implementaties van Grasshopper "Maths → Domain" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::{Domain, Domain1D, Domain2D, Value};

use super::{Component, ComponentResult};

const PIN_INDEX: &str = "I";
const PIN_NEIGHBOUR: &str = "N";
const PIN_RESULT: &str = "R";
const PIN_CLIPPED: &str = "C";
const PIN_U_MIN: &str = "U0";
const PIN_U_MAX: &str = "U1";
const PIN_V_MIN: &str = "V0";
const PIN_V_MAX: &str = "V1";
const PIN_SEGMENTS: &str = "S";
const PIN_START: &str = "S";
const PIN_END: &str = "E";
const PIN_DOMAIN: &str = "I";
const PIN_DOMAIN_2D: &str = "I²";
const PIN_DOMAINS: &str = "D";
const PIN_U_COMPONENT: &str = "U";
const PIN_V_COMPONENT: &str = "V";
const PIN_INCLUDES: &str = "I";
const PIN_DEVIATION: &str = "D";

const EPSILON: f32 = 1e-9;

/// Beschikbare componenten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    FindDomain,
    RemapNumbers,
    DeconstructDomain2,
    DivideDomain2,
    DivideDomain,
    DeconstructDomain,
    ConstructDomain2,
    ConstructDomain2Numbers,
    ConsecutiveDomains,
    RemapNumbersList,
    ConstructDomain,
    Bounds2D,
    DeconstructDomain2Components,
    Includes,
    Bounds,
    RemapNumbersSingle,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst van componentregistraties voor de maths-domain componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{0b5c7fad-0473-41aa-bf52-d7a861dcaa29}"],
        names: &["Find Domain", "FDom"],
        kind: ComponentKind::FindDomain,
    },
    Registration {
        guids: &["{2fcc2743-8339-4cdf-a046-a1f17439191d}"],
        names: &["Remap Numbers", "Remap"],
        kind: ComponentKind::RemapNumbers,
    },
    Registration {
        guids: &["{47c30f9d-b685-4d4d-9b20-5b60e48d5af8}"],
        names: &["Deconstruct Domain²", "DeDom2Num"],
        kind: ComponentKind::DeconstructDomain2,
    },
    Registration {
        guids: &["{75ac008b-1bc2-4edd-b967-667d628b9d24}"],
        names: &["Divide Domain²", "Divide Domain 2"],
        kind: ComponentKind::DivideDomain2,
    },
    Registration {
        guids: &["{75ef4190-91a2-42d9-a245-32a7162b0384}"],
        names: &["Divide Domain", "Div"],
        kind: ComponentKind::DivideDomain,
    },
    Registration {
        guids: &["{825ea536-aebb-41e9-af32-8baeb2ecb590}"],
        names: &["Deconstruct Domain", "DeDomain"],
        kind: ComponentKind::DeconstructDomain,
    },
    Registration {
        guids: &["{8555a743-36c1-42b8-abcc-06d9cb94519f}"],
        names: &["Construct Domain²", "Dom2"],
        kind: ComponentKind::ConstructDomain2,
    },
    Registration {
        guids: &["{9083b87f-a98c-4e41-9591-077ae4220b19}"],
        names: &["Construct Domain² Numbers", "Dom2Num"],
        kind: ComponentKind::ConstructDomain2Numbers,
    },
    Registration {
        guids: &["{95992b33-89e1-4d36-bd35-2754a11af21e}"],
        names: &["Consecutive Domains", "Consec"],
        kind: ComponentKind::ConsecutiveDomains,
    },
    Registration {
        guids: &["{9624aeeb-f2a1-49da-b1c7-8789db217177}"],
        names: &["Remap Numbers List", "Remap List"],
        kind: ComponentKind::RemapNumbersList,
    },
    Registration {
        guids: &["{d1a28e95-cf96-4936-bf34-8bf142d731bf}"],
        names: &["Construct Domain", "Dom"],
        kind: ComponentKind::ConstructDomain,
    },
    Registration {
        guids: &["{dd53b24c-003a-4a04-b185-a44d91633cbe}"],
        names: &["Bounds 2D", "Bnd2D"],
        kind: ComponentKind::Bounds2D,
    },
    Registration {
        guids: &["{f0adfc96-b175-46a6-80c7-2b0ee17395c4}"],
        names: &["Deconstruct Domain² Components", "DeDom2"],
        kind: ComponentKind::DeconstructDomain2Components,
    },
    Registration {
        guids: &["{f217f873-92f1-47ae-ad71-ca3c5a45c3f8}"],
        names: &["Includes", "Inc"],
        kind: ComponentKind::Includes,
    },
    Registration {
        guids: &["{f44b92b0-3b5b-493a-86f4-fd7408c3daf3}"],
        names: &["Bounds", "Bnd"],
        kind: ComponentKind::Bounds,
    },
    Registration {
        guids: &["{fa314286-867b-41fa-a7f6-3f474197bb81}"],
        names: &["Remap Numbers Single", "Remap Single"],
        kind: ComponentKind::RemapNumbersSingle,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::FindDomain => evaluate_find_domain(inputs),
            Self::RemapNumbers => evaluate_remap_numbers(inputs),
            Self::DeconstructDomain2 => evaluate_deconstruct_domain2(inputs),
            Self::DivideDomain2 => evaluate_divide_domain2(inputs),
            Self::DivideDomain => evaluate_divide_domain(inputs),
            Self::DeconstructDomain => evaluate_deconstruct_domain(inputs),
            Self::ConstructDomain2 => evaluate_construct_domain2(inputs),
            Self::ConstructDomain2Numbers => evaluate_construct_domain2_numbers(inputs),
            Self::ConsecutiveDomains => evaluate_consecutive_domains(inputs),
            Self::RemapNumbersList => evaluate_remap_numbers_list(inputs),
            Self::ConstructDomain => evaluate_construct_domain(inputs),
            Self::Bounds2D => evaluate_bounds_2d(inputs),
            Self::DeconstructDomain2Components => evaluate_deconstruct_domain2_components(inputs),
            Self::Includes => evaluate_includes(inputs),
            Self::Bounds => evaluate_bounds(inputs),
            Self::RemapNumbersSingle => evaluate_remap_numbers_single(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::FindDomain => "Find Domain",
            Self::RemapNumbers => "Remap Numbers",
            Self::DeconstructDomain2 => "Deconstruct Domain²",
            Self::DivideDomain2 => "Divide Domain²",
            Self::DivideDomain => "Divide Domain",
            Self::DeconstructDomain => "Deconstruct Domain",
            Self::ConstructDomain2 => "Construct Domain²",
            Self::ConstructDomain2Numbers => "Construct Domain² Numbers",
            Self::ConsecutiveDomains => "Consecutive Domains",
            Self::RemapNumbersList => "Remap Numbers List",
            Self::ConstructDomain => "Construct Domain",
            Self::Bounds2D => "Bounds 2D",
            Self::DeconstructDomain2Components => "Deconstruct Domain² Components",
            Self::Includes => "Includes",
            Self::Bounds => "Bounds",
            Self::RemapNumbersSingle => "Remap Numbers Single",
        }
    }
}

fn evaluate_find_domain(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Ok(default_index_output(-1, -1));
    }

    let domains = collect_domain_list(inputs.get(0));
    let value = inputs.get(1).and_then(extract_number);
    let strict = inputs
        .get(2)
        .map(|value| coerce_boolean(value, false))
        .unwrap_or(false);

    if domains.is_empty() || value.map_or(true, |v| !v.is_finite()) {
        return Ok(default_index_output(-1, -1));
    }

    let value = value.unwrap();
    let mut first_match = -1;
    let mut closest_index = -1;
    let mut closest_distance = f32::INFINITY;

    for (idx, domain) in domains.iter().enumerate() {
        if first_match == -1 && is_value_in_domain(value, domain, strict) {
            first_match = idx as i32;
        }
        let distance = domain_distance(value, domain);
        if distance < closest_distance - EPSILON {
            closest_distance = distance;
            closest_index = idx as i32;
        }
    }

    Ok(default_index_output(first_match, closest_index))
}

fn evaluate_remap_numbers(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Ok(BTreeMap::new());
    }
    let source = coerce_domain1d(inputs.get(1));
    let target = coerce_domain1d(inputs.get(2));
    let (Some(source), Some(target)) = (source, target) else {
        return Ok(BTreeMap::new());
    };

    let value = inputs.get(0).and_then(extract_number);
    let mapped = value
        .filter(|v| v.is_finite())
        .map(|v| remap_value(v, &source, &target))
        .unwrap_or(target.start);
    let clipped_source = value
        .filter(|v| v.is_finite())
        .map(|v| clamp_value_to_domain(v, &source))
        .unwrap_or(source.start);
    let clipped = remap_value(clipped_source, &source, &target);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_RESULT.to_owned(), Value::Number(mapped));
    outputs.insert(PIN_CLIPPED.to_owned(), Value::Number(clipped));
    Ok(outputs)
}

fn evaluate_deconstruct_domain2(inputs: &[Value]) -> ComponentResult {
    let Some(domain) = coerce_domain2d(inputs.get(0)) else {
        return Ok(BTreeMap::new());
    };
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_U_MIN.to_owned(), Value::Number(domain.u.start));
    outputs.insert(PIN_U_MAX.to_owned(), Value::Number(domain.u.end));
    outputs.insert(PIN_V_MIN.to_owned(), Value::Number(domain.v.start));
    outputs.insert(PIN_V_MAX.to_owned(), Value::Number(domain.v.end));
    Ok(outputs)
}

fn evaluate_divide_domain2(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Ok(BTreeMap::new());
    }
    let Some(domain) = coerce_domain2d(inputs.get(0)) else {
        return Ok(BTreeMap::new());
    };
    let u_count = inputs
        .get(1)
        .and_then(extract_number)
        .map_or(0, |value| sanitize_segment_count(value));
    let v_count = inputs
        .get(2)
        .and_then(extract_number)
        .map_or(0, |value| sanitize_segment_count(value));
    if u_count == 0 || v_count == 0 {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_SEGMENTS.to_owned(), Value::List(vec![]));
        return Ok(outputs);
    }

    let mut segments = Vec::with_capacity(u_count * v_count);
    for u in subdivide_domain(&domain.u, u_count) {
        for v in subdivide_domain(&domain.v, v_count) {
            segments.push(Value::Domain(Domain::Two(Domain2D { u: u.clone(), v })));
        }
    }
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_SEGMENTS.to_owned(), Value::List(segments));
    Ok(outputs)
}

fn evaluate_divide_domain(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Ok(BTreeMap::new());
    }
    let Some(domain) = coerce_domain1d(inputs.get(0)) else {
        return Ok(BTreeMap::new());
    };
    let count = inputs
        .get(1)
        .and_then(extract_number)
        .map_or(0, |value| sanitize_segment_count(value));
    let segments: Vec<Value> = subdivide_domain(&domain, count)
        .into_iter()
        .map(|segment| Value::Domain(Domain::One(segment)))
        .collect();
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_SEGMENTS.to_owned(), Value::List(segments));
    Ok(outputs)
}

fn evaluate_deconstruct_domain(inputs: &[Value]) -> ComponentResult {
    let Some(domain) = coerce_domain1d(inputs.get(0)) else {
        return Ok(BTreeMap::new());
    };
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_START.to_owned(), Value::Number(domain.start));
    outputs.insert(PIN_END.to_owned(), Value::Number(domain.end));
    Ok(outputs)
}

fn evaluate_construct_domain2(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Ok(BTreeMap::new());
    }
    let Some(u) = coerce_domain1d(inputs.get(0)) else {
        return Ok(BTreeMap::new());
    };
    let Some(v) = coerce_domain1d(inputs.get(1)) else {
        return Ok(BTreeMap::new());
    };
    let domain = Domain2D { u, v };
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_DOMAIN_2D.to_owned(), Value::Domain(Domain::Two(domain)));
    Ok(outputs)
}

fn evaluate_construct_domain2_numbers(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 4 {
        return Ok(BTreeMap::new());
    }
    let u0 = inputs.get(0).and_then(extract_number);
    let u1 = inputs.get(1).and_then(extract_number);
    let v0 = inputs.get(2).and_then(extract_number);
    let v1 = inputs.get(3).and_then(extract_number);
    let (Some(u0), Some(u1), Some(v0), Some(v1)) = (u0, u1, v0, v1) else {
        return Ok(BTreeMap::new());
    };
    let Some(u) = create_domain(u0, u1) else {
        return Ok(BTreeMap::new());
    };
    let Some(v) = create_domain(v0, v1) else {
        return Ok(BTreeMap::new());
    };
    let domain = Domain2D { u, v };
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_DOMAIN_2D.to_owned(), Value::Domain(Domain::Two(domain)));
    Ok(outputs)
}

fn evaluate_consecutive_domains(inputs: &[Value]) -> ComponentResult {
    let values = collect_numbers(inputs.get(0));
    if values.is_empty() {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_DOMAINS.to_owned(), Value::List(vec![]));
        return Ok(outputs);
    }

    let additive = inputs
        .get(1)
        .map(|value| coerce_boolean(value, false))
        .unwrap_or(false);

    let mut result = Vec::new();
    if additive {
        let mut start = 0.0;
        for length in values {
            if !length.is_finite() {
                continue;
            }
            let end = start + length;
            if let Some(domain) = create_domain(start, end) {
                result.push(Value::Domain(Domain::One(domain)));
            }
            start = end;
        }
    } else {
        let mut unique = values;
        unique.sort_by(|a, b| a.partial_cmp(b).unwrap());
        unique.dedup_by(|a, b| (*a - *b).abs() < EPSILON);
        if unique.len() > 1 {
            for pair in unique.windows(2) {
                if let Some(domain) = create_domain(pair[0], pair[1]) {
                    result.push(Value::Domain(Domain::One(domain)));
                }
            }
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_DOMAINS.to_owned(), Value::List(result));
    Ok(outputs)
}

fn evaluate_remap_numbers_list(inputs: &[Value]) -> ComponentResult {
    let values = collect_numbers(inputs.get(0));
    if values.is_empty() {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_RESULT.to_owned(), Value::List(vec![]));
        return Ok(outputs);
    }
    let target = coerce_domain1d(inputs.get(2));
    let Some(target) = target else {
        let mut outputs = BTreeMap::new();
        let list = values.into_iter().map(Value::Number).collect();
        outputs.insert(PIN_RESULT.to_owned(), Value::List(list));
        return Ok(outputs);
    };
    let source = coerce_domain1d(inputs.get(1)).or_else(|| compute_domain_from_numbers(&values));
    let Some(source) = source else {
        let mut outputs = BTreeMap::new();
        let list = values.iter().map(|_| Value::Number(target.start)).collect();
        outputs.insert(PIN_RESULT.to_owned(), Value::List(list));
        return Ok(outputs);
    };

    let remapped = values
        .into_iter()
        .map(|value| remap_value(value, &source, &target))
        .map(Value::Number)
        .collect();
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_RESULT.to_owned(), Value::List(remapped));
    Ok(outputs)
}

fn evaluate_construct_domain(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Ok(BTreeMap::new());
    }
    let (Some(start), Some(end)) = (
        inputs.get(0).and_then(extract_number),
        inputs.get(1).and_then(extract_number),
    ) else {
        return Ok(BTreeMap::new());
    };
    let Some(domain) = create_domain(start, end) else {
        return Ok(BTreeMap::new());
    };
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_DOMAIN.to_owned(), Value::Domain(Domain::One(domain)));
    Ok(outputs)
}

fn evaluate_bounds_2d(inputs: &[Value]) -> ComponentResult {
    let pairs = collect_coordinate_pairs(inputs.get(0));
    if pairs.is_empty() {
        return Ok(BTreeMap::new());
    }
    let (mut min_x, mut max_x) = (f32::INFINITY, f32::NEG_INFINITY);
    let (mut min_y, mut max_y) = (f32::INFINITY, f32::NEG_INFINITY);
    for (x, y) in pairs {
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
    }
    if !min_x.is_finite() || !max_x.is_finite() || !min_y.is_finite() || !max_y.is_finite() {
        return Ok(BTreeMap::new());
    }
    let Some(u) = create_domain(min_x, max_x) else {
        return Ok(BTreeMap::new());
    };
    let Some(v) = create_domain(min_y, max_y) else {
        return Ok(BTreeMap::new());
    };
    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_DOMAIN.to_owned(),
        Value::Domain(Domain::Two(Domain2D { u, v })),
    );
    Ok(outputs)
}

fn evaluate_deconstruct_domain2_components(inputs: &[Value]) -> ComponentResult {
    let Some(domain) = coerce_domain2d(inputs.get(0)) else {
        return Ok(BTreeMap::new());
    };
    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_U_COMPONENT.to_owned(),
        Value::Domain(Domain::One(domain.u.clone())),
    );
    outputs.insert(
        PIN_V_COMPONENT.to_owned(),
        Value::Domain(Domain::One(domain.v)),
    );
    Ok(outputs)
}

fn evaluate_includes(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Ok(BTreeMap::new());
    }
    let Some(domain) = coerce_domain1d(inputs.get(1)) else {
        return Ok(BTreeMap::new());
    };
    let Some(value) = inputs.get(0).and_then(extract_number) else {
        return Ok(BTreeMap::new());
    };
    if !value.is_finite() {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_INCLUDES.to_owned(), Value::Boolean(false));
        outputs.insert(PIN_DEVIATION.to_owned(), Value::Number(f32::NAN));
        return Ok(outputs);
    }
    let includes = is_value_in_domain(value, &domain, false);
    let deviation = if includes {
        0.0
    } else {
        domain_distance(value, &domain)
    };
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_INCLUDES.to_owned(), Value::Boolean(includes));
    outputs.insert(PIN_DEVIATION.to_owned(), Value::Number(deviation));
    Ok(outputs)
}

fn evaluate_bounds(inputs: &[Value]) -> ComponentResult {
    let numbers = collect_numbers(inputs.get(0));
    let Some(domain) = compute_domain_from_numbers(&numbers) else {
        return Ok(BTreeMap::new());
    };
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_DOMAIN.to_owned(), Value::Domain(Domain::One(domain)));
    Ok(outputs)
}

fn evaluate_remap_numbers_single(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Ok(BTreeMap::new());
    }
    let Some(source) = coerce_domain1d(inputs.get(1)) else {
        return Ok(BTreeMap::new());
    };
    let Some(target) = coerce_domain1d(inputs.get(2)) else {
        return Ok(BTreeMap::new());
    };
    let value = inputs.get(0).and_then(extract_number);
    let result = value
        .filter(|v| v.is_finite())
        .map(|v| remap_value(v, &source, &target))
        .unwrap_or(target.start);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_RESULT.to_owned(), Value::Number(result));
    Ok(outputs)
}

fn default_index_output(index: i32, neighbour: i32) -> BTreeMap<String, Value> {
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_INDEX.to_owned(), Value::Number(index as f32));
    outputs.insert(PIN_NEIGHBOUR.to_owned(), Value::Number(neighbour as f32));
    outputs
}

fn extract_number(value: &Value) -> Option<f32> {
    match value {
        Value::Number(number) if number.is_finite() => Some(*number),
        Value::Number(number) if number.is_nan() => None,
        Value::List(values) if !values.is_empty() => extract_number(&values[0]),
        _ => None,
    }
}

fn coerce_boolean(value: &Value, default: bool) -> bool {
    match value {
        Value::Boolean(state) => *state,
        Value::Number(number) => *number != 0.0,
        Value::List(values) if !values.is_empty() => coerce_boolean(&values[0], default),
        _ => default,
    }
}

fn coerce_domain1d(value: Option<&Value>) -> Option<Domain1D> {
    value.and_then(parse_domain1d)
}

fn parse_domain1d(value: &Value) -> Option<Domain1D> {
    match value {
        Value::Domain(Domain::One(domain)) => Some(domain.clone()),
        Value::Domain(Domain::Two(_)) => None,
        Value::Number(number) => create_domain(*number, *number),
        Value::List(values) => {
            if values.len() >= 2 {
                let start = values.get(0).and_then(extract_number);
                let end = values.get(1).and_then(extract_number);
                match (start, end) {
                    (Some(start), Some(end)) => create_domain(start, end),
                    _ => None,
                }
            } else if values.len() == 1 {
                coerce_domain1d(values.get(0))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn coerce_domain2d(value: Option<&Value>) -> Option<Domain2D> {
    value.and_then(parse_domain2d)
}

fn parse_domain2d(value: &Value) -> Option<Domain2D> {
    match value {
        Value::Domain(Domain::Two(domain)) => Some(domain.clone()),
        Value::List(values) => {
            if values.len() >= 4 {
                let u0 = values.get(0).and_then(extract_number);
                let u1 = values.get(1).and_then(extract_number);
                let v0 = values.get(2).and_then(extract_number);
                let v1 = values.get(3).and_then(extract_number);
                if let (Some(u0), Some(u1), Some(v0), Some(v1)) = (u0, u1, v0, v1) {
                    if let (Some(u), Some(v)) = (create_domain(u0, u1), create_domain(v0, v1)) {
                        return Some(Domain2D { u, v });
                    }
                }
            }
            if values.len() >= 2 {
                let u = coerce_domain1d(values.get(0));
                let v = coerce_domain1d(values.get(1));
                if let (Some(u), Some(v)) = (u, v) {
                    return Some(Domain2D { u, v });
                }
            }
            if values.len() == 1 {
                coerce_domain2d(values.get(0))
            } else {
                None
            }
        }
        Value::Domain(Domain::One(_)) => None,
        _ => None,
    }
}

fn create_domain(start: f32, end: f32) -> Option<Domain1D> {
    if !start.is_finite() || !end.is_finite() {
        return None;
    }
    let min = start.min(end);
    let max = start.max(end);
    let span = end - start;
    let length = max - min;
    let center = (start + end) / 2.0;
    Some(Domain1D {
        start,
        end,
        min,
        max,
        span,
        length,
        center,
    })
}

fn collect_domain_list(value: Option<&Value>) -> Vec<Domain1D> {
    let mut result = Vec::new();
    if let Some(value) = value {
        collect_domains_recursive(value, &mut result);
    }
    result
}

fn collect_domains_recursive(value: &Value, result: &mut Vec<Domain1D>) {
    match value {
        Value::List(values) => {
            for entry in values {
                if let Some(domain) = coerce_domain1d(Some(entry)) {
                    result.push(domain);
                } else {
                    collect_domains_recursive(entry, result);
                }
            }
        }
        _ => {
            if let Some(domain) = coerce_domain1d(Some(value)) {
                result.push(domain);
            }
        }
    }
}

fn domain_distance(value: f32, domain: &Domain1D) -> f32 {
    if value < domain.min {
        domain.min - value
    } else if value > domain.max {
        value - domain.max
    } else {
        0.0
    }
}

fn is_value_in_domain(value: f32, domain: &Domain1D, strict: bool) -> bool {
    if strict {
        if domain.length <= EPSILON {
            return false;
        }
        value > domain.min && value < domain.max
    } else {
        value >= domain.min - EPSILON && value <= domain.max + EPSILON
    }
}

fn clamp_value_to_domain(value: f32, domain: &Domain1D) -> f32 {
    if value < domain.min {
        domain.min
    } else if value > domain.max {
        domain.max
    } else {
        value
    }
}

fn remap_value(value: f32, source: &Domain1D, target: &Domain1D) -> f32 {
    let source_span = source.end - source.start;
    if source_span.abs() <= EPSILON {
        return target.start;
    }
    let ratio = (value - source.start) / source_span;
    target.start + ratio * (target.end - target.start)
}

fn collect_numbers(value: Option<&Value>) -> Vec<f32> {
    let mut result = Vec::new();
    if let Some(value) = value {
        collect_numbers_recursive(value, &mut result);
    }
    result
}

fn collect_numbers_recursive(value: &Value, result: &mut Vec<f32>) {
    match value {
        Value::Number(number) if number.is_finite() => result.push(*number),
        Value::List(values) => {
            if values.len() == 2 {
                if let (Some(a), Some(b)) = (extract_number(&values[0]), extract_number(&values[1]))
                {
                    result.push(a);
                    result.push(b);
                    return;
                }
            }
            for entry in values {
                collect_numbers_recursive(entry, result);
            }
        }
        Value::Domain(Domain::One(domain)) => {
            result.push(domain.start);
            result.push(domain.end);
        }
        Value::Domain(Domain::Two(domain)) => {
            collect_numbers_recursive(&Value::Domain(Domain::One(domain.u.clone())), result);
            collect_numbers_recursive(&Value::Domain(Domain::One(domain.v.clone())), result);
        }
        _ => {}
    }
}

fn collect_coordinate_pairs(value: Option<&Value>) -> Vec<(f32, f32)> {
    let mut result = Vec::new();
    if let Some(value) = value {
        collect_coordinate_pairs_recursive(value, &mut result);
    }
    result
}

fn collect_coordinate_pairs_recursive(value: &Value, result: &mut Vec<(f32, f32)>) {
    match value {
        Value::Point([x, y, _]) | Value::Vector([x, y, _]) => {
            if x.is_finite() && y.is_finite() {
                result.push((*x, *y));
            }
        }
        Value::List(values) => {
            if values.len() >= 2 {
                if let (Some(x), Some(y)) = (extract_number(&values[0]), extract_number(&values[1]))
                {
                    result.push((x, y));
                    return;
                }
            }
            for entry in values {
                collect_coordinate_pairs_recursive(entry, result);
            }
        }
        _ => {}
    }
}

fn compute_domain_from_numbers(numbers: &[f32]) -> Option<Domain1D> {
    if numbers.is_empty() {
        return None;
    }
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    for number in numbers {
        if !number.is_finite() {
            continue;
        }
        if *number < min {
            min = *number;
        }
        if *number > max {
            max = *number;
        }
    }
    if !min.is_finite() || !max.is_finite() {
        return None;
    }
    create_domain(min, max)
}

fn sanitize_segment_count(value: f32) -> usize {
    if !value.is_finite() {
        return 0;
    }
    let count = value.floor() as i64;
    if count <= 0 { 0 } else { count as usize }
}

fn subdivide_domain(domain: &Domain1D, count: usize) -> Vec<Domain1D> {
    if count == 0 {
        return Vec::new();
    }
    let step = (domain.end - domain.start) / count as f32;
    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        let start = domain.start + step * i as f32;
        let end = if i == count - 1 {
            domain.end
        } else {
            domain.start + step * (i + 1) as f32
        };
        if let Some(segment) = create_domain(start, end) {
            result.push(segment);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Component;
    use crate::graph::node::MetaMap;

    #[test]
    fn construct_domain_creates_valid_domain() {
        let component = ComponentKind::ConstructDomain;
        let outputs = component
            .evaluate(&[Value::Number(5.0), Value::Number(1.0)], &MetaMap::new())
            .expect("construct domain succeeds");
        let Value::Domain(Domain::One(domain)) = outputs.get(PIN_DOMAIN).unwrap() else {
            panic!("expected domain output");
        };
        assert!((domain.start - 5.0).abs() < EPSILON);
        assert!((domain.end - 1.0).abs() < EPSILON);
        assert_eq!(domain.min, 1.0);
        assert_eq!(domain.max, 5.0);
    }

    #[test]
    fn find_domain_returns_index_and_neighbour() {
        let component = ComponentKind::FindDomain;
        let inputs = [
            Value::List(vec![
                Value::List(vec![Value::Number(0.0), Value::Number(5.0)]),
                Value::List(vec![Value::Number(10.0), Value::Number(20.0)]),
            ]),
            Value::Number(12.0),
            Value::Boolean(false),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("find domain succeeds");
        assert!(
            matches!(outputs.get(PIN_INDEX), Some(Value::Number(value)) if (*value - 1.0).abs() < EPSILON)
        );
        assert!(
            matches!(outputs.get(PIN_NEIGHBOUR), Some(Value::Number(value)) if (*value - 1.0).abs() < EPSILON)
        );
    }

    #[test]
    fn divide_domain_produces_segments() {
        let component = ComponentKind::DivideDomain;
        let inputs = [
            Value::List(vec![Value::Number(0.0), Value::Number(10.0)]),
            Value::Number(4.0),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("divide domain succeeds");
        let Value::List(segments) = outputs.get(PIN_SEGMENTS).unwrap() else {
            panic!("expected segment list");
        };
        assert_eq!(segments.len(), 4);
    }

    #[test]
    fn remap_numbers_list_uses_implicit_source_domain() {
        let component = ComponentKind::RemapNumbersList;
        let inputs = [
            Value::List(vec![Value::Number(0.0), Value::Number(5.0)]),
            Value::List(vec![]),
            Value::List(vec![Value::Number(0.0), Value::Number(10.0)]),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("remap numbers list succeeds");
        let Value::List(values) = outputs.get(PIN_RESULT).unwrap() else {
            panic!("expected list output");
        };
        assert!(matches!(values.as_slice(),
            [Value::Number(a), Value::Number(b)] if (*a - 0.0).abs() < EPSILON && (*b - 10.0).abs() < EPSILON
        ));
    }

    #[test]
    fn includes_reports_deviation_outside_domain() {
        let component = ComponentKind::Includes;
        let inputs = [
            Value::Number(15.0),
            Value::List(vec![Value::Number(0.0), Value::Number(10.0)]),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("includes succeeds");
        assert!(matches!(
            outputs.get(PIN_INCLUDES),
            Some(Value::Boolean(false))
        ));
        assert!(
            matches!(outputs.get(PIN_DEVIATION), Some(Value::Number(value)) if (*value - 5.0).abs() < EPSILON)
        );
    }
}
