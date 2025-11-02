//! Grasshopper components for manipulating data trees.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

pub const REGISTRATIONS: &[Registration] = &[
    Registration::new(
        "Simplify Tree",
        &["{06b3086c-1e9d-41c2-bcfc-bb843156196e}", "{1303da7b-e339-4e65-a051-82c4dce8224d}"],
        &["Simplify Tree", "Simplify"],
        ComponentKind::SimplifyTree(SimplifyTreeComponent),
    ),
    Registration::new(
        "Clean Tree",
        &["{071c3940-a12d-4b77-bb23-42b5d3314a0d}", "{70ce4230-da08-4fce-b29d-63dc42a88585}", "{7991bc5f-8a01-4768-bfb0-a39357ac6b84}"],
        &["Clean Tree", "Clean"],
        ComponentKind::CleanTree(CleanTreeComponent),
    ),
    Registration::new(
        "Merge",
        &["{0b6c5dac-6c93-4158-b8d1-ca3187d45f25}", "{3cadddef-1e2b-4c09-9390-0e8f78f7609f}", "{86866576-6cc0-485a-9cd2-6f7d493f57f7}", "{22f66ff6-d281-453c-bd8c-36ed24026783}", "{481f0339-1299-43ba-b15c-c07891a8f822}", "{a70aa477-0109-4e75-ba73-78725dca0274}", "{ac9b4faf-c9d5-4f6a-a5e9-58c0c2cac116}", "{b5be5d1f-717f-493c-b958-816957f271fd}", "{f4b0f7b4-5a10-46c4-8191-58d7d66ffdff}"],
        &["Merge", "M10", "M3", "M8", "M6", "M4", "M5"],
        ComponentKind::Merge(MergeComponent),
    ),
    Registration::new(
        "Graft Tree",
        &["{10a8674b-f4bb-4fdf-a56e-94dc606ecf33}", "{87e1d9ef-088b-4d30-9dda-8a7448a17329}"],
        &["Graft Tree", "Graft"],
        ComponentKind::GraftTree(GraftTreeComponent),
    ),
    Registration::new(
        "Trim Tree",
        &["{1177d6ee-3993-4226-9558-52b7fd63e1e3}"],
        &["Trim Tree", "Trim"],
        ComponentKind::TrimTree(TrimTreeComponent),
    ),
    Registration::new(
        "Path Compare",
        &["{1d8b0e2c-e772-4fa9-b7f7-b158251b34b8}"],
        &["Path Compare", "Compare"],
        ComponentKind::PathCompare(PathCompareComponent),
    ),
    Registration::new(
        "Relative Items",
        &["{2653b135-4df1-4a6b-820c-55e2ad3bc1e0}", "{fac0d5be-e3ff-4bbb-9742-ec9a54900d41}"],
        &["Relative Items", "RelItem2", "RelItem"],
        ComponentKind::RelativeItems(RelativeItemsComponent),
    ),
    Registration::new(
        "Shift Paths",
        &["{2d61f4e0-47c5-41d6-a41d-6afa96ee63af}"],
        &["Shift Paths", "PShift"],
        ComponentKind::ShiftPaths(ShiftPathsComponent),
    ),
    Registration::new(
        "Tree Branch",
        &["{3a710c1e-1809-4e19-8c15-82adce31cd62}"],
        &["Tree Branch", "Branch"],
        ComponentKind::TreeBranch(TreeBranchComponent),
    ),
    Registration::new(
        "Stream Filter",
        &["{3e5582a1-901a-4f7c-b58d-f5d7e3166124}", "{eeafc956-268e-461d-8e73-ee05c6f72c01}"],
        &["Stream Filter", "Filter"],
        ComponentKind::StreamFilter(StreamFilterComponent),
    ),
    Registration::new(
        "Flip Matrix",
        &["{41aa4112-9c9b-42f4-847e-503b9d90e4c7}"],
        &["Flip Matrix", "Flip"],
        ComponentKind::FlipMatrix(FlipMatrixComponent),
    ),
    Registration::new(
        "Match Tree",
        &["{46372d0d-82dc-4acb-adc3-25d1fde04c4e}"],
        &["Match Tree", "Match"],
        ComponentKind::MatchTree(MatchTreeComponent),
    ),
    Registration::new(
        "Stream Gate",
        &["{71fcc052-6add-4d70-8d97-cfb37ea9d169}", "{d6313940-216b-487f-b511-6c8a5b87eae7}"],
        &["Stream Gate", "Gate"],
        ComponentKind::StreamGate(StreamGateComponent),
    ),
    Registration::new(
        "Explode Tree",
        &["{74cad441-2264-45fe-a57d-85034751208a}", "{8a470a35-d673-4779-a65e-ba95765e59e4}"],
        &["Explode Tree", "BANG!"],
        ComponentKind::ExplodeTree(ExplodeTreeComponent),
    ),
    Registration::new(
        "Construct Path",
        &["{946cb61e-18d2-45e3-8840-67b0efa26528}"],
        &["Construct Path", "Path"],
        ComponentKind::ConstructPath(ConstructPathComponent),
    ),
    Registration::new(
        "Tree Statistics",
        &["{99bee19d-588c-41a0-b9b9-1d00fb03ea1a}"],
        &["Tree Statistics", "TStat"],
        ComponentKind::TreeStatistics(TreeStatisticsComponent),
    ),
    Registration::new(
        "Flatten Tree",
        &["{a13fcd5d-81af-4337-a32e-28dd7e23ae4c}", "{f80cfe18-9510-4b89-8301-8e58faf423bb}"],
        &["Flatten Tree", "Flatten"],
        ComponentKind::FlattenTree(FlattenTreeComponent),
    ),
    Registration::new(
        "Unflatten Tree",
        &["{b8e2aa8f-8830-4ee1-bb59-613ea279c281}"],
        &["Unflatten Tree", "Unflatten"],
        ComponentKind::UnflattenTree(UnflattenTreeComponent),
    ),
    Registration::new(
        "Replace Paths",
        &["{bfaaf799-77dc-4f31-9ad8-2f7d1a80aeb0}"],
        &["Replace Paths", "Replace"],
        ComponentKind::ReplacePaths(ReplacePathsComponent),
    ),
    Registration::new(
        "Tree Item",
        &["{c1ec65a3-bda4-4fad-87d0-edf86ed9d81c}"],
        &["Tree Item", "Item"],
        ComponentKind::TreeItem(TreeItemComponent),
    ),
    Registration::new(
        "Entwine",
        &["{c9785b8e-2f30-4f90-8ee3-cca710f82402}"],
        &["Entwine"],
        ComponentKind::Entwine(EntwineComponent),
    ),
    Registration::new(
        "Split Tree",
        &["{d8b1e7ac-cd31-4748-b262-e07e53068afc}"],
        &["Split Tree", "Split"],
        ComponentKind::SplitTree(SplitTreeComponent),
    ),
    Registration::new(
        "Deconstruct Path",
        &["{df6d9197-9a6e-41a2-9c9d-d2221accb49e}"],
        &["Deconstruct Path", "DPath"],
        ComponentKind::DeconstructPath(DeconstructPathComponent),
    ),
    Registration::new(
        "Null Check",
        &["{e6859d1e-2b3d-4704-93ea-32714acae176}"],
        &["Null Check", "Null"],
        ComponentKind::NullCheck(NullCheckComponent),
    ),
    Registration::new(
        "Prune Tree",
        &["{fe769f85-8900-45dd-ba11-ec9cd6c778c6}"],
        &["Prune Tree", "Prune"],
        ComponentKind::PruneTree(PruneTreeComponent),
    ),
];

