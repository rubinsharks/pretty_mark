use std::{collections::HashMap, env, fs::{self, File}, io::Write, path::{Path, PathBuf}};

use parser::{get_node_for_markdown, node_to_html};
use serde_yaml::Value;
use toml_edit::{InlineTable, Table};

use crate::{html::HTMLView, layout::{common::{get_tomlview_for_key, layout_to_tomlview}, toml_to_html}, yaml::yaml_hashmap_to_inline_table};
pub mod parser;
pub mod common;

const MARKDOWN_TOML: &str = include_str!("../asset/markdown.toml");

fn get_markdown_path() -> Result<PathBuf, String> {
    let mut file_path = std::env::temp_dir();
    file_path.push("markdown.toml");

    if !file_path.exists() {
        fs::write(&file_path, MARKDOWN_TOML)
            .map_err(|e| format!("파일 쓰기 실패: {}", e))?;
    }
    Ok(file_path)
}

pub fn markdown_to_htmlview(md_path: &Path, is_dark: bool) -> Result<HTMLView, String> {
    let node = get_node_for_markdown(md_path)?;
    let htmlview = node_to_html(&node, None, None, is_dark);
    Ok(htmlview)
}

pub fn markdown_wrap_to_htmlview(md_path: &Path, layout_tables: HashMap<String, Table>) -> Result<HTMLView, String> {
    let md_wrap_path = get_markdown_path()?;
    let metas = metas_table_from_markdown(md_path)
        .unwrap_or_else(|_| InlineTable::new());

    let view = get_tomlview_for_key(md_wrap_path.as_path(), "root", Some(&metas), None, layout_tables)?;
    let md_html_view = markdown_to_htmlview(md_path, view.dark())?;

    let html_view = view.htmlview(None)
        .inflate_view("contents", md_html_view)
        .wrap_body(view.dark());
    Ok(html_view)
}

pub fn markdown_wrap_to_html(md_path: &Path, layout_tables: HashMap<String, Table>) -> Result<String, String> {
    let html_view = markdown_wrap_to_htmlview(md_path, layout_tables)?;
    let html = html_view.html();
    Ok(html)
}

// pub fn markdown_to_html(layout_path: &Path, layout_tables: HashMap<String, Table>) -> Result<String, String> {
//     let view = toml_to_html(layout_path, layout_tables.clone());
//     let is_dark = true;
//     let html_view = markdown_to_htmlview(layout_path, is_dark)?.wrap_body(is_dark);

//     // let metas = metas_table_from_markdown(layout_path)?;
//     // let html_view = add_meta_layout_to_html_view(html_view, metas, layout_tables)?;
//     let html = html_view.html();
//     return Ok(html);
// }

pub fn metas_table_from_markdown(index_path: &Path) -> Result<InlineTable, String> {
    let content = fs::read_to_string(index_path)
        .expect("Failed to read the markdown file");

    let mut metadata = InlineTable::new();
    let mut lines = content.lines();

    // 첫 줄이 --- 여야 frontmatter 시작
    if let Some(first_line) = lines.next() {
        if first_line.trim() != "---" {
            return Err("unable to find frontmatter first line".to_string());
        }
    } else {
        return Err("empty file".to_string()); // 빈 파일
    }

    // --- 사이의 메타데이터 추출
    let mut yaml_lines = Vec::new();
    let mut found_end = false;
    for line in lines {
        if line.trim() == "---" {
            found_end = true;
            break; // 종료 구분자 발견
        }
        yaml_lines.push(line);
    }

    // 종료 구분자가 없으면 빈 HashMap 반환
    if !found_end {
        return Err("unable to find frontmatter end line".to_string());
    }

    let yaml_str = yaml_lines.join("\n");
    
    let file_name: String = index_path
        .file_stem()          // Option<&OsStr>
        .and_then(|s| s.to_str()) // Option<&str>
        .map(|s| s.to_string())
        .unwrap_or("".to_string()); 
    metadata.insert("filename", toml_edit::Value::from(file_name));

    // YAML 파싱
    if let Ok(parsed) = serde_yaml::from_str::<HashMap<String, Value>>(&yaml_str) {
        let yaml_table = yaml_hashmap_to_inline_table(&parsed);
        metadata.extend(yaml_table);
    }
    Ok(metadata)
}