use std::{
    collections::{HashMap, HashSet}, fs::File, io::Read, path::{Path, PathBuf}
};
use toml_edit::{Array, DocumentMut, InlineTable, Item, Table, Value};
use super::view::{BoxView, ColumnView, NavView, RowView, TOMLView, TextView};
use crate::{html::HTMLView, layout::view::{ImageView, ListColumnView, ListRowView, MarkdownView}};

pub fn item_to_string(table: &Table, key: &str, default: &str, value: Option<&InlineTable>) -> String {
    let raw = table
        .get(key)
        .and_then(|item| item.as_str())
        .unwrap_or(default);

    // 중괄호 감싸진 형태인지 확인
    if raw.starts_with('{') && raw.ends_with('}') {
        if let Some(inline) = value {
            let inner_key = &raw[1..raw.len() - 1]; // {} 제거
            if let Some(val) = inline.get(inner_key).and_then(|v| v.as_str()) {
                return val.to_string();
            }
        }
    }

    raw.to_string()
}

pub fn item_to_strings(table: &Table, key: &str) -> Vec<String> {
    table
        .get(key)
        .and_then(|v| v.as_array()) // 배열 아니면 None
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_else(Vec::new)
}

pub fn item_to_bool(table: &Table, key: &str, default: bool, value: Option<&InlineTable>) -> bool {
    let raw = table
        .get(key)
        .and_then(|item| item.as_bool())
        .unwrap_or(default);
    raw
}

pub fn get_items(table: &Table, excluded: Vec<&str>) -> HashMap<String, Item> {
    table
        .iter()
        .filter(|(key, _)| !excluded.contains(key)) // 제외할 key는 건너뛰기
        .map(|(key, value)| {
            (key.to_string(), value.clone())
        })
        .collect()
}

pub fn get_value_in_inlinetable(key: &str, table: Option<InlineTable>) -> Option<Value> {
    if let Some(table) = table {
        for (k, v) in table {
            if k.as_str() == key {
                return Some(v);
            }
        }
    } 
    None
}

pub fn layout_to_tomlview(layout: String, index_path: &Path, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>) -> Result<Box<dyn TOMLView>, String> {
    if layout.ends_with(".toml") {
        Err("".to_string())
    } else {
        get_tomlview_for_key(index_path, layout.as_str(), value, super_view)
    }
}

pub fn get_tomlview_for_key(index_path: &Path, find_key: &str, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>) -> Result<Box<dyn TOMLView>, String> {
    let mut file = File::open(index_path).ok().ok_or("Failed to open the file")?;

    let mut config_str = String::new();
    file.read_to_string(&mut config_str)
        .ok()
        .ok_or("Failed to read the file")?;

    let doc = config_str
        .parse::<DocumentMut>()
        .map_err(|err| format!("{}", err))?;
        
    for (k, v) in doc.as_table() {
        if let Item::Table(table) = v {
            if k == find_key {
                let value = value.or_else(|| {
                    table.get("value").and_then(|item| item.as_inline_table())
                });
                let values = table.get("values").and_then(|item| item.as_array());
                if let Some(values) = values {
                    let values = values.clone();
                    for value in values {
                        if let Value::InlineTable(table) = value {
                            for (k, v) in table {
                                println!("key {}", k);
                            }
                        }
                    }
                }

                let view = table_to_tomlview(index_path, k, table, value, super_view);
                match view {
                    Ok(view) => {
                        return Ok(view);
                    }
                    Err(message) => {
                        println!("message {}", message);
                    }
                }
            }
        }
    }
    Err("".to_string())
}

pub fn table_to_tomlview(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>) -> Result<Box<dyn TOMLView>, String> {
    let shape = item_to_string(table, "shape", "", value);
    println!("table_to_tomlview {:?}", shape);
    match shape.as_str() {
        "nav" => {
            let nav_view = NavView::new(index_path, key, table, value, super_view);
            println!("nav_view {:?}", nav_view.height());
            Ok(Box::new(nav_view))
        }
        "box" => {
            let box_view = BoxView::new(index_path, key, table, value, super_view);
            println!("box_view {:?}", box_view.height());
            Ok(Box::new(box_view))
        }
        "column" => {
            let column_view = ColumnView::new(index_path, key, table, value, super_view, false);
            println!("column_view {:?}", column_view.height());
            Ok(Box::new(column_view))
        }
        "row" => {
            let row_view = RowView::new(index_path, key, table, value, super_view, false);
            println!("row_view {:?}", row_view.height());
            Ok(Box::new(row_view))
        }
        "scroll_column" => {
            let column_view = ColumnView::new(index_path, key, table, value, super_view, true);
            println!("column_view {:?}", column_view.height());
            Ok(Box::new(column_view))
        }
        "scroll_row" => {
            let row_view = RowView::new(index_path, key, table, value, super_view, true);
            println!("row_view {:?}", row_view.height());
            Ok(Box::new(row_view))
        }
        "text" => {
            let text_view = TextView::new(index_path, key, table, value, super_view);
            println!("text_view {:?}", text_view.height());
            Ok(Box::new(text_view))
        }
        "image" => {
            let image_view = ImageView::new(index_path, key, table, value, super_view);
            println!("text_view {:?}", image_view.height());
            Ok(Box::new(image_view))
        }
        "list_column" => {
            let list_column_view = ListColumnView::new(index_path, key, table, value, super_view, true);
            println!("list_column_view {:?}", list_column_view.height());
            Ok(Box::new(list_column_view))
        }
        "list_row" => {
            let list_row_view = ListRowView::new(index_path, key, table, value, super_view, true);
            println!("list_row_view {:?}", list_row_view.height());
            Ok(Box::new(list_row_view))
        }
        "markdown" => {
            let markdown_view = MarkdownView::new(index_path, key, table, value, super_view, true);
            println!("markdown_view {:?}", markdown_view.height());
            Ok(Box::new(markdown_view))
        }
        "grid" => {
            Err("".to_string())
        }
        _ => Err("".to_string()),
    }
}