pub struct Registration {
    pub name: &'static str,
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

impl Registration {
    pub const fn new(
        name: &'static str,
        guids: &'static [&'static str],
        names: &'static [&'static str],
        kind: ComponentKind,
    ) -> Self {
        Self {
            name,
            guids,
            names,
            kind,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    SimplifyTree(SimplifyTreeComponent),
    CleanTree(CleanTreeComponent),
    Merge(MergeComponent),
    GraftTree(GraftTreeComponent),
    TrimTree(TrimTreeComponent),
    PathCompare(PathCompareComponent),
    RelativeItems(RelativeItemsComponent),
    ShiftPaths(ShiftPathsComponent),
    TreeBranch(TreeBranchComponent),
    StreamFilter(StreamFilterComponent),
    FlipMatrix(FlipMatrixComponent),
    MatchTree(MatchTreeComponent),
    StreamGate(StreamGateComponent),
    ExplodeTree(ExplodeTreeComponent),
    ConstructPath(ConstructPathComponent),
    TreeStatistics(TreeStatisticsComponent),
    FlattenTree(FlattenTreeComponent),
    UnflattenTree(UnflattenTreeComponent),
    ReplacePaths(ReplacePathsComponent),
    TreeItem(TreeItemComponent),
    Entwine(EntwineComponent),
    SplitTree(SplitTreeComponent),
    DeconstructPath(DeconstructPathComponent),
    NullCheck(NullCheckComponent),
    PruneTree(PruneTreeComponent),
}

impl ComponentKind {
    pub fn evaluate(self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::SimplifyTree(c) => c.evaluate(inputs, meta),
            Self::CleanTree(c) => c.evaluate(inputs, meta),
            Self::Merge(c) => c.evaluate(inputs, meta),
            Self::GraftTree(c) => c.evaluate(inputs, meta),
            Self::TrimTree(c) => c.evaluate(inputs, meta),
            Self::PathCompare(c) => c.evaluate(inputs, meta),
            Self::RelativeItems(c) => c.evaluate(inputs, meta),
            Self::ShiftPaths(c) => c.evaluate(inputs, meta),
            Self::TreeBranch(c) => c.evaluate(inputs, meta),
            Self::StreamFilter(c) => c.evaluate(inputs, meta),
            Self::FlipMatrix(c) => c.evaluate(inputs, meta),
            Self::MatchTree(c) => c.evaluate(inputs, meta),
            Self::StreamGate(c) => c.evaluate(inputs, meta),
            Self::ExplodeTree(c) => c.evaluate(inputs, meta),
            Self::ConstructPath(c) => c.evaluate(inputs, meta),
            Self::TreeStatistics(c) => c.evaluate(inputs, meta),
            Self::FlattenTree(c) => c.evaluate(inputs, meta),
            Self::UnflattenTree(c) => c.evaluate(inputs, meta),
            Self::ReplacePaths(c) => c.evaluate(inputs, meta),
            Self::TreeItem(c) => c.evaluate(inputs, meta),
            Self::Entwine(c) => c.evaluate(inputs, meta),
            Self::SplitTree(c) => c.evaluate(inputs, meta),
            Self::DeconstructPath(c) => c.evaluate(inputs, meta),
            Self::NullCheck(c) => c.evaluate(inputs, meta),
            Self::PruneTree(c) => c.evaluate(inputs, meta),
        }
    }

    pub fn name(self) -> &'static str {
        for registration in REGISTRATIONS {
            // A pointer comparison should be sufficient.
            if std::ptr::eq(&registration.kind as *const _, &self as *const _) {
                return registration.name;
            }
        }
        "Unnamed SetsTree component"
    }
}


