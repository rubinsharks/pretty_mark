use std::{fs::File, path::Path};

use parser::{get_node_for_markdown, node_to_html};

use crate::{html::HTMLView};
pub mod parser;
pub mod common;

pub fn markdown_to_htmlview(index_path: &Path, is_dark: bool) -> Result<HTMLView, &'static str> {
    let node = get_node_for_markdown(index_path)?;
    let htmlview = node_to_html(&node, None, None, is_dark);
    Ok(htmlview)
}

pub fn markdown_to_html(index_path: &Path) -> Result<String, &'static str> {
    let is_dark = true;
    let html_view = markdown_to_htmlview(index_path, is_dark)?.wrap_body(is_dark);
    let html = html_view.html();
    return Ok(html);
}