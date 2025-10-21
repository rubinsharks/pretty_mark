use crate::file::find_files;
use crate::html::{filter_attrs, HTMLView};
use crate::layout::nav::make_nav;
use crate::markdown::{markdown_to_htmlview, metas_table_from_markdown};
use crate::page::Page;
use std::collections::HashMap;
use std::fmt::{self, Formatter};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use dateparser::parse;
use toml_edit::InlineTable;
use toml_edit::{value, DocumentMut, Item, Table, Value};

use std::any::Any;

use super::common::{get_items, item_to_bool, layout_to_tomlview};
use super::common::item_to_string;
use super::common::item_to_strings;
use super::common::table_to_tomlview;
use super::padding::PaddingType;

impl fmt::Display for dyn TOMLView {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", self.index_path().to_str().unwrap_or("")).unwrap();
        Ok(())
    }
}

impl fmt::Debug for dyn TOMLView {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}, {} - {}", self.index_path().to_str().unwrap_or(""), self.key(), self.shape()).unwrap();
        for view in self.views() {
            write!(f, "{:?}", view).unwrap();
        }
        Ok(())
    }
}

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
    align_absolute: String,
    align_subs: String,
    fixed: String,
    custom_class: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl ColumnView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, layout_tables: HashMap<String, Table>, is_scroll: bool) -> ColumnView {
        let table_value = table.get("value").and_then(|v| v.as_inline_table());
        let mut merged = InlineTable::new();
        if let Some(v) = value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        if let Some(v) = table_value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        let value = if merged.is_empty() {
            None
        } else {
            Some(&merged)
        };

        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);
        
        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        let custom_class = item_to_string(&table, "custom_class", "", value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);
        
        let mut view = ColumnView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "wrap", value),
            height: item_to_string(&table, "height", "wrap", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            is_scroll,
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            inner_padding: item_to_string(&table, "inner_padding", "0px", value),
            align_absolute: item_to_string(&table, "align_absolute", "", value),
            align_subs: item_to_string(&table, "align_subs", "start", value),
            fixed: item_to_string(&table, "fixed", "", value),
            custom_class,
            value: value.cloned(),
            dark: dark,
            views: vec![],
        };

        let views = table
            .iter()
            .filter_map(|(k, v)| {
                if let Item::Table(inner_table) = v {
                    match table_to_tomlview(index_path, k, inner_table, value, Some(&view), layout_tables.clone()) {
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
            // "flex-shrink: 0".to_string(),
        ];
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

        let mut class_parts = vec![
            format!("items-{}", self.align_subs),
        ];
        if self.width.starts_with("w-") {
            class_parts.push(self.width.clone());
        } else if self.width != "wrap" {
            class_parts.push(format!("w-[{}]", self.width));
        }
        if self.height.starts_with("h-") {
            class_parts.push(self.height.clone());
        } else if self.height != "wrap" {
            class_parts.push(format!("h-[{}]", self.height));
        }
        if !self.align_absolute.is_empty() {
            let mut align_class = self.align_absolute.clone();

            if !align_class.chars().any(|c| c.is_numeric()) {
                align_class = format!("{}-0", align_class);
            }
            class_parts.push(format!("absolute {}", align_class));
        }

        if !self.custom_class.is_empty() {
            class_parts.push(format!("{}", self.custom_class));
        }
        
        let class = class_parts.join(" ");

        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), self.key.clone());
        attrs.insert("style".to_string(), style);
        attrs.insert("class".to_string(), class);

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
    align_absolute: String,
    align_subs: String,
    fixed: String,
    custom_class: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl RowView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, layout_tables: HashMap<String, Table>, is_scroll: bool) -> RowView {
        let table_value = table.get("value").and_then(|v| v.as_inline_table());
        let mut merged = InlineTable::new();
        if let Some(v) = value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        if let Some(v) = table_value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        let value = if merged.is_empty() {
            None
        } else {
            Some(&merged)
        };

        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);

        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        let custom_class = item_to_string(&table, "custom_class", "", value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);
        
        let mut view = RowView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "wrap", value),
            height: item_to_string(&table, "height", "wrap", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            is_scroll,
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            inner_padding: item_to_string(&table, "inner_padding", "0px", value),
            align_absolute: item_to_string(&table, "align_absolute", "", value),
            align_subs: item_to_string(&table, "align_subs", "start", value),
            fixed: item_to_string(&table, "fixed", "", value),
            custom_class,
            value: value.cloned(),
            dark: dark,
            views: vec![],
        };

        let views = table
            .iter()
            .filter_map(|(k, v)| {
                if let Item::Table(inner_table) = v {
                    match table_to_tomlview(index_path, k, inner_table, value, Some(&view), layout_tables.clone()) {
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
            // "flex-shrink: 0".to_string(),
        ];
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

        let mut class_parts = vec![
            format!("items-{}", self.align_subs),
        ];
        if self.width.starts_with("w-") {
            class_parts.push(self.width.clone());
        } else if self.width != "wrap" {
            class_parts.push(format!("w-[{}]", self.width));
        }
        if self.height.starts_with("h-") {
            class_parts.push(self.height.clone());
        } else if self.height != "wrap" {
            class_parts.push(format!("h-[{}]", self.height));
        }
        if !self.align_absolute.is_empty() {
            let mut align_class = self.align_absolute.clone();

            if !align_class.chars().any(|c| c.is_numeric()) {
                align_class = format!("{}-0", align_class);
            }
            class_parts.push(format!("absolute {}", align_class));
        }
        if !self.custom_class.is_empty() {
            class_parts.push(format!("{}", self.custom_class));
        }
        let class = class_parts.join(" ");

        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), self.key.clone());
        attrs.insert("style".to_string(), style);
        attrs.insert("class".to_string(), class);

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
    inner_padding: String,
    align_absolute: String,
    align_subs: String,
    fixed: String,
    custom_class: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl BoxView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, layout_tables: HashMap<String, Table>) -> BoxView {
        let table_value = table.get("value").and_then(|v| v.as_inline_table());
        let mut merged = InlineTable::new();
        if let Some(v) = value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        if let Some(v) = table_value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        let value = if merged.is_empty() {
            None
        } else {
            Some(&merged)
        };
        
        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);

        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);
        
        let custom_class = item_to_string(&table, "custom_class", "", value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);
        
        let mut view = BoxView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "wrap", value),
            height: item_to_string(&table, "height", "wrap", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            inner_padding: item_to_string(&table, "inner_padding", "0px", value),
            align_absolute: item_to_string(&table, "align_absolute", "", value),
            align_subs: item_to_string(&table, "align_subs", "start", value),
            fixed: item_to_string(&table, "fixed", "", value),
            custom_class,
            value: value.cloned(),
            dark: dark,
            views: vec![],
        };

        let views = table
            .iter()
            .filter_map(|(k, v)| {
                if let Item::Table(inner_table) = v {
                    match table_to_tomlview(index_path, k, inner_table, value, Some(&view), layout_tables.clone()) {
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
            // "display:flex".to_string(),
            // "flex-shrink: 0".to_string(),
        ];
        if !self.fixed.is_empty() {
            style_parts.push(format!("position: fixed; {}: 0", self.fixed));
        }

        let style = style_parts.join("; ") + ";"; // 끝에 세미콜론

        let mut class_parts = vec![
            "relative".to_string(),
            format!("items-{}", self.align_subs),
        ];
        if self.width.starts_with("w-") {
            class_parts.push(self.width.clone());
        } else if self.width != "wrap" {
            class_parts.push(format!("w-[{}]", self.width));
        }
        if self.height.starts_with("h-") {
            class_parts.push(self.height.clone());
        } else if self.height != "wrap" {
            class_parts.push(format!("h-[{}]", self.height));
        }
        if !self.align_absolute.is_empty() {
            let mut align_class = self.align_absolute.clone();
            
            if !align_class.chars().any(|c| c.is_numeric()) {
                align_class = format!("{}-0", align_class);
            }
            class_parts.push(format!("absolute {}", align_class));
        }
        if !self.custom_class.is_empty() {
            class_parts.push(format!("{}", self.custom_class));
        }
        let class = class_parts.join(" ");

        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), self.key.clone());
        attrs.insert("style".to_string(), style);
        attrs.insert("class".to_string(), class);

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
    align_absolute: String,
    size: String,
    text: String,
    color: String,
    family: String,
    weight: String,
    underline: bool,
    horizontal_align: String,
    vertical_align: String,
    custom_class: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl TextView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, layout_tables: HashMap<String, Table>) -> TextView {
        let table_value = table.get("value").and_then(|v| v.as_inline_table());
        let mut merged = InlineTable::new();
        if let Some(v) = value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        if let Some(v) = table_value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        let value = if merged.is_empty() {
            None
        } else {
            Some(&merged)
        };
        
        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);

        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        let custom_class = item_to_string(&table, "custom_class", "", value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);
        
        TextView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "wrap", value),
            height: item_to_string(&table, "height", "wrap", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            align_absolute: item_to_string(&table, "align_absolute", "", value),
            size: item_to_string(&table, "size", "16px", value),
            text: item_to_string(&table, "text", "", value),
            color: item_to_string(&table, "color", "black", value),
            family: item_to_string(&table, "family", "Arial", value),
            weight: item_to_string(&table, "weight", "normal", value),
            underline: item_to_bool(&table, "underline", false, value),
            horizontal_align: item_to_string(&table, "horizontal_align", "left", value),
            vertical_align: item_to_string(&table, "vertical_align", "top", value),
            custom_class,
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
        let span_style = span_style_parts.join("; ") + ";"; // 끝에 세

        let mut class_parts = vec![
        ];
        if self.width.starts_with("w-") {
            class_parts.push(self.width.clone());
        } else if self.width != "wrap" {
            class_parts.push(format!("w-[{}]", self.width));
        }
        if self.height.starts_with("h-") {
            class_parts.push(self.height.clone());
        } else if self.height != "wrap" {
            class_parts.push(format!("h-[{}]", self.height));
        }
        if !self.align_absolute.is_empty() {
            let mut align_class = self.align_absolute.clone();

            if !align_class.chars().any(|c| c.is_numeric()) {
                align_class = format!("{}-0", align_class);
            }
            class_parts.push(format!("absolute {}", align_class));
        }
        if self.underline {
            class_parts.push("underline".to_string());
        }
        if !self.custom_class.is_empty() {
            class_parts.push(format!("{}", self.custom_class));
        }
        let class = class_parts.join(" ");

        let mut span_attrs = HashMap::new();
        span_attrs.insert("id".to_string(), self.key.clone());
        span_attrs.insert("style".to_string(), span_style);
        span_attrs.insert("class".to_string(), class);

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
    align_absolute: String,
    image_path: String,
    content_size: String,
    rounded: String,
    custom_class: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl ImageView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, layout_tables: HashMap<String, Table>) -> ImageView {
        let table_value = table.get("value").and_then(|v| v.as_inline_table());
        let mut merged = InlineTable::new();
        if let Some(v) = value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        if let Some(v) = table_value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        let value = if merged.is_empty() {
            None
        } else {
            Some(&merged)
        };
        
        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);

        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        let custom_class = item_to_string(&table, "custom_class", "", value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);

        ImageView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "wrap", value),
            height: item_to_string(&table, "height", "wrap", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            align_absolute: item_to_string(&table, "align_absolute", "", value),
            image_path: item_to_string(&table, "image_path", "", value),
            content_size: item_to_string(&table, "content_size", "cover", value),
            rounded: item_to_string(&table, "rounded", "", value),
            custom_class,
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

        let mut class_parts = vec![
        ];
        if self.width.starts_with("w-") {
            class_parts.push(self.width.clone());
        } else if self.width != "wrap" {
            class_parts.push(format!("w-[{}]", self.width));
        }
        if self.height.starts_with("h-") {
            class_parts.push(self.height.clone());
        } else if self.height != "wrap" {
            class_parts.push(format!("h-[{}]", self.height));
        }
        class_parts.push(format!("object-{}", self.content_size));
        if !self.align_absolute.is_empty() {
            let mut align_class = self.align_absolute.clone();

            if !align_class.chars().any(|c| c.is_numeric()) {
                align_class = format!("{}-0", align_class);
            }
            class_parts.push(format!("absolute {}", align_class));
        }
        if !self.rounded.is_empty() {
            class_parts.push(format!("rounded-[{}]", self.rounded));
        }
        if !self.custom_class.is_empty() {
            class_parts.push(format!("{}", self.custom_class));
        }
        let class = class_parts.join(" ");

        let mut img_attrs = HashMap::new();
        img_attrs.insert("id".to_string(), self.key.clone());
        img_attrs.insert("style".to_string(), img_style);
        img_attrs.insert("src".to_string(), self.image_path.clone());
        img_attrs.insert("class".to_string(), class);
        // img_attrs.insert("width".to_string(), self.width.clone());
        // img_attrs.insert("height".to_string(), self.height.clone());

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
    align_absolute: String,
    title: String,
    headers: Vec<String>,
    headers_map: HashMap<String, Item>,
    custom_class: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl NavView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, layout_tables: HashMap<String, Table>) -> NavView {
        let table_value = table.get("value").and_then(|v| v.as_inline_table());
        let mut merged = InlineTable::new();
        if let Some(v) = value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        if let Some(v) = table_value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        let value = if merged.is_empty() {
            None
        } else {
            Some(&merged)
        };
        
        let headers_map = get_items(&table, vec!["key", "shape", "width", "height", "background", "title", "values", "path", "dark"]);

        let custom_class = item_to_string(&table, "custom_class", "", value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);
        
        NavView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "wrap", value),
            height: item_to_string(&table, "height", "wrap", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            align_absolute: item_to_string(&table, "align_absolute", "", value),
            title: item_to_string(&table, "title", "", value),
            headers: item_to_strings(&table, "headers"),
            headers_map: headers_map,
            custom_class,
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
        let mut class_parts = vec![
        ];
        if self.width.starts_with("w-") {
            class_parts.push(self.width.clone());
        } else if self.width != "wrap" {
            class_parts.push(format!("w-[{}]", self.width));
        }
        if self.height.starts_with("h-") {
            class_parts.push(self.height.clone());
        } else if self.height != "wrap" {
            class_parts.push(format!("h-[{}]", self.height));
        }
        if !self.align_absolute.is_empty() {
            let mut align_class = self.align_absolute.clone();

            if !align_class.chars().any(|c| c.is_numeric()) {
                align_class = format!("{}-0", align_class);
            }
            class_parts.push(format!("absolute {}", align_class));
        }
        if !self.custom_class.is_empty() {
            class_parts.push(format!("{}", self.custom_class));
        }
        let class = class_parts.join(" ");

        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), self.key.clone());
        attrs.insert("class".to_string(), class);
        
        let nav_view = make_nav(self.title.clone(), self.headers.clone(), self.headers_map.clone(), self.dark);
        
        HTMLView {
            tag: "div".to_string(),
            attrs: attrs,
            value: "".to_string(),
            views: vec![nav_view],
        }
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
    align_absolute: String,
    align_subs: String,
    fixed: String,
    custom_class: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl ListColumnView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, layout_tables: HashMap<String, Table>, is_scroll: bool) -> ListColumnView {
        let table_value = table.get("value").and_then(|v| v.as_inline_table());
        let mut merged = InlineTable::new();
        if let Some(v) = value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        if let Some(v) = table_value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        let value = if merged.is_empty() {
            None
        } else {
            Some(&merged)
        };
        
        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);

        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        let custom_class = item_to_string(&table, "custom_class", "", value);
        
        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);

        let mut view = ListColumnView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "wrap", value),
            height: item_to_string(&table, "height", "wrap", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            is_scroll,
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            inner_padding: item_to_string(&table, "inner_padding", "0px", value),
            align_absolute: item_to_string(&table, "align_absolute", "", value),
            align_subs: item_to_string(&table, "align_subs", "start", value),
            fixed: item_to_string(&table, "fixed", "", value),
            custom_class,
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
                    .filter_map(|tbl| layout_to_tomlview(&view, layout.clone(), layout_tables.clone(), Some(tbl)).ok())
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

        let mut class_parts = vec![
            format!("items-{}", self.align_subs),
        ];
        if self.width.starts_with("w-") {
            class_parts.push(self.width.clone());
        } else if self.width != "wrap" {
            class_parts.push(format!("w-[{}]", self.width));
        }
        if self.height.starts_with("h-") {
            class_parts.push(self.height.clone());
        } else if self.height != "wrap" {
            class_parts.push(format!("h-[{}]", self.height));
        }
        if !self.align_absolute.is_empty() {
            let mut align_class = self.align_absolute.clone();

            if !align_class.chars().any(|c| c.is_numeric()) {
                align_class = format!("{}-0", align_class);
            }
            class_parts.push(format!("absolute {}", align_class));
        }
        if !self.custom_class.is_empty() {
            class_parts.push(format!("{}", self.custom_class));
        }
        let class = class_parts.join(" ");

        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), self.key.clone());
        attrs.insert("style".to_string(), style);
        attrs.insert("class".to_string(), class);

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
    align_absolute: String,
    align_subs: String,
    fixed: String,
    custom_class: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl ListRowView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, layout_tables: HashMap<String, Table>, is_scroll: bool) -> ListRowView {
        let table_value = table.get("value").and_then(|v| v.as_inline_table());
        let mut merged = InlineTable::new();
        if let Some(v) = value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        if let Some(v) = table_value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        let value = if merged.is_empty() {
            None
        } else {
            Some(&merged)
        };
        
        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);

        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        let custom_class = item_to_string(&table, "custom_class", "", value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);

        let mut view = ListRowView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "wrap", value),
            height: item_to_string(&table, "height", "wrap", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            is_scroll,
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            inner_padding: item_to_string(&table, "inner_padding", "0px", value),
            align_absolute: item_to_string(&table, "align_absolute", "", value),
            align_subs: item_to_string(&table, "align_subs", "start", value),
            fixed: item_to_string(&table, "fixed", "", value),
            custom_class,
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
                    .filter_map(|tbl| layout_to_tomlview(&view, layout.clone(), layout_tables.clone(), Some(tbl)).ok())
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

        let mut class_parts = vec![
            format!("items-{}", self.align_subs),
        ];
        if self.width.starts_with("w-") {
            class_parts.push(self.width.clone());
        } else if self.width != "wrap" {
            class_parts.push(format!("w-[{}]", self.width));
        }
        if self.height.starts_with("h-") {
            class_parts.push(self.height.clone());
        } else if self.height != "wrap" {
            class_parts.push(format!("h-[{}]", self.height));
        }
        if !self.align_absolute.is_empty() {
            let mut align_class = self.align_absolute.clone();

            if !align_class.chars().any(|c| c.is_numeric()) {
                align_class = format!("{}-0", align_class);
            }
            class_parts.push(format!("absolute {}", align_class));
        }
        if !self.custom_class.is_empty() {
            class_parts.push(format!("{}", self.custom_class));
        }
        let class = class_parts.join(" ");

        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), self.key.clone());
        attrs.insert("style".to_string(), style);
        attrs.insert("class".to_string(), class);

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

