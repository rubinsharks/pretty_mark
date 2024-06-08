use std::{fmt, fs, io};
use std::ffi::OsStr;
use std::fmt::Formatter;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use crate::parser::file::{find_md, read_dir_recursive};

mod markdown;
mod file;
mod html;

use markdown::*;
use crate::parser::html::{headers_highlight, HTMLNode, HTMLTag};

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

pub fn make_pages(path: &Path) -> Result<(), &'static str> {
    let page = read_dir_recursive(path)?;
    println!("{:?}", page);
    make_page(&page)?;
    Ok(())
}

pub fn make_page(page: &Page) -> Result<(), &'static str> {
    let _ = page_to_html("html", &page);
    for page in &page.pages {
        make_page(page)?;
    }
    Ok(())
}

pub fn page_to_html(root: &str, page: &Page) -> Result<(), &'static str> {
    let path = Path::new(&page.path);
    let md_path = find_md(path)?;
    let md_path = Path::new(&md_path);

    let html = match parser(md_path) {
        Ok(node) => {
            // println!("{:#?}", node);
            match md_to_html(&node, None) {
                None => { String::from("1") }
                Some(node) => { node.html(0) }
            }
        }
        Err(_) => { String::from("2") }
    };

    let mut components: Vec<_> = path.components().collect();
    components.remove(0);
    let mut html_path = components.iter()
        .map(|x| x.as_os_str().to_str().unwrap())
        .fold("".to_string(), |b, x| b + x);
    let new_path = root.to_string() + "/" + html_path.as_str();
    let new_path = Path::new(&new_path);
    let html_file_name = md_path.file_stem().unwrap().to_str().unwrap().to_string() + ".html";
    let html_path = new_path.join(html_file_name);

    let mut head = HTMLNode::new(HTMLTag::Head);
    let headers = headers_highlight();
    head.children = headers;

    let _ = fs::create_dir(new_path);

    let mut file = File::create(html_path).ok().ok_or("create fails")?;
    file.write(head.html(0).as_bytes()).ok().ok_or("head write fails")?;
    file.write(format!("\n").as_bytes()).ok().ok_or("")?;
    file.write_all(html.as_bytes()).ok().ok_or("all fails")?;

    Ok(())
}