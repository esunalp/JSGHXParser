//! Implementaties van Grasshopper "Sets -> List" componenten.

use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::graph::node::{MetaLookupExt, MetaMap, MetaValue};
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_LIST: &str = "L";
const PIN_OUTPUT_ELEMENT: &str = "E";
const PIN_OUTPUT_LIST_A: &str = "A";
const PIN_OUTPUT_LIST_B: &str = "B";
const PIN_OUTPUT_KEYS: &str = "K";
const PIN_OUTPUT_RESULT: &str = "R";
const PIN_OUTPUT_WEAVE: &str = "W";
const PIN_OUTPUT_CHUNKS: &str = "C";
const META_OUTPUT_PINS: &str = "OutputPins";

/// Beschikbare componenten binnen Sets -> List.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    ListItem,
    ListLength,
    ReverseList,
    Dispatch,
    SortList,
    ShiftList,
    InsertItems,
    PickAndChoose,
    Weave,
    SiftPattern,
    CrossReference,
    ShortestList,
    LongestList,
    PartitionList,
    SplitList,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Registraties van alle Sets -> List componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &[
            "{285ddd8a-5398-4a3e-b3c2-361025711a51}",
            "{59daf374-bc21-4a5e-8282-5504fb7ae9ae}",
            "{6e2ba21a-2252-42f4-8d3f-f5e0f49cc4ef}",
        ],
        names: &["List Item", "Item"],
        kind: ComponentKind::ListItem,
    },
    Registration {
        guids: &["{1817fd29-20ae-4503-b542-f0fb651e67d7}"],
        names: &["List Length", "Lng"],
        kind: ComponentKind::ListLength,
    },
    Registration {
        guids: &["{6ec97ea8-c559-47a2-8d0f-ce80c794d1f4}"],
        names: &["Reverse List", "Rev"],
        kind: ComponentKind::ReverseList,
    },
    Registration {
        guids: &["{d8332545-21b2-4716-96e3-8559a9876e17}"],
        names: &["Dispatch"],
        kind: ComponentKind::Dispatch,
    },
    Registration {
        guids: &[
            "{2b2628ea-3f43-4ce9-8435-9a045d54b5c6}",
            "{6f93d366-919f-4dda-a35e-ba03dd62799b}",
            "{cacb2c64-61b5-46db-825d-c61d5d09cc08}",
        ],
        names: &["Sort List", "Sort"],
        kind: ComponentKind::SortList,
    },
    Registration {
        guids: &["{4fdfe351-6c07-47ce-9fb9-be027fb62186}"],
        names: &["Shift List", "Shift"],
        kind: ComponentKind::ShiftList,
    },
    Registration {
        guids: &["{e2039b07-d3f3-40f8-af88-d74fed238727}"],
        names: &["Insert Items", "Ins"],
        kind: ComponentKind::InsertItems,
    },
    Registration {
        guids: &[
            "{03b801eb-87cd-476a-a591-257fe5d5bf0f}",
            "{4356ef8f-0ca1-4632-9c39-9e6dcd2b9496}",
        ],
        names: &["Pick'n'Choose", "P'n'C"],
        kind: ComponentKind::PickAndChoose,
    },
    Registration {
        guids: &[
            "{160c1df2-e2e8-48e5-b538-f2d6981007e3}",
            "{50faccbd-9c92-4175-a5fa-d65e36013db6}",
        ],
        names: &["Weave"],
        kind: ComponentKind::Weave,
    },
    Registration {
        guids: &["{3249222f-f536-467a-89f4-f0353fba455a}"],
        names: &["Sift Pattern", "Sift"],
        kind: ComponentKind::SiftPattern,
    },
    Registration {
        guids: &["{36947590-f0cb-4807-a8f9-9c90c9b20621}"],
        names: &["Cross Reference", "CrossRef"],
        kind: ComponentKind::CrossReference,
    },
    Registration {
        guids: &["{5a13ec19-e4e9-43da-bf65-f93025fa87ca}"],
        names: &["Shortest List", "Short"],
        kind: ComponentKind::ShortestList,
    },
    Registration {
        guids: &["{8440fd1b-b6e0-4bdb-aa93-4ec295c213e9}"],
        names: &["Longest List", "Long"],
        kind: ComponentKind::LongestList,
    },
    Registration {
        guids: &["{5a93246d-2595-4c28-bc2d-90657634f92a}"],
        names: &["Partition List", "Partition"],
        kind: ComponentKind::PartitionList,
    },
    Registration {
        guids: &["{9ab93e1a-ebdf-4090-9296-b000cff7b202}"],
        names: &["Split List", "Split"],
        kind: ComponentKind::SplitList,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::ListItem => evaluate_list_item(inputs, meta),
            Self::ListLength => evaluate_list_length(inputs, meta),
            Self::ReverseList => evaluate_reverse_list(inputs, meta),
            Self::Dispatch => evaluate_dispatch(inputs, meta),
            Self::SortList => evaluate_sort_list(inputs, meta),
            Self::ShiftList => evaluate_shift_list(inputs, meta),
            Self::InsertItems => evaluate_insert_items(inputs, meta),
            Self::PickAndChoose => evaluate_pick_and_choose(inputs, meta),
            Self::Weave => evaluate_weave(inputs, meta),
            Self::SiftPattern => evaluate_sift_pattern(inputs, meta),
            Self::CrossReference => evaluate_cross_reference(inputs, meta),
            Self::ShortestList => evaluate_shortest_list(inputs, meta),
            Self::LongestList => evaluate_longest_list(inputs, meta),
            Self::PartitionList => evaluate_partition_list(inputs, meta),
            Self::SplitList => evaluate_split_list(inputs, meta),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::ListItem => "List Item",
            Self::ListLength => "List Length",
            Self::ReverseList => "Reverse List",
            Self::Dispatch => "Dispatch",
            Self::SortList => "Sort List",
            Self::ShiftList => "Shift List",
            Self::InsertItems => "Insert Items",
            Self::PickAndChoose => "Pick'n'Choose",
            Self::Weave => "Weave",
            Self::SiftPattern => "Sift Pattern",
            Self::CrossReference => "Cross Reference",
            Self::ShortestList => "Shortest List",
            Self::LongestList => "Longest List",
            Self::PartitionList => "Partition List",
            Self::SplitList => "Split List",
        }
    }
}

