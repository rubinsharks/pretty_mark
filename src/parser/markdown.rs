

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use markdown::mdast::Node;
use markdown::message::Message;
use crate::parser::html::{HTMLNode, HTMLTag};


pub fn parser(path: &Path) -> Result<Node, &'static str> {
    let mut file = File::open(&path).ok().ok_or("open fails")?;
    let mut s = String::new();
    file.read_to_string(&mut s).ok().ok_or("read fails")?;

    match markdown::to_mdast(&s, &markdown::ParseOptions::default()) {
        Ok(node) => { Ok(node) }
        Err(message) => { Err("no file")}
    }
}

pub fn md_class(md: &Node, sup: Option<&Node>) -> String {
    match md {
        Node::Root(..) => String::from("container mx-auto"),
        Node::Paragraph(..) => {
            match sup {
                None => String::from(""),
                Some(node) => {
                    match node {
                        Node::BlockQuote(..) => String::from("text-xl italic font-medium leading-relaxed text-gray-900 dark:text-white"),
                        _ => String::from("text-gray-800 dark:text-gray-800 mt-1")
                    }
                }
            }
        }
        Node::Heading(value) => {
            match value.depth {
                1 => String::from("text-4xl font-bold mb-3 mt-5"),
                2 => String::from("text-3xl font-bold mb-3 mt-5"),
                3 => String::from("text-2xl font-bold mb-3 mt-5"),
                _ => String::from(""),
            }
        }
        Node::List(value) => {
            match value.start {
                None => String::from("list-disc pl-5"),
                Some(_) => String::from("list-decimal pl-5"),
            }
        }
        Node::BlockQuote(..) => String::from("p-4 my-4 border-s-4 border-gray-300 bg-gray-50 dark:border-gray-500 dark:bg-gray-800"),
        _ => String::from(""),
    }
}

pub fn md_to_html(md: &Node, sup: Option<&Node>) -> Option<HTMLNode> {
    match md {
        Node::Root(node) => {
            Some(HTMLNode {
                tag: HTMLTag::Body,
                children: node.children.iter().filter_map(|x| md_to_html(x, Some(md))).collect(),
                attributes: HashMap::from([
                    ("class", md_class(md, sup))
                ]),
                value: "".to_string(),
            })
        }
        Node::Heading(node) => {
            Some(HTMLNode {
                tag: HTMLTag::header_by(node.depth),
                children: node.children.iter().filter_map(|x| md_to_html(x, Some(md))).collect(),
                attributes: HashMap::from([
                    ("class", md_class(md, sup))
                ]),
                value: "".to_string(),
            })
        }
        Node::BlockQuote(node) => {
            Some(HTMLNode {
                tag: HTMLTag::Blockquote,
                children: node.children.iter().filter_map(|x| md_to_html(x, Some(md))).collect(),
                attributes: HashMap::from([
                    ("class", md_class(md, sup))
                ]),
                value: "".to_string(),
            })
        }
        Node::Paragraph(node) => {
            Some(HTMLNode {
                tag: HTMLTag::P,
                children: node.children.iter().filter_map(|x| md_to_html(x, Some(md))).collect(),
                attributes: HashMap::from([
                    ("class", md_class(md, sup))
                ]),
                value: String::from(""),
            })
        }
        Node::Text(node) => {
            Some(HTMLNode {
                tag: HTMLTag::Text,
                children: vec![],
                attributes: HashMap::from([
                    ("class", md_class(md, sup))
                ]),
                value: String::from(&node.value.replace("\n", "<br />")),
            })
        }
        Node::Strong(node) => {
            Some(HTMLNode {
                tag: HTMLTag::Strong,
                children: node.children.iter().filter_map(|x| md_to_html(x, Some(md))).collect(),
                attributes: HashMap::from([
                    ("class", md_class(md, sup))
                ]),
                value: String::from(""),
            })
        }
        Node::Emphasis(node) => {
            Some(HTMLNode {
                tag: HTMLTag::EM,
                children: node.children.iter().filter_map(|x| md_to_html(x, Some(md))).collect(),
                attributes: HashMap::from([
                    ("class", md_class(md, sup))
                ]),
                value: String::from(""),
            })
        }
        Node::Code(code) => {
            Some(HTMLNode {
                tag: HTMLTag::PRE,
                children: vec![
                    HTMLNode {
                        tag: HTMLTag::Code,
                        children: vec![],
                        attributes: match &code.lang {
                            None => { HashMap::new() }
                            Some(lang) => { HashMap::from([
                                ("class", format!("language-{}", lang))
                            ]) }
                        },
                        value: String::from(&code.value),
                    }
                ],
                attributes: HashMap::from([
                    ("class", md_class(md, sup))
                ]),
                value: "".to_string(),
            })
        }
        Node::List(node) => {
            Some(HTMLNode {
                tag: HTMLTag::ordered(md),
                children: node.children.iter().filter_map(|x| md_to_html(x, Some(md))).collect(),
                attributes: HashMap::from([
                    ("class", md_class(md, sup))
                ]),
                value: "".to_string(),
            })
        }
        Node::ListItem(node) => {
            Some(HTMLNode {
                tag: HTMLTag::LI,
                children: node.children.iter().filter_map(|x| md_to_html(x, Some(md))).collect(),
                attributes: HashMap::from([
                    ("class", md_class(md, sup))
                ]),
                value: "".to_string(),
            })
        }
        Node::Link(node) => {
            Some(HTMLNode {
                tag: HTMLTag::A,
                children: node.children.iter().filter_map(|x| md_to_html(x, Some(md))).collect(),
                attributes: HashMap::from([
                    ("class", md_class(md, sup)),
                    ("href", node.url.to_string()),
                ]),
                value: "".to_string(),
            })
        }
        Node::Image(node) => {
            Some(HTMLNode {
                tag: HTMLTag::IMG,
                children: vec![],
                attributes: HashMap::from([
                    ("class", md_class(md, sup)),
                    ("src", format!("root/{}", node.url.to_string())),
                    ("alt", node.alt.to_string()),
                    ("title", match &node.title {
                        None => "".to_string(),
                        Some(title) => title.to_string(),
                    })
                ]),
                value: "".to_string(),
            })
        }
        _ => {
            None
        }
    }
}