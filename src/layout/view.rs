use crate::html::{filter_attrs, HTMLView};
use crate::layout::nav::make_nav;
use crate::markdown::markdown_to_htmlview;
use crate::page::Page;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use toml_edit::InlineTable;
use toml_edit::{value, DocumentMut, Item, Table, Value};

use std::any::Any;

use super::common::{get_items, item_to_bool, layout_to_tomlview};
use super::common::item_to_string;
use super::common::item_to_strings;
use super::common::table_to_tomlview;
use super::padding::PaddingType;

pub trait TOMLView: Any {
    fn as_any(&self) -> &dyn Any;
    fn index_path(&self) -> PathBuf;
    fn shape(&self) -> String;
    fn key(&self) -> String;
    fn width(&self) -> String;
    fn height(&self) -> String;
    fn background(&self) -> String;
    fn path(&self) -> String;
    fn value(&self) -> Option<InlineTable>;
    fn dark(&self) -> bool;
    fn views(&self) -> &Vec<Box<dyn TOMLView>>;
    fn htmlview(&self, super_view: Option<&dyn TOMLView>) -> HTMLView;
}

/// column, row, box
trait AllocationView: TOMLView {
    fn fixed(&self) -> String;
    fn set_inner_padding(&mut self, value: String);
    fn set_outer_padding(&mut self, ptype: PaddingType, value: String);
}

pub struct ColumnView {
    index_path: PathBuf,
    key: String,
    width: String,
    height: String,
    background: String,
    path: String,
    is_scroll: bool,
    left_outer_padding: String,
    top_outer_padding: String,
    right_outer_padding: String,
    bottom_outer_padding: String,
    inner_padding: String,
    fixed: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl ColumnView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, is_scroll: bool) -> ColumnView {
        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);
        
        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);
        
        let mut view = ColumnView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "0px", value),
            height: item_to_string(&table, "height", "0px", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            is_scroll,
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            inner_padding: item_to_string(&table, "inner_padding", "0px", value),
            fixed: item_to_string(&table, "fixed", "", value),
            value: value.cloned(),
            dark: dark,
            views: vec![],
        };

        let views = table
            .iter()
            .filter_map(|(k, v)| {
                if let Item::Table(inner_table) = v {
                    match table_to_tomlview(index_path, k, inner_table, value, Some(&view)) {
                        Ok(view) => Some(view),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            })
            .collect();
        view.views = views;
        view
    }
}

impl TOMLView for ColumnView {
    fn index_path(&self) -> PathBuf {
        return self.index_path.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn shape(&self) -> String {
        if self.is_scroll {
            return "scroll_column".to_string();
        }
        return "column".to_string();
    }
    fn key(&self) -> String {
        return self.key.clone();
    }
    fn width(&self) -> String {
        return self.width.clone();
    }
    fn height(&self) -> String {
        return self.height.clone();
    }
    fn background(&self) -> String {
        return self.background.clone();
    }
    fn path(&self) -> String {
        return self.path.clone();
    }
    fn dark(&self) -> bool {
        return self.dark;
    }
    fn views(&self) -> &Vec<Box<dyn TOMLView>> {
        return &self.views;
    }
    fn htmlview(&self, super_view: Option<&dyn TOMLView>) -> HTMLView {
        let views = self.views.iter().map(|x| x.htmlview(Some(self))).collect();

        let mut style_parts = vec![
            format!("background:{}", self.background),
            format!(
                "padding:{} {} {} {}",
                self.top_outer_padding,
                self.right_outer_padding,
                self.bottom_outer_padding,
                self.left_outer_padding,
            ),
            "display:flex".to_string(),
            "flex-direction:column".to_string(),
            "flex-shrink: 0".to_string(),
        ];
        if self.width != "0px" {
            style_parts.push(format!("width:{}", self.width));
        }
        if self.height != "0px" {
            style_parts.push(format!("height:{}", self.height));
        }
        if self.is_scroll {
            style_parts.push("overflow-y: auto".to_string());
        } else {
            style_parts.push("overflow: hidden".to_string());
        }
        if !self.fixed.is_empty() {
            style_parts.push(format!("position: fixed; {}: 0", self.fixed));
        }

        if !self.inner_padding.trim().is_empty() {
            style_parts.push(format!("gap:{}", self.inner_padding));
        }

        let style = style_parts.join("; ") + ";"; // 끝에 세미콜론

        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), self.key.clone());
        attrs.insert("style".to_string(), style);

