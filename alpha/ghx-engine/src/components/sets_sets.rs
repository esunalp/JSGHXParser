//! Grasshopper components for set operations.
use crate::components::{Component, ComponentError, ComponentResult};
use crate::graph::value::Value;
use std::collections::{BTreeMap, HashMap, HashSet};

// --- Local Coercion Helpers ---

fn list(value: &Value) -> Result<&[Value], ComponentError> {
    match value {
        Value::List(l) => Ok(l),
        _ => Err(ComponentError::new(format!(
            "Expected a list, got {}",
            value.kind()
        ))),
    }
}

fn list_and_collect_into_hashset(value: &Value) -> Result<HashSet<Value>, ComponentError> {
    list(value).map(|l| l.iter().cloned().collect())
}

fn boolean(value: &Value) -> Result<bool, ComponentError> {
    match value {
        Value::Boolean(b) => Ok(*b),
        Value::Number(n) => Ok(n.abs() > 1e-9),
        Value::List(l) if l.len() == 1 => boolean(&l[0]),
        other => Err(ComponentError::new(format!(
            "Expected a boolean, got {}",
            other.kind()
        ))),
    }
}

// --- Component Implementations ---

/// Create Set component (GUID: `2cb4bf85-a282-464c-b42c-8e735d2a0a74`)
#[derive(Debug, Default, Clone, Copy)]
struct CreateSetSimpleComponent;
impl Component for CreateSetSimpleComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &crate::graph::node::MetaMap) -> ComponentResult {
        let values = list(&inputs[0])?;
        let mut unique_values = HashSet::new();
        let mut set = Vec::new();
        for value in values {
            if unique_values.insert(value.clone()) {
                set.push(value.clone());
            }
        }
        let mut outputs = BTreeMap::new();
        outputs.insert("Set".to_owned(), Value::List(set));
        Ok(outputs)
    }
}

/// Create Set component with map output (GUID: `98c3c63a-e78a-43ea-a111-514fcf312c95`)
#[derive(Debug, Default, Clone, Copy)]
struct CreateSetWithMapComponent;
impl Component for CreateSetWithMapComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &crate::graph::node::MetaMap) -> ComponentResult {
        let values = list(&inputs[0])?;
        let mut unique_values = HashMap::new();
        let mut set = Vec::new();
        let mut map = Vec::new();
        for value in values {
            let index = *unique_values.entry(value.clone()).or_insert_with(|| {
                let index = set.len();
                set.push(value.clone());
                index
            });
            map.push(Value::Number(index as f64));
        }
        let mut outputs = BTreeMap::new();
        outputs.insert("Set".to_owned(), Value::List(set));
        outputs.insert("Map".to_owned(), Value::List(map));
        Ok(outputs)
    }
}

/// Set Union component (GUID: `8eed5d78-7810-4ba1-968e-8a1f1db98e39`, etc.)
#[derive(Debug, Default, Clone, Copy)]
struct SetUnionComponent;
impl Component for SetUnionComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &crate::graph::node::MetaMap) -> ComponentResult {
        let set_a = list(&inputs[0])?;
        let set_b = list(&inputs[1])?;
        let mut unique_values = HashSet::new();
        let mut union = Vec::new();
        for value in set_a.iter().chain(set_b.iter()) {
            if unique_values.insert(value.clone()) {
                union.push(value.clone());
            }
        }
        let mut outputs = BTreeMap::new();
        outputs.insert("Union".to_owned(), Value::List(union));
        Ok(outputs)
    }
}

/// Set Intersection component (GUID: `82f19c48-9e73-43a4-ae6c-3a8368099b08`, etc.)
#[derive(Debug, Default, Clone, Copy)]
struct SetIntersectionComponent;
impl Component for SetIntersectionComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &crate::graph::node::MetaMap) -> ComponentResult {
        let set_a = list_and_collect_into_hashset(&inputs[0])?;
        let set_b = list(&inputs[1])?;
        let mut intersection = Vec::new();
        for value in set_b {
            if set_a.contains(value) {
                intersection.push(value.clone());
            }
        }
        let mut outputs = BTreeMap::new();
        outputs.insert("Intersection".to_owned(), Value::List(intersection));
        Ok(outputs)
    }
}

