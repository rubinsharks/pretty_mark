use std::collections::HashMap;
use markdown::mdast::Node;
use crate::parser::option::MDOption;
use crate::parser::markdown::filter_attrs;

pub fn headers_highlight(md_option: &Option<MDOption>) -> Vec<HTMLNode> {
    let mut is_night = false;
    match md_option {
        Some(option) => {
            is_night = option.is_night()
        }
        None => {}
    }
    Vec::from([
        HTMLNode::from_attributes(HTMLTag::Meta, HashMap::from([
            ("charset", String::from("UTF-8")),
        ]), md_option),
        HTMLNode::from_attributes(HTMLTag::Meta, HashMap::from([
            ("name", String::from("viewport")),
            ("content", String::from("width=device-width, initial-scale=1.0")),
        ]), md_option),
        HTMLNode::from_attributes(HTMLTag::Link, HashMap::from([
            ("rel", String::from("stylesheet")),
            ("href", format!("https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/{}.min.css", if is_night { "atom-one-dark" } else { "atom-one-light" })),
        ]), md_option),
        HTMLNode::from_attributes(HTMLTag::Script, HashMap::from([
            ("src", String::from("https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js")),
            ("value", String::from("")),
        ]), md_option),
        HTMLNode::from_attributes(HTMLTag::Script, HashMap::from([
            ("src", String::from("https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/go.min.js")),
            ("value", String::from("")),
        ]), md_option),
        HTMLNode::from_value(HTMLTag::Script, String::from("hljs.highlightAll();"), md_option),
        HTMLNode::from_attributes(HTMLTag::Script, HashMap::from([
            ("src", String::from("https://cdn.tailwindcss.com"))
        ]), md_option),
        HTMLNode::from_attributes(HTMLTag::Script, HashMap::from([
            ("src", String::from("https://flowbite.com/docs/flowbite.min.js?v=3.1.2a"))
        ]), md_option),
    ])
}

/// 사용가능한 HTML Tag
#[derive(Clone, Copy)]
pub enum HTMLTag {
    P,
    PRE,
    H1,
    H2,
    H3,
    Head,
    Body,
    Script,
    Link,
    Blockquote,
    Code,
    Text,
    UL,
    OL,
    LI,
    Strong,
    EM,
    IMG,
    A,
    Footer,
    Div,
    Span,
    Meta,
    HR,
    Table,
    THead,
    TR,
    TH,
    TBody,
    TD,
}

impl HTMLTag {
    pub fn header_by(depth: u8) -> Self {
        match depth {
            1 => Self::H1,
            2 => Self::H2,
            3 => Self::H3,
            _ => Self::P
        }
    }

    pub fn ordered(start: &Node) -> Self {
        match start {
            Node::List(..) => Self::OL,
            _ => Self::UL,
        }
    }
}

pub struct HTMLNode {
    pub tag: HTMLTag,
    pub children: Vec<HTMLNode>,
    pub attributes: HashMap<&'static str, String>,
    pub value: String
}

impl HTMLNode {
    pub fn new(tag: HTMLTag, md_option: &Option<MDOption>) -> HTMLNode {
        HTMLNode {
            tag,
            children: Vec::new(),
            attributes: HashMap::from([
                ("class", String::from(tag.class(md_option))),
            ]),
            value: String::from(""),
        }
    }

    pub fn from_attributes(tag: HTMLTag, attributes: HashMap<&'static str, String>, md_option: &Option<MDOption>) -> HTMLNode {
        HTMLNode {
            tag,
            attributes,
            ..HTMLNode::new(tag, md_option)
        }
    }

    pub fn from_value(tag: HTMLTag, value: String, md_option: &Option<MDOption>) -> HTMLNode {
        HTMLNode {
            tag,
            value,
            ..HTMLNode::new(tag, md_option)
        }
    }

    pub fn from_children(tag: HTMLTag, children: Vec<HTMLNode>, md_option: &Option<MDOption>) -> HTMLNode {
        HTMLNode {
            tag,
            children,
            ..HTMLNode::new(tag, md_option)
        }
    }

