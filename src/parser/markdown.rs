use crate::parser::html::{HTMLNode, HTMLTag};
use markdown::mdast::Node;
use markdown::message::Message;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::parser::option::ThemeValue;
use crate::parser::option::MDOption;

pub fn parser(path: &Path) -> Result<Node, &'static str> {
    let mut file = File::open(&path).ok().ok_or("open fails")?;
    let mut s = String::new();
    file.read_to_string(&mut s).ok().ok_or("read fails")?;

    match markdown::to_mdast(&s, &markdown::ParseOptions::gfm()) {
        Ok(node) => Ok(node),
        Err(message) => Err("no file"),
    }
}

pub fn filter_attrs(text: &str, md_option: &Option<MDOption>) -> String {
    if let Some(md_option) = md_option {
        if !md_option.is_night() {
            let result = text
                .split_whitespace() 
                .filter(|&word| !(word.starts_with("dark:") || word.starts_with("md:dark:")))
                .collect::<Vec<&str>>()
                .join(" "); 
            return result
        }
    }
    text.to_string()
}

pub fn md_class(md: &Node, sup: Option<&Node>, md_option: &Option<MDOption>, index: Option<usize>) -> String {
    match md {
        Node::Root(..) => filter_attrs("container mx-auto bg-white dark:bg-slate-900", md_option),
        Node::Paragraph(..) => match sup {
            None => String::from(""),
            Some(node) => match node {
                Node::Blockquote(..) => filter_attrs(
                    "text-xl italic font-medium leading-relaxed text-gray-900 dark:text-white", md_option
                ),
                _ => filter_attrs("text-slate-500 dark:text-slate-400 mt-1", md_option),
            },
        },
        Node::Heading(value) => match value.depth {
            1 => filter_attrs("text-4xl font-bold mb-8 mt-5 text-slate-900 dark:text-white", md_option),
            2 => filter_attrs("text-3xl font-bold mb-6 mt-5 text-slate-900 dark:text-white", md_option),
            3 => filter_attrs("text-2xl font-bold mb-4 mt-5 text-slate-900 dark:text-white", md_option),
            4 => filter_attrs("text-xl font-bold mb-3 mt-5 text-slate-900 dark:text-white", md_option),
            5 => filter_attrs("text-base font-bold mb-3 mt-5 text-slate-900 dark:text-white", md_option),
            6 => filter_attrs("text-sm font-bold mb-3 mt-5 text-slate-900 dark:text-white", md_option),
            _ => filter_attrs("", md_option),
        },
        Node::List(value) => match value.start {
            None => filter_attrs("list-disc pl-5 text-slate-500 dark:text-slate-400", md_option),
            Some(_) => filter_attrs("list-decimal pl-5 text-slate-500 dark:text-slate-400", md_option),
        },
        Node::Blockquote(..) => filter_attrs(
            "p-4 my-4 border-s-4 border-gray-300 bg-gray-50 dark:border-gray-500 dark:bg-gray-800", md_option
        ),
        Node::ThematicBreak(..) => filter_attrs("h-px bg-slate-200 dark:bg-slate-700 border-0 my-1.5", md_option),
        Node::Table(..) => filter_attrs("w-full text-sm text-left rtl:text-right text-gray-500 dark:text-gray-400", md_option),
        Node::TableRow(..) => match index {
            Some(0) => {
                filter_attrs("text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400", md_option)
            }
            _ => {
                filter_attrs("bg-white border-b dark:bg-gray-800 dark:border-gray-700 border-gray-200 hover:bg-gray-50 dark:hover:bg-gray-600", md_option)
            }
        },
        Node::TableCell(..) => filter_attrs("px-6 py-3", md_option),
        _ => filter_attrs("", md_option),
    }
}

pub fn md_to_html(md: &Node, sup: Option<&Node>, md_option: &Option<MDOption>, index: Option<usize>) -> Option<HTMLNode> {
    match md {
        Node::Root(node) => Some(HTMLNode {
            tag: HTMLTag::Body,
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option, None))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option, None))]),
            value: "".to_string(),
        }),
        Node::Heading(node) => Some(HTMLNode {
            tag: HTMLTag::header_by(node.depth),
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option, None))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option, None))]),
            value: "".to_string(),
        }),
        Node::Blockquote(node) => Some(HTMLNode {
            tag: HTMLTag::Blockquote,
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option, None))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option, None))]),
            value: "".to_string(),
        }),
        Node::Paragraph(node) => Some(HTMLNode {
            tag: HTMLTag::P,
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option, None))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option, None))]),
            value: String::from(""),
        }),
        Node::Text(node) => Some(HTMLNode {
            tag: HTMLTag::Text,
            children: vec![],
            attributes: HashMap::from([("class", md_class(md, sup, md_option, None))]),
            value: String::from(&node.value.replace("\n", "<br />")),
        }),
        Node::Strong(node) => Some(HTMLNode {
            tag: HTMLTag::Strong,
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option, None))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option, None))]),
            value: String::from(""),
        }),
        Node::Emphasis(node) => Some(HTMLNode {
            tag: HTMLTag::EM,
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option, None))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option, None))]),
            value: String::from(""),
        }),
        Node::Code(code) => Some(HTMLNode {
            tag: HTMLTag::PRE,
            children: vec![HTMLNode {
                tag: HTMLTag::Code,
                children: vec![],
                attributes: match &code.lang {
                    None => HashMap::new(),
                    Some(lang) => HashMap::from([("class", format!("language-{}", lang))]),
                },
                value: String::from(&code.value),
            }],
            attributes: HashMap::from([("class", md_class(md, sup, md_option, None))]),
            value: "".to_string(),
        }),
        Node::List(node) => Some(HTMLNode {
            tag: HTMLTag::ordered(md),
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option, None))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option, None))]),
            value: "".to_string(),
        }),
        Node::ListItem(node) => Some(HTMLNode {
            tag: HTMLTag::LI,
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option, None))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option, None))]),
            value: "".to_string(),
        }),
        Node::Link(node) => Some(HTMLNode {
            tag: HTMLTag::A,
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option, None))
                .collect(),
            attributes: HashMap::from([
                ("class", md_class(md, sup, md_option, None)),
                ("href", node.url.to_string()),
            ]),
            value: "".to_string(),
        }),
        Node::Image(node) => Some(HTMLNode {
            tag: HTMLTag::IMG,
            children: vec![],
            attributes: HashMap::from([
                ("class", md_class(md, sup, md_option, None)),
                ("src", format!("{}", node.url.to_string())),
                ("alt", node.alt.to_string()),
                (
                    "title",
                    match &node.title {
                        None => "".to_string(),
                        Some(title) => title.to_string(),
                    },
                ),
            ]),
            value: "".to_string(),
        }),
        Node::ThematicBreak(node) => Some(HTMLNode {
            tag: HTMLTag::HR,
            children: vec![],
            attributes: HashMap::from([("class", md_class(md, sup, md_option, None))]),
            value: String::from(""),
        }),
        Node::Table(node) => Some(HTMLNode {
            tag: HTMLTag::Table,
            children: node
                .children
                .iter()
                .enumerate()
                .filter_map(|(index, x)| md_to_html(x, Some(md), md_option, Some(index)))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option, None))]),
            value: "".to_string(),
        }),
        Node::TableRow(node) => Some(HTMLNode {
            tag: HTMLTag::TR,
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option, None))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option, index))]),
            value: "".to_string(),
        }),
        Node::TableCell(node) => Some(HTMLNode {
            tag: HTMLTag::TD,
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option, None))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option, None))]),
            value: "".to_string()
        }),
        _ => None,
    }
}