pub struct MarkdownListColumnView {
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
    align_absolute: String,
    align_subs: String,
    fixed: String,
    custom_class: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl MarkdownListColumnView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, layout_tables: HashMap<String, Table>, is_scroll: bool) -> MarkdownListColumnView {
        let table_value = table.get("value").and_then(|v| v.as_inline_table());
        let mut merged = InlineTable::new();
        if let Some(v) = value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        if let Some(v) = table_value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        let value = if merged.is_empty() {
            None
        } else {
            Some(&merged)
        };
        
        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);

        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        let custom_class = item_to_string(&table, "custom_class", "", value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);

        let mut view = MarkdownListColumnView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "wrap", value),
            height: item_to_string(&table, "height", "wrap", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            is_scroll,
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            inner_padding: item_to_string(&table, "inner_padding", "0px", value),
            align_absolute: item_to_string(&table, "align_absolute", "", value),
            align_subs: item_to_string(&table, "align_subs", "start", value),
            fixed: item_to_string(&table, "fixed", "", value),
            custom_class,
            value: value.cloned(),
            dark,
            views: vec![],
        };

        let layout = item_to_string(&table, "layout", "", value);
        let mut views: Vec<Box<dyn TOMLView>> = vec![];

        let index_folder = index_path.parent().unwrap_or(index_path);
        let files = item_to_string(&table, "files", "", value);
        let order_by = item_to_string(&table, "order_by", "", value);
        let file_paths = find_files(files.as_str(), index_folder);

        let order_by = order_by.trim(); // 예: "created" 또는 "created_desc"
        let (order_by, desc) = if let Some(k) = order_by.strip_suffix("_desc") {
            (k, true)
        } else {
            (order_by, false)
        };
        
        // (view, metas) 쌍으로 수집
        let mut view_with_meta: Vec<_> = file_paths
            .iter()
            .filter_map(|path| {
                let metas = metas_table_from_markdown(path.as_path()).ok()?;
                let view = layout_to_tomlview(&view, layout.clone(), layout_tables.clone(), Some(&metas)).ok()?;
                Some((view, metas))
            })
            .collect();

        if !order_by.is_empty() {
            if order_by == "created" {
                view_with_meta.sort_by_key(|(_, metas)| {
                    metas
                        .get("created")
                        .and_then(|v| v.as_str())
                        .and_then(|s| parse(s).ok()) // 여러 포맷 자동 인식
                        .unwrap_or_else(|| DateTime::<Utc>::MIN_UTC) // 실패 시 최소값
                });
            } else {
                view_with_meta.sort_by_key(|(_, metas)| {
                    metas
                        .get(&order_by)
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string()
                });
            }
        }
        if desc {
            view_with_meta.reverse();
        }

        // view만 꺼내서 views에 추가
        views.extend(view_with_meta.into_iter().map(|(v, _)| v));

        view.views = views;
        view
    }
}