fn wrap_index(index: i64, len: usize) -> usize {
    if len == 0 {
        return 0;
    }

    let len_i64 = len as i64;
    ((index % len_i64 + len_i64) as usize) % len
}

fn relative_output_offsets(meta: &MetaMap) -> Vec<(String, i64)> {
    let mut offsets = Vec::new();

    if let Some(MetaValue::List(entries)) = meta.get_normalized(META_OUTPUT_PINS) {
        for entry in entries {
            if let MetaValue::Text(text) = entry {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if let Ok(offset) = trimmed.parse::<i64>() {
                    offsets.push((trimmed.to_owned(), offset));
                }
            }
        }
    }

    offsets.sort_by_key(|(_, offset)| *offset);
    offsets
}

fn evaluate_list_item(inputs: &[Value], meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "List Item vereist een lijst en een index",
        ));
    }

    let list = coerce_list(inputs.get(0), "List Item L")?;
    let index = coerce_integer(inputs.get(1), "List Item i")?;
    let wrap = inputs
        .get(2)
        .map_or(Ok(false), |v| coerce_boolean(Some(v), "List Item W"))?;

    if list.is_empty() {
        return Ok(BTreeMap::new()); // Return lege map als de lijst leeg is
    }

    let final_index = if wrap {
        wrap_index(index, list.len())
    } else {
        index as usize
    };

    if final_index >= list.len() {
        return Err(ComponentError::new(format!(
            "Index {} is buiten de grenzen van de lijst (lengte {})",
            final_index,
            list.len()
        )));
    }

    let item = list[final_index].clone();
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_ELEMENT.to_owned(), item);
    let relative_outputs = relative_output_offsets(meta);
    let base_index = final_index as i64;
    for (pin_name, offset) in relative_outputs {
        let value = if wrap {
            let target_index = wrap_index(base_index + offset, list.len());
            list[target_index].clone()
        } else {
            let target_index = base_index + offset;
            if target_index < 0 || target_index >= list.len() as i64 {
                Value::Null
            } else {
                list[target_index as usize].clone()
            }
        };
        outputs.insert(pin_name, value);
    }
    // Sommige GHX-bestanden gebruiken andere pinnamen (bijv. "Item"/"i") voor de List Item-output.
    // Voeg aliases toe zodat verbindingen via die pinnamen ook een waarde ontvangen.
    outputs.insert(
        "Item".to_owned(),
        outputs
            .get(PIN_OUTPUT_ELEMENT)
            .cloned()
            .unwrap_or(Value::Null),
    );
    outputs.insert(
        "i".to_owned(),
        outputs
            .get(PIN_OUTPUT_ELEMENT)
            .cloned()
            .unwrap_or(Value::Null),
    );

    Ok(outputs)
}

