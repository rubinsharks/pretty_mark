mod markdown;
mod file;
mod html;

use std::{fmt, fs, io};
use std::fmt::Formatter;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use markdown::*;
use file::{change_root, find_images, find_md, read_dir_recursive};
use html::{headers_highlight, HTMLNode, HTMLTag};
use crate::parser::html::footer;

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
    let md = find_md(path)?;
    let images = find_images(path)?;
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
                None => { String::from("1") }
                Some(mut node) => {
                    node.children.push(footer());
                    node.html(0)
                }
            }
        }
        Err(_) => { String::from("2") }
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
    file.write(head.html(0).as_bytes()).ok().ok_or("head write fails")?;
    file.write(format!("\n").as_bytes()).ok().ok_or("")?;
    file.write_all(html.as_bytes()).ok().ok_or("all fails")?;

    for image in images {
        let _ = fs::copy(image.clone(), new_path.join(image.file_name().unwrap()));
    }

    Ok(())
}