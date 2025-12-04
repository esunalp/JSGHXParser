//! Grasshopper components for manipulating data trees.

use std::collections::BTreeMap;

use crate::components::coerce::{coerce_boolean, coerce_integer, coerce_text};
use crate::graph::node::{MetaLookupExt, MetaMap, MetaValue};
use crate::graph::value::Value;
use wildmatch::WildMatch;

use super::{Component, ComponentError, ComponentResult};

#[derive(Debug)]
struct Tree {
    branches: BTreeMap<Vec<usize>, Vec<Value>>,
}

impl From<&Value> for Tree {
    fn from(value: &Value) -> Self {
        let mut branches = BTreeMap::new();
        collect_branches_recursive(value, vec![], &mut branches, true);
        Self { branches }
    }
}

impl Tree {
    fn simplify(self) -> Self {
        if self.branches.len() <= 1 {
            return self;
        }

        let paths: Vec<_> = self.branches.keys().collect();
        let mut common_prefix_len = paths[0].len();
        for path in paths.iter().skip(1) {
            common_prefix_len = common_prefix_len.min(path.len()).min(
                paths[0]
                    .iter()
                    .zip(path.iter())
                    .take_while(|(a, b)| a == b)
                    .count(),
            );
        }

        if common_prefix_len == 0 {
            return self;
        }

        let branches = self
            .branches
            .into_iter()
            .map(|(path, values)| (path[common_prefix_len..].to_vec(), values))
            .collect();
        Self { branches }
    }

    fn to_value(&self) -> Value {
        build_tree_from_branches(&self.branches)
    }

    fn shift_paths(self, offset: isize) -> Self {
        let branches = self
            .branches
            .into_iter()
            .map(|(path, values)| {
                let new_path = path
                    .into_iter()
                    .map(|i| (i as isize + offset).max(0) as usize)
                    .collect();
                (new_path, values)
            })
            .collect();
        Self { branches }
    }

    fn flattened_items(&self) -> Vec<Value> {
        self.branches.values().flatten().cloned().collect()
    }

    fn flip_matrix(self) -> Self {
        let mut new_branches: BTreeMap<Vec<usize>, Vec<Value>> = BTreeMap::new();
        let mut max_len = 0;
        let paths: Vec<_> = self.branches.keys().cloned().collect();
        for (_, branch) in &self.branches {
            max_len = max_len.max(branch.len());
        }

        for i in 0..max_len {
            let mut new_branch = vec![];
            for path in &paths {
                new_branch.push(
                    self.branches
                        .get(path)
                        .and_then(|b| b.get(i))
                        .cloned()
                        .unwrap_or(Value::Null),
                );
            }
            new_branches.insert(vec![i], new_branch);
        }

        Self {
            branches: new_branches,
        }
    }

    fn unflatten_with(guide_tree: &Tree, items: &[Value]) -> Self {
        let mut new_branches = BTreeMap::new();
        let mut item_index = 0;
        for (path, guide_branch) in &guide_tree.branches {
            let mut new_branch = Vec::new();
            for _ in 0..guide_branch.len() {
                if item_index < items.len() {
                    new_branch.push(items[item_index].clone());
                    item_index += 1;
                } else {
                    break;
                }
            }
            if !new_branch.is_empty() {
                new_branches.insert(path.clone(), new_branch);
            }
        }
        Self {
            branches: new_branches,
        }
    }

    fn replace_paths(
        &self,
        search_masks: &[Value],
        replace_paths: &[Value],
    ) -> Result<Self, ComponentError> {
        let mut new_branches = self.branches.clone();
        for (search, replace) in search_masks.iter().zip(replace_paths.iter()) {
            let search_mask = coerce_text(search)?;
            let replace_path_str = coerce_text(replace)?;

            let search_mask = search_mask.trim_matches(|c| c == '{' || c == '}');
            let replace_path = parse_path(&replace_path_str)?;

            let wm = WildMatch::new(search_mask);
            let mut branches_to_replace = vec![];

            for (path, _) in &new_branches {
                let path_str = path
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(";");
                if wm.matches(&path_str) {
                    branches_to_replace.push(path.clone());
                }
            }

            for path in branches_to_replace {
                if let Some(branch) = new_branches.remove(&path) {
                    new_branches.insert(replace_path.clone(), branch);
                }
            }
        }
        Ok(Self {
            branches: new_branches,
        })
    }

