use std::{
    collections::{HashMap, HashSet}, fs::File, io::Read, path::{Path, PathBuf}
};
use maplit::hashmap;
use toml_edit::{Array, DocumentMut, InlineTable, Item, Table, Value};
use super::view::{BoxView, ColumnView, NavView, RowView, TOMLView, TextView};
use crate::{html::HTMLView, layout::view::{EmbedView, GridView, ImageView, ListColumnView, ListRowView, MarkdownListColumnView, MarkdownListRowView, MarkdownView}};

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

pub fn layout_to_tomlview(view: &dyn TOMLView, layout: String, layout_tables: HashMap<String, Table>, value: Option<&InlineTable>) -> Result<Box<dyn TOMLView>, String> {
    if let Some(layout_view) = layout_tables.iter()
        .filter_map(|(key, x)| table_to_tomlview(view.index_path().as_path(), key, x, value, Some(view), layout_tables.clone()).ok())
        .filter(|x| x.key() == layout)
        .next() {
            // println!("layout_to_tomlview shape: {:?} {:?}", layout_view.key(), layout_view.shape());
            return Ok(layout_view)
        }
    Err("Faild to load view in layout".to_string())
}

pub fn get_layout_tables_except_key(index_path: &Path, except_key: &str) -> Result<HashMap<String, Table>, String> {
    let mut file = File::open(index_path).ok().ok_or(format!("Failed to open the file in layout_tables in {:?}", index_path)?;

    let mut config_str = String::new();
    file.read_to_string(&mut config_str)
        .ok()
        .ok_or("Failed to read the file")?;

    let doc = config_str
        .parse::<DocumentMut>()
        .map_err(|err| format!("{}", err))?;

    let mut layout_tables: HashMap<String, Table> = hashmap! {};
    for (k, v) in doc.as_table() {
        if let Item::Table(table) = v {
            if k != except_key {
                layout_tables.insert(k.to_string(), table.clone());
            }
        }
    }
    Ok(layout_tables)
}

pub fn get_tomlview_for_key(index_path: &Path, find_key: &str, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, layout_tables: HashMap<String, Table>) -> Result<Box<dyn TOMLView>, String> {
    let mut file = File::open(index_path).ok().ok_or(format!("Failed to open the file in tomlview {:?}", index_path))?;

    let mut config_str = String::new();
    file.read_to_string(&mut config_str)
        .ok()
        .ok_or("Failed to read the file")?;

    let doc = config_str
        .parse::<DocumentMut>()
        .map_err(|err| format!("{}", err))?;

    for (k, v) in doc.as_table() {
        if let Item::Table(table) = v {
            println!("key! {} {:?}", k, index_path);
            if k == find_key {
                let value = value.or_else(|| {
                    table.get("value").and_then(|item| item.as_inline_table())
                });

                let view = table_to_tomlview(index_path, k, table, value, super_view, layout_tables);
                match view {
                    Ok(view) => {
                        return Ok(view);
                    }
                    Err(message) => {
                        println!("message {}", message);
                    }
                }
                break;
            }
        }
    }
    Err("".to_string())
}

pub fn table_to_tomlview(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, layout_tables: HashMap<String, Table>) -> Result<Box<dyn TOMLView>, String> {
    let shape = item_to_string(table, "shape", "", value);
    // println!("table_to_tomlview {:?} {:?}", key, shape);
    match shape.as_str() {
        "nav" => {
            let nav_view = NavView::new(index_path, key, table, value, super_view, layout_tables);
            Ok(Box::new(nav_view))
        }
        "box" => {
            let box_view = BoxView::new(index_path, key, table, value, super_view, layout_tables);
            Ok(Box::new(box_view))
        }
        "column" => {
            let column_view = ColumnView::new(index_path, key, table, value, super_view, layout_tables, false);
            Ok(Box::new(column_view))
        }
        "row" => {
            let row_view = RowView::new(index_path, key, table, value, super_view, layout_tables, false);
            Ok(Box::new(row_view))
        }
        "scroll_column" => {
            let column_view = ColumnView::new(index_path, key, table, value, super_view, layout_tables, true);
            Ok(Box::new(column_view))
        }
        "scroll_row" => {
            let row_view = RowView::new(index_path, key, table, value, super_view, layout_tables, true);
            Ok(Box::new(row_view))
        }
        "text" => {
            let text_view = TextView::new(index_path, key, table, value, super_view, layout_tables);
            Ok(Box::new(text_view))
        }
        "image" => {
            let image_view = ImageView::new(index_path, key, table, value, super_view, layout_tables);
            Ok(Box::new(image_view))
        }
        "list_column" => {
            let list_column_view = ListColumnView::new(index_path, key, table, value, super_view, layout_tables, true);
            Ok(Box::new(list_column_view))
        }
        "list_row" => {
            let list_row_view = ListRowView::new(index_path, key, table, value, super_view, layout_tables, true);
            Ok(Box::new(list_row_view))
        }
        "mdlist_column" => {
            let list_column_view = MarkdownListColumnView::new(index_path, key, table, value, super_view, layout_tables, true);
            Ok(Box::new(list_column_view))
        }
        "mdlist_row" => {
            let list_row_view = MarkdownListRowView::new(index_path, key, table, value, super_view, layout_tables, true);
            Ok(Box::new(list_row_view))
        }
        "markdown" => {
            let markdown_view = MarkdownView::new(index_path, key, table, value, super_view, layout_tables, true);
            Ok(Box::new(markdown_view))
        }
        "grid" => {
            let grid_view = GridView::new(index_path, key, table, value, super_view, layout_tables, true);
            Ok(Box::new(grid_view))
        }
        "embed" => {
            let embed_view = EmbedView::new(index_path, key, table, value, super_view, layout_tables, true);
            Ok(Box::new(embed_view))
        }
        _ => Err("".to_string()),
    }
}
