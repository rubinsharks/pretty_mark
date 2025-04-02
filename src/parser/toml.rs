
use std::fs::File;
use std::io::Read;
use std::path::{Display, Path};
use toml_edit::{DocumentMut, value, Value, Item, Table};

pub fn load_toml(path: &Path) -> String {
    let mut file = File::open(path).expect("Failed to open the file");

    let mut config_str = String::new();
    file.read_to_string(&mut config_str)
        .expect("Failed to read the file");


    let mut doc = config_str.parse::<DocumentMut>().expect("invalid doc");

    // 파일 순서대로 키를 접근
    for (key, value) in doc.as_table() {
        if key == "nav" {
            if let Item::Table(table) = value {
                let menus = table_to_menus(table);
                let html = menus_to_html(menus);
                println!("{:?}", html);
                for (key, value) in table {
                    println!("{} = {}", key, value);
                }
                return html;
            }
        }
    }
    
    "".to_string()
}

struct Menu {
    name: String,
    path: String,
    dropdowns: Vec<DropDown>,
}

struct DropDown {
    name: String,
    path: String,
}

fn menus_to_html(menus: Vec<Menu>) -> String {
    let mut html = String::new();
    html.push_str("<nav class=\"bg-white border-gray-200 dark:bg-gray-900 dark:border-gray-700\">");
    html.push_str(
        "<div class=\"max-w-screen-xl flex flex-wrap items-center justify-between mx-auto p-4\">",
    );
    html.push_str("<div class=\"hidden w-full md:block md:w-auto\" id=\"navbar-dropdown\">");
    html.push_str("<ul class=\"flex flex-col font-medium p-4 md:p-0 mt-4 border border-gray-100 rounded-lg bg-gray-50 md:space-x-8 rtl:space-x-reverse md:flex-row md:mt-0 md:border-0 md:bg-white dark:bg-gray-800 md:dark:bg-gray-900 dark:border-gray-700\">");
    for menu in menus {
        html.push_str("<li>");
        if menu.dropdowns.is_empty() {
            html.push_str(format!("<a href=\"{}\" class=\"block py-2 px-3 text-gray-900 rounded-sm hover:bg-gray-100 md:hover:bg-transparent md:border-0 md:hover:text-blue-700 md:p-0 dark:text-white md:dark:hover:text-blue-500 dark:hover:bg-gray-700 dark:hover:text-white md:dark:hover:bg-transparent\" aria-current=\"page\">{}</a>", menu.path, menu.name).as_str());
        } else {
            html.push_str(format!("<button id=\"dropdownNavbarLink\" data-dropdown-toggle=\"dropdownNavbar\" class=\"flex items-center justify-between w-full py-2 px-3 text-gray-900 rounded-sm hover:bg-gray-100 md:hover:bg-transparent md:border-0 md:hover:text-blue-700 md:p-0 md:w-auto dark:text-white md:dark:hover:text-blue-500 dark:focus:text-white dark:border-gray-700 dark:hover:bg-gray-700 md:dark:hover:bg-transparent\">{}", menu.name).as_str());
            html.push_str("<svg class=\"w-2.5 h-2.5 ms-2.5\" aria-hidden=\"true\" xmlns=\"http://www.w3.org/2000/svg\" fill=\"none\" viewBox=\"0 0 10 6\">");
            html.push_str("<path stroke=\"currentColor\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" d=\"m1 1 4 4 4-4\"/>");
            html.push_str("</svg>");
            html.push_str("</button>");
            // dropdown
            html.push_str("<div id=\"dropdownNavbar\" class=\"z-10 hidden font-normal bg-white divide-y divide-gray-100 rounded-lg shadow-sm w-44 dark:bg-gray-700 dark:divide-gray-600\">");
            html.push_str("<ul class=\"py-2 text-sm text-gray-700 dark:text-gray-400\" aria-labelledby=\"dropdownLargeButton\">");
            for dropdown in menu.dropdowns {
                html.push_str("<li>");
                html.push_str(format!("<a href=\"{}\" class=\"block px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-600 dark:hover:text-white\">{}</a>", dropdown.path, dropdown.name).as_str());
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
                path: path.to_string().trim().trim_matches('"').to_string(),
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