    fn entwine(inputs: &[Value]) -> Self {
        let mut new_branches = BTreeMap::new();
        for (i, input) in inputs.iter().enumerate() {
            if let Value::List(items) = input {
                if items.iter().all(|item| !matches!(item, Value::List(_))) {
                    new_branches.insert(vec![i], items.clone());
                } else {
                    let tree = Tree::from(input);
                    for (path, branch) in tree.branches {
                        let mut new_path = vec![i];
                        new_path.extend(path);
                        new_branches.insert(new_path, branch);
                    }
                }
            } else {
                new_branches.insert(vec![i], vec![input.clone()]);
            }
        }
        Self {
            branches: new_branches,
        }
    }

    fn split_tree(&self, search_mask: &str) -> (Self, Self) {
        let mut matching_branches = BTreeMap::new();
        let mut non_matching_branches = BTreeMap::new();

        let search_mask = search_mask.trim_matches(|c| c == '{' || c == '}');
        let wm = WildMatch::new(search_mask);

        for (path, branch) in &self.branches {
            let path_str = path
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(";");

            if wm.matches(&path_str) {
                matching_branches.insert(path.clone(), branch.clone());
            } else {
                non_matching_branches.insert(path.clone(), branch.clone());
            }
        }

        (
            Self {
                branches: matching_branches,
            },
            Self {
                branches: non_matching_branches,
            },
        )
    }

    fn prune(&self, n: i64) -> Self {
        let branches = self
            .branches
            .iter()
            .filter(|(_, branch)| {
                let len = branch.len();
                if n >= 0 {
                    len <= n as usize
                } else {
                    len >= n.abs() as usize
                }
            })
            .map(|(path, branch)| (path.clone(), branch.clone()))
            .collect();
        Self { branches }
    }
}

fn collect_branches_recursive(
    value: &Value,
    path: Vec<usize>,
    branches: &mut BTreeMap<Vec<usize>, Vec<Value>>,
    is_root: bool,
) {
    if let Value::List(items) = value {
        if is_root {
            for (i, item) in items.iter().enumerate() {
                collect_branches_recursive(item, vec![i], branches, false);
            }
        } else if items.iter().all(|item| !matches!(item, Value::List(_))) {
            branches.insert(path, items.clone());
        } else {
            for (i, item) in items.iter().enumerate() {
                let mut new_path = path.clone();
                new_path.push(i);
                collect_branches_recursive(item, new_path, branches, false);
            }
        }
    } else {
        branches.insert(path, vec![value.clone()]);
    }
}

#[derive(Debug)]
enum TempNode {
    Branch(Vec<Value>),
    SubTree(BTreeMap<usize, TempNode>),
}

fn build_tree_from_branches(branches: &BTreeMap<Vec<usize>, Vec<Value>>) -> Value {
    if branches.is_empty() {
        return Value::List(vec![]);
    }

    let mut root = TempNode::SubTree(BTreeMap::new());
    for (path, values) in branches {
        insert_branch(&mut root, path, values.clone());
    }
    temp_node_to_value(&root)
}

fn insert_branch(node: &mut TempNode, path: &[usize], values: Vec<Value>) {
    if path.is_empty() {
        *node = TempNode::Branch(values);
        return;
    }

    let index = path[0];
    let rest_of_path = &path[1..];

    if let TempNode::SubTree(map) = node {
        let child = map.entry(index).or_insert_with(|| {
            if rest_of_path.is_empty() {
                TempNode::Branch(vec![])
            } else {
                TempNode::SubTree(BTreeMap::new())
            }
        });
        insert_branch(child, rest_of_path, values);
    }
}

