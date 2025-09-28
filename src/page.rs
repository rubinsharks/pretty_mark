use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use maplit::hashmap;
use toml_edit::Table;

use crate::common::{copy_img_files_to_path, remove_code_indentation};
use crate::file::{self, has_stem_dir};
use crate::html::HTMLView;
use crate::layout::{self, layouts_from_toml, toml_to_html};
use crate::markdown::{markdown_wrap_to_html, markdown_wrap_to_htmlview};

use std::collections::{HashMap, HashSet};
use std::fmt::Formatter;
use std::fs::File;
use std::io::{Read, Write};
use std::{fmt};

pub struct Page {
    path: PathBuf,
    layout_path: PathBuf,
    pub layout_html: String,
    title: String,
    pages: Vec<Page>,
    markdowns: Vec<PathBuf>,
    markdowns_html: HashMap<PathBuf, String>,
}

impl Page {
    pub fn print(&self, depth: usize) -> String {
        let indent = "-".repeat(depth);
        let mut filename = format!("{}{}\n", indent,
            self.path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
        );
        for page in &self.pages {
            filename.push_str(&page.print(depth + 1));
        }
        filename
    }

    /// Page의 layout_html을 실제 html로 채운다. 
    pub fn inflate_html(&mut self, layout_tables: HashMap<String, Table>) -> Result<(), String> {
        let mut layout_tables = layout_tables;
        let layout_path: &Path = self.layout_path.as_ref();
        if let Some(ext) = layout_path.extension() {
            if ext == "md" {
                let html = markdown_wrap_to_html(layout_path, layout_tables.clone())?;
                self.layout_html = html;
            } else if ext == "toml" {
                // TODO : key가 중복되는 상위 테이블 제거 
                let layout_tables_local = layouts_from_toml(layout_path).unwrap_or(hashmap! {});
                layout_tables.extend(layout_tables_local);
                let html = toml_to_html(layout_path, layout_tables.clone())?;
                self.layout_html = html;
            }
        }
        self.layout_html = remove_code_indentation(self.layout_html.clone());

        let mut markdowns_html: HashMap<PathBuf, String> = hashmap! {};
        for md_path in &self.markdowns {
            let view = markdown_wrap_to_htmlview(&md_path, layout_tables.clone())?;
            markdowns_html.insert(md_path.clone(), view.html());
        }
        self.markdowns_html = markdowns_html; 
        
        for page in &mut self.pages {
            page.inflate_html(layout_tables.clone())?;
        }
        
        Ok(())
    }

    pub fn make_html_file(&mut self, root: &Path) -> Result<(), String> {
        // self.path의 첫 부분을 root로 교체
        // 예: self.path = "test/sub1", root = "a/b"
        // → "a/b/sub1"
        let relative = self.path.strip_prefix(
            self.path.components().next().unwrap().as_os_str()
        ).unwrap_or(self.path.as_path());

        let mut new_path = PathBuf::from(root);
        new_path.push(relative);

        // 디렉토리 생성
        fs::create_dir_all(&new_path).map_err(|e| format!("디렉토리 생성 실패: {e}"))?;

        // index.html 경로
        let index_file = new_path.join("index.html");
        fs::write(&index_file, &self.layout_html).map_err(|e| format!("HTML 파일 생성 실패: {e}"))?;

        // ✅ markdowns 처리
        for md_path in &self.markdowns {
            // stem 추출 (확장자 없는 파일 이름)
            let stem = md_path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| format!("잘못된 파일 이름: {}", md_path.display()))?;

            // stem 디렉토리 생성
            let stem_dir = new_path.join(stem);
            fs::create_dir_all(&stem_dir)
                .map_err(|e| format!("디렉토리 생성 실패: {e}"))?;

            // markdown html 내용 가져오기
            let content = self.markdowns_html.get(md_path)
                .ok_or_else(|| format!("markdowns_html 에서 {} 내용을 찾을 수 없음", md_path.display()))?;

            // stem/index.html 작성
            let md_index = stem_dir.join("index.html");
            fs::write(&md_index, content)
                .map_err(|e| format!("마크다운 HTML 파일 생성 실패: {e}"))?;
        }
        
        copy_img_files_to_path(self.path.as_path(), new_path.as_path())?;
        for page in &mut self.pages {
            page.make_html_file(root)?;
        }
        Ok(())
    }
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.path.to_str().unwrap_or("")).unwrap();
        Ok(())
    }
}

