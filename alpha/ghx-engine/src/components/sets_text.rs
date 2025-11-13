//! Text manipulation components for the GHX engine.

use std::collections::BTreeMap;
use crate::graph::node::MetaMap;
use crate::graph::value::Value;
use super::{Component, ComponentError, ComponentResult};
use regex::Regex;
use wildmatch::WildMatch;

// --- Local Coercion Helpers ---

fn coerce_string(value: &Value) -> Result<String, ComponentError> {
    match value {
        Value::Text(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Boolean(b) => Ok(b.to_string()),
        Value::List(l) if l.len() == 1 => coerce_string(&l[0]),
        other => Err(ComponentError::new(format!(
            "Expected a string, got {}",
            other.kind()
        ))),
    }
}

fn coerce_list(value: &Value) -> Result<Vec<Value>, ComponentError> {
    match value {
        Value::List(l) => Ok(l.clone()),
        _ => Err(ComponentError::new(format!(
            "Expected a list, got {}",
            value.kind()
        ))),
    }
}


// --- Component Implementations ---

#[derive(Debug, Default, Clone, Copy)]
struct ConcatenateComponent;
impl Component for ConcatenateComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new("Expected 2 inputs"));
        }
        let a = coerce_string(&inputs[0])?;
        let b = coerce_string(&inputs[1])?;
        let mut outputs = BTreeMap::new();
        outputs.insert("R".to_owned(), Value::Text(format!("{}{}", a, b)));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct TextLengthComponent;
impl Component for TextLengthComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Expected 1 input"));
        }
        let text = coerce_string(&inputs[0])?;
        let mut outputs = BTreeMap::new();
        outputs.insert("L".to_owned(), Value::Number(text.len() as f32));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct TextSplitComponent;
impl Component for TextSplitComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new("Expected 2 inputs"));
        }
        let text = coerce_string(&inputs[0])?;
        let separators = coerce_string(&inputs[1])?;
        let mut outputs = BTreeMap::new();
        let fragments = text
            .split(&separators.chars().collect::<Vec<char>>()[..])
            .map(|s| Value::Text(s.to_owned()))
            .collect();
        outputs.insert("R".to_owned(), Value::List(fragments));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct TextFragmentComponent;
impl Component for TextFragmentComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new("Expected at least 2 inputs"));
        }
        let text = coerce_string(&inputs[0])?;
        let start = coerce_string(&inputs[1])?.parse::<usize>().unwrap_or(0);
        let count = if inputs.len() > 2 {
            coerce_string(&inputs[2])?.parse::<usize>().ok()
        } else {
            None
        };
        let fragment = if let Some(n) = count {
            text.chars().skip(start).take(n).collect()
        } else {
            text.chars().skip(start).collect()
        };
        let mut outputs = BTreeMap::new();
        outputs.insert("F".to_owned(), Value::Text(fragment));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct TextJoinComponent;
impl Component for TextJoinComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new("Expected 2 inputs"));
        }
        let list = coerce_list(&inputs[0])?;
        let joiner = coerce_string(&inputs[1])?;
        let strings: Vec<String> = list.iter().map(|v| coerce_string(v).unwrap_or_default()).collect();
        let mut outputs = BTreeMap::new();
        outputs.insert("R".to_owned(), Value::Text(strings.join(&joiner)));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct CharactersComponent;
impl Component for CharactersComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Expected 1 input"));
        }
        let text = coerce_string(&inputs[0])?;
        let chars = text.chars().map(|c| Value::Text(c.to_string())).collect();
        let unicode = text.chars().map(|c| Value::Number(c as u32 as f32)).collect();
        let mut outputs = BTreeMap::new();
        outputs.insert("C".to_owned(), Value::List(chars));
        outputs.insert("U".to_owned(), Value::List(unicode));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct TextCaseComponent;
impl Component for TextCaseComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Expected 1 input"));
        }
        let text = coerce_string(&inputs[0])?;
        let mut outputs = BTreeMap::new();
        outputs.insert("U".to_owned(), Value::Text(text.to_uppercase()));
        outputs.insert("L".to_owned(), Value::Text(text.to_lowercase()));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct TextTrimComponent;