fn evaluate_list_length(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("List Length vereist een lijst"));
    }

    let list = coerce_list(inputs.get(0), "List Length L")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_LIST.to_owned(), Value::Number(list.len() as f64));

    Ok(outputs)
}

fn evaluate_reverse_list(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Reverse List vereist een lijst"));
    }

    let list = coerce_list(inputs.get(0), "Reverse List L")?;
    let mut reversed_list = list.to_vec();
    reversed_list.reverse();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_LIST.to_owned(), Value::List(reversed_list));

    Ok(outputs)
}

fn evaluate_dispatch(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Dispatch vereist een lijst en een patroon",
        ));
    }

    let list = coerce_list(inputs.get(0), "Dispatch L")?;
    let pattern = coerce_list(inputs.get(1), "Dispatch P")?;

    if pattern.is_empty() {
        return Err(ComponentError::new("Dispatch patroon kan niet leeg zijn"));
    }

    let mut list_a = Vec::new();
    let mut list_b = Vec::new();

    for (i, item) in list.iter().enumerate() {
        let pattern_value = pattern
            .get(i % pattern.len())
            .cloned()
            .unwrap_or(Value::Boolean(false));
        if coerce_boolean(Some(&pattern_value), "Dispatch P").unwrap_or(false) {
            list_a.push(item.clone());
        } else {
            list_b.push(item.clone());
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_LIST_A.to_owned(), Value::List(list_a));
    outputs.insert(PIN_OUTPUT_LIST_B.to_owned(), Value::List(list_b));

    Ok(outputs)
}

fn evaluate_sort_list(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Sort List vereist een lijst met sleutels",
        ));
    }

    let keys = coerce_list(inputs.get(0), "Sort List K")?;
    let mut indexed_keys: Vec<(usize, &Value)> = keys.iter().enumerate().collect();

    indexed_keys.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let sorted_keys: Vec<Value> = indexed_keys.iter().map(|(_, k)| (*k).clone()).collect();
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_KEYS.to_owned(), Value::List(sorted_keys));

    // Sorteer de synchrone lijsten
    for (i, pin) in ["A", "B", "C"].iter().enumerate() {
        if let Some(values) = inputs.get(i + 1) {
            let values_list = coerce_list(Some(values), &format!("Sort List {}", pin))?;
            let sorted_values: Vec<Value> = indexed_keys
                .iter()
                .map(|(idx, _)| values_list.get(*idx).cloned().unwrap_or(Value::Null))
                .collect();
            outputs.insert(pin.to_string(), Value::List(sorted_values));
        }
    }

    Ok(outputs)
}

fn evaluate_shift_list(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Shift List vereist een lijst en een shift offset",
        ));
    }

    let list = coerce_list(inputs.get(0), "Shift List L")?;
    let shift = coerce_integer(inputs.get(1), "Shift List S")?;
    let wrap = inputs
        .get(2)
        .map_or(Ok(false), |v| coerce_boolean(Some(v), "Shift List W"))?;

    if list.is_empty() {
        return Ok(BTreeMap::new());
    }

    let len = list.len();
    let mut shifted_list = list.to_vec();

    if wrap {
        let offset = shift.rem_euclid(len as i64) as usize;
        shifted_list.rotate_right(offset);
    } else {
        if shift > 0 {
            let offset = (shift as usize).min(len);
            shifted_list.rotate_right(offset);
            for i in 0..offset {
                shifted_list[i] = Value::Null;
            }
        } else {
            let offset = (-shift as usize).min(len);
            shifted_list.rotate_left(offset);
            for i in (len - offset)..len {
                shifted_list[i] = Value::Null;
            }
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_LIST.to_owned(), Value::List(shifted_list));

    Ok(outputs)
}