        HTMLView {
            tag: "div".to_string(),
            attrs: attrs,
            value: "".to_string(),
            views: views,
        }
        .wrap_href(self.path.clone())
    }
    fn value(&self) -> Option<InlineTable> {
        return self.value.clone();
    }
}

impl AllocationView for ColumnView {
    fn fixed(&self) -> String {
        return self.fixed.to_string();
    }
    fn set_inner_padding(&mut self, value: String) {
        self.inner_padding = value;
    }
    fn set_outer_padding(&mut self, ptype: PaddingType, value: String) {
        match ptype {
            PaddingType::LEFT => self.left_outer_padding = value,
            PaddingType::RIGHT => self.right_outer_padding = value,
            PaddingType::TOP => self.top_outer_padding = value,
            PaddingType::BOTTOM => self.bottom_outer_padding = value,
            PaddingType::HORIZONTAL => {
                self.left_outer_padding = value.clone();
                self.right_outer_padding = value;
            }
            PaddingType::VERTICAL => {
                self.top_outer_padding = value.clone();
                self.bottom_outer_padding = value;
            }
            PaddingType::ALL => {
                self.left_outer_padding = value.clone();
                self.right_outer_padding = value.clone();
                self.top_outer_padding = value.clone();
                self.bottom_outer_padding = value
            }
        }
    }
}

pub struct RowView {
    index_path: PathBuf,
    key: String,
    width: String,
    height: String,
    background: String,
    path: String,
    is_scroll: bool,
    left_outer_padding: String,
    top_outer_padding: String,
    right_outer_padding: String,
    bottom_outer_padding: String,
    inner_padding: String,
    fixed: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl RowView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, is_scroll: bool) -> RowView {

        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);

        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);
        
        let mut view = RowView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "0px", value),
            height: item_to_string(&table, "height", "0px", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            is_scroll,
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            inner_padding: item_to_string(&table, "inner_padding", "0px", value),
            fixed: item_to_string(&table, "fixed", "", value),
            value: value.cloned(),
            dark: dark,
            views: vec![],
        };
        let views = table
            .iter()
            .filter_map(|(k, v)| {
                if let Item::Table(inner_table) = v {
                    match table_to_tomlview(index_path, k, inner_table, value, Some(&view)) {
                        Ok(view) => Some(view),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            })
            .collect();
        view.views = views;
        view
    }
}

impl TOMLView for RowView {
    fn index_path(&self) -> PathBuf {
        return self.index_path.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn shape(&self) -> String {
        if self.is_scroll {
            return "scroll_row".to_string();
        }
        return "row".to_string();
    }
    fn key(&self) -> String {
        return self.key.clone();
    }
    fn width(&self) -> String {
        return self.width.clone();
    }
    fn height(&self) -> String {
        return self.height.clone();
    }
    fn background(&self) -> String {
        return self.background.clone();
    }
    fn path(&self) -> String {
        return self.path.clone();
    }
    fn dark(&self) -> bool {
        return self.dark;
    }
    fn views(&self) -> &Vec<Box<dyn TOMLView>> {
        return &self.views;
    }
    fn htmlview(&self, super_view: Option<&dyn TOMLView>) -> HTMLView {
        let views = self.views.iter().map(|x| x.htmlview(Some(self))).collect();

        let mut style_parts = vec![
            format!("background:{}", self.background),
            format!(
                "padding:{} {} {} {}",
                self.top_outer_padding,
                self.right_outer_padding,
                self.bottom_outer_padding,
                self.left_outer_padding,
            ),
            "display:flex".to_string(),
            "flex-direction:row".to_string(),
            "flex-shrink: 0".to_string(),
        ];
        if self.width != "0px" {
            style_parts.push(format!("width:{}", self.width));
        }
        if self.height != "0px" {
            style_parts.push(format!("height:{}", self.height));
        }
        if self.is_scroll {
            style_parts.push("overflow-x: auto".to_string());
        } else {
            style_parts.push("overflow: hidden".to_string());
        }
        if !self.inner_padding.trim().is_empty() {
            style_parts.push(format!("gap:{}", self.inner_padding));
        }
        if !self.fixed.is_empty() {
            style_parts.push(format!("position: fixed; {}: 0", self.fixed));
        }

        let style = style_parts.join("; ") + ";"; // 끝에 세미콜론

        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), self.key.clone());
        attrs.insert("style".to_string(), style);

