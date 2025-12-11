//! Ghx-engine componenten voor het `Sets` -> `Sequence` gedeelte.
use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::{Domain, Value};
use rand::Rng;
use rand::seq::SliceRandom;

use super::{
    Component, ComponentError, ComponentResult,
    coerce::{coerce_boolean, coerce_integer, coerce_number},
};

pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["9445ca40-cc73-4861-a455-146308676855"],
        names: &["Range"],
        nickname: Some("Range"),
        kind: ComponentKind::Range,
    },
    Registration {
        guids: &["e64c5fb1-845c-4ab1-8911-5f338516ba67"],
        names: &["Series"],
        nickname: Some("Series"),
        kind: ComponentKind::Series,
    },
    Registration {
        guids: &["fe99f302-3d0d-4389-8494-bd53f7935a02"],
        names: &["Fibonacci"],
        nickname: Some("Fib"),
        kind: ComponentKind::Fibonacci,
    },
    Registration {
        guids: &["008e9a6f-478a-4813-8c8a-546273bc3a6b"],
        names: &["Cull Pattern"],
        nickname: Some("Cull"),
        kind: ComponentKind::CullPattern,
    },
    Registration {
        guids: &[
            "501aecbb-c191-4d13-83d6-7ee32445ac50",
            "6568e019-f59c-4984-84d6-96bd5bfbe9e7",
        ],
        names: &["Cull Index"],
        nickname: Some("Cull i"),
        kind: ComponentKind::CullIndex,
    },
    Registration {
        guids: &["932b9817-fcc6-4ac3-b9fd-c0e8eeadc53f"],
        names: &["Cull Nth"],
        nickname: Some("CullN"),
        kind: ComponentKind::CullNth,
    },
    Registration {
        guids: &[
            "2ab17f9a-d852-4405-80e1-938c5e57e78d",
            "b7e4e0ef-a01d-48c4-93be-2a12d4417e22",
        ],
        names: &["Random"],
        nickname: Some("Random"),
        kind: ComponentKind::Random,
    },
    Registration {
        guids: &["455925fd-23ff-4e57-a0e7-913a4165e659"],
        names: &["Random Reduce"],
        nickname: Some("Reduce"),
        kind: ComponentKind::RandomReduce,
    },
    Registration {
        guids: &["5fa4e736-0d82-4af0-97fb-30a79f4cbf41"],
        names: &["Stack Data"],
        nickname: Some("Stack"),
        kind: ComponentKind::StackData,
    },
    Registration {
        guids: &["c40dc145-9e36-4a69-ac1a-6d825c654993"],
        names: &["Repeat Data"],
        nickname: Some("Repeat"),
        kind: ComponentKind::RepeatData,
    },
    Registration {
        guids: &["dd8134c0-109b-4012-92be-51d843edfff7"],
        names: &["Duplicate Data"],
        nickname: Some("Dup"),
        kind: ComponentKind::DuplicateData,
    },
    Registration {
        guids: &["f02a20f6-bb49-4e3d-b155-8ed5d3c6b000"],
        names: &["Jitter"],
        nickname: Some("Jitter"),
        kind: ComponentKind::Jitter,
    },
];

#[derive(Debug, Clone, Copy)]
pub struct Registration {
    guids: &'static [&'static str],
    names: &'static [&'static str],
    nickname: Option<&'static str>,
    kind: ComponentKind,
}

impl Registration {
    pub fn guids(&self) -> &'static [&'static str] {
        self.guids
    }

    pub fn names(&self) -> &'static [&'static str] {
        self.names
    }

    pub fn nickname(&self) -> Option<&'static str> {
        self.nickname
    }

    pub fn kind(&self) -> ComponentKind {
        self.kind
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Range,
    Series,
    Fibonacci,
    CullPattern,
    CullIndex,
    CullNth,
    Random,
    RandomReduce,
    StackData,
    RepeatData,
    DuplicateData,
    Jitter,
}

