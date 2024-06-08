
mod parser;

use std::any::{Any, type_name};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Display, Path};
use markdown::mdast::{Heading, List, Node};
use markdown::message::Message;

fn main() -> Result<(), &'static str> {
    let path = Path::new("root");
    parser::make_pages(path)?;
    // let mut file = File::create("index.html")?;

    // let mut head = HTMLNode::new(HTMLTag::Head);
    // let headers = headers_highlight();
    // head.children = headers;
    //
    // let path = Path::new("root/index.md");
    // let html = match parser(path) {
    //     Ok(node) => {
    //         println!("{:#?}", node);
    //         match md_to_html(&node, None) {
    //             None => { String::from("1") }
    //             Some(node) => { node.html(0) }
    //         }
    //     }
    //     Err(_) => { String::from("2") }
    // };
    //
    // file.write(head.html(0).as_bytes());
    // file.write(format!("\n").as_bytes());
    // file.write_all(html.as_bytes())?;
    Ok(())
}

