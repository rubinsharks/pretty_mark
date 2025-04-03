mod file;
mod html;
mod markdown;
mod toml;

use std::fmt::Formatter;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::{fmt, fs, io};

use crate::parser::html::footer;
use file::{change_root, find_images, find_md, find_option, read_dir_recursive};
use html::{headers_highlight, HTMLNode, HTMLTag};
use markdown::*;
use toml::{load_nav_from_toml, load_footer_from_toml};

struct Page {
    path: String,
    pages: Vec<Page>,
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
pub fn make_pages(path: &Path, html_path: &Path) -> Result<(), &'static str> {
    // let path = Path::new("root/option.toml");
    // toml::load_toml(path);
    let page = read_dir_recursive(path)?;
    println!("{:?}", page);
    make_page(html_path, &page)?;
    Ok(())
}

///
pub fn make_page(html_path: &Path, page: &Page) -> Result<(), &'static str> {
    println!("{:?}", page.path);
    let _ = page_to_html(html_path, &page);
    for page in &page.pages {
        make_page(html_path, page)?;
    }
    Ok(())
}

pub fn page_to_html(root: &Path, page: &Page) -> Result<(), &'static str> {
    let path = Path::new(&page.path);
    let md = find_md(path)?;
    let images = find_images(path)?;
    let mut nav_html = "".to_string();
    let mut footer_html = "".to_string();
    match find_option(path) {
        Ok(option) => {
            nav_html = load_nav_from_toml(option.as_path()).unwrap_or_else(|_| "".to_string());
            footer_html = load_footer_from_toml(option.as_path()).unwrap_or_else(|_| "".to_string());
        }
        Err(_) => {}
    }
    
    println!("{:?}", page.path);
    let md_path = Path::new(&md);

    match fs::metadata(md_path) {
        Ok(metadata) => {
            if let created = metadata.created().ok().unwrap() {
                println!("{:?}", created);
            }
            let updated = metadata.modified();
        }
        Err(_) => {}
    }

    let html = match parser(md_path) {
        Ok(node) => {
            println!("{:#?}", node);
            match md_to_html(&node, None) {
                None => String::from("1"),
                Some(mut node) => {
                    node.children.push(footer());
                    node.html(0)
                }
            }
        }
        Err(_) => String::from("2"),
    };

    let new_path = change_root(root, path);
    let new_path = Path::new(&new_path);
    let html_file_name = md_path.file_stem().unwrap().to_str().unwrap().to_string() + ".html";
    let html_path = new_path.join(html_file_name);

    let mut head = HTMLNode::new(HTMLTag::Head);
    let headers = headers_highlight();
    head.children = headers;

    let _ = fs::create_dir(new_path);

    let mut file = File::create(html_path).ok().ok_or("create fails")?;
    file.write(head.html(0).as_bytes())
        .ok()
        .ok_or("head write fails")?;
    file.write(format!("\n").as_bytes()).ok().ok_or("")?;
    file.write(format!("<body class=\"container mx-auto bg-white dark:bg-slate-900\">\n").as_bytes()).ok().ok_or("")?;
    if !nav_html.is_empty() {
        file.write(format!("{}", nav_html.as_str()).as_bytes()).ok().ok_or("")?;
    }
    file.write_all(html.as_bytes()).ok().ok_or("all fails")?;
    if !footer_html.is_empty() {
        file.write(format!("{}", footer_html.as_str()).as_bytes()).ok().ok_or("")?;
    }
    file.write(format!("</body>").as_bytes()).ok().ok_or("")?;

    for image in images {
        let _ = fs::copy(image.clone(), new_path.join(image.file_name().unwrap()));
    }

    Ok(())
}