impl ComponentKind {
    pub fn evaluate(self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::Range => Range.evaluate(inputs, meta),
            Self::Series => Series.evaluate(inputs, meta),
            Self::Fibonacci => Fibonacci.evaluate(inputs, meta),
            Self::CullPattern => CullPattern.evaluate(inputs, meta),
            Self::CullIndex => CullIndex.evaluate(inputs, meta),
            Self::CullNth => CullNth.evaluate(inputs, meta),
            Self::Random => Random.evaluate(inputs, meta),
            Self::RandomReduce => RandomReduce.evaluate(inputs, meta),
            Self::StackData => StackData.evaluate(inputs, meta),
            Self::RepeatData => RepeatData.evaluate(inputs, meta),
            Self::DuplicateData => DuplicateData.evaluate(inputs, meta),
            Self::Jitter => Jitter.evaluate(inputs, meta),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Range => "Range",
            Self::Series => "Series",
            Self::Fibonacci => "Fibonacci",
            Self::CullPattern => "Cull Pattern",
            Self::CullIndex => "Cull Index",
            Self::CullNth => "Cull Nth",
            Self::Random => "Random",
            Self::RandomReduce => "Random Reduce",
            Self::StackData => "Stack Data",
            Self::RepeatData => "Repeat Data",
            Self::DuplicateData => "Duplicate Data",
            Self::Jitter => "Jitter",
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Range;
impl Component for Range {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new("Expected 2 inputs"));
        }
        let domain = match &inputs[0] {
            Value::Domain(Domain::One(domain)) => domain,
            _ => return Err(ComponentError::new("Expected a 1D domain")),
        };
        let steps = coerce_integer(&inputs[1])? as usize;
        let mut numbers = Vec::with_capacity(steps + 1);
        let step_size = (domain.end - domain.start) / steps as f64;
        for i in 0..=steps {
            numbers.push(Value::Number(domain.start + i as f64 * step_size));
        }
        let mut outputs = BTreeMap::new();
        outputs.insert("R".to_owned(), Value::List(numbers));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Series;
impl Component for Series {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 3 {
            return Err(ComponentError::new("Expected 3 inputs"));
        }
        let start = coerce_number(&inputs[0], None)?;
        let step = coerce_number(&inputs[1], None)?;
        let count = coerce_integer(&inputs[2])? as usize;
        let mut numbers = Vec::with_capacity(count);
        for i in 0..count {
            numbers.push(Value::Number(start + i as f64 * step));
        }
        let mut outputs = BTreeMap::new();
        outputs.insert("S".to_owned(), Value::List(numbers));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Fibonacci;
impl Component for Fibonacci {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 3 {
            return Err(ComponentError::new("Expected 3 inputs"));
        }
        let a = coerce_number(&inputs[0], None)?;
        let b = coerce_number(&inputs[1], None)?;
        let n = coerce_integer(&inputs[2])? as usize;

        let mut sequence = Vec::with_capacity(n);
        if n >= 1 {
            sequence.push(Value::Number(a));
        }
        if n >= 2 {
            sequence.push(Value::Number(b));
        }

        let mut prev = a;
        let mut current = b;

        for _ in 2..n {
            let next = prev + current;
            sequence.push(Value::Number(next));
            prev = current;
            current = next;
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("S".to_owned(), Value::List(sequence));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct CullPattern;
impl Component for CullPattern {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new("Expected 2 inputs"));
        }
        let list = match &inputs[0] {
            Value::List(list) => list,
            _ => return Err(ComponentError::new("Expected a list for the first input")),
        };
        let pattern = match &inputs[1] {
            Value::List(list) => list,
            _ => return Err(ComponentError::new("Expected a list for the second input")),
        };
        if pattern.is_empty() {
            return Err(ComponentError::new("Pattern cannot be empty"));
        }

        let bool_pattern: Vec<bool> = pattern
            .iter()
            .map(coerce_boolean)
            .collect::<Result<_, _>>()?;

        let culled_list: Vec<Value> = list
            .iter()
            .zip(bool_pattern.iter().cycle())
            .filter(|&(_, &should_cull)| !should_cull)
            .map(|(val, _)| val.clone())
            .collect();