fn evaluate_insert_items(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Insert Items vereist een lijst, items en indices",
        ));
    }

    let list = coerce_list(inputs.get(0), "Insert Items L")?;
    let items = coerce_list(inputs.get(1), "Insert Items I")?;
    let indices = coerce_list(inputs.get(2), "Insert Items i")?;
    let wrap = inputs
        .get(3)
        .map_or(Ok(false), |v| coerce_boolean(Some(v), "Insert Items W"))?;

    let mut result_list = list.to_vec();

    for (item_index, index_val) in indices.iter().enumerate() {
        let mut index = coerce_integer(Some(index_val), "Insert Items i")? as isize;
        let item = items.get(item_index).cloned().unwrap_or(Value::Null);

        if wrap {
            let len = result_list.len() as isize;
            if len > 0 {
                index %= len;
                if index < 0 {
                    index += len;
                }
            } else {
                index = 0;
            }
        }

        if index >= 0 && index <= result_list.len() as isize {
            result_list.insert(index as usize, item);
        } else {
            // Index is buiten bereik, niet wrappen
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_LIST.to_owned(), Value::List(result_list));

    Ok(outputs)
}

fn evaluate_pick_and_choose(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 1 {
        return Err(ComponentError::new("Pick'n'Choose vereist een patroon"));
    }

    let pattern = coerce_list(inputs.get(0), "Pick'n'Choose P")?;
    let streams: Vec<_> = inputs
        .iter()
        .skip(1)
        .map(|v| coerce_list(Some(v), "Pick'n'Choose Stream"))
        .collect::<Result<Vec<_>, _>>()?;

    let mut result = Vec::new();
    for (i, p) in pattern.iter().enumerate() {
        let stream_index = coerce_integer(Some(p), "Pick'n'Choose P")? as usize;
        if let Some(stream) = streams.get(stream_index) {
            if let Some(item) = stream.get(i) {
                result.push(item.clone());
            }
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_RESULT.to_owned(), Value::List(result));
    Ok(outputs)
}

fn evaluate_weave(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 1 {
        return Err(ComponentError::new("Weave vereist een patroon"));
    }

    let pattern = coerce_list(inputs.get(0), "Weave P")?;
    let streams: Vec<_> = inputs
        .iter()
        .skip(1)
        .map(|v| coerce_list(Some(v), "Weave Stream"))
        .collect::<Result<Vec<_>, _>>()?;

    let mut stream_counters = vec![0; streams.len()];
    let mut result = Vec::new();

    for p in pattern.iter() {
        let stream_index = coerce_integer(Some(p), "Weave P")? as usize;
        if let Some(stream) = streams.get(stream_index) {
            if let Some(item) = stream.get(stream_counters[stream_index]) {
                result.push(item.clone());
                stream_counters[stream_index] += 1;
            }
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_WEAVE.to_owned(), Value::List(result));
    Ok(outputs)
}

fn evaluate_sift_pattern(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Sift Pattern vereist een lijst en een patroon",
        ));
    }

    let list = coerce_list(inputs.get(0), "Sift Pattern L")?;
    let pattern = coerce_list(inputs.get(1), "Sift Pattern P")?;

    if pattern.is_empty() {
        return Err(ComponentError::new("Sift patroon kan niet leeg zijn"));
    }

    let max_pattern_index = pattern
        .iter()
        .map(|p| coerce_integer(Some(p), "Sift Pattern P").unwrap_or(0))
        .max()
        .unwrap_or(0) as usize;

    let mut outputs_vec: Vec<Vec<Value>> = vec![Vec::new(); max_pattern_index + 1];

    for (i, item) in list.iter().enumerate() {
        let pattern_index =
            coerce_integer(pattern.get(i % pattern.len()), "Sift Pattern P").unwrap_or(0) as usize;
        if let Some(output_list) = outputs_vec.get_mut(pattern_index) {
            output_list.push(item.clone());
        }
    }

    let mut outputs = BTreeMap::new();
    for (i, out_list) in outputs_vec.iter().enumerate() {
        outputs.insert(i.to_string(), Value::List(out_list.clone()));
    }

    Ok(outputs)
}

fn evaluate_cross_reference(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Cross Reference vereist twee lijsten"));
    }

    let list_a = coerce_list(inputs.get(0), "Cross Reference A")?;
    let list_b = coerce_list(inputs.get(1), "Cross Reference B")?;

    let mut new_a = Vec::new();
    let mut new_b = Vec::new();

    for a in list_a.iter() {
        for b in list_b.iter() {
            new_a.push(a.clone());
            new_b.push(b.clone());
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_LIST_A.to_owned(), Value::List(new_a));
    outputs.insert(PIN_OUTPUT_LIST_B.to_owned(), Value::List(new_b));
    Ok(outputs)
}