impl TOMLView for MarkdownListColumnView {
    fn index_path(&self) -> PathBuf {
        return self.index_path.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn shape(&self) -> String {
        if self.is_scroll {
            return "mdlist_column".to_string();
        }
        return "mdlist_column".to_string();
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

        let mut class_parts = vec![
            format!("items-{}", self.align_subs),
        ];
        if self.width.starts_with("w-") {
            class_parts.push(self.width.clone());
        } else if self.width != "wrap" {
            class_parts.push(format!("w-[{}]", self.width));
        }
        if self.height.starts_with("h-") {
            class_parts.push(self.height.clone());
        } else if self.height != "wrap" {
            class_parts.push(format!("h-[{}]", self.height));
        }
        if !self.align_absolute.is_empty() {
            let mut align_class = self.align_absolute.clone();

            if !align_class.chars().any(|c| c.is_numeric()) {
                align_class = format!("{}-0", align_class);
            }
            class_parts.push(format!("absolute {}", align_class));
        }
        if !self.custom_class.is_empty() {
            class_parts.push(format!("{}", self.custom_class));
        }
        let class = class_parts.join(" ");

        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), self.key.clone());
        attrs.insert("style".to_string(), style);
        attrs.insert("class".to_string(), class);

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

impl AllocationView for MarkdownListColumnView {
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

pub struct MarkdownListRowView {
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
    align_absolute: String,
    align_subs: String,
    fixed: String,
    custom_class: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl MarkdownListRowView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, layout_tables: HashMap<String, Table>, is_scroll: bool) -> MarkdownListRowView {
        let table_value = table.get("value").and_then(|v| v.as_inline_table());
        let mut merged = InlineTable::new();
        if let Some(v) = value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        if let Some(v) = table_value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        let value = if merged.is_empty() {
            None
        } else {
            Some(&merged)
        };
        
        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);

        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        let custom_class = item_to_string(&table, "custom_class", "", value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);

