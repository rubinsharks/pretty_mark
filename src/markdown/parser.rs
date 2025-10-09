use crate::html::{filter_attrs, HTMLView};
use crate::html::{HTMLNode, HTMLTag};
use maplit::hashmap;
use markdown::mdast::Node;
use markdown::message::Message;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::option::ThemeValue;
use crate::option::MDOption;

use super::common::remove_frontmatter;

pub fn get_node_for_markdown(index_path: &Path) -> Result<Node, &'static str> {
    let mut file = File::open(&index_path).ok().ok_or("open fails")?;
    let mut markdown_contents = String::new();
    file.read_to_string(&mut markdown_contents).ok().ok_or("read fails")?;
    let markdown_contents_removed_frontmatter = remove_frontmatter(&markdown_contents);

    match markdown::to_mdast(&markdown_contents_removed_frontmatter, &markdown::ParseOptions::gfm()) {
        Ok(node) => Ok(node),
        Err(message) => Err("no file"),
    }
}

pub fn node_to_html(md: &Node, sup: Option<&Node>, index: Option<usize>, is_dark: bool) -> HTMLView {
    match md {
        Node::Root(node) => HTMLView {
            tag: "div".to_string(),
            attrs: hashmap! {
                "class".to_string() => filter_attrs("w-full h-full dark:bg-slate-900", is_dark)
            },
            value: "".to_string(),
            views: node
                .children
                .iter()
                .map(|x| node_to_html(x, Some(md), None, is_dark))
                .collect(),
        },
        Node::Heading(node) => HTMLView {
            tag: match node.depth {
                1 => "h1".to_string(),
                2 => "h2".to_string(),
                3 => "h3".to_string(),
                _ => "p".to_string()
            },
            attrs: hashmap! {
                "class".to_string() => match node.depth {
                    1 => filter_attrs("text-4xl font-bold mb-8 mt-5 text-slate-900 dark:text-white", is_dark),
                    2 => filter_attrs("text-3xl font-bold mb-6 mt-5 text-slate-900 dark:text-white", is_dark),
                    3 => filter_attrs("text-2xl font-bold mb-4 mt-5 text-slate-900 dark:text-white", is_dark),
                    4 => filter_attrs("text-xl font-bold mb-3 mt-5 text-slate-900 dark:text-white", is_dark),
                    5 => filter_attrs("text-base font-bold mb-3 mt-5 text-slate-900 dark:text-white", is_dark),
                    6 => filter_attrs("text-sm font-bold mb-3 mt-5 text-slate-900 dark:text-white", is_dark),
                    _ => filter_attrs("", is_dark),
                },
            },
            value: "".to_string(),
            views: node
                .children
                .iter()
                .map(|x| node_to_html(x, Some(md), None, is_dark))
                .collect(),
        },
        Node::List(node) => HTMLView {
            tag: match md {
                Node::List(..) => "ol".to_string(),
                _ => "ul".to_string(),
            },
            attrs: hashmap! {
                "class".to_string() => match node.start {
                    None => filter_attrs("list-disc pl-5 text-slate-500 dark:text-slate-400", is_dark),
                    Some(_) => filter_attrs("list-decimal pl-5 text-slate-500 dark:text-slate-400", is_dark),
                },
            },
            value: "".to_string(),
            views: node
                .children
                .iter()
                .map(|x| node_to_html(x, Some(md), None, is_dark))
                .collect(),
        },
        Node::ListItem(node) => HTMLView {
            tag: "li".to_string(),
            attrs: hashmap! {
            },
            value: "".to_string(),
            views: node
                .children
                .iter()
                .map(|x| node_to_html(x, Some(md), None, is_dark))
                .collect(),
        },
        Node::Blockquote(node) => HTMLView {
            tag: "blockquote".to_string(),
            attrs: hashmap! {
                "class".to_string() => filter_attrs(
                    "p-4 my-4 border-s-4 border-gray-300 bg-gray-50 dark:border-gray-500 dark:bg-gray-800", is_dark
                ),
            },
            value: "".to_string(),
            views: node
                .children
                .iter()
                .map(|x| node_to_html(x, Some(md), None, is_dark))
                .collect(),
        },
        Node::Paragraph(node) => HTMLView {
            tag: "p".to_string(),
            attrs: hashmap! {
                "class".to_string() => match sup {
                    None => String::from(""),
                    Some(node) => match node {
                        Node::Blockquote(..) => filter_attrs(
                            "text-xl italic font-medium leading-relaxed text-gray-900 dark:text-white", is_dark
                        ),
                        _ => filter_attrs("text-slate-500 dark:text-slate-400 mt-1", is_dark),
                    },
                }
            },
            value: "".to_string(),
            views: node
                .children
                .iter()
                .map(|x| node_to_html(x, Some(md), None, is_dark))
                .collect(),
        },
        Node::Text(node) => HTMLView {
            tag: "span".to_string(),
            attrs: hashmap! {
            },
            value: String::from(&node.value.replace("\n", "<br />")),
            views: vec![],
        },
        Node::Strong(node) => HTMLView {
            tag: "strong".to_string(),
            attrs: hashmap! {
            },
            value: "".to_string(),
            views: node
                .children
                .iter()
                .map(|x| node_to_html(x, Some(md), None, is_dark))
                .collect(),
        },
        Node::Emphasis(node) => HTMLView {
            tag: "em".to_string(),
            attrs: hashmap! {
            },
            value: "".to_string(),
            views: node
                .children
                .iter()
                .map(|x| node_to_html(x, Some(md), None, is_dark))
                .collect(),
        },
        Node::Code(node) => HTMLView {
            tag: "pre".to_string(),
            attrs: hashmap! {
            },
            value: "".to_string(),
            views: vec![
                HTMLView {
                    tag: "code".to_string(),
                    attrs: match &node.lang {
                        None => hashmap! {},
                        Some(lang) => hashmap! { "class".to_string() => format!("language-{}", lang) },
                    },
                    value: node.value.to_string(),
                    views: vec![],
                },
            ]
        },
        Node::Link(node) => HTMLView {
            tag: "a".to_string(),
            attrs: hashmap! {
                "href".to_string() => node.url.to_string(),
            },
            value: "".to_string(),
            views: node
                .children
                .iter()
                .map(|x| node_to_html(x, Some(md), None, is_dark))
                .collect(),
        },
        Node::Image(node) => HTMLView {
            tag: "img".to_string(),
            attrs: hashmap! {
                "src".to_string() => format!("{}", node.url.to_string()),
                "alt".to_string() => node.alt.to_string(),
                "title".to_string() => match &node.title {
                    None => "".to_string(),
                    Some(title) => title.to_string(),
                },
            },
            value: "".to_string(),
            views: vec![],
        },
        Node::ThematicBreak(node) => HTMLView {
            tag: "hr".to_string(),
            attrs: hashmap! {
                "class".to_string() => filter_attrs("h-px bg-slate-200 dark:bg-slate-700 border-0 my-1.5", is_dark),
            },
            value: "".to_string(),
            views: vec![],
        },
        Node::Table(node) => HTMLView {
            tag: "table".to_string(),
            attrs: hashmap! {
                "class".to_string() => filter_attrs("w-full text-sm text-left rtl:text-right text-gray-500 dark:text-gray-400", is_dark),
            },
            value: "".to_string(),
            views: node
                .children
                .iter()
                .enumerate()
                .map(|(index, x)| node_to_html(x, Some(md), Some(index), is_dark))
                .collect(),
        },
        Node::TableRow(node) => HTMLView {
            tag: "tr".to_string(),
            attrs: hashmap! {
                "class".to_string() => match index {
                    Some(0) => {
                        filter_attrs("text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400", is_dark)
                    }
                    _ => {
                        filter_attrs("bg-white border-b dark:bg-gray-800 dark:border-gray-700 border-gray-200 hover:bg-gray-50 dark:hover:bg-gray-600", is_dark)
                    }
                },
            },
            value: "".to_string(),
            views: node
                .children
                .iter()
                .map(|x| node_to_html(x, Some(md), None, is_dark))
                .collect(),
        },
        Node::TableCell(node) => HTMLView {
            tag: "td".to_string(),
            attrs: hashmap! {
                "class".to_string() => filter_attrs("px-6 py-3", is_dark),
            },
            value: "".to_string(),
            views: node
                .children
                .iter()
                .map(|x| node_to_html(x, Some(md), None, is_dark))
                .collect(),
        },
        _ => HTMLView::zero(),
    }
}