fn evaluate_shortest_list(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Shortest List vereist twee lijsten"));
    }
    let list_a = coerce_list(inputs.get(0), "Shortest List A")?;
    let list_b = coerce_list(inputs.get(1), "Shortest List B")?;
    let min_len = list_a.len().min(list_b.len());

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_LIST_A.to_owned(),
        Value::List(list_a[..min_len].to_vec()),
    );
    outputs.insert(
        PIN_OUTPUT_LIST_B.to_owned(),
        Value::List(list_b[..min_len].to_vec()),
    );
    Ok(outputs)
}

fn evaluate_longest_list(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Longest List vereist twee lijsten"));
    }
    let mut list_a = coerce_list(inputs.get(0), "Longest List A")?.to_vec();
    let mut list_b = coerce_list(inputs.get(1), "Longest List B")?.to_vec();
    let max_len = list_a.len().max(list_b.len());

    if let Some(last) = list_a.last().cloned() {
        list_a.resize(max_len, last);
    }
    if let Some(last) = list_b.last().cloned() {
        list_b.resize(max_len, last);
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_LIST_A.to_owned(), Value::List(list_a));
    outputs.insert(PIN_OUTPUT_LIST_B.to_owned(), Value::List(list_b));
    Ok(outputs)
}

fn evaluate_partition_list(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Partition List vereist een lijst en een grootte",
        ));
    }
    let list = coerce_list(inputs.get(0), "Partition List L")?;
    let size = coerce_integer(inputs.get(1), "Partition List S")? as usize;

    if size == 0 {
        return Err(ComponentError::new("Partition grootte kan niet nul zijn"));
    }

    let chunks: Vec<Value> = list.chunks(size).map(|c| Value::List(c.to_vec())).collect();
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CHUNKS.to_owned(), Value::List(chunks));
    Ok(outputs)
}

fn evaluate_split_list(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Split List vereist een lijst en een index",
        ));
    }
    let list = coerce_list(inputs.get(0), "Split List L")?;
    let index = coerce_integer(inputs.get(1), "Split List i")? as usize;

    if index > list.len() {
        return Err(ComponentError::new(format!(
            "Split index {} is buiten de grenzen van de lijst",
            index
        )));
    }

    let (a, b) = list.split_at(index);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_LIST_A.to_owned(), Value::List(a.to_vec()));
    outputs.insert(PIN_OUTPUT_LIST_B.to_owned(), Value::List(b.to_vec()));
    Ok(outputs)
}

fn coerce_list<'a>(
    value: Option<&'a Value>,
    context: &str,
) -> Result<Cow<'a, [Value]>, ComponentError> {
    match value {
        Some(Value::List(list)) => Ok(Cow::Borrowed(list)),
        Some(other) => Ok(Cow::Owned(vec![other.clone()])),
        None => Err(ComponentError::new(format!(
            "{} vereist een lijst",
            context
        ))),
    }
}

fn coerce_integer(value: Option<&Value>, context: &str) -> Result<i64, ComponentError> {
    match value {
        Some(Value::List(values)) if !values.is_empty() => coerce_integer(values.get(0), context),
        Some(other) => super::coerce::coerce_integer(other).map_err(|_| {
            ComponentError::new(format!(
                "{} verwacht een geheel getal, kreeg {}",
                context,
                other.kind()
            ))
        }),
        None => Err(ComponentError::new(format!(
            "{} vereist een geheel getal",
            context
        ))),
    }
}