        let mut view = MarkdownListRowView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "wrap", value),
            height: item_to_string(&table, "height", "wrap", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            is_scroll,
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            inner_padding: item_to_string(&table, "inner_padding", "0px", value),
            align_absolute: item_to_string(&table, "align_absolute", "", value),
            align_subs: item_to_string(&table, "align_subs", "start", value),
            fixed: item_to_string(&table, "fixed", "", value),
            custom_class,
            value: value.cloned(),
            dark,
            views: vec![],
        };

        let layout = item_to_string(&table, "layout", "", value);
        let mut views: Vec<Box<dyn TOMLView>> = vec![];

        let index_folder = index_path.parent().unwrap_or(index_path);
        let files = item_to_string(&table, "files", "", value);
        let file_paths = find_files(files.as_str(), index_folder);

        views.extend(
            file_paths
                .iter()
                .filter_map(|v| metas_table_from_markdown(v.as_path()).ok() )
                .filter_map(|tbl| layout_to_tomlview(&view, layout.clone(), layout_tables.clone(), Some(&tbl)).ok())
        );

        view.views = views;
        view
    }
}

impl TOMLView for MarkdownListRowView {
    fn index_path(&self) -> PathBuf {
        return self.index_path.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn shape(&self) -> String {
        if self.is_scroll {
            return "mdlist_row".to_string();
        }
        return "mdlist_row".to_string();
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

        let mut class_parts = vec![
            format!("items-{}", self.align_subs),
        ];
        if self.width.starts_with("w-") {
            class_parts.push(self.width.clone());
        } else if self.width != "wrap" {
            class_parts.push(format!("w-[{}]", self.width));
        }
        if self.height.starts_with("h-") {
            class_parts.push(self.height.clone());
        } else if self.height != "wrap" {
            class_parts.push(format!("h-[{}]", self.height));
        }
        if !self.align_absolute.is_empty() {
            let mut align_class = self.align_absolute.clone();

            if !align_class.chars().any(|c| c.is_numeric()) {
                align_class = format!("{}-0", align_class);
            }
            class_parts.push(format!("absolute {}", align_class));
        }
        if !self.custom_class.is_empty() {
            class_parts.push(format!("{}", self.custom_class));
        }
        let class = class_parts.join(" ");

        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), self.key.clone());
        attrs.insert("style".to_string(), style);
        attrs.insert("class".to_string(), class);

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

impl AllocationView for MarkdownListRowView {
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
    align_absolute: String,
    fixed: String,
    markdown_path: String,
    custom_class: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl MarkdownView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, layout_tables: HashMap<String, Table>, is_scroll: bool) -> MarkdownView {
        let table_value = table.get("value").and_then(|v| v.as_inline_table());
        let mut merged = InlineTable::new();
        if let Some(v) = value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        if let Some(v) = table_value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        let value = if merged.is_empty() {
            None
        } else {
            Some(&merged)
        };
        
        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);

        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        let custom_class = item_to_string(&table, "custom_class", "", value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);

