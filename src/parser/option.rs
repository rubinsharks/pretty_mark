use std::fs;
use std::path::{Path, PathBuf};
use crate::parser::Page;
use std::fs::File;
use std::io::Read;
use toml_edit::{DocumentMut, value, Value, Item, Table};
use std::collections::HashMap;
use crate::parser::markdown::filter_attrs;

pub fn load_option(path: &Path) -> Result<MDOption, &'static str> {
    match find_option(path) {
        Ok(option) => {
            match load_option_from_toml(option.as_path()) {
                Ok(option) => {
                    return Ok(option);
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
        Err(_) => {
            Err("no option file detected")
        }
    }
}

pub fn find_option(path: &Path) -> Result<PathBuf, &'static str> {
    let paths = fs::read_dir(path).ok().ok_or("")?;
    let mut option_paths: Vec<PathBuf> = vec![];
    for path in paths.filter_map(|x| x.ok()).map(|x| x.path()) {
        match path.extension() {
            None => { }
            Some(extension) => {
                if extension == "toml" {
                    option_paths.push(path);
                }
            }
        }
    }
    match option_paths.len() {
        1 => Ok(option_paths.first().unwrap().to_path_buf()),
        0 => Err("no md file detected"),
        _ => Err("multiple md files detected")
    }
}

pub fn load_option_from_toml(path: &Path) -> Result<MDOption, &'static str> {
    let mut file = File::open(path).ok().ok_or("Failed to open the file")?;

    let mut config_str = String::new();
    file.read_to_string(&mut config_str)
        .ok().ok_or("Failed to read the file")?;

    let mut doc = config_str.parse::<DocumentMut>()
        .ok().ok_or("invalid doc")?;

    let mut menus = vec![];
    let mut themes = HashMap::new();
    let mut footer = Footer { title: "".to_string(), snss: vec![] };
    let mut basic = Basic { title: "".to_string(), created: "".to_string(), tags: vec![] };

    // 파일 순서대로 키를 접근
    for (key, value) in doc.as_table() {
        if key == "nav" {
            if let Item::Table(table) = value {
                menus = table_to_menus(table);
            }
        } else if key == "theme" {
            if let Item::Table(table) = value {
                themes = table_to_themes(table);
            }
        } else if key == "footer" {
            if let Item::Table(table) = value {
                footer = table_to_footer(table);
            }
        } else if key == "basic" {
            if let Item::Table(table) = value {
                basic = table_to_basic(table);
            }
        }
    }

    let option = MDOption {
        menus,
        themes,
        footer,
        basic,
    };

    Ok(option)
}

#[derive(Clone)]
pub struct MDOption {
    pub menus: Vec<Menu>,
    pub themes: HashMap<String, ThemeValue>,
    pub footer: Footer,
    pub basic: Basic,
}

impl MDOption {
    pub fn menus_to_html(&self) -> String {
        if (*self).menus.is_empty() {
            "".to_string()
        } else {
            menus_to_html(&self.menus, &Some(self.clone()))
        }
    }

    pub fn footer_to_html(&self) -> String {
        if (*self).footer.title.is_empty() && (*self).footer.snss.is_empty() {
            "".to_string()
        } else {
            footer_to_html(&self.footer, &Some(self.clone()))
        }
    }

    pub fn is_night(&self) -> bool {
        if let Some(ThemeValue::Bool(is_night)) = self.themes.get("night") {
            return is_night.clone();
        }
        false
    }

    pub fn tag(&self) -> &Vec<String> {
        &self.basic.tags
    }
}

#[derive(Clone)]
struct Menu {
    name: String,
    path: String,
    dropdowns: Vec<DropDown>,
}

#[derive(Clone)]
struct DropDown {
    name: String,
    path: String,
}