fn coerce_boolean(value: Option<&Value>, context: &str) -> Result<bool, ComponentError> {
    match value {
        Some(Value::List(values)) if !values.is_empty() => coerce_boolean(values.get(0), context),
        Some(other) => super::coerce::coerce_boolean(other).map_err(|_| {
            ComponentError::new(format!(
                "{} verwacht een boolean, kreeg {}",
                context,
                other.kind()
            ))
        }),
        None => Err(ComponentError::new(format!(
            "{} vereist een boolean",
            context
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Component, ComponentKind, META_OUTPUT_PINS, PIN_OUTPUT_CHUNKS, PIN_OUTPUT_ELEMENT,
        PIN_OUTPUT_KEYS, PIN_OUTPUT_LIST, PIN_OUTPUT_LIST_A, PIN_OUTPUT_LIST_B, PIN_OUTPUT_RESULT,
        PIN_OUTPUT_WEAVE,
    };
    use crate::graph::node::{MetaMap, MetaValue};
    use crate::graph::value::Value;

    fn meta_with_output_pins(pins: &[&str]) -> MetaMap {
        let mut meta = MetaMap::new();
        if pins.is_empty() {
            return meta;
        }

        let pin_values = pins
            .iter()
            .map(|pin| MetaValue::Text(pin.to_string()))
            .collect::<Vec<_>>();

        meta.insert(META_OUTPUT_PINS.to_owned(), MetaValue::List(pin_values));
        meta
    }

    #[test]
    fn list_length_correct() {
        let component = ComponentKind::ListLength;
        let inputs = &[Value::List(vec![Value::Number(1.0), Value::Number(2.0)])];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let length = outputs.get(PIN_OUTPUT_LIST).unwrap();
        assert!(matches!(length, Value::Number(x) if (*x - 2.0).abs() < 1e-6));
    }

    #[test]
    fn list_item_correct() {
        let component = ComponentKind::ListItem;
        let inputs = &[
            Value::List(vec![
                Value::Number(10.0),
                Value::Number(20.0),
                Value::Number(30.0),
            ]),
            Value::Number(1.0),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let item = outputs.get(PIN_OUTPUT_ELEMENT).unwrap();
        assert!(matches!(item, Value::Number(x) if (*x - 20.0).abs() < 1e-6));
    }

    #[test]
    fn list_item_wrap() {
        let component = ComponentKind::ListItem;
        let inputs = &[
            Value::List(vec![
                Value::Number(10.0),
                Value::Number(20.0),
                Value::Number(30.0),
            ]),
            Value::Number(4.0),
            Value::Boolean(true),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let item = outputs.get(PIN_OUTPUT_ELEMENT).unwrap();
        assert!(matches!(item, Value::Number(x) if (*x - 20.0).abs() < 1e-6));
    }

    #[test]
    fn list_item_out_of_bounds() {
        let component = ComponentKind::ListItem;
        let inputs = &[
            Value::List(vec![Value::Number(10.0), Value::Number(20.0)]),
            Value::Number(2.0),
            Value::Boolean(false), // No wrap
        ];
        let err = component.evaluate(inputs, &MetaMap::new()).unwrap_err();
        assert!(err.message().contains("buiten de grenzen"));
    }

    #[test]
    fn list_item_additional_outputs_relative() {
        let component = ComponentKind::ListItem;
        let meta = meta_with_output_pins(&["i", "+1", "-1"]);
        let inputs = &[
            Value::List(vec![
                Value::Number(10.0),
                Value::Number(20.0),
                Value::Number(30.0),
                Value::Number(40.0),
            ]),
            Value::Number(1.0),
        ];
        let outputs = component.evaluate(inputs, &meta).unwrap();
        let plus_one = outputs.get("+1").expect("contains +1 pin");
        assert!(matches!(plus_one, Value::Number(x) if (*x - 30.0).abs() < 1e-6));
        let minus_one = outputs.get("-1").expect("contains -1 pin");
        assert!(matches!(minus_one, Value::Number(x) if (*x - 10.0).abs() < 1e-6));
    }

    #[test]
    fn list_item_additional_outputs_wraps() {
        let component = ComponentKind::ListItem;
        let meta = meta_with_output_pins(&["+1", "-1"]);
        let inputs = &[
            Value::List(vec![
                Value::Number(11.0),
                Value::Number(22.0),
                Value::Number(33.0),
            ]),
            Value::Number(2.0),
            Value::Boolean(true),
        ];
        let outputs = component.evaluate(inputs, &meta).unwrap();
        let plus_one = outputs.get("+1").expect("contains +1 pin");
        assert!(matches!(plus_one, Value::Number(x) if (*x - 11.0).abs() < 1e-6));
        let minus_one = outputs.get("-1").expect("contains -1 pin");
        assert!(matches!(minus_one, Value::Number(x) if (*x - 22.0).abs() < 1e-6));
    }

    #[test]
    fn list_item_additional_output_out_of_range_is_null() {
        let component = ComponentKind::ListItem;
        let meta = meta_with_output_pins(&["+1"]);
        let inputs = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::Number(1.0),
        ];
        let outputs = component.evaluate(inputs, &meta).unwrap();
        assert_eq!(outputs.get("+1"), Some(&Value::Null));
    }

    #[test]
    fn reverse_list_correct() {
        let component = ComponentKind::ReverseList;
        let inputs = &[Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let reversed = match outputs.get(PIN_OUTPUT_LIST).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(reversed.len(), 3);
        assert!(matches!(reversed[0], Value::Number(x) if (x - 3.0).abs() < 1e-6));
        assert!(matches!(reversed[2], Value::Number(x) if (x - 1.0).abs() < 1e-6));
    }

    #[test]
    fn dispatch_correct() {
        let component = ComponentKind::Dispatch;
        let inputs = &[
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Number(4.0),
            ]),
            Value::List(vec![
                Value::Boolean(true),
                Value::Boolean(false),
                Value::Boolean(true),
            ]),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let list_a = match outputs.get(PIN_OUTPUT_LIST_A).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        let list_b = match outputs.get(PIN_OUTPUT_LIST_B).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(list_a.len(), 3);
        assert_eq!(list_b.len(), 1);
        assert!(matches!(list_a[0], Value::Number(x) if (x - 1.0).abs() < 1e-6));
        assert!(matches!(list_a[1], Value::Number(x) if (x - 3.0).abs() < 1e-6));
        assert!(matches!(list_a[2], Value::Number(x) if (x - 4.0).abs() < 1e-6));
        assert!(matches!(list_b[0], Value::Number(x) if (x - 2.0).abs() < 1e-6));
    }

    #[test]
    fn sort_list_correct() {
        let component = ComponentKind::SortList;
        let inputs = &[
            Value::List(vec![
                Value::Number(3.0),
                Value::Number(1.0),
                Value::Number(2.0),
            ]),
            Value::List(vec![
                Value::Text("C".into()),
                Value::Text("A".into()),
                Value::Text("B".into()),
            ]),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let sorted_keys = match outputs.get(PIN_OUTPUT_KEYS).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        let sorted_a = match outputs.get(PIN_OUTPUT_LIST_A).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(sorted_keys.len(), 3);
        assert!(matches!(sorted_keys[0], Value::Number(x) if (x - 1.0).abs() < 1e-6));
        assert!(matches!(sorted_keys[2], Value::Number(x) if (x - 3.0).abs() < 1e-6));
        assert_eq!(sorted_a.len(), 3);
        assert!(matches!(&sorted_a[0], Value::Text(s) if s == "A"));
        assert!(matches!(&sorted_a[2], Value::Text(s) if s == "C"));
    }

    #[test]
    fn shift_list_correct() {
        let component = ComponentKind::ShiftList;
        let inputs = &[
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
            ]),
            Value::Number(1.0),
            Value::Boolean(true),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let shifted = match outputs.get(PIN_OUTPUT_LIST).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(shifted.len(), 3);
        assert!(matches!(shifted[0], Value::Number(x) if (x - 3.0).abs() < 1e-6));
        assert!(matches!(shifted[1], Value::Number(x) if (x - 1.0).abs() < 1e-6));
    }

    #[test]
    fn insert_items_correct() {
        let component = ComponentKind::InsertItems;
        let inputs = &[
            Value::List(vec![Value::Number(10.0), Value::Number(30.0)]),
            Value::List(vec![Value::Number(20.0)]),
            Value::List(vec![Value::Number(1.0)]),
            Value::Boolean(false),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let result = match outputs.get(PIN_OUTPUT_LIST).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(result.len(), 3);
        assert!(matches!(result[1], Value::Number(x) if (x - 20.0).abs() < 1e-6));
    }

    #[test]
    fn pick_and_choose_correct() {
        let component = ComponentKind::PickAndChoose;
        let inputs = &[
            Value::List(vec![
                Value::Number(0.0),
                Value::Number(1.0),
                Value::Number(0.0),
            ]),
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
            ]),
            Value::List(vec![
                Value::Number(4.0),
                Value::Number(5.0),
                Value::Number(6.0),
            ]),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let result = match outputs.get(PIN_OUTPUT_RESULT).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(result.len(), 3);
        assert!(matches!(result[0], Value::Number(x) if (x - 1.0).abs() < 1e-6));
        assert!(matches!(result[1], Value::Number(x) if (x - 5.0).abs() < 1e-6));
        assert!(matches!(result[2], Value::Number(x) if (x - 3.0).abs() < 1e-6));
    }

    #[test]
    fn weave_correct() {
        let component = ComponentKind::Weave;
        let inputs = &[
            Value::List(vec![
                Value::Number(0.0),
                Value::Number(1.0),
                Value::Number(0.0),
            ]),
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(3.0)]),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let result = match outputs.get(PIN_OUTPUT_WEAVE).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(result.len(), 3);
        assert!(matches!(result[0], Value::Number(x) if (x - 1.0).abs() < 1e-6));
        assert!(matches!(result[1], Value::Number(x) if (x - 3.0).abs() < 1e-6));
        assert!(matches!(result[2], Value::Number(x) if (x - 2.0).abs() < 1e-6));
    }

    #[test]
    fn sift_pattern_correct() {
        let component = ComponentKind::SiftPattern;
        let inputs = &[
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Number(4.0),
            ]),
            Value::List(vec![Value::Number(0.0), Value::Number(1.0)]),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let list_0 = match outputs.get("0").unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        let list_1 = match outputs.get("1").unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(list_0.len(), 2);
        assert_eq!(list_1.len(), 2);
        assert!(matches!(list_0[0], Value::Number(x) if (x - 1.0).abs() < 1e-6));
        assert!(matches!(list_0[1], Value::Number(x) if (x - 3.0).abs() < 1e-6));
        assert!(matches!(list_1[0], Value::Number(x) if (x - 2.0).abs() < 1e-6));
        assert!(matches!(list_1[1], Value::Number(x) if (x - 4.0).abs() < 1e-6));
    }

    #[test]
    fn cross_reference_correct() {
        let component = ComponentKind::CrossReference;
        let inputs = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Text("A".into()), Value::Text("B".into())]),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let list_a = match outputs.get(PIN_OUTPUT_LIST_A).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        let list_b = match outputs.get(PIN_OUTPUT_LIST_B).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(list_a.len(), 4);
        assert_eq!(list_b.len(), 4);
        assert!(matches!(list_a[3], Value::Number(x) if (x - 2.0).abs() < 1e-6));
        assert!(matches!(&list_b[3], Value::Text(s) if s == "B"));
    }

    #[test]
    fn shortest_list_correct() {
        let component = ComponentKind::ShortestList;
        let inputs = &[
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
            ]),
            Value::List(vec![Value::Number(4.0), Value::Number(5.0)]),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let list_a = match outputs.get(PIN_OUTPUT_LIST_A).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(list_a.len(), 2);
    }

    #[test]
    fn longest_list_correct() {
        let component = ComponentKind::LongestList;
        let inputs = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![
                Value::Number(3.0),
                Value::Number(4.0),
                Value::Number(5.0),
            ]),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let list_a = match outputs.get(PIN_OUTPUT_LIST_A).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(list_a.len(), 3);
        assert!(matches!(list_a[2], Value::Number(x) if (x - 2.0).abs() < 1e-6));
    }

    #[test]
    fn partition_list_correct() {
        let component = ComponentKind::PartitionList;
        let inputs = &[
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Number(4.0),
            ]),
            Value::Number(2.0),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let chunks = match outputs.get(PIN_OUTPUT_CHUNKS).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(chunks.len(), 2);
    }

    #[test]
    fn split_list_correct() {
        let component = ComponentKind::SplitList;
        let inputs = &[
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
            ]),
            Value::Number(1.0),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let list_a = match outputs.get(PIN_OUTPUT_LIST_A).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        let list_b = match outputs.get(PIN_OUTPUT_LIST_B).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(list_a.len(), 1);
        assert_eq!(list_b.len(), 2);
    }

    #[test]
    fn list_item_accepts_boolean_index() {
        let component = ComponentKind::ListItem;
        let inputs = &[
            Value::List(vec![Value::Number(5.0), Value::Number(9.0)]),
            Value::Boolean(true), // coerces to index 1
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert!(matches!(
            outputs.get(super::PIN_OUTPUT_ELEMENT),
            Some(Value::Number(n)) if (*n - 9.0).abs() < 1e-9
        ));
    }

    #[test]
    fn list_item_accepts_single_geometry_without_list_wrapper() {
        let component = ComponentKind::ListItem;
        let brep = Value::Surface {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            faces: vec![vec![0, 1, 2]],
        };
        let inputs = &[brep.clone(), Value::Number(0.0)];

        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();

        assert!(matches!(
            outputs.get(super::PIN_OUTPUT_ELEMENT),
            Some(Value::Surface { .. })
        ));
        // Ensure the geometry itself is passed through unchanged.
        assert_eq!(outputs.get(super::PIN_OUTPUT_ELEMENT), Some(&brep));
    }

    #[test]
    fn list_item_exposes_item_alias_pins() {
        let component = ComponentKind::ListItem;
        let inputs = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::Number(1.0),
        ];

        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert!(matches!(
            outputs.get("Item"),
            Some(Value::Number(n)) if (*n - 2.0).abs() < 1e-9
        ));
        assert!(matches!(
            outputs.get("i"),
            Some(Value::Number(n)) if (*n - 2.0).abs() < 1e-9
        ));
    }
}
