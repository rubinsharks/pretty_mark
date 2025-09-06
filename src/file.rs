use crate::option::load_option;
use crate::page::Page;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use glob::glob;

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

pub fn find_all_ext_files(path: &Path, ext: fn(&str) -> bool) -> Result<Vec<PathBuf>, String> {
    let paths = fs::read_dir(path).ok().ok_or("failed to read dir")?;
    let mut ext_paths: Vec<PathBuf> = vec![];
    for path in paths.filter_map(|x| x.ok()).map(|x| x.path()) {
        match path.extension() {
            None => {}
            Some(extension) => {
                if ext(extension.to_str().unwrap_or("")) {
                    ext_paths.push(path);
                }
            }
        }
    }
    Ok(ext_paths)
}

/// 해당 디렉토리에 있는 모든 이미지 패스를 찾아내 추가한다.
pub fn find_images(path: &Path) -> Result<Vec<PathBuf>, String> {
    let image_paths = find_all_ext_files(path, |ext| {
        ext == "png" || ext == "jpeg" || ext == "jpg" || ext == "svg"
    })?;
    Ok(image_paths)
}

pub fn change_root(root: &Path, path: &Path) -> PathBuf {
    let path: PathBuf = path.iter().skip(1).collect();
    root.join(path)
}

pub fn get_file_timestamps(path: &Path) -> io::Result<(Option<SystemTime>, Option<SystemTime>)> {
    let metadata = fs::metadata(path)?;

    let created = metadata.created().ok();
    let modified = metadata.modified().ok(); // 일반적으로 지원됨

    Ok((created, modified))
}

pub fn find_files(pattern: &str, base_path: &Path) -> Vec<PathBuf> {
    println!("find_files pattern: {}", pattern);
    let mut results = Vec::new();

    // base_path + pattern (예: /tmp/*.md)
    let full_pattern = base_path.join(pattern).to_string_lossy().to_string();

    for entry in glob(&full_pattern).expect("Invalid glob pattern") {
        match entry {
            Ok(path) => {
                if !path.is_dir() {
                    results.push(path)
                }
            },
            Err(e) => eprintln!("Glob error: {:?}", e),
        }
    }

    results
}