    pub fn from_attributes_children(tag: HTMLTag, attributes: HashMap<&'static str, String>, children: Vec<HTMLNode>, md_option: &Option<MDOption>) -> HTMLNode {
        HTMLNode {
            tag,
            children,
            attributes,
            ..HTMLNode::new(tag, md_option)
        }
    }

    pub fn from_attributes_value(tag: HTMLTag, attributes: HashMap<&'static str, String>, value: String, md_option: &Option<MDOption>) -> HTMLNode {
        HTMLNode {
            tag,
            attributes,
            value,
            ..HTMLNode::new(tag, md_option)
        }
    }

    pub fn html(&self, depth: usize) -> String {
        let mut str = String::new();
        let tag = self.tag.tag();
        if !tag.is_empty() {
            if tag != "body" {
                str.push_str(format!("<{}", self.tag.tag()).as_str());
                for (key, value) in &self.attributes {
                    if !value.is_empty() {
                        str.push_str(format!(" {}=", key).as_str());
                        str.push_str(format!("\"{}\"",value).as_str());
                    }
                }
                str.push_str(">");
            }

            if self.children.is_empty() {
                str.push_str(format!("{}", self.value).as_str());
            } else {
                for child in &self.children {
                    str.push_str("\n");
                    str.push_str("\t");
                    (0..depth).for_each(|x| str.push_str("\t"));
                    str.push_str(child.html(depth + 1).as_str());
                }
                str.push_str("\n");
                (0..depth).for_each(|x| str.push_str("\t"));
            }
            if tag != "body" {
                str.push_str(format!("</{}>", self.tag.tag()).as_str());
            }
        } else {
            str.push_str(format!("{}", self.value).as_str());
        }
        str
    }
}

impl HTMLTag {

    pub fn class(&self, md_option: &Option<MDOption>) -> String {
        match self {
            HTMLTag::Body => filter_attrs("container mx-auto bg-white dark:bg-black", md_option),
            _ => "".to_string()
        }
    }
    pub fn tag(&self) -> &'static str {
        match self {
            HTMLTag::P => "p",
            HTMLTag::PRE => "pre",
            HTMLTag::H1 => "h1",
            HTMLTag::Script => "script",
            HTMLTag::Link => "link",
            HTMLTag::Head => "head",
            HTMLTag::Body => "body",
            HTMLTag::H2 => "h2",
            HTMLTag::H3 => "h3",
            HTMLTag::Blockquote => "blockquote",
            HTMLTag::Code => "code",
            HTMLTag::UL => "ul",
            HTMLTag::OL => "ol",
            HTMLTag::LI => "li",
            HTMLTag::Strong => "strong",
            HTMLTag::EM => "em",
            HTMLTag::IMG => "img",
            HTMLTag::A => "a",
            HTMLTag::Footer => "footer",
            HTMLTag::Div => "div",
            HTMLTag::Span => "span",
            HTMLTag::Meta => "meta",
            HTMLTag::HR => "hr",
            HTMLTag::Table => "table",
            HTMLTag::THead => "thead",
            HTMLTag::TR => "tr",
            HTMLTag::TH => "th",
            HTMLTag::TBody => "tbody",
            HTMLTag::TD => "td",
            _ => { "" }
        }
    }
}

pub fn footer(md_option: &Option<MDOption>) -> HTMLNode {
    HTMLNode::from_attributes_children(HTMLTag::Footer, HashMap::from([
        ("class", filter_attrs("bg-white dark:bg-gray-900", md_option)),
    ]), vec![
        HTMLNode::from_attributes_children(HTMLTag::Div, HashMap::from([
            ("class", filter_attrs("px-4 py-6 bg-gray-100 dark:bg-gray-700 md:flex md:items-center md:justify-between", md_option)),
        ]), vec![
            HTMLNode::from_attributes_value(HTMLTag::Span, HashMap::from([
                ("class", filter_attrs("text-sm text-gray-500 dark:text-gray-300 sm:text-center", md_option)),
            ]), "Auto generated by <a href=\"https://rubinsharks.github.io\">rubinsharks.github.io</a>".to_string(), md_option)
        ], md_option)
    ], md_option)
}