fn temp_node_to_value(node: &TempNode) -> Value {
    match node {
        TempNode::Branch(items) => Value::List(items.clone()),
        TempNode::SubTree(map) => {
            if map.is_empty() {
                return Value::List(vec![]);
            }
            let max_index = *map.keys().max().unwrap_or(&0);
            let mut list = Vec::with_capacity(max_index + 1);
            for i in 0..=max_index {
                let value = map
                    .get(&i)
                    .map(temp_node_to_value)
                    .unwrap_or_else(|| Value::List(vec![]));
                list.push(value);
            }
            Value::List(list)
        }
    }
}

fn parse_path(path_str: &str) -> Result<Vec<usize>, ComponentError> {
    let trimmed = path_str.trim_matches(|c| c == '{' || c == '}');
    if trimmed.is_empty() {
        return Ok(vec![]);
    }
    trimmed
        .split(';')
        .map(|s| {
            s.parse::<usize>()
                .map_err(|_| ComponentError::new(format!("Invalid path segment: {}", s)))
        })
        .collect()
}

pub const REGISTRATIONS: &[Registration] = &[
    Registration::new(
        "Simplify Tree",
        &[
            "{06b3086c-1e9d-41c2-bcfc-bb843156196e}",
            "{1303da7b-e339-4e65-a051-82c4dce8224d}",
        ],
        &["Simplify Tree", "Simplify"],
        ComponentKind::SimplifyTree(SimplifyTreeComponent),
    ),
    Registration::new(
        "Clean Tree",
        &[
            "{071c3940-a12d-4b77-bb23-42b5d3314a0d}",
            "{70ce4230-da08-4fce-b29d-63dc42a88585}",
            "{7991bc5f-8a01-4768-bfb0-a39357ac6b84}",
        ],
        &["Clean Tree", "Clean"],
        ComponentKind::CleanTree(CleanTreeComponent),
    ),
    Registration::new(
        "Merge",
        &[
            "{0b6c5dac-6c93-4158-b8d1-ca3187d45f25}",
            "{3cadddef-1e2b-4c09-9390-0e8f78f7609f}",
            "{86866576-6cc0-485a-9cd2-6f7d493f57f7}",
            "{22f66ff6-d281-453c-bd8c-36ed24026783}",
            "{481f0339-1299-43ba-b15c-c07891a8f822}",
            "{a70aa477-0109-4e75-ba73-78725dca0274}",
            "{ac9b4faf-c9d5-4f6a-a5e9-58c0c2cac116}",
            "{b5be5d1f-717f-493c-b958-816957f271fd}",
            "{f4b0f7b4-5a10-46c4-8191-58d7d66ffdff}",
        ],
        &["Merge", "M10", "M3", "M8", "M6", "M4", "M5"],
        ComponentKind::Merge(MergeComponent),
    ),
    Registration::new(
        "Graft Tree",
        &[
            "{10a8674b-f4bb-4fdf-a56e-94dc606ecf33}",
            "{87e1d9ef-088b-4d30-9dda-8a7448a17329}",
        ],
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
        &[
            "{2653b135-4df1-4a6b-820c-55e2ad3bc1e0}",
            "{fac0d5be-e3ff-4bbb-9742-ec9a54900d41}",
        ],
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
        &[
            "{3e5582a1-901a-4f7c-b58d-f5d7e3166124}",
            "{eeafc956-268e-461d-8e73-ee05c6f72c01}",
        ],
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
        &[
            "{71fcc052-6add-4d70-8d97-cfb37ea9d169}",
            "{d6313940-216b-487f-b511-6c8a5b87eae7}",
        ],
        &["Stream Gate", "Gate"],
        ComponentKind::StreamGate(StreamGateComponent),
    ),
    Registration::new(
        "Explode Tree",
        &[
            "{74cad441-2264-45fe-a57d-85034751208a}",
            "{8a470a35-d673-4779-a65e-ba95765e59e4}",
        ],
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
        &[
            "{a13fcd5d-81af-4337-a32e-28dd7e23ae4c}",
            "{f80cfe18-9510-4b89-8301-8e58faf423bb}",
        ],
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
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new(
                "Simplify Tree component requires at least one input.",
            ));
        }
        let tree = Tree::from(&inputs[0]);
        let simplified_tree = tree.simplify();
        let mut outputs = BTreeMap::new();
        outputs.insert("T".to_string(), simplified_tree.to_value());
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CleanTreeComponent;