        MarkdownView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "wrap", value),
            height: item_to_string(&table, "height", "wrap", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            is_scroll,
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            inner_padding: item_to_string(&table, "inner_padding", "0px", value),
            align_absolute: item_to_string(&table, "align_absolute", "", value),
            fixed: item_to_string(&table, "fixed", "", value),
            markdown_path: item_to_string(&table, "markdown_path", "", value),
            custom_class,
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
            // "flex-shrink: 0".to_string(),
        ];
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

        let mut class_parts = vec![
        ];
        if self.width.starts_with("w-") {
            class_parts.push(self.width.clone());
        } else if self.width != "wrap" {
            class_parts.push(format!("w-[{}]", self.width));
        }
        if self.height.starts_with("h-") {
            class_parts.push(self.height.clone());
        } else if self.height != "wrap" {
            class_parts.push(format!("h-[{}]", self.height));
        }
        if !self.align_absolute.is_empty() {
            let mut align_class = self.align_absolute.clone();

            if !align_class.chars().any(|c| c.is_numeric()) {
                align_class = format!("{}-0", align_class);
            }
            class_parts.push(format!("absolute {}", align_class));
        }
        if !self.custom_class.is_empty() {
            class_parts.push(format!("{}", self.custom_class));
        }
        let class = class_parts.join(" ");

        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), self.key.clone());
        attrs.insert("style".to_string(), style);
        attrs.insert("class".to_string(), class);

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

