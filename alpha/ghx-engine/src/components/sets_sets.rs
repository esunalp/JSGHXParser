//! Implementaties van Grasshopper "Sets -> Sets" componenten.

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

// Definieer hier de output pin namen als const
const OUTPUT_SET: &str = "S";
const OUTPUT_COUNT: &str = "N";
const OUTPUT_RESULT: &str = "R";
const OUTPUT_INDEX: &str = "I";
const OUTPUT_UNION: &str = "U";
const OUTPUT_MAP: &str = "M";
const OUTPUT_HIT: &str = "H";
const OUTPUT_SYM_DIFF: &str = "X";
const OUTPUT_PRODUCT: &str = "P";

/// Beschikbare componenten binnen Sets -> Sets.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    DeleteConsecutive,
    KeyValueSearch,
    CreateSet,
    CreateSetWithMap,
    MemberIndex,
    SubSet,
    Disjoint,
    SetIntersection,
    SetUnion,
    FindSimilarMember,
    ReplaceMembers,
    SetDifferenceSymmetric,
    SetMajority,
    CartesianProduct,
    SetDifference,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Registraties van alle Sets -> Sets componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["190d042c-2270-4bc1-81c0-4f90c170c9c9"],
        names: &["Delete Consecutive", "DCon"],
        kind: ComponentKind::DeleteConsecutive,
    },
    Registration {
        guids: &["1edcc3cf-cf84-41d4-8204-561162cfe510"],
        names: &["Key/Value Search", "KeySearch"],
        kind: ComponentKind::KeyValueSearch,
    },
    Registration {
        guids: &["2cb4bf85-a282-464c-b42c-8e735d2a0a74"],
        names: &["Create Set"], // Let op: CSet nickname wordt gedeeld
        kind: ComponentKind::CreateSet,
    },
    Registration {
        guids: &["98c3c63a-e78a-43ea-a111-514fcf312c95"],
        names: &["Create Set"],
        kind: ComponentKind::CreateSetWithMap,
    },
    Registration {
        guids: &["3ff27857-b988-417a-b495-b24c733dbd00"],
        names: &["Member Index", "MIndex"],
        kind: ComponentKind::MemberIndex,
    },
    Registration {
        guids: &["4cfc0bb0-0745-4772-a520-39f9bf3d99bc"],
        names: &["SubSet"],
        kind: ComponentKind::SubSet,
    },
    Registration {
        guids: &["81800098-1060-4e2b-80d4-17f835cc825f"],
        names: &["Disjoint"],
        kind: ComponentKind::Disjoint,
    },
    Registration {
        guids: &["82f19c48-9e73-43a4-ae6c-3a8368099b08", "8a55f680-cf53-4634-a486-b828de92b71d"],
        names: &["Set Intersection", "Intersection"],
        kind: ComponentKind::SetIntersection,
    },
    Registration {
        guids: &["8eed5d78-7810-4ba1-968e-8a1f1db98e39", "ab34845d-4ab9-4ff4-8870-eedd0c5594cb"],
        names: &["Set Union", "SUnion"],
        kind: ComponentKind::SetUnion,
    },
    Registration {
        guids: &["b4d4235f-14ff-4d4e-a29a-b358dcd2baf4"],
        names: &["Find similar member", "FSim"],
        kind: ComponentKind::FindSimilarMember,
    },
    Registration {
        guids: &["bafac914-ede4-4a59-a7b2-cc41bc3de961"],
        names: &["Replace Members", "Replace"],
        kind: ComponentKind::ReplaceMembers,
    },
    Registration {
        guids: &["d2461702-3164-4894-8c10-ed1fc4b52965"],
        names: &["Set Difference (S)", "ExDiff"],
        kind: ComponentKind::SetDifferenceSymmetric,
    },
    Registration {
        guids: &["d4136a7b-7422-4660-9404-640474bd2725"],
        names: &["Set Majority", "Majority"],
        kind: ComponentKind::SetMajority,
    },
    Registration {
        guids: &["deffaf1e-270a-4c15-a693-9216b68afd4a"],
        names: &["Carthesian Product", "CProd"],
        kind: ComponentKind::CartesianProduct,
    },
    Registration {
        guids: &["e3b1a10c-4d49-4140-b8e6-0b5732a26c31"],
        names: &["Set Difference", "Difference"],
        kind: ComponentKind::SetDifference,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::CreateSet => evaluate_create_set(inputs, meta),
            Self::CreateSetWithMap => evaluate_create_set_with_map(inputs, meta),
            Self::SetUnion => evaluate_set_union(inputs, meta),
            Self::SetIntersection => evaluate_set_intersection(inputs, meta),
            Self::SetDifference => evaluate_set_difference(inputs, meta),
            Self::SetDifferenceSymmetric => evaluate_set_difference_symmetric(inputs, meta),
            Self::SubSet => evaluate_subset(inputs, meta),
            Self::Disjoint => evaluate_disjoint(inputs, meta),
            Self::MemberIndex => evaluate_member_index(inputs, meta),
            Self::KeyValueSearch => evaluate_key_value_search(inputs, meta),
            Self::DeleteConsecutive => evaluate_delete_consecutive(inputs, meta),
            Self::ReplaceMembers => evaluate_replace_members(inputs, meta),
            _ => Err(ComponentError::new("Component is nog niet geïmplementeerd.")),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::DeleteConsecutive => "Delete Consecutive",
            Self::KeyValueSearch => "Key/Value Search",
            Self::CreateSet => "Create Set",
            Self::CreateSetWithMap => "Create Set",
            Self::MemberIndex => "Member Index",
            Self::SubSet => "SubSet",
            Self::Disjoint => "Disjoint",
            Self::SetIntersection => "Set Intersection",
            Self::SetUnion => "Set Union",
            Self::FindSimilarMember => "Find similar member",
            Self::ReplaceMembers => "Replace Members",
            Self::SetDifferenceSymmetric => "Set Difference (S)",
            Self::SetMajority => "Set Majority",
            Self::CartesianProduct => "Carthesian Product",
            Self::SetDifference => "Set Difference",
        }
    }
}