        let mut outputs = BTreeMap::new();
        outputs.insert("L".to_owned(), Value::List(culled_list));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct CullIndex;
impl Component for CullIndex {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        let (list, indices, wrap) = match inputs.len() {
            2 => {
                let list = match &inputs[0] {
                    Value::List(l) => l,
                    _ => return Err(ComponentError::new("Input L must be a list.")),
                };
                let indices = match &inputs[1] {
                    Value::List(i) => i
                        .iter()
                        .map(coerce_integer)
                        .collect::<Result<Vec<_>, _>>()?,
                    _ => return Err(ComponentError::new("Input I must be a list.")),
                };
                (list, indices, false)
            }
            3 => {
                let list = match &inputs[0] {
                    Value::List(l) => l,
                    _ => return Err(ComponentError::new("Input L must be a list.")),
                };
                let indices = match &inputs[1] {
                    Value::List(i) => i
                        .iter()
                        .map(coerce_integer)
                        .collect::<Result<Vec<_>, _>>()?,
                    _ => return Err(ComponentError::new("Input I must be a list.")),
                };
                let wrap = coerce_boolean(&inputs[2])?;
                (list, indices, wrap)
            }
            _ => return Err(ComponentError::new("Expected 2 or 3 inputs.")),
        };

        if list.is_empty() {
            let mut outputs = BTreeMap::new();
            outputs.insert("L".to_owned(), Value::List(vec![]));
            return Ok(outputs);
        }

        let mut indices_to_remove = std::collections::HashSet::new();
        for index in indices {
            let mut i = index;
            if wrap {
                i %= list.len() as i64;
                if i < 0 {
                    i += list.len() as i64;
                }
            }
            if i >= 0 && i < list.len() as i64 {
                indices_to_remove.insert(i as usize);
            }
        }

        let mut culled_list = Vec::new();
        for (i, item) in list.iter().enumerate() {
            if !indices_to_remove.contains(&i) {
                culled_list.push(item.clone());
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("L".to_owned(), Value::List(culled_list));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct CullNth;
impl Component for CullNth {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new("Expected 2 inputs"));
        }

        let list = match &inputs[0] {
            Value::List(l) => l,
            _ => return Err(ComponentError::new("Input L must be a list.")),
        };

        let n = coerce_integer(&inputs[1])? as usize;

        if n == 0 {
            return Err(ComponentError::new("N cannot be zero."));
        }

        let culled_list = list
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                if (i + 1) % n == 0 {
                    None
                } else {
                    Some(item.clone())
                }
            })
            .collect();

        let mut outputs = BTreeMap::new();
        outputs.insert("L".to_owned(), Value::List(culled_list));
        Ok(outputs)
    }
}
#[derive(Debug, Default, Clone, Copy)]
struct Random;
impl Component for Random {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        let (range, number, seed, integers) = match inputs.len() {
            3 => {
                let range = match &inputs[0] {
                    Value::Domain(Domain::One(d)) => d,
                    _ => return Err(ComponentError::new("Input R must be a 1D domain.")),
                };
                let number = coerce_integer(&inputs[1])? as usize;
                let seed = coerce_integer(&inputs[2])?;
                (range.clone(), number, seed, false)
            }
            4 => {
                let range = match &inputs[0] {
                    Value::Domain(Domain::One(d)) => d,
                    _ => return Err(ComponentError::new("Input R must be a 1D domain.")),
                };
                let number = coerce_integer(&inputs[1])? as usize;
                let seed = coerce_integer(&inputs[2])?;
                let integers = coerce_boolean(&inputs[3])?;
                (range.clone(), number, seed, integers)
            }
            _ => return Err(ComponentError::new("Expected 3 or 4 inputs.")),
        };

        let mut rng: rand::prelude::StdRng = rand::SeedableRng::seed_from_u64(seed as u64);
        let mut random_numbers = Vec::with_capacity(number);