/// Set Difference component (GUID: `e3b1a10c-4d49-4140-b8e6-0b5732a26c31`)
#[derive(Debug, Default, Clone, Copy)]
struct SetDifferenceComponent;
impl Component for SetDifferenceComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &crate::graph::node::MetaMap) -> ComponentResult {
        let set_a = list(&inputs[0])?;
        let set_b_hs: HashSet<_> = list(&inputs[1])?.iter().cloned().collect();
        let mut difference = Vec::new();
        for value in set_a {
            if !set_b_hs.contains(value) {
                difference.push(value.clone());
            }
        }
        let mut outputs = BTreeMap::new();
        outputs.insert("Difference".to_owned(), Value::List(difference));
        Ok(outputs)
    }
}

/// Set Difference Symmetric component (GUID: `d2461702-3164-4894-8c10-ed1fc4b52965`)
#[derive(Debug, Default, Clone, Copy)]
struct SetDifferenceSymmetricComponent;
impl Component for SetDifferenceSymmetricComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &crate::graph::node::MetaMap) -> ComponentResult {
        let set_a: HashSet<_> = list(&inputs[0])?.iter().cloned().collect();
        let set_b: HashSet<_> = list(&inputs[1])?.iter().cloned().collect();
        let difference: Vec<_> = set_a.symmetric_difference(&set_b).cloned().collect();
        let mut outputs = BTreeMap::new();
        outputs.insert("ExDifference".to_owned(), Value::List(difference));
        Ok(outputs)
    }
}

/// Member Index component (GUID: `3ff27857-b988-417a-b495-b24c733dbd00`)
#[derive(Debug, Default, Clone, Copy)]
struct MemberIndexComponent;
impl Component for MemberIndexComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &crate::graph::node::MetaMap) -> ComponentResult {
        let set = list(&inputs[0])?;
        let member = &inputs[1];
        let mut indices = Vec::new();
        for (i, value) in set.iter().enumerate() {
            if value == member {
                indices.push(Value::Number(i as f64));
            }
        }
        let count = Value::Number(indices.len() as f64);
        let mut outputs = BTreeMap::new();
        outputs.insert("Index".to_owned(), Value::List(indices));
        outputs.insert("Count".to_owned(), count);
        Ok(outputs)
    }
}

/// Disjoint component (GUID: `81800098-1060-4e2b-80d4-17f835cc825f`)
#[derive(Debug, Default, Clone, Copy)]
struct DisjointComponent;
impl Component for DisjointComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &crate::graph::node::MetaMap) -> ComponentResult {
        let set_a = list_and_collect_into_hashset(&inputs[0])?;
        let set_b = list(&inputs[1])?;
        let is_disjoint = set_b.iter().all(|v| !set_a.contains(v));
        let mut outputs = BTreeMap::new();
        outputs.insert("Result".to_owned(), Value::Boolean(is_disjoint));
        Ok(outputs)
    }
}

/// SubSet component (GUID: `4cfc0bb0-0745-4772-a520-39f9bf3d99bc`)
#[derive(Debug, Default, Clone, Copy)]
struct SubSetComponent;
impl Component for SubSetComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &crate::graph::node::MetaMap) -> ComponentResult {
        let set_a = list_and_collect_into_hashset(&inputs[0])?;
        let set_b = list(&inputs[1])?;
        let is_subset = set_b.iter().all(|v| set_a.contains(v));
        let mut outputs = BTreeMap::new();
        outputs.insert("Result".to_owned(), Value::Boolean(is_subset));
        Ok(outputs)
    }
}

/// Key/Value Search component (GUID: `1edcc3cf-cf84-41d4-8204-561162cfe510`)
#[derive(Debug, Default, Clone, Copy)]
struct KeyValueSearchComponent;
impl Component for KeyValueSearchComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &crate::graph::node::MetaMap) -> ComponentResult {
        let keys = list(&inputs[0])?;
        let values = list(&inputs[1])?;
        let search_key = &inputs[2];
        if keys.len() != values.len() {
            return Err(ComponentError::new( "Keys and Values must have the same number of elements."));
        }
        let result = keys
            .iter()
            .position(|k| k == search_key)
            .map(|i| values[i].clone())
            .unwrap_or(Value::Null);
        let mut outputs = BTreeMap::new();
        outputs.insert("Result".to_owned(), result);
        Ok(outputs)
    }
}