fn evaluate_create_set(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Create Set vereist een lijst"));
    }
    let list = coerce_list(inputs.get(0), "Create Set L")?;

    let mut set = Vec::new();
    let mut seen = HashSet::new();

    for item in list {
        if seen.insert(item) {
            set.push(item.clone());
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(OUTPUT_SET.to_owned(), Value::List(set));
    Ok(outputs)
}

fn evaluate_create_set_with_map(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Create Set vereist een lijst"));
    }
    let list = coerce_list(inputs.get(0), "Create Set L")?;

    let mut set = Vec::new();
    let mut map = Vec::new();
    let mut seen = HashMap::new();

    for (_i, item) in list.iter().enumerate() {
        let index = *seen.entry(item).or_insert_with(|| {
            set.push(item.clone());
            set.len() - 1
        });
        map.push(Value::Number(index as f64));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(OUTPUT_SET.to_owned(), Value::List(set));
    outputs.insert(OUTPUT_MAP.to_owned(), Value::List(map));
    Ok(outputs)
}

fn evaluate_set_union(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Set Union vereist twee lijsten"));
    }
    let list_a = coerce_list(inputs.get(0), "Set Union A")?;
    let list_b = coerce_list(inputs.get(1), "Set Union B")?;

    let mut set = Vec::new();
    let mut seen = HashSet::new();

    for item in list_a.iter().chain(list_b.iter()) {
        if seen.insert(item) {
            set.push(item.clone());
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(OUTPUT_UNION.to_owned(), Value::List(set));
    Ok(outputs)
}

fn evaluate_set_intersection(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Set Intersection vereist twee lijsten"));
    }
    let list_a = coerce_list(inputs.get(0), "Set Intersection A")?;
    let list_b = coerce_list(inputs.get(1), "Set Intersection B")?;

    let set_a: HashSet<_> = list_a.iter().collect();
    let set_b: HashSet<_> = list_b.iter().collect();

    let intersection: Vec<Value> = set_a.intersection(&set_b).map(|v| (*v).clone()).collect();

    let mut outputs = BTreeMap::new();
    outputs.insert(OUTPUT_RESULT.to_owned(), Value::List(intersection));
    Ok(outputs)
}

fn evaluate_set_difference(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Set Difference vereist twee lijsten"));
    }
    let list_a = coerce_list(inputs.get(0), "Set Difference A")?;
    let list_b = coerce_list(inputs.get(1), "Set Difference B")?;

    let set_b: HashSet<_> = list_b.iter().collect();

    let difference: Vec<Value> = list_a.iter().filter(|item| !set_b.contains(item)).cloned().collect();

    let mut outputs = BTreeMap::new();
    outputs.insert(OUTPUT_RESULT.to_owned(), Value::List(difference));
    Ok(outputs)
}