impl Component for TextTrimComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Expected 1 input"));
        }
        let text = coerce_string(&inputs[0])?;
        let start = if inputs.len() > 1 {
            coerce_string(&inputs[1])?.parse::<bool>().unwrap_or(true)
        } else {
            true
        };
        let end = if inputs.len() > 2 {
            coerce_string(&inputs[2])?.parse::<bool>().unwrap_or(true)
        } else {
            true
        };

        let mut result = text.as_str();
        if start {
            result = result.trim_start();
        }
        if end {
            result = result.trim_end();
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("R".to_owned(), Value::Text(result.to_owned()));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct ReplaceTextComponent;
impl Component for ReplaceTextComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new("Expected at least 2 inputs"));
        }
        let text = coerce_string(&inputs[0])?;
        let find = coerce_string(&inputs[1])?;
        let replace = if inputs.len() > 2 {
            coerce_string(&inputs[2])?
        } else {
            String::new()
        };
        let mut outputs = BTreeMap::new();
        outputs.insert("R".to_owned(), Value::Text(text.replace(&find, &replace)));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct TextDistanceComponent;
impl Component for TextDistanceComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new("Expected 2 inputs"));
        }
        let a = coerce_string(&inputs[0])?;
        let b = coerce_string(&inputs[1])?;
        let case_sensitive = if inputs.len() > 2 {
            coerce_string(&inputs[2])?.parse::<bool>().unwrap_or(true)
        } else {
            true
        };

        let distance = if case_sensitive {
            levenshtein::levenshtein(&a, &b)
        } else {
            levenshtein::levenshtein(&a.to_lowercase(), &b.to_lowercase())
        };

        let mut outputs = BTreeMap::new();
        outputs.insert("D".to_owned(), Value::Number(distance as f32));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct SortTextComponent;
impl Component for SortTextComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Expected at least 1 input"));
        }
        let keys = coerce_list(&inputs[0])?;
        let mut keys: Vec<String> = keys.iter().map(|v| coerce_string(v).unwrap_or_default()).collect();

        let mut values = if inputs.len() > 1 {
            coerce_list(&inputs[1])?
        } else {
            vec![]
        };

        let mut pairs: Vec<_> = keys.clone().into_iter().zip(values.clone().into_iter()).collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));

        keys = pairs.iter().map(|(k, _)| k.clone()).collect();
        values = pairs.iter().map(|(_, v)| v.clone()).collect();

        let mut outputs = BTreeMap::new();
        outputs.insert("K".to_owned(), Value::List(keys.into_iter().map(Value::Text).collect()));
        outputs.insert("V".to_owned(), Value::List(values));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct MatchTextComponent;
impl Component for MatchTextComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Expected at least 1 input"));
        }
        let text = coerce_string(&inputs[0])?;
        let pattern = if inputs.len() > 1 {
            coerce_string(&inputs[1]).ok()
        } else {
            None
        };
        let regex_str = if inputs.len() > 2 {
            coerce_string(&inputs[2]).ok()
        } else {
            None
        };
        let case_sensitive = if inputs.len() > 3 {
            coerce_string(&inputs[3])?.parse::<bool>().unwrap_or(true)
        } else {
            true
        };

        let mut matches = true;

        if let Some(p) = pattern {
            if case_sensitive {
                matches &= WildMatch::new(&p).matches(&text);
            } else {
                matches &= WildMatch::new(&p.to_lowercase()).matches(&text.to_lowercase());
            }
        }

        if let Some(r) = regex_str {
            let re = if case_sensitive {
                Regex::new(&r)
            } else {
                Regex::new(&format!("(?i){}", r))
            };
            if let Ok(re) = re {
                matches &= re.is_match(&text);
            } else {
                return Err(ComponentError::new("Invalid RegEx pattern"));
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("M".to_owned(), Value::Boolean(matches));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct FormatComponent;
impl Component for FormatComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Expected at least 1 input"));
        }
        let format_str = coerce_string(&inputs[0])?;
        let mut result = format_str;

        for (i, input) in inputs.iter().skip(1).enumerate() {
            let placeholder = format!("{{{}}}", i);
            let value_str = coerce_string(input).unwrap_or_default();
            result = result.replace(&placeholder, &value_str);
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("T".to_owned(), Value::Text(result));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct TextOnSurfaceComponent;
impl Component for TextOnSurfaceComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new(
            "Component 'Text On Surface' is not yet implemented.",
        ))
    }
}

// --- Registration ---