impl Component for CleanTreeComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 4 {
            return Err(ComponentError::new(
                "Clean Tree component requires four inputs.",
            ));
        }

        let remove_nulls = coerce_boolean(&inputs[0]).unwrap_or(true);
        let remove_invalid = coerce_boolean(&inputs[1]).unwrap_or(true);
        let remove_empty = coerce_boolean(&inputs[2]).unwrap_or(true);
        let tree = &inputs[3];

        let cleaned_tree = clean_tree(tree, remove_nulls, remove_invalid, remove_empty);

        let mut outputs = BTreeMap::new();
        outputs.insert("T".to_string(), cleaned_tree);
        Ok(outputs)
    }
}

fn clean_tree(
    value: &Value,
    remove_nulls: bool,
    remove_invalid: bool,
    remove_empty: bool,
) -> Value {
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
            .filter_map(|value| match value {
                Value::Null => None,
                Value::List(list) => Some(list.clone()),
                other => Some(vec![other.clone()]),
            })
            .collect();

        if lists.is_empty() {
            return Ok(BTreeMap::new());
        }

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
        // Grasshopper labels the merge output pin as "R" (Result), but we also
        // keep the legacy "S" key to stay backwards compatible with earlier
        // assumptions in our engine.
        let merged_value = Value::List(merged);
        outputs.insert("R".to_string(), merged_value.clone());
        outputs.insert("S".to_string(), merged_value);
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GraftTreeComponent;

impl Component for GraftTreeComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new(
                "Graft Tree component requires at least one input.",
            ));
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
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new(
                "Trim Tree component requires two inputs.",
            ));
        }
        let tree = Tree::from(&inputs[0]);
        let depth = coerce_integer(&inputs[1])? as usize;

        let mut new_branches = BTreeMap::new();
        for (path, values) in tree.branches {
            let new_path = if path.len() > depth {
                path[..path.len() - depth].to_vec()
            } else {
                vec![]
            };
            new_branches
                .entry(new_path)
                .or_insert_with(Vec::new)
                .extend(values);
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(
            "T".to_string(),
            Tree {
                branches: new_branches,
            }
            .to_value(),
        );
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PathCompareComponent;

impl Component for PathCompareComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new(
                "Path Compare component requires two inputs.",
            ));
        }
        let path_str = coerce_text(&inputs[0])?;
        let mask = coerce_text(&inputs[1])?;

        let path_str = path_str.trim_matches(|c| c == '{' || c == '}');
        let mask = mask.trim_matches(|c| c == '{' || c == '}');

        let wm = WildMatch::new(mask);
        let result = wm.matches(path_str);

        let mut outputs = BTreeMap::new();
        outputs.insert("C".to_string(), Value::Boolean(result));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RelativeItemsComponent;

impl Component for RelativeItemsComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 3 {
            return Err(ComponentError::new(
                "Relative Items component requires at least three inputs.",
            ));
        }
        let tree_a = Tree::from(&inputs[0]);
        let tree_b = if inputs.len() > 3 {
            Tree::from(&inputs[1])
        } else {
            tree_a.clone_tree()
        };
        let offset = coerce_integer(if inputs.len() > 3 {
            &inputs[2]
        } else {
            &inputs[1]
        })?;
        let wrap = if inputs.len() > 3 {
            coerce_boolean(&inputs[3])?
        } else {
            false
        };
        let mut outputs = BTreeMap::new();
        let mut result_a = BTreeMap::new();
        let mut result_b = BTreeMap::new();

        for (path, branch_a) in &tree_a.branches {
            if let Some(branch_b) = tree_b.branches.get(path) {
                let mut new_branch_a = vec![];
                let mut new_branch_b = vec![];

                for (i, item_a) in branch_a.iter().enumerate() {
                    let new_index = i as i64 + offset;
                    let item_b = if wrap {
                        branch_b.get((new_index.rem_euclid(branch_b.len() as i64)) as usize)
                    } else {
                        branch_b.get(new_index as usize)
                    };

                    if let Some(item_b) = item_b {
                        new_branch_a.push(item_a.clone());
                        new_branch_b.push(item_b.clone());
                    }
                }
                result_a.insert(path.clone(), new_branch_a);
                result_b.insert(path.clone(), new_branch_b);
            }
        }

        outputs.insert("A".to_string(), Tree { branches: result_a }.to_value());
        outputs.insert("B".to_string(), Tree { branches: result_b }.to_value());
        Ok(outputs)
    }
}