        HTMLView {
            tag: "div".to_string(),
            attrs: attrs,
            value: "".to_string(),
            views: views,
        }
        .wrap_href(self.path.clone())
    }
    fn value(&self) -> Option<InlineTable> {
        return self.value.clone();
    }
}

impl AllocationView for RowView {
    fn fixed(&self) -> String {
        self.fixed.to_string()
    }
    fn set_inner_padding(&mut self, value: String) {
        self.inner_padding = value;
    }
    fn set_outer_padding(&mut self, ptype: PaddingType, value: String) {
        match ptype {
            PaddingType::LEFT => self.left_outer_padding = value,
            PaddingType::RIGHT => self.right_outer_padding = value,
            PaddingType::TOP => self.top_outer_padding = value,
            PaddingType::BOTTOM => self.bottom_outer_padding = value,
            PaddingType::HORIZONTAL => {
                self.left_outer_padding = value.clone();
                self.right_outer_padding = value;
            }
            PaddingType::VERTICAL => {
                self.top_outer_padding = value.clone();
                self.bottom_outer_padding = value;
            }
            PaddingType::ALL => {
                self.left_outer_padding = value.clone();
                self.right_outer_padding = value.clone();
                self.top_outer_padding = value.clone();
                self.bottom_outer_padding = value
            }
        }
    }
}

pub struct BoxView {
    index_path: PathBuf,
    key: String,
    width: String,
    height: String,
    background: String,
    path: String,
    left_outer_padding: String,
    top_outer_padding: String,
    right_outer_padding: String,
    bottom_outer_padding: String,
    fixed: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl BoxView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>) -> BoxView {

        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);

        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);
        
        let mut view = BoxView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "0px", value),
            height: item_to_string(&table, "height", "0px", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            fixed: item_to_string(&table, "fixed", "", value),
            value: value.cloned(),
            dark: dark,
            views: vec![],
        };

        let views = table
            .iter()
            .filter_map(|(k, v)| {
                if let Item::Table(inner_table) = v {
                    match table_to_tomlview(index_path, k, inner_table, value, Some(&view)) {
                        Ok(view) => Some(view),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            })
            .collect();
        view.views = views;
        view
    }
}

impl TOMLView for BoxView {
    fn index_path(&self) -> PathBuf {
        return self.index_path.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn shape(&self) -> String {
        return "box".to_string();
    }
    fn key(&self) -> String {
        return self.key.clone();
    }
    fn width(&self) -> String {
        return self.width.clone();
    }
    fn height(&self) -> String {
        return self.height.clone();
    }
    fn background(&self) -> String {
        return self.background.clone();
    }
    fn path(&self) -> String {
        return self.path.clone();
    }
    fn dark(&self) -> bool {
        return self.dark;
    }
    fn views(&self) -> &Vec<Box<dyn TOMLView>> {
        return &self.views;
    }
    fn htmlview(&self, super_view: Option<&dyn TOMLView>) -> HTMLView {
        let views = self.views.iter().map(|x| x.htmlview(Some(self))).collect();

        let mut style_parts = vec![
            format!("background:{}", self.background),
            format!(
                "padding:{} {} {} {}",
                self.top_outer_padding,
                self.right_outer_padding,
                self.bottom_outer_padding,
                self.left_outer_padding,
            ),
            "display:flex".to_string(),
            "flex-direction:row".to_string(),
            "flex-shrink: 0".to_string(),
        ];
        if self.width != "0px" {
            style_parts.push(format!("width:{}", self.width));
        }
        if self.height != "0px" {
            style_parts.push(format!("height:{}", self.height));
        }
        if !self.fixed.is_empty() {
            style_parts.push(format!("position: fixed; {}: 0", self.fixed));
        }

        let style = style_parts.join("; ") + ";"; // 끝에 세미콜론

        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), self.key.clone());
        attrs.insert("style".to_string(), style);

        HTMLView {
            tag: "div".to_string(),
            attrs: attrs,
            value: "".to_string(),
            views: views,
        }
        .wrap_href(self.path.clone())
    }
    fn value(&self) -> Option<InlineTable> {
        return self.value.clone();
    }
}