        for _ in 0..number {
            let value = if integers {
                let val = rng.gen_range(range.start.round() as i64..=range.end.round() as i64);
                Value::Number(val as f64)
            } else {
                let val = rng.gen_range(range.start..=range.end);
                Value::Number(val)
            };
            random_numbers.push(value);
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("R".to_owned(), Value::List(random_numbers));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct RandomReduce;
impl Component for RandomReduce {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 3 {
            return Err(ComponentError::new("Expected 3 inputs"));
        }
        let list = match &inputs[0] {
            Value::List(l) => l,
            _ => return Err(ComponentError::new("Input L must be a list.")),
        };
        let reduction = coerce_integer(&inputs[1])? as usize;
        let seed = coerce_integer(&inputs[2])?;

        if reduction >= list.len() {
            let mut outputs = BTreeMap::new();
            outputs.insert("L".to_owned(), Value::List(vec![]));
            return Ok(outputs);
        }

        let mut rng: rand::prelude::StdRng = rand::SeedableRng::seed_from_u64(seed as u64);
        let mut indices: Vec<usize> = (0..list.len()).collect();
        indices.shuffle(&mut rng);
        let indices_to_keep: std::collections::HashSet<usize> =
            indices[reduction..].iter().cloned().collect();

        let mut reduced_list = Vec::with_capacity(list.len() - reduction);
        for (i, item) in list.iter().enumerate() {
            if indices_to_keep.contains(&i) {
                reduced_list.push(item.clone());
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("L".to_owned(), Value::List(reduced_list));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct StackData;
impl Component for StackData {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new("Expected 2 inputs"));
        }
        let data = match &inputs[0] {
            Value::List(d) => d,
            _ => return Err(ComponentError::new("Input D must be a list.")),
        };
        let stack = match &inputs[1] {
            Value::List(s) => s
                .iter()
                .map(coerce_integer)
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err(ComponentError::new("Input S must be a list.")),
        };

        let mut stacked_data = Vec::new();
        for (item, &count) in data.iter().zip(stack.iter().cycle()) {
            for _ in 0..count {
                stacked_data.push(item.clone());
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("D".to_owned(), Value::List(stacked_data));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct RepeatData;
impl Component for RepeatData {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new("Expected 2 inputs"));
        }
        let data = match &inputs[0] {
            Value::List(d) => d,
            _ => return Err(ComponentError::new("Input D must be a list.")),
        };
        let length = coerce_integer(&inputs[1])? as usize;

        if data.is_empty() {
            let mut outputs = BTreeMap::new();
            outputs.insert("D".to_owned(), Value::List(vec![]));
            return Ok(outputs);
        }

        let repeated_data: Vec<Value> = data.iter().cycle().take(length).cloned().collect();

        let mut outputs = BTreeMap::new();
        outputs.insert("D".to_owned(), Value::List(repeated_data));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct DuplicateData;
impl Component for DuplicateData {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new("Expected 2 inputs"));
        }
        let data = match &inputs[0] {
            Value::List(d) => d,
            _ => return Err(ComponentError::new("Input D must be a list.")),
        };
        let number = coerce_integer(&inputs[1])? as usize;
        let order = if inputs.len() > 2 {
            coerce_boolean(&inputs[2])?
        } else {
            false
        };

        let mut duplicated_data = Vec::with_capacity(data.len() * number);
        if order {
            for item in data {
                for _ in 0..number {
                    duplicated_data.push(item.clone());
                }
            }
        } else {
            for _ in 0..number {
                duplicated_data.extend_from_slice(data);
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("D".to_owned(), Value::List(duplicated_data));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Jitter;
impl Component for Jitter {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 3 {
            return Err(ComponentError::new("Expected 3 inputs"));
        }
        let list = match &inputs[0] {
            Value::List(l) => l,
            _ => return Err(ComponentError::new("Input L must be a list.")),
        };
        let jitter = coerce_number(&inputs[1], None)?;
        let seed = coerce_integer(&inputs[2])?;

        let mut rng: rand::prelude::StdRng = rand::SeedableRng::seed_from_u64(seed as u64);
        let mut shuffled_list = list.clone();
        if jitter > 0.0 {
            let k = (list.len() as f64 * jitter).round() as usize;
            // Partial Fisher-Yates shuffle
            for i in 0..k.min(list.len()) {
                let j = rng.gen_range(i..list.len());
                shuffled_list.swap(i, j);
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("V".to_owned(), Value::List(shuffled_list.clone()));

        let mut indices_map: Vec<Value> = Vec::with_capacity(list.len());
        for item in list.iter() {
            if let Some(pos) = shuffled_list.iter().position(|r| r == item) {
                indices_map.push(Value::Number(pos as f64));
            }
        }
        outputs.insert("I".to_owned(), Value::List(indices_map));

        Ok(outputs)
    }
}