impl Tree {
    fn clone_tree(&self) -> Self {
        Tree {
            branches: self.branches.clone(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ShiftPathsComponent;

impl Component for ShiftPathsComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new(
                "Shift Paths component requires two inputs.",
            ));
        }
        let tree = Tree::from(&inputs[0]);
        let offset = coerce_integer(&inputs[1])? as isize;

        let shifted_tree = tree.shift_paths(offset);

        let mut outputs = BTreeMap::new();
        outputs.insert("D".to_string(), shifted_tree.to_value());
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TreeBranchComponent;

impl Component for TreeBranchComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new(
                "Tree Branch component requires two inputs.",
            ));
        }
        let tree = Tree::from(&inputs[0]);
        let path_str = coerce_text(&inputs[1])?;
        let path = parse_path(&path_str)?;

        let mut outputs = BTreeMap::new();
        if let Some(branch) = tree.branches.get(&path) {
            outputs.insert("B".to_string(), Value::List(branch.clone()));
        } else {
            outputs.insert("B".to_string(), Value::List(vec![]));
        }
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StreamFilterComponent;

fn stream_filter_output_pin(meta: &MetaMap) -> String {
    meta.get_normalized("OutputPins")
        .and_then(|value| match value {
            MetaValue::List(list) => list.iter().find_map(|entry| {
                if let MetaValue::Text(text) = entry {
                    if !text.is_empty() {
                        return Some(text.clone());
                    }
                }
                None
            }),
            MetaValue::Text(text) if !text.is_empty() => Some(text.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "S".to_owned())
}

impl Component for StreamFilterComponent {
    fn evaluate(&self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new(
                "Stream Filter component requires at least two inputs.",
            ));
        }
        let gate = coerce_integer(&inputs[0])? as usize;
        if gate < inputs.len() - 1 {
            let mut outputs = BTreeMap::new();
            let output_pin = stream_filter_output_pin(meta);
            outputs.insert(output_pin, inputs[gate + 1].clone());
            Ok(outputs)
        } else {
            Err(ComponentError::new("Gate index out of bounds."))
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FlipMatrixComponent;

impl Component for FlipMatrixComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new(
                "Flip Matrix component requires one input.",
            ));
        }
        let tree = Tree::from(&inputs[0]);
        let flipped_tree = tree.flip_matrix();
        let mut outputs = BTreeMap::new();
        outputs.insert("D".to_string(), flipped_tree.to_value());
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MatchTreeComponent;

impl Component for MatchTreeComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new(
                "Match Tree component requires two inputs.",
            ));
        }
        let tree_to_modify = Tree::from(&inputs[0]);
        let guide_tree = Tree::from(&inputs[1]);

        let items = tree_to_modify.flattened_items();
        let new_tree = Tree::unflatten_with(&guide_tree, &items);

        let mut outputs = BTreeMap::new();
        outputs.insert("T".to_string(), new_tree.to_value());
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StreamGateComponent;

impl Component for StreamGateComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new(
                "Stream Gate component requires at least two inputs.",
            ));
        }
        let gate = coerce_integer(&inputs[0])? as usize;
        let mut outputs = BTreeMap::new();

        for i in 1..inputs.len() {
            let output_name = (i - 1).to_string();
            if i - 1 == gate {
                outputs.insert(output_name, inputs[i].clone());
            } else {
                outputs.insert(output_name, Value::Null);
            }
        }
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ExplodeTreeComponent;