impl AllocationView for BoxView {
    fn fixed(&self) -> String {
        return self.fixed.to_string()
    }
    fn set_inner_padding(&mut self, value: String) {}
    fn set_outer_padding(&mut self, ptype: PaddingType, value: String) {
        match ptype {
            PaddingType::LEFT => self.left_outer_padding = value,
            PaddingType::RIGHT => self.right_outer_padding = value,
            PaddingType::TOP => self.top_outer_padding = value,
            PaddingType::BOTTOM => self.bottom_outer_padding = value,
            PaddingType::HORIZONTAL => {
                self.left_outer_padding = value.clone();
                self.right_outer_padding = value;
            }
            PaddingType::VERTICAL => {
                self.top_outer_padding = value.clone();
                self.bottom_outer_padding = value;
            }
            PaddingType::ALL => {
                self.left_outer_padding = value.clone();
                self.right_outer_padding = value.clone();
                self.top_outer_padding = value.clone();
                self.bottom_outer_padding = value
            }
        }
    }
}

pub struct TextView {
    index_path: PathBuf,
    key: String,
    width: String,
    height: String,
    background: String,
    path: String,
    left_outer_padding: String,
    top_outer_padding: String,
    right_outer_padding: String,
    bottom_outer_padding: String,
    size: String,
    text: String,
    color: String,
    family: String,
    weight: String,
    horizontal_align: String,
    vertical_align: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl TextView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>) -> TextView {
        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);

        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);
        
        TextView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "0px", value),
            height: item_to_string(&table, "height", "0px", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            size: item_to_string(&table, "size", "16px", value),
            text: item_to_string(&table, "text", "", value),
            color: item_to_string(&table, "color", "black", value),
            family: item_to_string(&table, "family", "Arial", value),
            weight: item_to_string(&table, "weight", "normal", value),
            horizontal_align: item_to_string(&table, "horizontal_align", "left", value),
            vertical_align: item_to_string(&table, "vertical_align", "top", value),
            value: value.cloned(),
            dark: dark,
            views: vec![],
        }
    }
}

impl TOMLView for TextView {
    fn index_path(&self) -> PathBuf {
        return self.index_path.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn shape(&self) -> String {
        return "text".to_string();
    }
    fn key(&self) -> String {
        return self.key.clone();
    }
    fn width(&self) -> String {
        return self.width.clone();
    }
    fn height(&self) -> String {
        return self.height.clone();
    }
    fn background(&self) -> String {
        return self.background.clone();
    }
    fn path(&self) -> String {
        return self.path.clone();
    }
    fn dark(&self) -> bool {
        return self.dark;
    }
    fn views(&self) -> &Vec<Box<dyn TOMLView>> {
        return &self.views;
    }
    fn htmlview(&self, super_view: Option<&dyn TOMLView>) -> HTMLView {
        let mut span_style_parts = vec![
            format!("background:{}", self.background),
            format!(
                "padding:{} {} {} {}",
                self.top_outer_padding,
                self.right_outer_padding,
                self.bottom_outer_padding,
                self.left_outer_padding,
            ),
            format!("display: {}", "flex"),
            format!("font-size:{}", self.size),
            format!("color:{}", self.color),
            format!("font-family:{}", self.family),
            format!("font-weight:{}", self.weight),
            format!("justify-content: {}", self.horizontal_align),
            format!("align-items: {}", self.vertical_align),
        ];
        if self.width != "0px" && self.width != "wrap" {
            span_style_parts.push(format!("width:{}", self.width));
        }
        if self.height != "0px" && self.height != "wrap" {
            span_style_parts.push(format!("height:{}", self.height));
        }
        let span_style = span_style_parts.join("; ") + ";"; // 끝에 세

        let mut span_attrs = HashMap::new();
        span_attrs.insert("id".to_string(), self.key.clone());
        span_attrs.insert("style".to_string(), span_style);

        HTMLView {
            tag: "span".to_string(),
            attrs: span_attrs,
            value: self.text.clone(),
            views: vec![],
        }
        .wrap_href(self.path.clone())
    }
    fn value(&self) -> Option<InlineTable> {
        return self.value.clone();
    }
}

pub struct ImageView {
    index_path: PathBuf,
    key: String,
    width: String,
    height: String,
    background: String,
    path: String,
    left_outer_padding: String,
    top_outer_padding: String,
    right_outer_padding: String,
    bottom_outer_padding: String,
    image_path: String,
    content_size: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl ImageView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>) -> ImageView {
        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);

        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);

        ImageView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "0px", value),
            height: item_to_string(&table, "height", "0px", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            image_path: item_to_string(&table, "image_path", "", value),
            content_size: item_to_string(&table, "content_size", "cover", value),
            value: value.cloned(),
            dark: dark,
            views: vec![],
        }
    }
}