fn evaluate_set_difference_symmetric(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Set Difference (S) vereist twee lijsten"));
    }
    let list_a = coerce_list(inputs.get(0), "Set Difference (S) A")?;
    let list_b = coerce_list(inputs.get(1), "Set Difference (S) B")?;

    let set_a: HashSet<_> = list_a.iter().collect();
    let set_b: HashSet<_> = list_b.iter().collect();

    let sym_diff: Vec<Value> = set_a.symmetric_difference(&set_b).map(|v| (*v).clone()).collect();

    let mut outputs = BTreeMap::new();
    outputs.insert(OUTPUT_SYM_DIFF.to_owned(), Value::List(sym_diff));
    Ok(outputs)
}

fn evaluate_subset(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("SubSet vereist twee lijsten"));
    }
    let list_a = coerce_list(inputs.get(0), "SubSet A")?;
    let list_b = coerce_list(inputs.get(1), "SubSet B")?;

    let set_a: HashSet<_> = list_a.iter().collect();
    let set_b: HashSet<_> = list_b.iter().collect();

    let is_subset = set_b.is_subset(&set_a);

    let mut outputs = BTreeMap::new();
    outputs.insert(OUTPUT_RESULT.to_owned(), Value::Boolean(is_subset));
    Ok(outputs)
}

fn evaluate_disjoint(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Disjoint vereist twee lijsten"));
    }
    let list_a = coerce_list(inputs.get(0), "Disjoint A")?;
    let list_b = coerce_list(inputs.get(1), "Disjoint B")?;

    let set_a: HashSet<_> = list_a.iter().collect();
    let set_b: HashSet<_> = list_b.iter().collect();

    let is_disjoint = set_a.is_disjoint(&set_b);

    let mut outputs = BTreeMap::new();
    outputs.insert(OUTPUT_RESULT.to_owned(), Value::Boolean(is_disjoint));
    Ok(outputs)
}

fn evaluate_member_index(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Member Index vereist een set en een lid"));
    }
    let set = coerce_list(inputs.get(0), "Member Index S")?;
    let member = inputs.get(1).ok_or_else(|| ComponentError::new("Lid ontbreekt in Member Index"))?;

    let indices: Vec<Value> = set.iter().enumerate()
        .filter(|(_, item)| *item == member)
        .map(|(i, _)| Value::Number(i as f64))
        .collect();

    let count = Value::Number(indices.len() as f64);

    let mut outputs = BTreeMap::new();
    outputs.insert(OUTPUT_INDEX.to_owned(), Value::List(indices));
    outputs.insert(OUTPUT_COUNT.to_owned(), count);
    Ok(outputs)
}

fn evaluate_key_value_search(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new("Key/Value Search vereist sleutels, waarden en een zoekterm"));
    }
    let keys = coerce_list(inputs.get(0), "Key/Value Search K")?;
    let values = coerce_list(inputs.get(1), "Key/Value Search V")?;
    let search = inputs.get(2).ok_or_else(|| ComponentError::new("Zoekterm ontbreekt in Key/Value Search"))?;

    if keys.len() != values.len() {
        return Err(ComponentError::new("Sleutel- en waardenlijsten moeten even lang zijn"));
    }

    let result = keys.iter().position(|key| key == search)
        .and_then(|i| values.get(i).cloned())
        .unwrap_or(Value::Null);

    let mut outputs = BTreeMap::new();
    outputs.insert(OUTPUT_RESULT.to_owned(), result);
    Ok(outputs)
}