impl Component for ExplodeTreeComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new(
                "Explode Tree component requires one input.",
            ));
        }
        let tree = Tree::from(&inputs[0]);
        let mut outputs = BTreeMap::new();

        for (i, (path, branch)) in tree.branches.iter().enumerate() {
            let mut branch_tree = BTreeMap::new();
            branch_tree.insert(path.clone(), branch.clone());
            outputs.insert(
                format!("Branch {}", i),
                Tree {
                    branches: branch_tree,
                }
                .to_value(),
            );
        }

        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ConstructPathComponent;

impl Component for ConstructPathComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new(
                "Construct Path component requires one input.",
            ));
        }

        let indices = match &inputs[0] {
            Value::List(l) => l,
            _ => {
                return Err(ComponentError::new(
                    "Construct Path component requires a list of integers.",
                ));
            }
        };

        let path = indices
            .iter()
            .map(|v| coerce_integer(v).map(|i| i.to_string()))
            .collect::<Result<Vec<_>, _>>()?
            .join(";");

        let mut outputs = BTreeMap::new();
        outputs.insert("B".to_string(), Value::Text(format!("{{{}}}", path)));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TreeStatisticsComponent;

impl Component for TreeStatisticsComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new(
                "Tree Statistics component requires one input.",
            ));
        }

        let tree = Tree::from(&inputs[0]);

        let paths: Vec<Value> = tree
            .branches
            .keys()
            .map(|path| {
                let path_str = path
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(";");
                Value::Text(format!("{{{}}}", path_str))
            })
            .collect();

        let lengths: Vec<Value> = tree
            .branches
            .values()
            .map(|branch| Value::Number(branch.len() as f64))
            .collect();

        let count = Value::Number(tree.branches.len() as f64);

        let mut outputs = BTreeMap::new();
        outputs.insert("P".to_string(), Value::List(paths));
        outputs.insert("L".to_string(), Value::List(lengths));
        outputs.insert("C".to_string(), count);
        Ok(outputs)
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
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new(
                "Unflatten Tree component requires two inputs.",
            ));
        }

        let items_to_unflatten = Tree::from(&inputs[0]).flattened_items();
        let guide_tree = Tree::from(&inputs[1]);

        let result_tree = Tree::unflatten_with(&guide_tree, &items_to_unflatten);
        let result_value = result_tree.to_value();

        let mut outputs = BTreeMap::new();
        outputs.insert("T".to_string(), result_value);
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ReplacePathsComponent;

impl Component for ReplacePathsComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 3 {
            return Err(ComponentError::new(
                "Replace Paths component requires three inputs.",
            ));
        }

        let tree = Tree::from(&inputs[0]);
        let search_masks = match &inputs[1] {
            Value::List(l) => l,
            _ => {
                return Err(ComponentError::new(
                    "Replace Paths component requires a list of search masks.",
                ));
            }
        };
        let replace_paths = match &inputs[2] {
            Value::List(l) => l,
            _ => {
                return Err(ComponentError::new(
                    "Replace Paths component requires a list of replace paths.",
                ));
            }
        };

        let result_tree = tree.replace_paths(search_masks, replace_paths)?;
        let result_value = result_tree.to_value();

        let mut outputs = BTreeMap::new();
        outputs.insert("D".to_string(), result_value);
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TreeItemComponent;

impl Component for TreeItemComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 3 {
            return Err(ComponentError::new(
                "Tree Item component requires three inputs.",
            ));
        }

        let tree = Tree::from(&inputs[0]);
        let path_str = coerce_text(&inputs[1])?;
        let path = parse_path(&path_str)?;
        let index = coerce_integer(&inputs[2])?;
        let wrap = if inputs.len() > 3 {
            coerce_boolean(&inputs[3])?
        } else {
            false
        };

        let mut outputs = BTreeMap::new();
        if let Some(branch) = tree.branches.get(&path) {
            if branch.is_empty() {
                outputs.insert("E".to_string(), Value::Null);
            } else {
                let item = if wrap {
                    let wrapped_index = index.rem_euclid(branch.len() as i64) as usize;
                    branch[wrapped_index].clone()
                } else if index >= 0 && (index as usize) < branch.len() {
                    branch[index as usize].clone()
                } else {
                    Value::Null
                };
                outputs.insert("E".to_string(), item);
            }
        } else {
            outputs.insert("E".to_string(), Value::Null);
        }

        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EntwineComponent;