#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Concatenate,
    TextLength,
    TextSplit,
    TextFragment,
    TextJoin,
    Characters,
    TextCase,
    TextTrim,
    ReplaceText,
    TextDistance,
    SortText,
    MatchText,
    Format,
    TextOnSurface,
}

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], meta: &crate::graph::node::MetaMap) -> ComponentResult {
        match self {
            Self::Concatenate => ConcatenateComponent.evaluate(inputs, meta),
            Self::TextLength => TextLengthComponent.evaluate(inputs, meta),
            Self::TextSplit => TextSplitComponent.evaluate(inputs, meta),
            Self::TextFragment => TextFragmentComponent.evaluate(inputs, meta),
            Self::TextJoin => TextJoinComponent.evaluate(inputs, meta),
            Self::Characters => CharactersComponent.evaluate(inputs, meta),
            Self::TextCase => TextCaseComponent.evaluate(inputs, meta),
            Self::TextTrim => TextTrimComponent.evaluate(inputs, meta),
            Self::ReplaceText => ReplaceTextComponent.evaluate(inputs, meta),
            Self::TextDistance => TextDistanceComponent.evaluate(inputs, meta),
            Self::SortText => SortTextComponent.evaluate(inputs, meta),
            Self::MatchText => MatchTextComponent.evaluate(inputs, meta),
            Self::Format => FormatComponent.evaluate(inputs, meta),
            Self::TextOnSurface => TextOnSurfaceComponent.evaluate(inputs, meta),
        }
    }
}