impl fmt::Debug for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.path.to_str().unwrap_or("")).unwrap();
        for page in &self.pages {
            write!(f, "{:?}", page).unwrap();
        }
        Ok(())
    }
}

/// read files without directory file recursively
pub fn read_dir_recursive(path: &Path) -> Result<Page, String> {
    let this_path = path.to_path_buf();
    let index_path = find_index_path(path)?;
    let title = find_title(path)?;

    let markdown_paths = find_md_paths_except_index(&path)?;
    
    let mut page = Page {
        path: this_path.clone(),
        layout_path: index_path,
        layout_html: String::new(),
        title,
        pages: vec![],
        markdowns: markdown_paths,
        markdowns_html: hashmap! {},
    };

    let paths = fs::read_dir(path).ok().ok_or("")?;
    for path in paths.filter_map(|x| x.ok()) {
        let path = path.path();
        if path.is_dir() {
            match read_dir_recursive(path.as_path()) {
                Ok(sub_page) => {
                    page.pages.push(sub_page)
                }
                Err(message) => {
                    println!("{}", message);
                }
            }
        }
    }
    Ok(page)
}

pub fn find_index_path(path: &Path) -> Result<PathBuf, String> {
    let index_paths = file::find_all_ext_files(path, |ext| {
        ext == "md" || ext == "toml"
    })?;

    // index.md, index.toml 만 걸러내기
    let mut index_candidates: Vec<PathBuf> = index_paths
        .into_iter()
        .filter(|p| {
            matches!(
                p.file_name().and_then(|f| f.to_str()),
                Some("index.md") | Some("index.toml")
            )
        })
        .collect();

    // index.* 파일이 둘 이상 있으면 에러
    if index_candidates.len() > 1 {
        return Err(format!(
            "multiple index files detected in {:?}. please keep only one of index.md or index.toml",
            path
        ));
    }

    if let Some(p) = index_candidates.pop() {
        Ok(p)
    } else {
        // 없으면 index.md 생성
        let new_index = path.join("index.md");
        if !new_index.exists() {
            fs::write(&new_index, "")
                .map_err(|e| format!("failed to create {:?}: {}", new_index, e))?;
        }
        Ok(new_index)
    }
}

pub fn find_md_paths_except_index(path: &Path) -> Result<Vec<PathBuf>, String> {
    let md_paths = file::find_all_ext_files(path, |ext| {
       ext == "md"
    })?;

    // index.md 제외
    let filtered: Vec<PathBuf> = md_paths
        .into_iter()
        .filter(|p| p.file_name().map_or(true, |f| f != "index.md"))
        .collect();

    // 디렉토리 이름 모으기
    let mut folder_names = HashSet::new();
    for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let meta = entry.metadata().map_err(|e| e.to_string())?;
        if meta.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                folder_names.insert(name.to_string());
            }
        }
    }

    // md 파일과 폴더 이름 충돌 검사
    for md in &filtered {
        if let Some(stem) = md.file_stem().and_then(|s| s.to_str()) {
            if folder_names.contains(stem) {
                return Err(format!(
                    "Folder name conflicts with markdown file: {}",
                    md.display()
                ));
            }
        }
    }
    
    Ok(filtered)
}

pub fn make_md_files_to_folder_except_index(path: &Path) -> Result<Vec<PathBuf>, String> {
    let md_paths = find_md_paths_except_index(path)?;
    let mut new_paths = Vec::new();

    for md in md_paths {
        let stem = md
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("Invalid file name: {}", md.display()))?;

        let parent = md.parent().unwrap_or_else(|| Path::new("."));
        let new_dir = parent.join(stem);
        let new_path = new_dir.join("index.md");

        fs::create_dir_all(&new_dir)
            .map_err(|e| format!("Failed to create {:?}: {}", new_dir, e))?;

        if new_path.exists() {
            fs::remove_file(&new_path)
                .map_err(|e| format!("Failed to remove existing {:?}: {}", new_path, e))?;
        }

        fs::rename(&md, &new_path)
            .map_err(|e| format!("Failed to move {:?} -> {:?}: {}", md, new_path, e))?;

        new_paths.push(new_path);
    }

    Ok(new_paths)
}

pub fn find_title(path: &Path) -> Result<String, String> {
    Ok("".to_string())
}