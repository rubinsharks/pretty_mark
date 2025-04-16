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

    match markdown::to_mdast(&s, &markdown::ParseOptions::default()) {
        Ok(node) => Ok(node),
        Err(message) => Err("no file"),
    }
}

pub fn filter_attrs(text: &str, md_option: &Option<MDOption>) -> String {
    // "dark:"로 시작하는 모든 부분을 제거
    if let Some(md_option) = md_option {
        if !md_option.is_night() {
            let result = text
                .split_whitespace() // 공백으로 구분하여
                .filter(|&word| !(word.starts_with("dark:") || word.starts_with("md:dark:")))
                .collect::<Vec<&str>>()
                .join(" "); // 다시 공백으로 합친다
            return result
        }
    }
    text.to_string()
}

pub fn md_class(md: &Node, sup: Option<&Node>, md_option: &Option<MDOption>) -> String {
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
            1 => filter_attrs("text-4xl font-bold mb-3 mt-5 text-slate-900 dark:text-white", md_option),
            2 => filter_attrs("text-3xl font-bold mb-3 mt-5 text-slate-900 dark:text-white", md_option),
            3 => filter_attrs("text-2xl font-bold mb-3 mt-5 text-slate-900 dark:text-white", md_option),
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
        _ => filter_attrs("", md_option),
    }
}

pub fn md_to_html(md: &Node, sup: Option<&Node>, md_option: &Option<MDOption>) -> Option<HTMLNode> {
    match md {
        Node::Root(node) => Some(HTMLNode {
            tag: HTMLTag::Body,
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option))]),
            value: "".to_string(),
        }),
        Node::Heading(node) => Some(HTMLNode {
            tag: HTMLTag::header_by(node.depth),
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option))]),
            value: "".to_string(),
        }),
        Node::Blockquote(node) => Some(HTMLNode {
            tag: HTMLTag::Blockquote,
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option))]),
            value: "".to_string(),
        }),
        Node::Paragraph(node) => Some(HTMLNode {
            tag: HTMLTag::P,
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option))]),
            value: String::from(""),
        }),
        Node::Text(node) => Some(HTMLNode {
            tag: HTMLTag::Text,
            children: vec![],
            attributes: HashMap::from([("class", md_class(md, sup, md_option))]),
            value: String::from(&node.value.replace("\n", "<br />")),
        }),
        Node::Strong(node) => Some(HTMLNode {
            tag: HTMLTag::Strong,
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option))]),
            value: String::from(""),
        }),
        Node::Emphasis(node) => Some(HTMLNode {
            tag: HTMLTag::EM,
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option))]),
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
            attributes: HashMap::from([("class", md_class(md, sup, md_option))]),
            value: "".to_string(),
        }),
        Node::List(node) => Some(HTMLNode {
            tag: HTMLTag::ordered(md),
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option))]),
            value: "".to_string(),
        }),
        Node::ListItem(node) => Some(HTMLNode {
            tag: HTMLTag::LI,
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option))
                .collect(),
            attributes: HashMap::from([("class", md_class(md, sup, md_option))]),
            value: "".to_string(),
        }),
        Node::Link(node) => Some(HTMLNode {
            tag: HTMLTag::A,
            children: node
                .children
                .iter()
                .filter_map(|x| md_to_html(x, Some(md), md_option))
                .collect(),
            attributes: HashMap::from([
                ("class", md_class(md, sup, md_option)),
                ("href", node.url.to_string()),
            ]),
            value: "".to_string(),
        }),
        Node::Image(node) => Some(HTMLNode {
            tag: HTMLTag::IMG,
            children: vec![],
            attributes: HashMap::from([
                ("class", md_class(md, sup, md_option)),
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
        _ => None,
    }
}
