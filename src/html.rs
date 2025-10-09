use core::fmt;
use std::collections::HashMap;
use std::fmt::Formatter;
use maplit::hashmap;
use crate::option::MDOption;
use std::fs::File;
use std::io::Write;
use std::path::Path;


pub fn filter_attrs(text: &str, is_dark: bool) -> String {
    if !is_dark {
        let result = text
            .split_whitespace() 
            .filter(|&word| !(word.starts_with("dark:") || word.starts_with("md:dark:")))
            .collect::<Vec<&str>>()
            .join(" "); 
        return result
    }
    text.to_string()
}

impl fmt::Display for HTMLView {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} - {:?}", self.tag, self.attrs).unwrap();
        Ok(())
    }
}

impl fmt::Debug for HTMLView {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?} - {:?}", self.tag, self.attrs).unwrap();
        for view in &self.views {
            write!(f, "{:?}", view).unwrap();
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct HTMLView {
    pub tag: String,
    pub attrs: HashMap<String, String>,
    pub value: String,
    pub views: Vec<HTMLView>,
}

impl HTMLView {
    pub fn zero() -> HTMLView {
        HTMLView {
            tag: "div".to_string(),
            attrs: HashMap::new(),
            value: "".to_string(),
            views: vec![],
        }
    }

    pub fn new(tag: &str, attrs: HashMap<String, String>, value: &str, views: Vec<HTMLView>) -> HTMLView {
        HTMLView { tag: tag.to_string(), attrs, value: value.to_string(), views }
    }

    pub fn html(&self) -> String {
        let attr_string = self
            .attrs
            .iter()
            .map(|(k, v)| format!(r#"{}="{}""#, k, v))
            .collect::<Vec<_>>()
            .join(" ");

        let children_html = self
            .views
            .iter()
            .map(|child| format!("{}", child.html()))
            .collect::<Vec<_>>()
            .join("");

        let start_tag = if attr_string.is_empty() {
            format!("<{}>", self.tag)
        } else {
            format!("<{} {}>", self.tag, attr_string)
        };

        let sub_value = children_html
            .replace("\n", "\n\t");
        let sub_value = sub_value.trim_end_matches(&['\n', '\t'][..]);
        if sub_value.is_empty() && self.value.is_empty() {
            format!(
                "{}</{}>\n",
                start_tag,
                self.tag
            )
        } else {
            format!(
                "{}\n{}\t{}\n</{}>\n",
                start_tag,
                self.value,
                sub_value,
                self.tag
            )
        }
    }

    pub fn wrap_body(&self, is_dark: bool) -> HTMLView {
        let self_view = self.clone();
        let body_view = HTMLView {
            tag: "body".to_string(),
            attrs: hashmap! {
                "class".to_string() => filter_attrs("mx-auto bg-white w-screen h-screen dark:bg-slate-900", is_dark),
            },
            value: "".to_string(),
            views: vec![self_view]
        };
        let head_view = metas(is_dark);
        let html_view = HTMLView {
            tag: "html".to_string(),
            attrs: hashmap! {},
            value: "".to_string(),
            views: vec![head_view, body_view]
        };
        html_view
    }

    pub fn wrap_href(&self, path: String) -> HTMLView {
        if path.is_empty() {
            return self.clone();
        }
        let self_view = self.clone();
        let href_view = HTMLView {
            tag: "a".to_string(),
            attrs: hashmap! {
                "href".to_string() => format!("{}", path)
            },
            value: "".to_string(),
            views: vec![self_view]
        };
        href_view
    }

    pub fn wrap_div(&self, attrs: HashMap<String, String>) -> HTMLView {
        self.wrap_tag("div", attrs)
    }

    pub fn wrap_tag(&self, tag: &str, attrs: HashMap<String, String>) -> HTMLView {
        let self_view = self.clone();
        let div_view = HTMLView {
            tag: tag.to_string(),
            attrs: attrs,
            value: "".to_string(),
            views: vec![self_view]
        };
        div_view
    }

    pub fn inflate_view(&mut self, key: &str, inflating_view: HTMLView) -> HTMLView {
        for view in &mut self.views {
            if view.attrs.get("id").map(|s| s.as_str()) == Some(key) {
                view.views.push(inflating_view.clone());
            }
        }
        self.clone() // 수정된 self 를 반환
    }

    pub fn insert_header_footer(&mut self, header: Option<HTMLView>, footer: Option<HTMLView>) -> Result<(), String> {
        // body 노드 찾기
        let body = self.find_body_mut().ok_or("body tag not found")?;

        if let Some(h) = header {
            body.views.insert(0, h);
        }
        if let Some(f) = footer {
            body.views.push(f);
        }

        Ok(())
    }

    fn find_body_mut(&mut self) -> Option<&mut HTMLView> {
        if self.tag == "body" {
            return Some(self);
        }
        for child in &mut self.views {
            if let Some(found) = child.find_body_mut() {
                return Some(found);
            }
        }
        None
    }
}

pub fn metas(is_dark: bool) -> HTMLView {
    let charset = HTMLView::new("meta", hashmap! {
        "charset".to_string() => "UTF-8".to_string(),
    }, "", vec![]);
    let viewport = HTMLView::new("meta", hashmap! {
        "name".to_string() => "viewport".to_string(),
        "content".to_string() => "width=device-width, initial-scale=1.0".to_string()
    }, "", vec![]);
    let highlight_dark = HTMLView::new("link", hashmap! {
        "rel".to_string() => "stylesheet".to_string(),
        "href".to_string() => format!("https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/atom-one-{}.min.css", if (is_dark) { "dark" } else { "light" }),
    }, "", vec![]);
    let highlight = HTMLView::new("script", hashmap! {
        "src".to_string() => "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js".to_string(),
    }, "", vec![]);
    let highlight_go = HTMLView::new("script", hashmap! {
        "src".to_string() => "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/go.min.js".to_string(),
    }, "", vec![]);
    let highlight_all = HTMLView::new("script", hashmap! {}, "hljs.highlightAll();", vec![]);
    let tailwind = HTMLView::new("script", hashmap! {
        "src".to_string() => "https://cdn.tailwindcss.com".to_string(),
    }, "", vec![]);
    let flowbite = HTMLView::new("script", hashmap! {
        "src".to_string() => "https://flowbite.com/docs/flowbite.min.js?v=3.1.2a".to_string(),
    }, "", vec![]);
    HTMLView::new("head", hashmap! {}, "", vec![charset, viewport, highlight_dark, highlight, highlight_go, highlight_all, tailwind, flowbite])
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
        let mut is_night = false;
        if let Some(option) = md_option {
            is_night = option.is_night();
        }
        match self {
            HTMLTag::Body => filter_attrs("container mx-auto bg-white dark:bg-black", is_night),
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
    let mut is_night = false;
    if let Some(option) = md_option {
        is_night = option.is_night();
    }
    HTMLNode::from_attributes_children(HTMLTag::Footer, HashMap::from([
        ("class", filter_attrs("bg-white dark:bg-gray-900", is_night)),
    ]), vec![
        HTMLNode::from_attributes_children(HTMLTag::Div, HashMap::from([
            ("class", filter_attrs("px-4 py-6 bg-gray-100 dark:bg-gray-700 md:flex md:items-center md:justify-between", is_night)),
        ]), vec![
            HTMLNode::from_attributes_value(HTMLTag::Span, HashMap::from([
                ("class", filter_attrs("text-sm text-gray-500 dark:text-gray-300 sm:text-center", is_night)),
            ]), "Auto generated by <a href=\"https://rubinsharks.github.io\">rubinsharks.github.io</a>".to_string(), md_option)
        ], md_option)
    ], md_option)
}

fn write_html(html_path: &Path, md_option: &Option<MDOption>, html: &String) -> Result<(), String> {
    let mut is_night = false;
    if let Some(option) = md_option {
        is_night = option.is_night();
    }
    
    let mut head = HTMLNode::new(HTMLTag::Head, &md_option);
    let mut file = File::create(html_path).ok().ok_or("create fails")?;

    file.write(head.html(0).as_bytes())
        .ok()
        .ok_or("head write fails")?;
    file.write(format!("\n").as_bytes()).ok().ok_or("")?;
    file.write(
        format!(
            "<body class=\"{}\">\n",
            filter_attrs("container mx-auto bg-white dark:bg-slate-900", is_night)
        )
        .as_bytes(),
    )
    .ok()
    .ok_or("")?;
    if let Some(option) = &md_option {
        file.write(format!("{}", option.menus_to_html().as_str()).as_bytes())
            .ok()
            .ok_or("")?;
    }

    file.write_all(r#"<div class="max-w-screen-xl mx-auto px-4">"#.as_bytes()).ok().ok_or("")?;
    file.write_all(r#"<div class="left-0 h-2 w-full"></div>"#.as_bytes())
    .ok()
    .ok_or("all fails")?;
    file.write_all(html.as_bytes()).ok().ok_or("all fails")?;
    file.write_all(r#"</div>"#.as_bytes()).ok().ok_or("")?;
    file.write(format!("\n").as_bytes()).ok().ok_or("")?;
    file.write_all(r#"<div class="left-0 h-20 w-full"><br></div>"#.as_bytes())
        .ok()
        .ok_or("all fails")?;
    file.write(format!("\n").as_bytes()).ok().ok_or("")?;
    if let Some(option) = &md_option {
        file.write(format!("{}", option.footer_to_html().as_str()).as_bytes())
            .ok()
            .ok_or("")?;
    }
    file.write(format!("\n").as_bytes()).ok().ok_or("")?;
    file.write(format!("</body>").as_bytes()).ok().ok_or("")?;

    Ok(())
}