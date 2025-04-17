mod file;
mod html;
mod markdown;
mod option;

use std::collections::{HashMap, HashSet};
use std::fmt::Formatter;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::{fmt, fs, io};

use crate::parser::html::footer;
use file::{change_root, find_images, find_md, read_dir_recursive};
use html::{headers_highlight, HTMLNode, HTMLTag};
use markdown::*;
use option::{find_option, load_option_from_toml, MDOption};
use std::time::SystemTime;

/// 일단 첫줄을 title로 본다.
struct Page {
    path: String,
    title: String,
    option: Option<MDOption>,
    pages: Vec<Page>,
}

impl Page {
    fn collect_tags(&self) -> HashSet<String> {
        let mut tags = HashSet::new();

        // 현재 페이지의 태그를 추가
        if let Some(option) = &self.option {
            if !option.basic.tags.is_empty() {
                tags.extend(option.basic.tags.iter().cloned());
            }
        }

        // 하위 페이지들을 재귀적으로 탐색하여 태그 수집
        for page in &self.pages {
            let sub_page_tags = page.collect_tags();
            tags.extend(sub_page_tags); // 하위 페이지에서 수집된 태그를 추가
        }

        tags
    }
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl fmt::Debug for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.path).unwrap();
        for page in &self.pages {
            print!("{:?}", page);
        }
        Ok(())
    }
}

/// 해당 Path에 있는 폴더를 재귀로 분석해 html로 변환한다.
pub fn make_pages(path: &Path, html_root_path: &Path) -> Result<(), String> {
    // let path = Path::new("root/option.toml");
    // toml::load_toml(path);
    let page = read_dir_recursive(path)?;
    let tags = page.collect_tags();
    make_page(html_root_path, &page, &None)?;
    Ok(())
}

///
pub fn make_page(html_root_path: &Path, page: &Page, md_sup_option: &Option<MDOption>) -> Result<(), String> {
    let md_option = page_to_html(html_root_path, &page, md_sup_option)?;
    for page in &page.pages {
        make_page(html_root_path, page, &md_option)?;
    }
    Ok(())
}

pub fn page_to_html(html_root_path: &Path, page: &Page, md_sup_option: &Option<MDOption>) -> Result<Option<MDOption>, String> {
    let path = Path::new(&page.path);
    let md = find_md(path)?;
    let images = find_images(path)?;
    
    let mut md_option: Option<MDOption> = None;
    match find_option(path) {
        Ok(option) => match load_option_from_toml(option.as_path()) {
            Ok(option) => {
                md_option = Some(option);
            }
            Err(_) => {
                if let Some(md_sup_option) = md_sup_option {
                    md_option = Some(md_sup_option.clone());
                }
            }
        },
        Err(_) => {
            if let Some(md_sup_option) = md_sup_option {
                md_option = Some(md_sup_option.clone());
            }
        }
    }

    let new_path = change_root(html_root_path, path);
    let new_path = Path::new(&new_path);
    let html_path = new_path.join("index.html");
    
    if let Some(md) = md {
        let md_path = Path::new(&md);

        // let (created, updated) = get_file_timestamps(&path)?;

        let html = match parser(md_path) {
            Ok(node) => {
                println!("{node:?}");
                match md_to_html(&node, None, &md_option) {
                    None => String::from("1"),
                    Some(mut node) => {
                        // node.children.push(footer(&md_option));
                        node.html(0)
                    }
                }
            }
            Err(_) => String::from("2"),
        };

        let _ = fs::create_dir(new_path);
        write_html(html_path.as_path(), &md_option, &html)?;

        for image in images {
            let _ = fs::copy(image.clone(), new_path.join(image.file_name().unwrap()));
        }
    } else {
        let _ = fs::create_dir(new_path);
        let mut file = File::create(html_path).ok().ok_or("create fails")?;
    }

    Ok(md_option)
}

fn get_file_timestamps(path: &Path) -> io::Result<(Option<SystemTime>, Option<SystemTime>)> {
    let metadata = fs::metadata(path)?;

    let created = metadata.created().ok();
    let modified = metadata.modified().ok(); // 일반적으로 지원됨

    Ok((created, modified))
}


fn write_html(html_path: &Path, md_option: &Option<MDOption>, html: &String) -> Result<(), String> {

    let mut head = HTMLNode::new(HTMLTag::Head, &md_option);
    let headers = headers_highlight(&md_option);
    head.children = headers;

    let mut file = File::create(html_path).ok().ok_or("create fails")?;

    file.write(head.html(0).as_bytes())
        .ok()
        .ok_or("head write fails")?;
    file.write(format!("\n").as_bytes()).ok().ok_or("")?;
    file.write(
        format!(
            "<body class=\"{}\">\n",
            filter_attrs("container mx-auto bg-white dark:bg-slate-900", &md_option)
        )
        .as_bytes(),
    )
    .ok()
    .ok_or("")?;
    if let Some(option) = &md_option {
        file.write(format!("{}", option.menus_to_html().as_str()).as_bytes())
            .ok()
            .ok_or("")?;
    }
    
    file.write_all(r#"<div class="max-w-screen-xl mx-auto px-4">"#.as_bytes()).ok().ok_or("")?;
    file.write_all(r#"<div class="left-0 h-2 w-full"></div>"#.as_bytes())
    .ok()
    .ok_or("all fails")?;
    file.write_all(html.as_bytes()).ok().ok_or("all fails")?;
    file.write_all(r#"</div>"#.as_bytes()).ok().ok_or("")?;
    file.write(format!("\n").as_bytes()).ok().ok_or("")?;
    file.write_all(r#"<div class="left-0 h-20 w-full"><br></div>"#.as_bytes())
        .ok()
        .ok_or("all fails")?;
    file.write(format!("\n").as_bytes()).ok().ok_or("")?;
    if let Some(option) = &md_option {
        file.write(format!("{}", option.footer_to_html().as_str()).as_bytes())
            .ok()
            .ok_or("")?;
    }
    file.write(format!("\n").as_bytes()).ok().ok_or("")?;
    file.write(format!("</body>").as_bytes()).ok().ok_or("")?;
    
    Ok(())
}