/// Delete Consecutive component (GUID: `190d042c-2270-4bc1-81c0-4f90c170c9c9`)
#[derive(Debug, Default, Clone, Copy)]
struct DeleteConsecutiveComponent;
impl Component for DeleteConsecutiveComponent {
     fn evaluate(&self, inputs: &[Value], _meta: &crate::graph::node::MetaMap) -> ComponentResult {
        let set = list(&inputs[0])?;
        let wrap = boolean(&inputs[1]).unwrap_or(false);
        if set.is_empty() {
            let mut outputs = BTreeMap::new();
            outputs.insert("Set".to_owned(), Value::List(vec![]));
            outputs.insert("Count".to_owned(), Value::Number(0.0));
            return Ok(outputs);
        }
        let mut result_set: Vec<Value> = Vec::new();
        result_set.push(set[0].clone());
        for i in 1..set.len() {
            if set[i] != set[i - 1] {
                result_set.push(set[i].clone());
            }
        }
        let mut removed_count = set.len() - result_set.len();
        if wrap && result_set.len() > 1 && result_set.first() == result_set.last() {
            result_set.remove(0);
            removed_count += 1;
        }
        let mut outputs = BTreeMap::new();
        outputs.insert("Set".to_owned(), Value::List(result_set));
        outputs.insert("Count".to_owned(),Value::Number(removed_count as f64));
        Ok(outputs)
    }
}

/// Replace Members component (GUID: `bafac914-ede4-4a59-a7b2-cc41bc3de961`)
#[derive(Debug, Default, Clone, Copy)]
struct ReplaceMembersComponent;
impl Component for ReplaceMembersComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &crate::graph::node::MetaMap) -> ComponentResult {
        let set = list(&inputs[0])?;
        let find = list(&inputs[1])?;
        let replace = list(&inputs[2])?;
        let find_map: HashMap<_, _> = find.iter().zip(replace.iter().cycle()).collect();
        let result: Vec<_> = set
            .iter()
            .map(|v| find_map.get(v).map(|r| (*r).clone()).unwrap_or_else(|| v.clone()))
            .collect();
        let mut outputs = BTreeMap::new();
        outputs.insert("Result".to_owned(), Value::List(result));
        Ok(outputs)
    }
}

/// Set Majority component (GUID: `d4136a7b-7422-4660-9404-640474bd2725`)
#[derive(Debug, Default, Clone, Copy)]
struct SetMajorityComponent;
impl Component for SetMajorityComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &crate::graph::node::MetaMap) -> ComponentResult {
        let set_a: HashSet<_> = list(&inputs[0])?.iter().cloned().collect();
        let set_b: HashSet<_> = list(&inputs[1])?.iter().cloned().collect();
        let set_c: HashSet<_> = list(&inputs[2])?.iter().cloned().collect();

        let ab_intersect: HashSet<_> = set_a.intersection(&set_b).cloned().collect();
        let ac_intersect: HashSet<_> = set_a.intersection(&set_c).cloned().collect();
        let bc_intersect: HashSet<_> = set_b.intersection(&set_c).cloned().collect();

        let result: Vec<_> = ab_intersect
            .union(&ac_intersect)
            .cloned()
            .collect::<HashSet<_>>()
            .union(&bc_intersect)
            .cloned()
            .collect();

        let mut outputs = BTreeMap::new();
        outputs.insert("Result".to_owned(), Value::List(result));
        Ok(outputs)
    }
}

/// Cartesian Product component (GUID: `deffaf1e-270a-4c15-a693-9216b68afd4a`)
#[derive(Debug, Default, Clone, Copy)]
struct CartesianProductComponent;
impl Component for CartesianProductComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &crate::graph::node::MetaMap) -> ComponentResult {
        let set_a = list(&inputs[0])?;
        let set_b = list(&inputs[1])?;
        let mut product = Vec::new();
        for _a in set_a {
            let mut row = Vec::new();
            for b in set_b {
                row.push(b.clone());
            }
            product.push(Value::List(row));
        }
        let mut outputs = BTreeMap::new();
        outputs.insert("Product".to_owned(), Value::List(product));
        Ok(outputs)
    }
}

/// Find Similar Member component (GUID: `b4d4235f-14ff-4d4e-a29a-b358dcd2baf4`)
#[derive(Debug, Default, Clone, Copy)]
struct FindSimilarMemberComponent;
impl Component for FindSimilarMemberComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &crate::graph::node::MetaMap) -> ComponentResult {
        let set = list(&inputs[0])?;
        let data = &inputs[1];

        if set.is_empty() {
            let mut outputs = BTreeMap::new();
            outputs.insert("Hit".to_owned(), Value::Null);
            outputs.insert("Index".to_owned(), Value::Null);
            return Ok(outputs);
        }

        let mut min_dist = f64::MAX;
        let mut best_index = 0;