#[derive(Debug, Default, Clone, Copy)]
pub struct SimplifyTreeComponent;

impl Component for SimplifyTreeComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Simplify Tree component is not yet implemented."))
    }
}

use super::coerce::{coerce_to_boolean};

#[derive(Debug, Default, Clone, Copy)]
pub struct CleanTreeComponent;

impl Component for CleanTreeComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 4 {
            return Err(ComponentError::new(
                "Clean Tree component requires four inputs.",
            ));
        }

        let remove_nulls = coerce_to_boolean(&inputs[0]).unwrap_or(true);
        let remove_invalid = coerce_to_boolean(&inputs[1]).unwrap_or(true);
        let remove_empty = coerce_to_boolean(&inputs[2]).unwrap_or(true);
        let tree = &inputs[3];

        let cleaned_tree = clean_tree(tree, remove_nulls, remove_invalid, remove_empty);

        let mut outputs = BTreeMap::new();
        outputs.insert("T".to_string(), cleaned_tree);
        Ok(outputs)
    }
}

fn clean_tree(value: &Value, remove_nulls: bool, remove_invalid: bool, remove_empty: bool) -> Value {
    match value {
        Value::List(items) => {
            let cleaned_items: Vec<Value> = items
                .iter()
                .filter_map(|item| {
                    let cleaned_item = clean_tree(item, remove_nulls, remove_invalid, remove_empty);
                    match &cleaned_item {
                        Value::Null if remove_nulls || remove_invalid => None,
                        Value::List(list) if list.is_empty() && remove_empty => None,
                        _ => Some(cleaned_item),
                    }
                })
                .collect();
            Value::List(cleaned_items)
        }
        Value::Null if remove_nulls || remove_invalid => Value::Null,
        _ => value.clone(),
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MergeComponent;

impl Component for MergeComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Ok(BTreeMap::new());
        }

        let lists: Vec<_> = inputs
            .iter()
            .map(|v| if let Value::List(l) = v { l.clone() } else { vec![v.clone()] })
            .collect();

        let max_len = lists.iter().map(|l| l.len()).max().unwrap_or(0);
        let mut merged = Vec::with_capacity(max_len * lists.len());

        for i in 0..max_len {
            for list in &lists {
                if let Some(item) = list.get(i) {
                    merged.push(item.clone());
                }
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("S".to_string(), Value::List(merged));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GraftTreeComponent;

impl Component for GraftTreeComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Graft Tree component requires at least one input."));
        }

        let mut grafted_tree = Vec::new();
        if let Value::List(items) = &inputs[0] {
            for item in items {
                grafted_tree.push(Value::List(vec![item.clone()]));
            }
        } else {
            grafted_tree.push(Value::List(vec![inputs[0].clone()]));
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("T".to_string(), Value::List(grafted_tree));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TrimTreeComponent;

impl Component for TrimTreeComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Trim Tree component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PathCompareComponent;

impl Component for PathCompareComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Path Compare component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RelativeItemsComponent;

impl Component for RelativeItemsComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Relative Items component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ShiftPathsComponent;

impl Component for ShiftPathsComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Shift Paths component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TreeBranchComponent;

impl Component for TreeBranchComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Tree Branch component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StreamFilterComponent;

impl Component for StreamFilterComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Stream Filter component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FlipMatrixComponent;

impl Component for FlipMatrixComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Flip Matrix component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MatchTreeComponent;

impl Component for MatchTreeComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Match Tree component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StreamGateComponent;

impl Component for StreamGateComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Stream Gate component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ExplodeTreeComponent;

impl Component for ExplodeTreeComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Explode Tree component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ConstructPathComponent;

impl Component for ConstructPathComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Construct Path component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TreeStatisticsComponent;

impl Component for TreeStatisticsComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Tree Statistics component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FlattenTreeComponent;

impl Component for FlattenTreeComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new(
                "Flatten Tree component requires at least one input.",
            ));
        }

        let mut flattened = Vec::new();
        flatten_recursive(&inputs[0], &mut flattened);

        let mut outputs = BTreeMap::new();
        outputs.insert("T".to_string(), Value::List(flattened));
        Ok(outputs)
    }
}