impl TOMLView for ImageView {
    fn index_path(&self) -> PathBuf {
        return self.index_path.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn shape(&self) -> String {
        return "text".to_string();
    }
    fn key(&self) -> String {
        return self.key.clone();
    }
    fn width(&self) -> String {
        return self.width.clone();
    }
    fn height(&self) -> String {
        return self.height.clone();
    }
    fn background(&self) -> String {
        return self.background.clone();
    }
    fn path(&self) -> String {
        return self.path.clone();
    }
    fn dark(&self) -> bool {
        return self.dark;
    }
    fn views(&self) -> &Vec<Box<dyn TOMLView>> {
        return &self.views;
    }
    fn htmlview(&self, super_view: Option<&dyn TOMLView>) -> HTMLView {
        let mut img_style_parts = vec![
            format!("background:{}", self.background),
            format!(
                "padding:{} {} {} {}",
                self.top_outer_padding,
                self.right_outer_padding,
                self.bottom_outer_padding,
                self.left_outer_padding,
            ),
            format!("display: {}", "flex"),
        ];
        let img_style = img_style_parts.join("; ") + ";"; // 끝에 세

        let mut img_attrs = HashMap::new();
        img_attrs.insert("id".to_string(), self.key.clone());
        img_attrs.insert("style".to_string(), img_style);
        img_attrs.insert("src".to_string(), self.image_path.clone());
        img_attrs.insert("width".to_string(), self.width.clone());
        img_attrs.insert("height".to_string(), self.height.clone());

        HTMLView {
            tag: "img".to_string(),
            attrs: img_attrs,
            value: "".to_string(),
            views: vec![],
        }
        .wrap_href(self.path.clone())
    }
    fn value(&self) -> Option<InlineTable> {
        return self.value.clone();
    }
}

pub struct NavView {
    index_path: PathBuf,
    key: String,
    width: String,
    height: String,
    background: String,
    path: String,
    title: String,
    headers: Vec<String>,
    headers_map: HashMap<String, Item>,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl NavView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>) -> NavView {
        let headers_map = get_items(&table, vec!["key", "shape", "width", "height", "background", "title", "values", "path", "dark"]);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);
        
        NavView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "0px", value),
            height: item_to_string(&table, "height", "0px", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            title: item_to_string(&table, "title", "", value),
            headers: item_to_strings(&table, "headers"),
            headers_map: headers_map,
            value: value.cloned(),
            dark: dark,
            views: vec![],
        }
    }
}

impl TOMLView for NavView {
    fn index_path(&self) -> PathBuf {
        return self.index_path.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn shape(&self) -> String {
        return "nav".to_string();
    }
    fn key(&self) -> String {
        return self.key.clone();
    }
    fn width(&self) -> String {
        return self.width.clone();
    }
    fn height(&self) -> String {
        return self.height.clone();
    }
    fn background(&self) -> String {
        return self.background.clone();
    }
    fn path(&self) -> String {
        return self.path.clone();
    }
    fn dark(&self) -> bool {
        return self.dark;
    }
    fn views(&self) -> &Vec<Box<dyn TOMLView>> {
        return &self.views;
    }
    fn htmlview(&self, super_view: Option<&dyn TOMLView>) -> HTMLView {  
        println!("headers_map {:?}", self.headers_map);
        make_nav(self.title.clone(), self.headers.clone(), self.headers_map.clone(), self.dark)
    }
    fn value(&self) -> Option<InlineTable> {
        return self.value.clone();
    }
}

pub struct ListColumnView {
    index_path: PathBuf,
    key: String,
    width: String,
    height: String,
    background: String,
    path: String,
    is_scroll: bool,
    left_outer_padding: String,
    top_outer_padding: String,
    right_outer_padding: String,
    bottom_outer_padding: String,
    inner_padding: String,
    fixed: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl ListColumnView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, is_scroll: bool) -> ListColumnView {
        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);

        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        
        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);

        let mut view = ListColumnView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "0px", value),
            height: item_to_string(&table, "height", "0px", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            is_scroll,
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            inner_padding: item_to_string(&table, "inner_padding", "0px", value),
            fixed: item_to_string(&table, "fixed", "", value),
            value: value.cloned(),
            dark: dark,
            views: vec![],
        };
        
        let layout = item_to_string(&table, "layout", "", value);
        let mut views: Vec<Box<dyn TOMLView>> = vec![];
        if let Some(values) = table.get("values").and_then(|item| item.as_array()) {
            views.extend(
                values
                    .iter()
                    .filter_map(|v| if let Value::InlineTable(tbl) = v { Some(tbl) } else { None })
                    .filter_map(|tbl| layout_to_tomlview(layout.clone(), index_path, Some(tbl), Some(&view)).ok())
            );
        }

        view.views = views;
        view
    }
}

