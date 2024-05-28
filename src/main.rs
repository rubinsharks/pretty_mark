use std::any::{Any, type_name};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Display, Path};
use markdown::mdast::{Heading, Node};
use markdown::message::Message;

fn main() -> std::io::Result<()> {
    let mut file = File::create("rust.html")?;

    let mut head = HTMLNode::new(HTMLTag::Head);
    let headers = headers_highlight();
    head.children = headers;

    let path = Path::new("root/rust.md");
    let html = match parser(path) {
        Ok(node) => {
            println!("{:#?}", node);
            match md_to_html(&node, None) {
                None => { String::from("1") }
                Some(node) => { node.html() }
            }
        }
        Err(_) => { String::from("2") }
    };

    file.write(head.html().as_bytes());
    file.write(format!("\n").as_bytes());
    file.write_all(html.as_bytes())?;
    Ok(())
}

fn headers_highlight() -> Vec<HTMLNode> {
    Vec::from([
        HTMLNode::from_attributes(HTMLTag::Link, HashMap::from([
            ("rel", String::from("stylesheet")),
            ("href", String::from("https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/default.min.css")),
        ])),
        HTMLNode::from_attributes(HTMLTag::Script, HashMap::from([
            ("src", String::from("https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js")),
            ("value", String::from("")),
        ])),
        HTMLNode::from_attributes(HTMLTag::Script, HashMap::from([
            ("src", String::from("https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/go.min.js")),
            ("value", String::from("")),
        ])),
        HTMLNode::from_value(HTMLTag::Script, String::from("hljs.highlightAll();")),
        HTMLNode::from_attributes(HTMLTag::Script, HashMap::from([
            ("src", String::from("https://cdn.tailwindcss.com"))
        ])),
    ])
}

fn parser(path: &Path) -> Result<Node, Message> {
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => {}
    }

    // let mdast = markdown::to_mdast("# Hey, *you*!", &markdown::ParseOptions::default());
    // println!(
    //     "{:#?}",
    //     markdown::to_mdast(&s, &markdown::ParseOptions::default())
    // );

    // markdown::to_html(&s)
    markdown::to_mdast(&s, &markdown::ParseOptions::default())
}

#[derive(Clone, Copy)]
enum HTMLTag {
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
}

impl HTMLTag {
    fn header_by(depth: u8) -> Self {
        match depth {
            1 => Self::H1,
            2 => Self::H2,
            3 => Self::H3,
            _ => Self::P
        }
    }
}

pub struct HTMLNode {
    tag: HTMLTag,
    children: Vec<HTMLNode>,
    attributes: HashMap<&'static str, String>,
    value: String
}

fn md_class(md: &Node, sup: Option<&Node>) -> String {
    match md {
        Node::Root(..) => String::from("container mx-auto"),
        Node::Paragraph(..) => {
            match sup {
                None => String::from(""),
                Some(node) => {
                    match node {
                        Node::BlockQuote(..) => String::from("text-xl italic font-medium leading-relaxed text-gray-900 dark:text-white"),
                        _ => String::from("text-gray-800 dark:text-gray-800")
                    }
                }
            }
        }
        Node::Heading(value) => {
            match value.depth {
                1 => String::from("text-4xl font-bold mb-3"),
                2 => String::from("text-3xl font-bold mb-3"),
                3 => String::from("text-2xl font-bold mb-3"),
                _ => String::from(""),
            }
        }
        Node::BlockQuote(..) => String::from("p-4 my-4 border-s-4 border-gray-300 bg-gray-50 dark:border-gray-500 dark:bg-gray-800"),
        _ => String::from(""),
    }
}

fn md_to_html(md: &Node, sup: Option<&Node>) -> Option<HTMLNode> {
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
        _ => {
            None
        }
    }
}

impl HTMLNode {

    fn new(tag: HTMLTag) -> HTMLNode {
        HTMLNode {
            tag,
            children: Vec::new(),
            attributes: HashMap::from([
                ("class", String::from(tag.class())),
            ]),
            value: String::from(""),
        }
    }

    fn from_attributes(tag: HTMLTag, attributes: HashMap<&'static str, String>) -> HTMLNode {
        HTMLNode {
            tag,
            attributes,
            ..HTMLNode::new(tag)
        }
    }

    fn from_value(tag: HTMLTag, value: String) -> HTMLNode {
        HTMLNode {
            tag,
            value,
            ..HTMLNode::new(tag)
        }
    }

    fn from_children(tag: HTMLTag, children: Vec<HTMLNode>) -> HTMLNode {
        HTMLNode {
            tag,
            children,
            ..HTMLNode::new(tag)
        }
    }

    fn html(&self) -> String {
        let mut str = String::new();
        let tag = self.tag.tag();
        if !tag.is_empty() {
            str.push_str(format!("<{}", self.tag.tag()).as_str());
            for (key, value) in &self.attributes {
                if !value.is_empty() {
                    str.push_str(format!(" {}=", key).as_str());
                    str.push_str(format!("\"{}\"",value).as_str());
                }
            }
            str.push_str(">");

            if self.children.is_empty() {
                str.push_str(format!("{}", self.value).as_str());
            } else {
                for child in &self.children {
                    str.push_str("\n\t");
                    str.push_str(child.html().as_str());
                }
                str.push_str("\n");
            }
            str.push_str(format!("</{}>", self.tag.tag()).as_str());
        } else {
            str.push_str(format!("{}", self.value).as_str());
        }
        str
    }
}

impl HTMLTag {

    fn class(&self) -> &'static str {
        match self {
            HTMLTag::Body => "container mx-auto",
            _ => ""
        }
    }
    fn tag(&self) -> &'static str {
        match self {
            HTMLTag::P => { "p" }
            HTMLTag::PRE => { "pre" }
            HTMLTag::H1 => { "h1" }
            HTMLTag::Script => { "script" }
            HTMLTag::Link => { "link" }
            HTMLTag::Head => { "head" }
            HTMLTag::Body => { "body" }
            HTMLTag::H2 => { "h2" }
            HTMLTag::H3 => { "h3" }
            HTMLTag::Blockquote => { "blockquote" }
            HTMLTag::Code => { "code" }
            HTMLTag::Text => { "" }
        }
    }
}