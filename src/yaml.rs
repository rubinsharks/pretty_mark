use std::collections::HashMap;
use serde_yaml::Value as YamlValue;
use toml_edit::{value, InlineTable, Value as TomlEditValue};

pub fn yaml_hashmap_to_inline_table(map: &HashMap<String, YamlValue>) -> InlineTable {
    let mut table = InlineTable::new();

    for (k, v) in map {
        if let Some(value) = yaml_to_toml_edit_value(v) {
            table.insert(k.clone(), value);
        }
    }

    table
}

fn yaml_to_toml_edit_value(yaml: &YamlValue) -> Option<TomlEditValue> {
    match yaml {
        // YamlValue::Mapping(m) => {
        //     let mut inline = InlineTable::new();
        //     for (k, v) in m {
        //         if let YamlValue::String(key) = k {
        //             inline.insert(key.clone(), yaml_to_toml_edit_value(v));
        //         }
        //     }
        //     TomlEditValue::InlineTable(inline)
        // },
        // YamlValue::Sequence(seq) => {
        //     let arr: Vec<TomlEditValue> = seq.iter().map(yaml_to_toml_edit_value).collect();
        //     TomlEditValue::Array(arr.into())
        // },
        YamlValue::String(s) => Some(TomlEditValue::from(s)),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(TomlEditValue::from(i))
            } else if let Some(f) = n.as_f64() {
                Some(TomlEditValue::from(f))
            } else {
                Some(TomlEditValue::from(n.to_string()))
            }
        },
        YamlValue::Bool(b) => Some(TomlEditValue::from(*b)),
        YamlValue::Null => None,
        _ => None,
    }
}