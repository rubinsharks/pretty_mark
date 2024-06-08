use std::fs;
use std::path::{Path, PathBuf};
use crate::parser::Page;

/// read files without directory file recursively
pub fn read_dir_recursive(path: &Path) -> Result<Page, &'static str> {
    let paths = fs::read_dir(path).ok().ok_or("")?;
    let path_string = path.to_str().unwrap().to_string();
    let mut page = Page { path: path_string, pages: vec![] };
    for path in paths.filter_map(|x| x.ok()) {
        match read_dir_recursive(path.path().as_path()) {
            Ok(sub_page) => { page.pages.push(sub_page) }
            Err(_) => { }
        }
    }
    Ok(page)
}

pub fn find_md(path: &Path) -> Result<String, &'static str> {
    let paths = fs::read_dir(path).ok().ok_or("")?;
    let mut md_paths: Vec<PathBuf> = vec![];
    for path in paths.filter_map(|x| x.ok()).map(|x| x.path()) {
        match path.extension() {
            None => { }
            Some(extension) => {
                if extension == "md" {
                    md_paths.push(path);
                }
            }
        }
    }
    match md_paths.len() {
        1 => Ok(md_paths.first().unwrap().to_str().ok_or("")?.to_string()),
        0 => Err("no md file detected"),
        _ => Err("multiple md files detected")
    }
}