pub struct GridView {
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
    align_absolute: String,
    fixed: String,
    row_count: String,
    custom_class: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl GridView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, layout_tables: HashMap<String, Table>, is_scroll: bool) -> GridView {
        let table_value = table.get("value").and_then(|v| v.as_inline_table());
        let mut merged = InlineTable::new();
        if let Some(v) = value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        if let Some(v) = table_value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        let value = if merged.is_empty() {
            None
        } else {
            Some(&merged)
        };
        
        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);

        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        let custom_class = item_to_string(&table, "custom_class", "", value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);

        let mut view = GridView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "wrap", value),
            height: item_to_string(&table, "height", "wrap", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            is_scroll,
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            inner_padding: item_to_string(&table, "inner_padding", "0px", value),
            align_absolute: item_to_string(&table, "align_absolute", "", value),
            fixed: item_to_string(&table, "fixed", "", value),
            row_count: item_to_string(&table, "row_count", "3", value),
            custom_class,
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
                    .filter_map(|tbl| layout_to_tomlview(&view, layout.clone(), layout_tables.clone(), Some(tbl)).ok())
            );
        }

        view.views = views;
        view
    }
}

impl TOMLView for GridView {
    fn index_path(&self) -> PathBuf {
        return self.index_path.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn shape(&self) -> String {
        if self.is_scroll {
            return "grid".to_string();
        }
        return "grid".to_string();
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
            // "display:flex".to_string(),
            // "flex-direction:row".to_string(),
        ];
        if self.is_scroll {
            style_parts.push("overflow-x: auto".to_string());
        } else {
            style_parts.push("overflow: hidden".to_string());
        }
        if !self.fixed.is_empty() {
            style_parts.push(format!("position: fixed; {}: 0", self.fixed));
        }

        // if !self.inner_padding.trim().is_empty() {
        //     style_parts.push(format!("gap:{}", self.inner_padding));
        // }

        let style = style_parts.join("; ") + ";"; // 끝에 세미콜론

        let mut class_parts = vec![
            format!("grid grid-cols-{} gap-[{}]", self.row_count, self.inner_padding),
        ];
        if self.width.starts_with("w-") {
            class_parts.push(self.width.clone());
        } else if self.width != "wrap" {
            class_parts.push(format!("w-[{}]", self.width));
        }
        if self.height.starts_with("h-") {
            class_parts.push(self.height.clone());
        } else if self.height != "wrap" {
            class_parts.push(format!("h-[{}]", self.height));
        }
        if !self.align_absolute.is_empty() {
            let mut align_class = self.align_absolute.clone();

            if !align_class.chars().any(|c| c.is_numeric()) {
                align_class = format!("{}-0", align_class);
            }
            class_parts.push(format!("absolute {}", align_class));
        }
        if !self.custom_class.is_empty() {
            class_parts.push(format!("{}", self.custom_class));
        }
        let class = class_parts.join(" ");

        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), self.key.clone());
        attrs.insert("style".to_string(), style);
        attrs.insert("class".to_string(), class);

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