fn flatten_recursive(value: &Value, flattened: &mut Vec<Value>) {
    if let Value::List(items) = value {
        for item in items {
            flatten_recursive(item, flattened);
        }
    } else {
        flattened.push(value.clone());
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UnflattenTreeComponent;

impl Component for UnflattenTreeComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Unflatten Tree component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ReplacePathsComponent;

impl Component for ReplacePathsComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Replace Paths component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TreeItemComponent;

impl Component for TreeItemComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Tree Item component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EntwineComponent;

impl Component for EntwineComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Entwine component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SplitTreeComponent;

impl Component for SplitTreeComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Split Tree component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DeconstructPathComponent;

impl Component for DeconstructPathComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Deconstruct Path component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NullCheckComponent;

impl Component for NullCheckComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Null Check component is not yet implemented."))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PruneTreeComponent;

impl Component for PruneTreeComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new("Prune Tree component is not yet implemented."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::value::Value;

    #[test]
    fn test_flatten_tree_recursive() {
        let component = FlattenTreeComponent;
        let inputs = vec![Value::List(vec![
            Value::List(vec![Value::List(vec![Value::Number(1.0)])]),
            Value::Number(2.0),
        ])];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let expected = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        assert_eq!(outputs.get("T"), Some(&expected));
    }

    #[test]
    fn test_graft_tree() {
        let component = GraftTreeComponent;
        let inputs = vec![Value::List(vec![Value::Number(1.0), Value::Number(2.0)])];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let expected = Value::List(vec![
            Value::List(vec![Value::Number(1.0)]),
            Value::List(vec![Value::Number(2.0)]),
        ]);
        assert_eq!(outputs.get("T"), Some(&expected));
    }

    #[test]
    fn test_merge_interleaves() {
        let component = MergeComponent;
        let inputs = vec![
            Value::List(vec![Value::Number(1.0), Value::Number(3.0)]),
            Value::List(vec![Value::Number(2.0), Value::Number(4.0)]),
        ];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let expected = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ]);
        assert_eq!(outputs.get("S"), Some(&expected));
    }

    #[test]
    fn test_clean_tree() {
        let component = CleanTreeComponent;
        let inputs = vec![
            Value::Boolean(true),
            Value::Boolean(true),
            Value::Boolean(true),
            Value::List(vec![
                Value::Number(1.0),
                Value::Null,
                Value::List(vec![]),
                Value::Number(2.0),
            ]),
        ];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let expected = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        assert_eq!(outputs.get("T"), Some(&expected));
    }
}