fn evaluate_delete_consecutive(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Delete Consecutive vereist een lijst"));
    }
    let list = coerce_list(inputs.get(0), "Delete Consecutive S")?;
    let wrap = coerce_boolean(inputs.get(1), "Delete Consecutive W").unwrap_or(false);

    if list.is_empty() {
        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_SET.to_owned(), Value::List(vec![]));
        outputs.insert(OUTPUT_COUNT.to_owned(), Value::Number(0.0));
        return Ok(outputs);
    }

    let mut new_list: Vec<Value> = Vec::new();
    let mut removed_count = 0;

    if wrap && list.first() == list.last() {
        // Find first non-equal element
        let first = list.first().unwrap();
        let start_index = list.iter().position(|item| item != first).unwrap_or(list.len());
        for item in list.iter().skip(start_index) {
            if new_list.last() != Some(item) {
                new_list.push(item.clone());
            } else {
                removed_count += 1;
            }
        }
        removed_count += start_index;

    } else {
        for item in list {
            if new_list.last() != Some(item) {
                new_list.push(item.clone());
            } else {
                removed_count += 1;
            }
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(OUTPUT_SET.to_owned(), Value::List(new_list));
    outputs.insert(OUTPUT_COUNT.to_owned(), Value::Number(removed_count as f64));
    Ok(outputs)
}

fn evaluate_replace_members(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new("Replace Members vereist een set, zoek- en vervanglijsten"));
    }
    let set = coerce_list(inputs.get(0), "Replace Members S")?;
    let find = coerce_list(inputs.get(1), "Replace Members F")?;
    let replace = coerce_list(inputs.get(2), "Replace Members R")?;

    let mut replacement_map: HashMap<&Value, &Value> = HashMap::new();
    for (f, r) in find.iter().zip(replace.iter()) {
        replacement_map.insert(f, r);
    }

    let result: Vec<Value> = set.iter()
        .map(|item| replacement_map.get(item).map_or(item, |&v| v).clone())
        .collect();

    let mut outputs = BTreeMap::new();
    outputs.insert(OUTPUT_RESULT.to_owned(), Value::List(result));
    Ok(outputs)
}

fn coerce_list<'a>(value: Option<&'a Value>, context: &str) -> Result<&'a [Value], ComponentError> {
    match value {
        Some(Value::List(list)) => Ok(list),
        Some(other) => Err(ComponentError::new(format!("{} verwacht een lijst, kreeg {}", context, other.kind()))),
        None => Err(ComponentError::new(format!("{} vereist een lijst", context))),
    }
}

fn coerce_boolean(value: Option<&Value>, context: &str) -> Result<bool, ComponentError> {
    match value {
        Some(Value::Boolean(b)) => Ok(*b),
        Some(Value::Number(n)) => Ok(*n != 0.0),
        Some(Value::List(values)) if !values.is_empty() => coerce_boolean(values.get(0), context),
        Some(other) => Err(ComponentError::new(format!("{} verwacht een boolean, kreeg {}", context, other.kind()))),
        None => Err(ComponentError::new(format!("{} vereist een boolean", context))),
    }
}