pub struct EmbedView {
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
    align_absolute: String,
    fixed: String,
    custom_class: String,
    value: Option<InlineTable>,
    dark: bool,
    views: Vec<Box<dyn TOMLView>>,
}

impl EmbedView {
    pub fn new(index_path: &Path, key: &str, table: &Table, value: Option<&InlineTable>, super_view: Option<&dyn TOMLView>, layout_tables: HashMap<String, Table>, is_scroll: bool) -> EmbedView {
        let table_value = table.get("value").and_then(|v| v.as_inline_table());
        let mut merged = InlineTable::new();
        if let Some(v) = value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        if let Some(v) = table_value {
            for (k, v) in v.iter() {
                merged.insert(k, v.clone());
            }
        }
        let value = if merged.is_empty() {
            None
        } else {
            Some(&merged)
        };
        
        let horizontal_padding = item_to_string(&table, "horizontal_padding", "0px", value);
        let vertical_padding = item_to_string(&table, "vertical_padding", "0px", value);

        let left_outer_padding = item_to_string(&table, "left_outer_padding", horizontal_padding.as_str(), value);
        let top_outer_padding = item_to_string(&table, "top_outer_padding", vertical_padding.as_str(), value);
        let right_outer_padding = item_to_string(&table, "right_outer_padding", horizontal_padding.as_str(), value);
        let bottom_outer_padding = item_to_string(&table, "bottom_outer_padding", vertical_padding.as_str(), value);

        let custom_class = item_to_string(&table, "custom_class", "", value);

        let default_dark = super_view
            .and_then(|x| Some(x.dark()))
            .unwrap_or(false);
        let dark = item_to_bool(&table, "dark", default_dark, value);

        let mut view = EmbedView {
            index_path: index_path.to_path_buf(),
            key: key.to_string(),
            width: item_to_string(&table, "width", "wrap", value),
            height: item_to_string(&table, "height", "wrap", value),
            background: item_to_string(&table, "background", "transparent", value),
            path: item_to_string(&table, "path", "", value),
            is_scroll,
            left_outer_padding,
            top_outer_padding,
            right_outer_padding,
            bottom_outer_padding,
            inner_padding: item_to_string(&table, "inner_padding", "0px", value),
            align_absolute: item_to_string(&table, "align_absolute", "", value),
            fixed: item_to_string(&table, "fixed", "", value),
            custom_class,
            value: value.cloned(),
            dark: dark,
            views: vec![],
        };

        let layout = item_to_string(&table, "layout", "", value);

        if let Some(layout_view) = layout_tables.iter()
            .filter_map(|(key, x)| table_to_tomlview(index_path, key, x, value, Some(&view), layout_tables.clone()).ok())
            .filter(|x| x.key() == layout)
            .next() {
                view.views = vec![layout_view];
            }
        view
    }
}