impl TOMLView for ListColumnView {
    fn index_path(&self) -> PathBuf {
        return self.index_path.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn shape(&self) -> String {
        if self.is_scroll {
            return "list_column".to_string();
        }
        return "list_column".to_string();
    }
    fn key(&self) -> String {
        return self.key.clone();
    }
    fn width(&self) -> String {
        return self.width.clone();
    }
    fn height(&self) -> String {
        return self.height.clone();
    }
    fn background(&self) -> String {
        return self.background.clone();
    }
    fn path(&self) -> String {
        return self.path.clone();
    }
    fn dark(&self) -> bool {
        return self.dark;
    }
    fn views(&self) -> &Vec<Box<dyn TOMLView>> {
        return &self.views;
    }
    fn htmlview(&self, super_view: Option<&dyn TOMLView>) -> HTMLView {
        let views = self.views.iter().map(|x| x.htmlview(Some(self))).collect();

        let mut style_parts = vec![
            format!("background:{}", self.background),
            format!(
                "padding:{} {} {} {}",
                self.top_outer_padding,
                self.right_outer_padding,
                self.bottom_outer_padding,
                self.left_outer_padding,
            ),
            "display:flex".to_string(),
            "flex-direction:column".to_string(),
        ];
        if self.width != "0px" {
            style_parts.push(format!("width:{}", self.width));
        }
        if self.height != "0px" {
            style_parts.push(format!("height:{}", self.height));
        }
        if self.is_scroll {
            style_parts.push("overflow-y: auto".to_string());
        } else {
            style_parts.push("overflow: hidden".to_string());
        }
        if !self.fixed.is_empty() {
            style_parts.push(format!("position: fixed; {}: 0", self.fixed));
        }

        if !self.inner_padding.trim().is_empty() {
            style_parts.push(format!("gap:{}", self.inner_padding));
        }

        let style = style_parts.join("; ") + ";"; // 끝에 세미콜론

        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), self.key.clone());
        attrs.insert("style".to_string(), style);

        HTMLView {
            tag: "div".to_string(),
            attrs: attrs,
            value: "".to_string(),
            views: views,
        }
        .wrap_href(self.path.clone())
    }
    fn value(&self) -> Option<InlineTable> {
        return self.value.clone();
    }
}

impl AllocationView for ListColumnView {
    fn fixed(&self) -> String {
        return self.fixed.to_string();
    }
    fn set_inner_padding(&mut self, value: String) {
        self.inner_padding = value;
    }
    fn set_outer_padding(&mut self, ptype: PaddingType, value: String) {
        match ptype {
            PaddingType::LEFT => self.left_outer_padding = value,
            PaddingType::RIGHT => self.right_outer_padding = value,
            PaddingType::TOP => self.top_outer_padding = value,
            PaddingType::BOTTOM => self.bottom_outer_padding = value,
            PaddingType::HORIZONTAL => {
                self.left_outer_padding = value.clone();
                self.right_outer_padding = value;
            }
            PaddingType::VERTICAL => {
                self.top_outer_padding = value.clone();
                self.bottom_outer_padding = value;
            }
            PaddingType::ALL => {
                self.left_outer_padding = value.clone();
                self.right_outer_padding = value.clone();
                self.top_outer_padding = value.clone();
                self.bottom_outer_padding = value
            }
        }
    }
}

