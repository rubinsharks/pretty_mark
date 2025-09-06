use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use maplit::hashmap;
use toml_edit::Table;

use crate::common::{copy_img_files_to_path, remove_code_indentation};
use crate::file;
use crate::layout::{self, layouts_from_toml, toml_to_html};
use crate::markdown::markdown_to_html;

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
    
    pub fn inflate_html(&mut self, layout_tables: HashMap<String, Table>) {
        let mut layout_tables = layout_tables;
        if let Some(ext) = self.layout_path.extension() {
            println!("layout_path {:?}", ext);
            if ext == "md" {
                match markdown_to_html(self.layout_path.as_path()) {
                    Ok(html) => {
                        self.layout_html = html;
                    },
                    Err(message) => {
                        println!("message {}", message);
                    }
                }
            } else if ext == "toml" {
                // TODO : key가 중복되는 상위 테이블 제거 
                let layout_tables_local = layouts_from_toml(self.layout_path.as_path()).unwrap_or(hashmap! {});
                layout_tables.extend(layout_tables_local);
                match toml_to_html(self.layout_path.as_path(), layout_tables.clone()) {
                    Ok(html) => {
                        self.layout_html = html;
                    },
                    Err(message) => {
                        println!("message {}", message);
                    }
                }
            }
        }
        self.layout_html = remove_code_indentation(self.layout_html.clone());
        for page in &mut self.pages {
            page.inflate_html(layout_tables.clone());
        }
    }

    pub fn make_html_file(&mut self, root: &Path) {
        // self.path의 첫 부분을 root로 교체
        // 예: self.path = "test/sub1", root = "a/b"
        // → "a/b/sub1"
        let relative = self.path.strip_prefix(
            self.path.components().next().unwrap().as_os_str()
        ).unwrap_or(self.path.as_path());

        let mut new_path = PathBuf::from(root);
        new_path.push(relative);

        // 디렉토리 생성
        if let Err(e) = fs::create_dir_all(&new_path) {
            eprintln!("디렉토리 생성 실패: {:?}", e);
            return;
        }

        // index.html 경로
        let index_file = new_path.join("index.html");

        // layout_html 내용을 파일에 쓰기
        if let Err(e) = fs::write(&index_file, &self.layout_html) {
            eprintln!("HTML 파일 생성 실패: {:?}", e);
        }
        copy_img_files_to_path(self.path.as_path(), new_path.as_path());

        for page in &mut self.pages {
            page.make_html_file(root);
        }
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
            writeln!(f, "{:?}", page).unwrap();
        }
        Ok(())
    }
}

/// read files without directory file recursively
pub fn read_dir_recursive(path: &Path) -> Result<Page, String> {
    let paths = fs::read_dir(path).ok().ok_or("")?;
    
    let this_path = path.to_path_buf();
    let index_path = find_index_path(path)?
        .ok_or(format!("{:?}: have no index", path.to_str()))?;
    let title = find_title(path)?;

    let mut page = Page {
        path: this_path,
        layout_path: index_path,
        layout_html: String::new(),
        title,
        pages: vec![],
    };

    

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

pub fn find_index_path(path: &Path) -> Result<Option<PathBuf>, String> {
    let index_paths = file::find_all_ext_files(path, |ext| {
       ext == "md" || ext == "toml"
    })?;
    println!("paths {:?}", index_paths);

    let selected = index_paths
        .iter()
        .find(|p| p.file_name() == Some("index.toml".as_ref()))
        .or_else(|| index_paths.iter().find(|p| p.file_name() == Some("index.md".as_ref())))
        .cloned(); // <-- 여기서 &PathBuf → PathBuf로 복사
    Ok(selected)
}

pub fn find_md_paths_except_index(path: &Path) -> Result<Vec<PathBuf>, String> {
    let md_paths = file::find_all_ext_files(path, |ext| {
       ext == "md"
    })?;
    let filtered = md_paths
        .into_iter()
        .filter(|p| p.file_name().map_or(true, |f| f != "index.md"))
        .collect();
    Ok(filtered)
}

pub fn find_title(path: &Path) -> Result<String, String> {
    Ok("".to_string())
}