impl Component for EntwineComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        let result_tree = Tree::entwine(inputs);
        let result_value = result_tree.to_value();

        let mut outputs = BTreeMap::new();
        outputs.insert("R".to_string(), result_value);
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SplitTreeComponent;

impl Component for SplitTreeComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new(
                "Split Tree component requires two inputs.",
            ));
        }

        let tree = Tree::from(&inputs[0]);
        let mask = coerce_text(&inputs[1])?;

        let (matching_tree, non_matching_tree) = tree.split_tree(&mask);

        let mut outputs = BTreeMap::new();
        outputs.insert("P".to_string(), matching_tree.to_value());
        outputs.insert("N".to_string(), non_matching_tree.to_value());
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DeconstructPathComponent;

impl Component for DeconstructPathComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new(
                "Deconstruct Path component requires one input.",
            ));
        }

        let path_str = coerce_text(&inputs[0])?;
        let path = parse_path(&path_str)?;

        let segments: Vec<Value> = path
            .into_iter()
            .map(|segment| Value::Number(segment as f64))
            .collect();

        let mut outputs = BTreeMap::new();
        outputs.insert("I".to_string(), Value::List(segments));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NullCheckComponent;

impl Component for NullCheckComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new(
                "Null Check component requires one input.",
            ));
        }

        let tree = Tree::from(&inputs[0]);
        let items = tree.flattened_items();

        let mut null_items = Vec::new();
        let mut valid_items = Vec::new();
        let mut null_indices = Vec::new();
        let mut valid_indices = Vec::new();

        for (i, item) in items.into_iter().enumerate() {
            if matches!(item, Value::Null) {
                null_items.push(item);
                null_indices.push(Value::Number(i as f64));
            } else {
                valid_items.push(item);
                valid_indices.push(Value::Number(i as f64));
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("N".to_string(), Value::List(null_items));
        outputs.insert("V".to_string(), Value::List(valid_items));
        outputs.insert("In".to_string(), Value::List(null_indices));
        outputs.insert("Iv".to_string(), Value::List(valid_indices));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PruneTreeComponent;

impl Component for PruneTreeComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new(
                "Prune Tree component requires two inputs.",
            ));
        }

        let tree = Tree::from(&inputs[0]);
        let n = coerce_integer(&inputs[1])?;

        let pruned_tree = tree.prune(n);
        let result_value = pruned_tree.to_value();

        let mut outputs = BTreeMap::new();
        outputs.insert("T".to_string(), result_value);
        Ok(outputs)
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
    fn test_tree_item() {
        let component = TreeItemComponent;
        let inputs = vec![
            Value::List(vec![
                Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
                Value::List(vec![Value::Number(3.0)]),
            ]),
            Value::Text("{0}".to_string()),
            Value::Number(2.0),
            Value::Boolean(true),
        ];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let expected = Value::Number(1.0);
        assert_eq!(outputs.get("E"), Some(&expected));
    }

    #[test]
    fn test_replace_paths() {
        let component = ReplacePathsComponent;
        let inputs = vec![
            Value::List(vec![
                Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
                Value::List(vec![Value::Number(3.0)]),
            ]),
            Value::List(vec![Value::Text("{0}".to_string())]),
            Value::List(vec![Value::Text("{1}".to_string())]),
        ];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let expected = Value::List(vec![
            Value::List(vec![]),
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
        ]);
        assert_eq!(outputs.get("D"), Some(&expected));
    }

    #[test]
    fn test_unflatten_tree() {
        let component = UnflattenTreeComponent;
        let inputs = vec![
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
            ]),
            Value::List(vec![
                Value::List(vec![Value::Null, Value::Null]),
                Value::List(vec![Value::Null]),
            ]),
        ];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let expected = Value::List(vec![
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(3.0)]),
        ]);
        assert_eq!(outputs.get("T"), Some(&expected));
    }

    #[test]
    fn test_tree_statistics() {
        let component = TreeStatisticsComponent;
        let inputs = vec![Value::List(vec![
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(3.0)]),
        ])];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();

        let expected_paths = Value::List(vec![
            Value::Text("{0}".to_string()),
            Value::Text("{1}".to_string()),
        ]);
        let expected_lengths = Value::List(vec![Value::Number(2.0), Value::Number(1.0)]);
        let expected_count = Value::Number(2.0);

        assert_eq!(outputs.get("P"), Some(&expected_paths));
        assert_eq!(outputs.get("L"), Some(&expected_lengths));
        assert_eq!(outputs.get("C"), Some(&expected_count));
    }

    #[test]
    fn test_construct_path() {
        let component = ConstructPathComponent;
        let inputs = vec![Value::List(vec![
            Value::Number(0.0),
            Value::Number(1.0),
            Value::Number(2.0),
        ])];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let expected = Value::Text("{0;1;2}".to_string());
        assert_eq!(outputs.get("B"), Some(&expected));
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

    #[test]
    fn test_entwine() {
        let component = EntwineComponent;
        let inputs = vec![
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(3.0), Value::Number(4.0)]),
        ];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let expected = Value::List(vec![
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(3.0), Value::Number(4.0)]),
        ]);
        assert_eq!(outputs.get("R"), Some(&expected));
    }

    #[test]
    fn test_split_tree() {
        let component = SplitTreeComponent;
        let inputs = vec![
            Value::List(vec![
                Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
                Value::List(vec![Value::Number(3.0)]),
            ]),
            Value::Text("{0}".to_string()),
        ];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let expected_matching = Value::List(vec![Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
        ])]);
        let expected_non_matching = Value::List(vec![
            Value::List(vec![]),
            Value::List(vec![Value::Number(3.0)]),
        ]);
        assert_eq!(outputs.get("P"), Some(&expected_matching));
        assert_eq!(outputs.get("N"), Some(&expected_non_matching));
    }

    #[test]
    fn test_deconstruct_path() {
        let component = DeconstructPathComponent;
        let inputs = vec![Value::Text("{0;1;2}".to_string())];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let expected = Value::List(vec![
            Value::Number(0.0),
            Value::Number(1.0),
            Value::Number(2.0),
        ]);
        assert_eq!(outputs.get("I"), Some(&expected));
    }

    #[test]
    fn test_null_check() {
        let component = NullCheckComponent;
        let inputs = vec![Value::List(vec![
            Value::Number(1.0),
            Value::Null,
            Value::Text("hello".to_string()),
            Value::Null,
        ])];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();

        let expected_null_items = Value::List(vec![Value::Null, Value::Null]);
        let expected_valid_items =
            Value::List(vec![Value::Number(1.0), Value::Text("hello".to_string())]);
        let expected_null_indices = Value::List(vec![Value::Number(1.0), Value::Number(3.0)]);
        let expected_valid_indices = Value::List(vec![Value::Number(0.0), Value::Number(2.0)]);

        assert_eq!(outputs.get("N"), Some(&expected_null_items));
        assert_eq!(outputs.get("V"), Some(&expected_valid_items));
        assert_eq!(outputs.get("In"), Some(&expected_null_indices));
        assert_eq!(outputs.get("Iv"), Some(&expected_valid_indices));
    }

    #[test]
    fn test_prune_tree_positive() {
        let component = PruneTreeComponent;
        let inputs = vec![
            Value::List(vec![
                Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
                Value::List(vec![Value::Number(3.0)]),
            ]),
            Value::Number(1.0),
        ];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let expected = Value::List(vec![
            Value::List(vec![]),
            Value::List(vec![Value::Number(3.0)]),
        ]);
        assert_eq!(outputs.get("T"), Some(&expected));
    }

    #[test]
    fn test_prune_tree_negative() {
        let component = PruneTreeComponent;
        let inputs = vec![
            Value::List(vec![
                Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
                Value::List(vec![Value::Number(3.0)]),
            ]),
            Value::Number(-2.0),
        ];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let expected = Value::List(vec![Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
        ])]);
        assert_eq!(outputs.get("T"), Some(&expected));
    }
}