fn menus_to_html(menus: &Vec<Menu>, md_option: &Option<MDOption>) -> String {
    let mut html = String::new();
    html.push_str(&format!(r#"<nav class="{}">"#, filter_attrs("bg-white border-gray-200 dark:bg-gray-900 dark:border-gray-700", md_option)));
    html.push_str(
        &format!("<div class=\"{}\">", filter_attrs("max-w-screen-xl flex flex-wrap items-center justify-between mx-auto p-4", md_option))
    );

    // logo
    html.push_str(&format!(r###"<a href="/" class="{}">"###, filter_attrs("flex items-center space-x-3 rtl:space-x-reverse", md_option)));
    // html.push_str(&format!(r#"<img src="https://flowbite.com/docs/images/logo.svg" class="{}" alt="Flowbite Logo" />"#, filter_attrs("h-8", md_option)));
    if let Some(option) = md_option {
        html.push_str(&format!(r#"<span class="{}">{}</span>"#, filter_attrs("self-center text-2xl font-semibold whitespace-nowrap dark:text-white", md_option), option.basic.title));
    }
    html.push_str("</a>");

    // collapse
    html.push_str(&format!(r#"<button data-collapse-toggle="navbar-dropdown" type="button" class="{}" aria-controls="navbar-dropdown" aria-expanded="false">"#, filter_attrs("inline-flex items-center p-2 w-10 h-10 justify-center text-sm text-gray-500 rounded-lg md:hidden hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-gray-200 dark:text-gray-400 dark:hover:bg-gray-700 dark:focus:ring-gray-600", md_option)));
    html.push_str(&format!(r#"<span class="{}">Open main menu</span>"#, filter_attrs("sr-only", md_option)));
    html.push_str(&format!(r#"<svg class="{}" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 17 14">"#, filter_attrs("w-5 h-5", md_option)));
    html.push_str(r#"<path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M1 1h15M1 7h15M1 13h15"/>"#);
    html.push_str("</svg>");
    html.push_str("</button>");

    // menus
    html.push_str(&format!(r#"<div class="{}" id="navbar-dropdown">"#, filter_attrs("hidden w-full md:block md:w-auto", md_option)));
    html.push_str(&format!(r#"<ul class="{}">"#, filter_attrs("flex flex-col font-medium p-4 md:p-0 mt-4 border border-gray-100 rounded-lg bg-gray-50 md:space-x-8 rtl:space-x-reverse md:flex-row md:mt-0 md:border-0 md:bg-white dark:bg-gray-800 md:dark:bg-gray-900 dark:border-gray-700", md_option)));
    for menu in menus {
        html.push_str("<li>");
        if menu.dropdowns.is_empty() {
            html.push_str(&format!("<a href=\"{}\" class=\"{}\" aria-current=\"page\">{}</a>", menu.path, filter_attrs("block py-2 px-3 text-gray-900 rounded-sm hover:bg-gray-100 md:hover:bg-transparent md:border-0 md:hover:text-blue-700 md:p-0 dark:text-white md:dark:hover:text-blue-500 dark:hover:bg-gray-700 dark:hover:text-white md:dark:hover:bg-transparent", md_option), menu.name));
        } else {
            html.push_str(&format!(r#"<button id="dropdownNavbarLink" data-dropdown-toggle="dropdownNavbar-{}" class="{}">{}"#, menu.name, filter_attrs("flex items-center justify-between w-full py-2 px-3 text-gray-900 rounded-sm hover:bg-gray-100 md:hover:bg-transparent md:border-0 md:hover:text-blue-700 md:p-0 md:w-auto dark:text-white md:dark:hover:text-blue-500 dark:focus:text-white dark:border-gray-700 dark:hover:bg-gray-700 md:dark:hover:bg-transparent", md_option), menu.name));
            html.push_str("<svg class=\"w-2.5 h-2.5 ms-2.5\" aria-hidden=\"true\" xmlns=\"http://www.w3.org/2000/svg\" fill=\"none\" viewBox=\"0 0 10 6\">");
            html.push_str("<path stroke=\"currentColor\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" d=\"m1 1 4 4 4-4\"/>");
            html.push_str("</svg>");
            html.push_str("</button>");
            // dropdown
            html.push_str(&format!(r#"<div id="dropdownNavbar-{}" class="{}">"#, menu.name, filter_attrs("z-10 hidden font-normal bg-white divide-y divide-gray-100 rounded-lg shadow-sm w-44 dark:bg-gray-700 dark:divide-gray-600", md_option)));
            html.push_str(&format!("<ul class=\"{}\" aria-labelledby=\"dropdownLargeButton\">", filter_attrs("py-2 text-sm text-gray-700 dark:text-gray-400", md_option)));
            for dropdown in &menu.dropdowns {
                html.push_str("<li>");
                html.push_str(&format!("<a href=\"{}\" class=\"{}\">{}</a>", dropdown.path, filter_attrs("block px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-600 dark:hover:text-white", md_option), dropdown.name));
                html.push_str("</li>");
            }
            html.push_str("</ul>");
        }
        html.push_str("</li>");
    }
    html.push_str("</ul>");
    html.push_str("</div>");
    html.push_str("</div>");
    html.push_str("</nav>");
    html.push_str("\n");
    html
}

fn table_to_menus(table: &Table) -> Vec<Menu> {
    let mut menus: Vec<Menu> = vec![];
    for (key, value) in table {
        if let Item::Table(table) = value {
            let dropdowns = table_to_dropdowns(table);
            let menu = Menu {
                name: key.to_string(),
                path: "".to_string(),
                dropdowns,
            };
            menus.push(menu);
        } else if let Item::Value(Value::String(path)) = value {
            let menu = Menu {
                name: key.to_string(),
                path: ensure_leading_slash(path.to_string().trim().trim_matches('"')),
                dropdowns: vec![],
            };
            menus.push(menu);
        } else {
            let menu = Menu {
                name: key.to_string(),
                path: "".to_string(),
                dropdowns: vec![],
            };
            menus.push(menu);
        }
    }
    menus
}

fn ensure_leading_slash(s: &str) -> String {
    if s.starts_with('/') {
        s.to_string()
    } else {
        format!("/{}", s)
    }
}

fn table_to_dropdowns(table: &Table) -> Vec<DropDown> {
    let mut dropdowns: Vec<DropDown> = vec![];
    for (key, value) in table {
        if let Item::Value(Value::String(path)) = value {
            let dropdown = DropDown {
                name: key.to_string(),
                path: path.to_string().trim().trim_matches('"').to_string(),
            };
            dropdowns.push(dropdown);
        } else {
            let dropdown = DropDown {
                name: key.to_string(),
                path: "".to_string(),
            };
            dropdowns.push(dropdown);
        }
    }
    dropdowns
}

#[derive(Clone)]
pub struct Basic {
    title: String,
    created: String,
    pub tags: Vec<String>,
}

#[derive(Clone)]
pub struct Footer {
    title: String,
    snss: Vec<FooterSNS>,
}

#[derive(Clone)]
pub struct FooterSNS {
    name: String,
    path: String
}

impl FooterSNS {
    fn svg_html(&self) -> &str {
        match self.name.as_str() {
            "facebook" => r#"
                <svg class="w-4 h-4" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 8 19">
                    <path fill-rule="evenodd" d="M6.135 3H8V0H6.135a4.147 4.147 0 0 0-4.142 4.142V6H0v3h2v9.938h3V9h2.021l.592-3H5V3.591A.6.6 0 0 1 5.592 3h.543Z" clip-rule="evenodd"/>
                </svg>
            "#,
            "discord" => r#"
                <svg class="w-4 h-4" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 21 16">
                    <path d="M16.942 1.556a16.3 16.3 0 0 0-4.126-1.3 12.04 12.04 0 0 0-.529 1.1 15.175 15.175 0 0 0-4.573 0 11.585 11.585 0 0 0-.535-1.1 16.274 16.274 0 0 0-4.129 1.3A17.392 17.392 0 0 0 .182 13.218a15.785 15.785 0 0 0 4.963 2.521c.41-.564.773-1.16 1.084-1.785a10.63 10.63 0 0 1-1.706-.83c.143-.106.283-.217.418-.33a11.664 11.664 0 0 0 10.118 0c.137.113.277.224.418.33-.544.328-1.116.606-1.71.832a12.52 12.52 0 0 0 1.084 1.785 16.46 16.46 0 0 0 5.064-2.595 17.286 17.286 0 0 0-2.973-11.59ZM6.678 10.813a1.941 1.941 0 0 1-1.8-2.045 1.93 1.93 0 0 1 1.8-2.047 1.919 1.919 0 0 1 1.8 2.047 1.93 1.93 0 0 1-1.8 2.045Zm6.644 0a1.94 1.94 0 0 1-1.8-2.045 1.93 1.93 0 0 1 1.8-2.047 1.918 1.918 0 0 1 1.8 2.047 1.93 1.93 0 0 1-1.8 2.045Z"/>
                </svg>
            "#,
            "twitter" => r#"
                <svg class="w-4 h-4" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 17">
                    <path fill-rule="evenodd" d="M20 1.892a8.178 8.178 0 0 1-2.355.635 4.074 4.074 0 0 0 1.8-2.235 8.344 8.344 0 0 1-2.605.98A4.13 4.13 0 0 0 13.85 0a4.068 4.068 0 0 0-4.1 4.038 4 4 0 0 0 .105.919A11.705 11.705 0 0 1 1.4.734a4.006 4.006 0 0 0 1.268 5.392 4.165 4.165 0 0 1-1.859-.5v.05A4.057 4.057 0 0 0 4.1 9.635a4.19 4.19 0 0 1-1.856.07 4.108 4.108 0 0 0 3.831 2.807A8.36 8.36 0 0 1 0 14.184 11.732 11.732 0 0 0 6.291 16 11.502 11.502 0 0 0 17.964 4.5c0-.177 0-.35-.012-.523A8.143 8.143 0 0 0 20 1.892Z" clip-rule="evenodd"/>
                </svg>
            "#,
            "github" => r#"
                <svg class="w-4 h-4" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M10 .333A9.911 9.911 0 0 0 6.866 19.65c.5.092.678-.215.678-.477 0-.237-.01-1.017-.014-1.845-2.757.6-3.338-1.169-3.338-1.169a2.627 2.627 0 0 0-1.1-1.451c-.9-.615.07-.6.07-.6a2.084 2.084 0 0 1 1.518 1.021 2.11 2.11 0 0 0 2.884.823c.044-.503.268-.973.63-1.325-2.2-.25-4.516-1.1-4.516-4.9A3.832 3.832 0 0 1 4.7 7.068a3.56 3.56 0 0 1 .095-2.623s.832-.266 2.726 1.016a9.409 9.409 0 0 1 4.962 0c1.89-1.282 2.717-1.016 2.717-1.016.366.83.402 1.768.1 2.623a3.827 3.827 0 0 1 1.02 2.659c0 3.807-2.319 4.644-4.525 4.889a2.366 2.366 0 0 1 .673 1.834c0 1.326-.012 2.394-.012 2.72 0 .263.18.572.681.475A9.911 9.911 0 0 0 10 .333Z" clip-rule="evenodd"/>
                  </svg>
            "#,
            "dribble" => r#"
                <svg class="w-4 h-4" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M10 0a10 10 0 1 0 10 10A10.009 10.009 0 0 0 10 0Zm6.613 4.614a8.523 8.523 0 0 1 1.93 5.32 20.094 20.094 0 0 0-5.949-.274c-.059-.149-.122-.292-.184-.441a23.879 23.879 0 0 0-.566-1.239 11.41 11.41 0 0 0 4.769-3.366ZM8 1.707a8.821 8.821 0 0 1 2-.238 8.5 8.5 0 0 1 5.664 2.152 9.608 9.608 0 0 1-4.476 3.087A45.758 45.758 0 0 0 8 1.707ZM1.642 8.262a8.57 8.57 0 0 1 4.73-5.981A53.998 53.998 0 0 1 9.54 7.222a32.078 32.078 0 0 1-7.9 1.04h.002Zm2.01 7.46a8.51 8.51 0 0 1-2.2-5.707v-.262a31.64 31.64 0 0 0 8.777-1.219c.243.477.477.964.692 1.449-.114.032-.227.067-.336.1a13.569 13.569 0 0 0-6.942 5.636l.009.003ZM10 18.556a8.508 8.508 0 0 1-5.243-1.8 11.717 11.717 0 0 1 6.7-5.332.509.509 0 0 1 .055-.02 35.65 35.65 0 0 1 1.819 6.476 8.476 8.476 0 0 1-3.331.676Zm4.772-1.462A37.232 37.232 0 0 0 13.113 11a12.513 12.513 0 0 1 5.321.364 8.56 8.56 0 0 1-3.66 5.73h-.002Z" clip-rule="evenodd"/>
                </svg>
            "#,
            _ => "", // Return empty string if name doesn't match
        }
    }
}

/// z-10이 footer의 height
fn footer_to_html(footer: &Footer, md_option: &Option<MDOption>) -> String {
    let mut html = String::new();

    html.push_str(&format!(r#"<footer class="{}">"#, filter_attrs("fixed bottom-0 left-0 z-20 w-full p-4 bg-white border-t border-gray-200 shadow-sm md:p-4 dark:bg-gray-800 dark:border-gray-600", md_option)));
    
    html.push_str(&format!(r#"<div class="{}">"#, filter_attrs("max-w-screen-xl mx-auto sm:flex sm:items-center justify-between", md_option)));
    html.push_str(&format!("<span class=\"{}\">{}", filter_attrs("text-sm text-gray-500 sm:text-center dark:text-gray-400 truncate", md_option), footer.title));
    html.push_str("</span>");
    html.push_str(&format!("<div class=\"{}\">", filter_attrs("flex flex-wrap items-center mt-1 text-sm font-medium text-gray-500 dark:text-gray-400 sm:mt-0", md_option)));
    for sns in &footer.snss {
        html.push_str(&format!("<a href={} class=\"{}\">", sns.path, filter_attrs("text-gray-500 hover:text-gray-900 dark:hover:text-white ms-5", md_option)));
        html.push_str(sns.svg_html());
        html.push_str("</a>");
    }
    html.push_str("</div>");
    html.push_str("</div>");
    html.push_str("</footer>");
    html
}

fn table_to_footer(table: &Table) -> Footer {
    let mut footer = Footer {
        title: "".to_string(),
        snss: vec![],
    };
    for (key, value) in table {
        if let Item::Table(table) = value {
            let snss = table_to_snss(table);
            footer.snss = snss;
        } else if let Item::Value(Value::String(path)) = value {
            if key == "title" {
                footer.title = path.to_string().trim().trim_matches('"').to_string();
            }
        } else {
        }
    }
    footer
}

fn table_to_snss(table: &Table) -> Vec<FooterSNS> {
    let mut snss: Vec<FooterSNS> = vec![];
    for (key, value) in table {
        if let Item::Value(Value::String(path)) = value {
            let sns = FooterSNS {
                name: key.to_string(),
                path: path.to_string().trim().trim_matches('"').to_string(),
            };
            snss.push(sns);
        } else {
            let sns = FooterSNS {
                name: key.to_string(),
                path: "".to_string(),
            };
            snss.push(sns);
        }
    }
    snss
}

#[derive(Clone)]    
pub enum ThemeValue {
    String(String),
    Bool(bool),
    Float(f64),
}

fn table_to_themes(table: &Table) -> HashMap<String, ThemeValue> {
    let mut hash_map: HashMap<String, ThemeValue> = HashMap::new();

    for (key, value) in table {
        if let Item::Value(Value::Boolean(bool)) = value {
            hash_map.insert(key.to_string(), ThemeValue::Bool(bool.value().clone()));
        } else if let Item::Value(Value::String(str)) = value {
            hash_map.insert(key.to_string(), ThemeValue::String(str.value().clone()));
        } else {

        }
    }
    hash_map
}

fn table_to_basic(table: &Table) -> Basic {
    let mut basic = Basic {
        title: "".to_string(),
        created: "".to_string(),
        tags: vec![],
    };
    for (key, value) in table {
        if let Item::Value(Value::String(path)) = value {
            if key == "title" {
                basic.title = path.to_string().trim().trim_matches('"').to_string();
            } else if key == "created" {
                basic.created = path.to_string().trim().trim_matches('"').to_string();
            } else if key == "tag" {
                basic.tags = vec![path.to_string().trim().trim_matches('"').to_string()];
            }
        } else if let Item::Value(Value::Array(path)) = value {
            basic.tags = path.iter().filter_map(|x| {
                    if let Value::String(s) = x {
                        Some(s.to_string().trim().trim_matches('"').to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>();
        }
    }
    basic
}
