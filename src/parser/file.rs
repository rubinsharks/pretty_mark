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

pub fn find_md(path: &Path) -> Result<PathBuf, &'static str> {
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
        1 => Ok(md_paths.first().unwrap().to_path_buf()),
        0 => Err("no md file detected"),
        _ => Err("multiple md files detected")
    }
}

pub fn find_images(path: &Path) -> Result<Vec<PathBuf>, &'static str> {
    let paths = fs::read_dir(path).ok().ok_or("")?;
    let mut md_paths: Vec<PathBuf> = vec![];
    for path in paths.filter_map(|x| x.ok()).map(|x| x.path()) {
        match path.extension() {
            None => { }
            Some(extension) => {
                if extension == "png"
                    || extension == "jpeg"
                    || extension == "jpg" {
                    md_paths.push(path);
                }
            }
        }
    }
    Ok(md_paths)
}

pub fn change_root(root: &str, path: &Path) -> PathBuf {
    let mut components: Vec<_> = path.components().collect();
    components.remove(0);
    let path: PathBuf = path.iter().skip(1).collect();
    Path::new(root).join(path)
}