#[cfg(test)]
mod tests {
    use super::{Component, ComponentKind, OUTPUT_MAP, OUTPUT_SET, OUTPUT_UNION, OUTPUT_SYM_DIFF, OUTPUT_RESULT, OUTPUT_INDEX, OUTPUT_COUNT};
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn create_set_correct() {
        let component = ComponentKind::CreateSet;
        let inputs = &[Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(1.0),
        ])];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let set = match outputs.get(OUTPUT_SET).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn create_set_with_map_correct() {
        let component = ComponentKind::CreateSetWithMap;
        let inputs = &[Value::List(vec![
            Value::Text("A".into()),
            Value::Text("B".into()),
            Value::Text("A".into()),
        ])];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let map = match outputs.get(OUTPUT_MAP).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(map.len(), 3);
        assert_eq!(map[0], Value::Number(0.0));
        assert_eq!(map[2], Value::Number(0.0));
    }

    #[test]
    fn set_union_correct() {
        let component = ComponentKind::SetUnion;
        let inputs = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let union = match outputs.get(OUTPUT_UNION).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(union.len(), 3);
    }

    #[test]
    fn set_intersection_correct() {
        let component = ComponentKind::SetIntersection;
        let inputs = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let intersection = match outputs.get(OUTPUT_RESULT).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(intersection.len(), 1);
        assert!(intersection.contains(&Value::Number(2.0)));
    }

    #[test]
    fn set_difference_correct() {
        let component = ComponentKind::SetDifference;
        let inputs = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let difference = match outputs.get(OUTPUT_RESULT).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(difference.len(), 1);
        assert!(difference.contains(&Value::Number(1.0)));
    }

    #[test]
    fn set_difference_symmetric_correct() {
        let component = ComponentKind::SetDifferenceSymmetric;
        let inputs = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let sym_diff = match outputs.get(OUTPUT_SYM_DIFF).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(sym_diff.len(), 2);
    }

    #[test]
    fn subset_correct() {
        let component = ComponentKind::SubSet;
        let inputs = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]),
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get(OUTPUT_RESULT), Some(&Value::Boolean(true)));
    }

    #[test]
    fn disjoint_correct() {
        let component = ComponentKind::Disjoint;
        let inputs = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(3.0), Value::Number(4.0)]),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get(OUTPUT_RESULT), Some(&Value::Boolean(true)));
    }

    #[test]
    fn member_index_correct() {
        let component = ComponentKind::MemberIndex;
        let inputs = &[
            Value::List(vec![Value::Text("A".into()), Value::Text("B".into()), Value::Text("A".into())]),
            Value::Text("A".into()),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let indices = match outputs.get(OUTPUT_INDEX).unwrap() {
            Value::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(indices, &[Value::Number(0.0), Value::Number(2.0)]);
        assert_eq!(outputs.get(OUTPUT_COUNT), Some(&Value::Number(2.0)));
    }

    #[test]
    fn key_value_search_correct() {
        let component = ComponentKind::KeyValueSearch;
        let inputs = &[
            Value::List(vec![Value::Text("A".into()), Value::Text("B".into())]),
            Value::List(vec![Value::Number(10.0), Value::Number(20.0)]),
            Value::Text("B".into()),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get(OUTPUT_RESULT), Some(&Value::Number(20.0)));
    }

    #[test]
    fn delete_consecutive_correct() {
        let component = ComponentKind::DeleteConsecutive;
        let inputs = &[
            Value::List(vec![Value::Number(1.0), Value::Number(1.0), Value::Number(2.0)]),
            Value::Boolean(false),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let set = outputs.get(OUTPUT_SET).unwrap();
        assert_eq!(set, &Value::List(vec![Value::Number(1.0), Value::Number(2.0)]));
        assert_eq!(outputs.get(OUTPUT_COUNT), Some(&Value::Number(1.0)));
    }

    #[test]
    fn delete_consecutive_with_wrap() {
        let component = ComponentKind::DeleteConsecutive;
        let inputs = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(1.0), Value::Number(1.0)]),
            Value::Boolean(true),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let set = outputs.get(OUTPUT_SET).unwrap();
        assert_eq!(set, &Value::List(vec![Value::Number(2.0), Value::Number(1.0)]));
        assert_eq!(outputs.get(OUTPUT_COUNT), Some(&Value::Number(2.0)));
    }

    #[test]
    fn replace_members_correct() {
        let component = ComponentKind::ReplaceMembers;
        let inputs = &[
            Value::List(vec![Value::Text("A".into()), Value::Text("B".into()), Value::Text("C".into())]),
            Value::List(vec![Value::Text("A".into()), Value::Text("C".into())]),
            Value::List(vec![Value::Text("X".into()), Value::Text("Z".into())]),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let result = outputs.get(OUTPUT_RESULT).unwrap();
        assert_eq!(result, &Value::List(vec![Value::Text("X".into()), Value::Text("B".into()), Value::Text("Z".into())]));
    }
}
