use std::{collections::HashMap, fs::{self, File}, path::Path};

use parser::{get_node_for_markdown, node_to_html};
use serde_yaml::Value;
use toml_edit::InlineTable;

use crate::{html::HTMLView, yaml::yaml_hashmap_to_inline_table};
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