        for (i, member) in set.iter().enumerate() {
            let dist = match (data, member) {
                (Value::Number(a), Value::Number(b)) => (a - b).abs(),
                (Value::Point(a), Value::Point(b)) => {
                    let dx = a[0] - b[0];
                    let dy = a[1] - b[1];
                    let dz = a[2] - b[2];
                    (dx * dx + dy * dy + dz * dz).sqrt()
                }
                (Value::Vector(a), Value::Vector(b)) => {
                    let dx = a[0] - b[0];
                    let dy = a[1] - b[1];
                    let dz = a[2] - b[2];
                    (dx * dx + dy * dy + dz * dz).sqrt()
                }
                _ if data == member => 0.0,
                _ => f64::MAX,
            };

            if dist < min_dist {
                min_dist = dist;
                best_index = i;
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("Hit".to_owned(), set[best_index].clone());
        outputs.insert("Index".to_owned(), Value::Number(best_index as f64));
        Ok(outputs)
    }
}


// --- Registration ---

#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    CreateSetSimple,
    CreateSetWithMap,
    SetUnion,
    SetIntersection,
    SetDifference,
    SetDifferenceSymmetric,
    MemberIndex,
    Disjoint,
    SubSet,
    KeyValueSearch,
    DeleteConsecutive,
    ReplaceMembers,
    SetMajority,
    CartesianProduct,
    FindSimilarMember,
}

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], meta: &crate::graph::node::MetaMap) -> ComponentResult {
        match self {
            Self::CreateSetSimple => CreateSetSimpleComponent.evaluate(inputs, meta),
            Self::CreateSetWithMap => CreateSetWithMapComponent.evaluate(inputs, meta),
            Self::SetUnion => SetUnionComponent.evaluate(inputs, meta),
            Self::SetIntersection => SetIntersectionComponent.evaluate(inputs, meta),
            Self::SetDifference => SetDifferenceComponent.evaluate(inputs, meta),
            Self::SetDifferenceSymmetric => SetDifferenceSymmetricComponent.evaluate(inputs, meta),
            Self::MemberIndex => MemberIndexComponent.evaluate(inputs, meta),
            Self::Disjoint => DisjointComponent.evaluate(inputs, meta),
            Self::SubSet => SubSetComponent.evaluate(inputs, meta),
            Self::KeyValueSearch => KeyValueSearchComponent.evaluate(inputs, meta),
            Self::DeleteConsecutive => DeleteConsecutiveComponent.evaluate(inputs, meta),
            Self::ReplaceMembers => ReplaceMembersComponent.evaluate(inputs, meta),
            Self::SetMajority => SetMajorityComponent.evaluate(inputs, meta),
            Self::CartesianProduct => CartesianProductComponent.evaluate(inputs, meta),
            Self::FindSimilarMember => FindSimilarMemberComponent.evaluate(inputs, meta),
        }
    }
}

impl ComponentKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::CreateSetSimple | Self::CreateSetWithMap => "Create Set",
            Self::SetUnion => "Set Union",
            Self::SetIntersection => "Set Intersection",
            Self::SetDifference => "Set Difference",
            Self::SetDifferenceSymmetric => "Set Difference (S)",
            Self::MemberIndex => "Member Index",
            Self::Disjoint => "Disjoint",
            Self::SubSet => "SubSet",
            Self::KeyValueSearch => "Key/Value Search",
            Self::DeleteConsecutive => "Delete Consecutive",
            Self::ReplaceMembers => "Replace Members",
            Self::SetMajority => "Set Majority",
            Self::CartesianProduct => "Carthesian Product",
            Self::FindSimilarMember => "Find similar member",
        }
    }
}

pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["2cb4bf85-a282-464c-b42c-8e735d2a0a74"],
        names: &["Create Set", "CSet"],
        kind: ComponentKind::CreateSetSimple,
    },
    Registration {
        guids: &["98c3c63a-e78a-43ea-a111-514fcf312c95"],
        names: &["Create Set", "CSet"],
        kind: ComponentKind::CreateSetWithMap,
    },
    Registration {
        guids: &["8eed5d78-7810-4ba1-968e-8a1f1db98e39", "ab34845d-4ab9-4ff4-8870-eedd0c5594cb"],
        names: &["Set Union", "SUnion"],
        kind: ComponentKind::SetUnion,
    },
    Registration {
        guids: &["82f19c48-9e73-43a4-ae6c-3a8368099b08", "8a55f680-cf53-4634-a486-b828de92b71d"],
        names: &["Set Intersection", "Intersection"],
        kind: ComponentKind::SetIntersection,
    },
    Registration {
        guids: &["e3b1a10c-4d49-4140-b8e6-0b5732a26c31"],
        names: &["Set Difference", "Difference"],
        kind: ComponentKind::SetDifference,
    },
    Registration {
        guids: &["d2461702-3164-4894-8c10-ed1fc4b52965"],
        names: &["Set Difference (S)", "ExDiff"],
        kind: ComponentKind::SetDifferenceSymmetric,
    },
    Registration {
        guids: &["3ff27857-b988-417a-b495-b24c733dbd00"],
        names: &["Member Index", "MIndex"],
        kind: ComponentKind::MemberIndex,
    },
    Registration {
        guids: &["81800098-1060-4e2b-80d4-17f835cc825f"],
        names: &["Disjoint"],
        kind: ComponentKind::Disjoint,
    },
    Registration {
        guids: &["4cfc0bb0-0745-4772-a520-39f9bf3d99bc"],
        names: &["SubSet"],
        kind: ComponentKind::SubSet,
    },
    Registration {
        guids: &["1edcc3cf-cf84-41d4-8204-561162cfe510"],
        names: &["Key/Value Search", "KeySearch"],
        kind: ComponentKind::KeyValueSearch,
    },
    Registration {
        guids: &["190d042c-2270-4bc1-81c0-4f90c170c9c9"],
        names: &["Delete Consecutive", "DCon"],
        kind: ComponentKind::DeleteConsecutive,
    },
    Registration {
        guids: &["bafac914-ede4-4a59-a7b2-cc41bc3de961"],
        names: &["Replace Members", "Replace"],
        kind: ComponentKind::ReplaceMembers,
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
        guids: &["b4d4235f-14ff-4d4e-a29a-b358dcd2baf4"],
        names: &["Find similar member", "FSim"],
        kind: ComponentKind::FindSimilarMember,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::MetaMap;

    #[test]
    fn test_create_set_simple() {
        let component = CreateSetSimpleComponent;
        let inputs = &[Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(1.0),
            Value::Number(3.0),
            Value::Number(2.0),
        ])];
        let result = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let set = result.get("Set").unwrap();
        assert_eq!(
            set,
            &Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0)
            ])
        );
    }

    #[test]
    fn test_create_set_with_map() {
        let component = CreateSetWithMapComponent;
        let inputs = &[Value::List(vec![
            Value::Number(10.0),
            Value::Number(20.0),
            Value::Number(10.0),
        ])];
        let result = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let set = result.get("Set").unwrap();
        let map = result.get("Map").unwrap();
        assert_eq!(
            set,
            &Value::List(vec![Value::Number(10.0), Value::Number(20.0)])
        );
        assert_eq!(
            map,
            &Value::List(vec![
                Value::Number(0.0),
                Value::Number(1.0),
                Value::Number(0.0)
            ])
        );
    }

    #[test]
    fn test_set_union() {
        let component = SetUnionComponent;
        let inputs = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
        ];
        let result = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let union = result.get("Union").unwrap();
        assert_eq!(
            union,
            &Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0)
            ])
        );
    }

    #[test]
    fn test_set_intersection() {
        let component = SetIntersectionComponent;
        let inputs = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
        ];
        let result = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let intersection = result.get("Intersection").unwrap();
        assert_eq!(
            intersection,
            &Value::List(vec![Value::Number(2.0)])
        );
    }

    #[test]
    fn test_set_difference() {
        let component = SetDifferenceComponent;
        let inputs = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
        ];
        let result = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let difference = result.get("Difference").unwrap();
        assert_eq!(
            difference,
            &Value::List(vec![Value::Number(1.0)])
        );
    }

    #[test]
    fn test_set_difference_symmetric() {
        let component = SetDifferenceSymmetricComponent;
        let inputs = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
        ];
        let result = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let mut difference: Vec<_> = list(result.get("ExDifference").unwrap()).unwrap().to_vec();
        difference.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(
            difference,
            vec![Value::Number(1.0), Value::Number(3.0)]
        );
    }

    #[test]
    fn test_member_index() {
        let component = MemberIndexComponent;
        let inputs = &[
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(1.0),
            ]),
            Value::Number(1.0),
        ];
        let result = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let indices = result.get("Index").unwrap();
        let count = result.get("Count").unwrap();
        assert_eq!(
            indices,
            &Value::List(vec![Value::Number(0.0), Value::Number(2.0)])
        );
        assert_eq!(count, &Value::Number(2.0));
    }

    #[test]
    fn test_disjoint() {
        let component = DisjointComponent;
        let inputs_disjoint = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(3.0), Value::Number(4.0)]),
        ];
        let result_disjoint = component.evaluate(inputs_disjoint, &MetaMap::new()).unwrap();
        assert_eq!(result_disjoint.get("Result").unwrap(), &Value::Boolean(true));

        let inputs_not_disjoint = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
        ];
        let result_not_disjoint = component.evaluate(inputs_not_disjoint, &MetaMap::new()).unwrap();
        assert_eq!(
            result_not_disjoint.get("Result").unwrap(),
            &Value::Boolean(false)
        );
    }

    #[test]
    fn test_subset() {
        let component = SubSetComponent;
        let inputs_subset = &[
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
            ]),
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
        ];
        let result_subset = component.evaluate(inputs_subset, &MetaMap::new()).unwrap();
        assert_eq!(result_subset.get("Result").unwrap(), &Value::Boolean(true));

        let inputs_not_subset = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
        ];
        let result_not_subset = component.evaluate(inputs_not_subset, &MetaMap::new()).unwrap();
        assert_eq!(
            result_not_subset.get("Result").unwrap(),
            &Value::Boolean(false)
        );
    }

    #[test]
    fn test_key_value_search() {
        let component = KeyValueSearchComponent;
        let inputs = &[
            Value::List(vec![
                Value::Text("a".to_string()),
                Value::Text("b".to_string()),
            ]),
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::Text("b".to_string()),
        ];
        let result = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert_eq!(result.get("Result").unwrap(), &Value::Number(2.0));
    }

    #[test]
    fn test_delete_consecutive() {
        let component = DeleteConsecutiveComponent;
        let inputs_no_wrap = &[
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(2.0),
                Value::Number(1.0),
            ]),
            Value::Boolean(false),
        ];
        let result_no_wrap = component.evaluate(inputs_no_wrap, &MetaMap::new()).unwrap();
        assert_eq!(
            result_no_wrap.get("Set").unwrap(),
            &Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(1.0)
            ])
        );
        assert_eq!(result_no_wrap.get("Count").unwrap(), &Value::Number(2.0));

        let inputs_wrap = &[
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(2.0),
                Value::Number(1.0),
            ]),
            Value::Boolean(true),
        ];
        let result_wrap = component.evaluate(inputs_wrap, &MetaMap::new()).unwrap();
        assert_eq!(
            result_wrap.get("Set").unwrap(),
            &Value::List(vec![Value::Number(2.0), Value::Number(1.0)])
        );
        assert_eq!(result_wrap.get("Count").unwrap(), &Value::Number(3.0));
    }

    #[test]
    fn test_replace_members() {
        let component = ReplaceMembersComponent;
        let inputs = &[
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
            ]),
            Value::List(vec![Value::Number(1.0), Value::Number(3.0)]),
            Value::List(vec![Value::Number(10.0), Value::Number(30.0)]),
        ];
        let result = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let replaced = result.get("Result").unwrap();
        assert_eq!(
            replaced,
            &Value::List(vec![
                Value::Number(10.0),
                Value::Number(2.0),
                Value::Number(30.0)
            ])
        );
    }

    #[test]
    fn test_set_majority() {
        let component = SetMajorityComponent;
        let inputs = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
            Value::List(vec![Value::Number(3.0), Value::Number(4.0)]),
        ];
        let result = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let mut majority: Vec<_> = list(result.get("Result").unwrap()).unwrap().to_vec();
        majority.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(
            majority,
            vec![Value::Number(2.0), Value::Number(3.0)]
        );
    }

    #[test]
    fn test_cartesian_product() {
        let component = CartesianProductComponent;
        let inputs = &[
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Text("a".to_string()), Value::Text("b".to_string())]),
        ];
        let result = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let product = result.get("Product").unwrap();
        assert_eq!(
            product,
            &Value::List(vec![
                Value::List(vec![
                    Value::Text("a".to_string()),
                    Value::Text("b".to_string())
                ]),
                Value::List(vec![
                    Value::Text("a".to_string()),
                    Value::Text("b".to_string())
                ]),
            ])
        );
    }

    #[test]
    fn test_find_similar_member() {
        let component = FindSimilarMemberComponent;
        let inputs = &[
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
            ]),
            Value::Number(2.1),
        ];
        let result = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert_eq!(result.get("Hit").unwrap(), &Value::Number(2.0));
        assert_eq!(result.get("Index").unwrap(), &Value::Number(1.0));
    }
}
