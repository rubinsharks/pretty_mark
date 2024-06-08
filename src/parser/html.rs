use std::collections::HashMap;
use markdown::mdast::Node;

pub fn headers_highlight() -> Vec<HTMLNode> {
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
    pub fn new(tag: HTMLTag) -> HTMLNode {
        HTMLNode {
            tag,
            children: Vec::new(),
            attributes: HashMap::from([
                ("class", String::from(tag.class())),
            ]),
            value: String::from(""),
        }
    }

    pub fn from_attributes(tag: HTMLTag, attributes: HashMap<&'static str, String>) -> HTMLNode {
        HTMLNode {
            tag,
            attributes,
            ..HTMLNode::new(tag)
        }
    }

    pub fn from_value(tag: HTMLTag, value: String) -> HTMLNode {
        HTMLNode {
            tag,
            value,
            ..HTMLNode::new(tag)
        }
    }

    pub fn from_children(tag: HTMLTag, children: Vec<HTMLNode>) -> HTMLNode {
        HTMLNode {
            tag,
            children,
            ..HTMLNode::new(tag)
        }
    }

    pub fn html(&self, depth: usize) -> String {
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
                    str.push_str("\n");
                    str.push_str("\t");
                    (0..depth).for_each(|x| str.push_str("\t"));
                    str.push_str(child.html(depth + 1).as_str());
                }
                str.push_str("\n");
                (0..depth).for_each(|x| str.push_str("\t"));
            }
            str.push_str(format!("</{}>", self.tag.tag()).as_str());
        } else {
            str.push_str(format!("{}", self.value).as_str());
        }
        str
    }
}

impl HTMLTag {

    pub fn class(&self) -> &'static str {
        match self {
            HTMLTag::Body => "container mx-auto",
            _ => ""
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
            _ => { "" }
        }
    }
}