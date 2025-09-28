use std::{collections::HashMap, path::Path};
use common::{get_layout_tables_except_key, get_tomlview_for_key, table_to_tomlview};
use toml_edit::{value, Table};

mod padding;
mod view;
pub mod common;
mod svg;
mod nav;

pub fn toml_to_html(layout_path: &Path, layout_tables: HashMap<String, Table>) -> Result<String, String> {
    let view = get_tomlview_for_key(layout_path, "root", None, None, layout_tables)?;
    let html_view = view.htmlview(None).wrap_body(view.dark());
    let html = html_view.html();
    return Ok(html);
}

pub fn layouts_from_toml(index_path: &Path) -> Result<HashMap<String, Table>, String> {
    let tables = get_layout_tables_except_key(index_path, "root")?;
    Ok(tables)
}