impl ComponentKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Concatenate => "Concatenate",
            Self::TextLength => "Text Length",
            Self::TextSplit => "Text Split",
            Self::TextFragment => "Text Fragment",
            Self::TextJoin => "Text Join",
            Self::Characters => "Characters",
            Self::TextCase => "Text Case",
            Self::TextTrim => "Text Trim",
            Self::ReplaceText => "Replace Text",
            Self::TextDistance => "Text Distance",
            Self::SortText => "Sort Text",
            Self::MatchText => "Match Text",
            Self::Format => "Format",
            Self::TextOnSurface => "Text On Surface",
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
        guids: &[
            "01cbd6e3-ccbe-4c24-baeb-46e10553e18b",
            "2013e425-8713-42e2-a661-b57e78840337",
        ],
        names: &["Concatenate", "Concat"],
        kind: ComponentKind::Concatenate,
    },
    Registration {
        guids: &["dca05f6f-e3d9-42e3-b3bb-eb20363fb335"],
        names: &["Text Length", "Len"],
        kind: ComponentKind::TextLength,
    },
    Registration {
        guids: &["04887d01-504c-480e-b2a2-01ea19cc5922"],
        names: &["Text Split", "Split"],
        kind: ComponentKind::TextSplit,
    },
    Registration {
        guids: &["07e0811f-034a-4504-bca0-2d03b2c46217"],
        names: &["Text Fragment", "Fragment"],
        kind: ComponentKind::TextFragment,
    },
    Registration {
        guids: &["1274d51a-81e6-4ccf-ad1f-0edf4c769cac"],
        names: &["Text Join", "Join"],
        kind: ComponentKind::TextJoin,
    },
    Registration {
        guids: &["86503240-d884-43f9-9323-efe30488a6e1"],
        names: &["Characters", "Chars"],
        kind: ComponentKind::Characters,
    },
    Registration {
        guids: &[
            "b1991128-8bf1-4dea-8497-4b7188a64e9d",
            "bdd2a14a-1302-4152-a484-7198716d1a11",
        ],
        names: &["Text Case", "Case"],
        kind: ComponentKind::TextCase,
    },
    Registration {
        guids: &["e4cb7168-5e32-4c54-b425-5a31c6fd685a"],
        names: &["Text Trim", "Trim"],
        kind: ComponentKind::TextTrim,
    },
    Registration {
        guids: &["4df8df00-3635-45bd-95e6-f9206296c110"],
        names: &["Replace Text", "Rep"],
        kind: ComponentKind::ReplaceText,
    },
    Registration {
        guids: &["f7608c4d-836c-4adf-9d1f-3b04e6a2647d"],
        names: &["Text Distance", "TDist"],
        kind: ComponentKind::TextDistance,
    },
    Registration {
        guids: &[
            "1ff80a00-1b1d-4fb3-926a-0c246261fc55",
            "cec16c67-7b8b-41f7-a5a5-f675177e524b",
        ],
        names: &["Sort Text", "TSort"],
        kind: ComponentKind::SortText,
    },
    Registration {
        guids: &["3756c55f-95c3-442c-a027-6b3ab0455a94"],
        names: &["Match Text", "TMatch"],
        kind: ComponentKind::MatchText,
    },
    Registration {
        guids: &[
            "758d91a0-4aec-47f8-9671-16739a8a2c5d",
            "c8203c3c-6bcd-4f8c-a906-befd92ebf0cb",
        ],
        names: &["Format"],
        kind: ComponentKind::Format,
    },
    Registration {
        guids: &["28504f1f-a8d9-40c8-b8aa-529413456258"],
        names: &["Text On Surface", "TextSrf"],
        kind: ComponentKind::TextOnSurface,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn test_concatenate() {
        let component = ConcatenateComponent;
        let inputs = &[Value::Text("a".to_string()), Value::Text("b".to_string())];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("R"), Some(&Value::Text("ab".to_string())));
    }

    #[test]
    fn test_text_length() {
        let component = TextLengthComponent;
        let inputs = &[Value::Text("hello".to_string())];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("L"), Some(&Value::Number(5.0)));
    }

    #[test]
    fn test_text_split() {
        let component = TextSplitComponent;
        let inputs = &[Value::Text("a,b,c".to_string()), Value::Text(",".to_string())];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let expected = Value::List(vec![
            Value::Text("a".to_string()),
            Value::Text("b".to_string()),
            Value::Text("c".to_string()),
        ]);
        assert_eq!(outputs.get("R"), Some(&expected));
    }

    #[test]
    fn test_text_fragment() {
        let component = TextFragmentComponent;
        let inputs = &[
            Value::Text("hello world".to_string()),
            Value::Text("6".to_string()),
            Value::Text("5".to_string()),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("F"), Some(&Value::Text("world".to_string())));
    }

    #[test]
    fn test_text_join() {
        let component = TextJoinComponent;
        let inputs = &[
            Value::List(vec![
                Value::Text("a".to_string()),
                Value::Text("b".to_string()),
                Value::Text("c".to_string()),
            ]),
            Value::Text(",".to_string()),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("R"), Some(&Value::Text("a,b,c".to_string())));
    }

    #[test]
    fn test_characters() {
        let component = CharactersComponent;
        let inputs = &[Value::Text("abc".to_string())];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let expected_c = Value::List(vec![
            Value::Text("a".to_string()),
            Value::Text("b".to_string()),
            Value::Text("c".to_string()),
        ]);
        let expected_u = Value::List(vec![
            Value::Number(97.0),
            Value::Number(98.0),
            Value::Number(99.0),
        ]);
        assert_eq!(outputs.get("C"), Some(&expected_c));
        assert_eq!(outputs.get("U"), Some(&expected_u));
    }

    #[test]
    fn test_text_case() {
        let component = TextCaseComponent;
        let inputs = &[Value::Text("HeLLo".to_string())];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("U"), Some(&Value::Text("HELLO".to_string())));
        assert_eq!(outputs.get("L"), Some(&Value::Text("hello".to_string())));
    }

    #[test]
    fn test_text_trim() {
        let component = TextTrimComponent;
        let inputs = &[Value::Text("  hello  ".to_string())];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("R"), Some(&Value::Text("hello".to_string())));
    }

    #[test]
    fn test_replace_text() {
        let component = ReplaceTextComponent;
        let inputs = &[
            Value::Text("hello world".to_string()),
            Value::Text("world".to_string()),
            Value::Text("rust".to_string()),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("R"), Some(&Value::Text("hello rust".to_string())));
    }

    #[test]
    fn test_text_distance() {
        let component = TextDistanceComponent;
        let inputs = &[Value::Text("kitten".to_string()), Value::Text("sitting".to_string())];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("D"), Some(&Value::Number(3.0)));
    }

    #[test]
    fn test_sort_text() {
        let component = SortTextComponent;
        let inputs = &[
            Value::List(vec![
                Value::Text("c".to_string()),
                Value::Text("a".to_string()),
                Value::Text("b".to_string()),
            ]),
            Value::List(vec![
                Value::Number(3.0),
                Value::Number(1.0),
                Value::Number(2.0),
            ]),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        let expected_k = Value::List(vec![
            Value::Text("a".to_string()),
            Value::Text("b".to_string()),
            Value::Text("c".to_string()),
        ]);
        let expected_v = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        assert_eq!(outputs.get("K"), Some(&expected_k));
        assert_eq!(outputs.get("V"), Some(&expected_v));
    }

    #[test]
    fn test_match_text_regex() {
        let component = MatchTextComponent;
        let inputs = &[
            Value::Text("hello".to_string()),
            Value::Null,
            Value::Text("^h".to_string()),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("M"), Some(&Value::Boolean(true)));
    }

    #[test]
    fn test_match_text_wildcard() {
        let component = MatchTextComponent;
        let inputs = &[
            Value::Text("hello".to_string()),
            Value::Text("h*o".to_string()),
            Value::Null,
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("M"), Some(&Value::Boolean(true)));
    }

    #[test]
    fn test_format() {
        let component = FormatComponent;
        let inputs = &[
            Value::Text("Hello, {0}! Welcome to {1}.".to_string()),
            Value::Text("world".to_string()),
            Value::Text("Rust".to_string()),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("T"), Some(&Value::Text("Hello, world! Welcome to Rust.".to_string())));
    }
}
