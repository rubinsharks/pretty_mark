use crate::parser::option::load_option;
use crate::parser::Page;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

/// read files without directory file recursively
pub fn read_dir_recursive(path: &Path) -> Result<Page, &'static str> {
    let paths = fs::read_dir(path).ok().ok_or("")?;
    let path_string = path.to_str().unwrap().to_string();
    let option = match load_option(path) {
        Ok(option) => Some(option),
        Err(_) => None,
    };

    let mut title = "".to_string();
    let md_path = match find_md(path) {
        Ok(md_path) => {
            if let Some(path) = md_path {
                title = read_first_non_empty_line(path.as_path())?;
                Some(path)
            } else {
                None
            }
        }
        Err(_) => None,
    };

    let mut page = Page {
        path: path_string,
        title,
        option,
        pages: vec![],
    };

    for path in paths.filter_map(|x| x.ok()) {
        let path = path.path();
        if path.is_dir() {
            match read_dir_recursive(path.as_path()) {
                Ok(sub_page) => {
                    page.pages.push(sub_page)
                }
                Err(_) => {}
            }
        }
    }
    Ok(page)
}

pub fn read_first_non_empty_line(path: &Path) -> Result<String, &'static str> {
    let file = fs::File::open(path).ok().ok_or("failed to open")?;
    let reader = BufReader::new(file);

    for line_result in reader.lines() {
        let line = line_result.ok().ok_or("failed to read line")?;
        let trimmed = line.trim();

        if trimmed.starts_with('#') {
            let content = trimmed.trim_start_matches('#').trim();
            return Ok(content.to_string());
        }
    }

    Ok("".to_string()) // 공백이 아닌 줄이 없음
}

/// 확장자가 md인 파일을 찾아낸다.
/// 기본적으로 한 디렉토리에 하나의 md파일이 있을 것이라고 예상한다.
pub fn find_md(path: &Path) -> Result<Option<PathBuf>, String> {
    let paths = fs::read_dir(path).ok().ok_or("failed to read dir")?;
    let mut md_paths: Vec<PathBuf> = vec![];
    for path in paths.filter_map(|x| x.ok()).map(|x| x.path()) {
        match path.extension() {
            None => {}
            Some(extension) => {
                if extension == "md" {
                    md_paths.push(path);
                }
            }
        }
    }

    match md_paths.len() {
        1 => Ok(Some(md_paths.first().unwrap().to_path_buf())),
        0 => Ok(None),
        _ => Err(format!("multiple md files detected in {:?}", path)),
    }
}

/// 해당 디렉토리에 있는 모든 이미지 패스를 찾아내 추가한다.
pub fn find_images(path: &Path) -> Result<Vec<PathBuf>, &'static str> {
    let paths = fs::read_dir(path).ok().ok_or("")?;
    let mut md_paths: Vec<PathBuf> = vec![];
    for path in paths.filter_map(|x| x.ok()).map(|x| x.path()) {
        match path.extension() {
            None => {}
            Some(extension) => {
                if extension == "png" || extension == "jpeg" || extension == "jpg" || extension == "svg" {
                    md_paths.push(path);
                }
            }
        }
    }
    Ok(md_paths)
}

pub fn change_root(root: &Path, path: &Path) -> PathBuf {
    let path: PathBuf = path.iter().skip(1).collect();
    root.join(path)
}