impl TOMLView for EmbedView {
    fn index_path(&self) -> PathBuf {
        return self.index_path.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn shape(&self) -> String {
        if self.is_scroll {
            return "embed".to_string();
        }
        return "embed".to_string();
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
        let views: Vec<HTMLView> = self.views.iter().map(|x| x.htmlview(Some(self))).collect();
        let sub_classes: Vec<String> = views.clone()
            .first()
            .and_then(|first_view| first_view.attrs.get("class"))  // Option<&String>
            .map(|class_str| {
                class_str
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_else(Vec::new);  // None이면 빈 Vec 반환

        let mut style_parts = vec![
            format!("background:{}", self.background),
            format!(
                "padding:{} {} {} {}",
                self.top_outer_padding,
                self.right_outer_padding,
                self.bottom_outer_padding,
                self.left_outer_padding,
            ),
        ];
        if !self.fixed.is_empty() {
            style_parts.push(format!("position: fixed; {}: 0", self.fixed));
        }

        if !self.inner_padding.trim().is_empty() {
            style_parts.push(format!("gap:{}", self.inner_padding));
        }

        let style = style_parts.join("; ") + ";"; // 끝에 세미콜론

        let mut class_parts = vec![
        ];
        if self.width.starts_with("w-") {
            class_parts.push(self.width.clone());
        } else if self.width != "wrap" {
            class_parts.push(format!("w-[{}]", self.width));
        } else {
            let sub_width = sub_classes.iter().find(|&s| s.starts_with("w-"));
            sub_width.map(|width_class| class_parts.push(width_class.clone()));
        }
        if self.height.starts_with("h-") {
            class_parts.push(self.height.clone());
        } else if self.height != "wrap" {
            class_parts.push(format!("h-[{}]", self.height));
        } else {
            let sub_height = sub_classes.iter().find(|&s| s.starts_with("h-"));
            sub_height.map(|width_class| class_parts.push(width_class.clone()));
        }
        if !self.align_absolute.is_empty() {
            let mut align_class = self.align_absolute.clone();

            if !align_class.chars().any(|c| c.is_numeric()) {
                align_class = format!("{}-0", align_class);
            }
            class_parts.push(format!("absolute {}", align_class));
        }
        if !self.custom_class.is_empty() {
            class_parts.push(format!("{}", self.custom_class));
        }
        let class = class_parts.join(" ");

        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), format!("embed_{}", self.key.clone()));
        attrs.insert("style".to_string(), style);
        attrs.insert("class".to_string(), class);

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