pub struct ListRowView {
    index_path: PathBuf,
    key: String,
    width: String,
    height: String,
    background: String,
    path: String,
    is_scroll: bool,
    left_outer_padding: String,
    top_outer_padding: String,
    right_outer_padding: String,
    bottom_outer_padding: String,
    inner_padding: String,
    fixed: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl ListRowView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, is_scroll: bool) -> ListRowView {
        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);

        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);

        let mut view = ListRowView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "0px", value),
            height: item_to_string(&table, "height", "0px", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            is_scroll,
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            inner_padding: item_to_string(&table, "inner_padding", "0px", value),
            fixed: item_to_string(&table, "fixed", "", value),
            value: value.cloned(),
            dark: dark,
            views: vec![],
        };

        let layout = item_to_string(&table, "layout", "", value);
        let mut views: Vec<Box<dyn TOMLView>> = vec![];
        if let Some(values) = table.get("values").and_then(|item| item.as_array()) {
            views.extend(
                values
                    .iter()
                    .filter_map(|v| if let Value::InlineTable(tbl) = v { Some(tbl) } else { None })
                    .filter_map(|tbl| layout_to_tomlview(layout.clone(), index_path, Some(tbl), Some(&view)).ok())
            );
        }

        view.views = views;
        view
    }
}

impl TOMLView for ListRowView {
    fn index_path(&self) -> PathBuf {
        return self.index_path.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn shape(&self) -> String {
        if self.is_scroll {
            return "list_row".to_string();
        }
        return "list_row".to_string();
    }
    fn key(&self) -> String {
        return self.key.clone();
    }
    fn width(&self) -> String {
        return self.width.clone();
    }
    fn height(&self) -> String {
        return self.height.clone();
    }
    fn background(&self) -> String {
        return self.background.clone();
    }
    fn path(&self) -> String {
        return self.path.clone();
    }
    fn dark(&self) -> bool {
        return self.dark;
    }
    fn views(&self) -> &Vec<Box<dyn TOMLView>> {
        return &self.views;
    }
    fn htmlview(&self, super_view: Option<&dyn TOMLView>) -> HTMLView {
        let views = self.views.iter().map(|x| x.htmlview(Some(self))).collect();

        let mut style_parts = vec![
            format!("background:{}", self.background),
            format!(
                "padding:{} {} {} {}",
                self.top_outer_padding,
                self.right_outer_padding,
                self.bottom_outer_padding,
                self.left_outer_padding,
            ),
            "display:flex".to_string(),
            "flex-direction:row".to_string(),
        ];
        if self.width != "0px" {
            style_parts.push(format!("width:{}", self.width));
        }
        if self.height != "0px" {
            style_parts.push(format!("height:{}", self.height));
        }
        if self.is_scroll {
            style_parts.push("overflow-x: auto".to_string());
        } else {
            style_parts.push("overflow: hidden".to_string());
        }
        if !self.fixed.is_empty() {
            style_parts.push(format!("position: fixed; {}: 0", self.fixed));
        }

        if !self.inner_padding.trim().is_empty() {
            style_parts.push(format!("gap:{}", self.inner_padding));
        }

        let style = style_parts.join("; ") + ";"; // 끝에 세미콜론

        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), self.key.clone());
        attrs.insert("style".to_string(), style);

        HTMLView {
            tag: "div".to_string(),
            attrs: attrs,
            value: "".to_string(),
            views: views,
        }
        .wrap_href(self.path.clone())
    }
    fn value(&self) -> Option<InlineTable> {
        return self.value.clone();
    }
}

impl AllocationView for ListRowView {
    fn fixed(&self) -> String {
        return self.fixed.to_string();
    }
    fn set_inner_padding(&mut self, value: String) {
        self.inner_padding = value;
    }
    fn set_outer_padding(&mut self, ptype: PaddingType, value: String) {
        match ptype {
            PaddingType::LEFT => self.left_outer_padding = value,
            PaddingType::RIGHT => self.right_outer_padding = value,
            PaddingType::TOP => self.top_outer_padding = value,
            PaddingType::BOTTOM => self.bottom_outer_padding = value,
            PaddingType::HORIZONTAL => {
                self.left_outer_padding = value.clone();
                self.right_outer_padding = value;
            }
            PaddingType::VERTICAL => {
                self.top_outer_padding = value.clone();
                self.bottom_outer_padding = value;
            }
            PaddingType::ALL => {
                self.left_outer_padding = value.clone();
                self.right_outer_padding = value.clone();
                self.top_outer_padding = value.clone();
                self.bottom_outer_padding = value
            }
        }
    }
}

