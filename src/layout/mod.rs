use std::path::Path;
use common::get_tomlview_for_key;

mod padding;
mod view;
mod common;
mod svg;
mod nav;

pub fn toml_to_html(index_path: &Path) -> Result<String, String> {
    let view = get_tomlview_for_key(index_path, "root", None, None)?;
    let html_view = view.htmlview(None).wrap_body(view.dark());
    let html = html_view.html();
    return Ok(html);
}