pub struct MarkdownView {
    index_path: PathBuf,
    key: String,
    width: String,
    height: String,
    background: String,
    path: String,
    is_scroll: bool,
    left_outer_padding: String,
    top_outer_padding: String,
    right_outer_padding: String,
    bottom_outer_padding: String,
    inner_padding: String,
    fixed: String,
    markdown_path: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl MarkdownView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, is_scroll: bool) -> MarkdownView {
        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);

        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);

        MarkdownView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "0px", value),
            height: item_to_string(&table, "height", "0px", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            is_scroll,
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            inner_padding: item_to_string(&table, "inner_padding", "0px", value),
            fixed: item_to_string(&table, "fixed", "", value),
            markdown_path: item_to_string(&table, "markdown_path", "", value),
            value: value.cloned(),
            dark: dark,
            views: vec![],
        }
    }
}

impl TOMLView for MarkdownView {
    fn index_path(&self) -> PathBuf {
        return self.index_path.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn shape(&self) -> String {
        if self.is_scroll {
            return "scroll_column".to_string();
        }
        return "column".to_string();
    }
    fn key(&self) -> String {
        return self.key.clone();
    }
    fn width(&self) -> String {
        return self.width.clone();
    }
    fn height(&self) -> String {
        return self.height.clone();
    }
    fn background(&self) -> String {
        return self.background.clone();
    }
    fn path(&self) -> String {
        return self.path.clone();
    }
    fn dark(&self) -> bool {
        return self.dark;
    }
    fn views(&self) -> &Vec<Box<dyn TOMLView>> {
        return &self.views;
    }
    fn htmlview(&self, super_view: Option<&dyn TOMLView>) -> HTMLView {
        let views = self.views.iter().map(|x| x.htmlview(Some(self))).collect();

        let mut style_parts = vec![
            format!("background:{}", self.background),
            format!(
                "padding:{} {} {} {}",
                self.top_outer_padding,
                self.right_outer_padding,
                self.bottom_outer_padding,
                self.left_outer_padding,
            ),
            "display:flex".to_string(),
            "flex-direction:column".to_string(),
            "flex-shrink: 0".to_string(),
        ];
        if self.width != "wrap" {
            style_parts.push(format!("width:{}", self.width));
        }
        if self.height != "wrap" {
            style_parts.push(format!("height:{}", self.height));
        }
        if self.is_scroll {
            style_parts.push("overflow-y: auto".to_string());
        } else {
            style_parts.push("overflow: hidden".to_string());
        }
        if !self.fixed.is_empty() {
            style_parts.push(format!("position: fixed; {}: 0", self.fixed));
        }

        if !self.inner_padding.trim().is_empty() {
            style_parts.push(format!("gap:{}", self.inner_padding));
        }

        let style = style_parts.join("; ") + ";"; // 끝에 세미콜론
        

        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), self.key.clone());
        attrs.insert("style".to_string(), style);

        let parent = self.index_path.parent().expect("index_path has no parent");
        let target_path = parent.join(self.markdown_path.clone());
        let markdown_view = match markdown_to_htmlview(target_path.as_path(), self.dark) {
            Ok(markdown_view) => {
                markdown_view
            },
            Err(message) => {
                println!("markdown_to_htmlview ... {}, {}", self.markdown_path.clone(), self.index_path.to_str().unwrap_or(""));
                HTMLView {
                    tag: "div".to_string(),
                    attrs: attrs.clone(),
                    value: "".to_string(),
                    views: views,
                }
                .wrap_href(self.path.clone())
            }
        };

        HTMLView {
            tag: "div".to_string(),
            attrs: attrs,
            value: "".to_string(),
            views: vec![markdown_view],
        }
        .wrap_href(self.path.clone())
    }
    fn value(&self) -> Option<InlineTable> {
        return self.value.clone();
    }
}