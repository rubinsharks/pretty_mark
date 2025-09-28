use std::collections::HashMap;

use maplit::hashmap;
use toml_edit::{Item, Table};
use crate::html::{filter_attrs, HTMLView};
use super::svg::{svg_dropdown, svg_menu};
use crate::common::SlashNormalize;


pub fn make_nav_sub_menus(title: String, sub_menus: &Table, is_dark: bool) -> HTMLView {
  let svg_dropdown = svg_dropdown();
  let dropdown_navbar_link = HTMLView::new("button", hashmap! {
      "id".to_string() => "dropdownNavbarLink".to_string(),
      "data-dropdown-toggle".to_string() => format!("dropdownNavbar:{}", title.as_str()),
      "class".to_string() => filter_attrs("flex items-center justify-between w-full py-2 px-3 text-gray-900 rounded-sm hover:bg-gray-100 md:hover:bg-transparent md:border-0 md:hover:text-blue-700 md:p-0 md:w-auto dark:text-white md:dark:hover:text-blue-500 dark:focus:text-white dark:border-gray-700 dark:hover:bg-gray-700 md:dark:hover:bg-transparent", is_dark),
  }, title.as_str(), vec![svg_dropdown]);

  let sub_menu_views = sub_menus
      .iter()
      .map(|(key, value)| 
          HTMLView::new("a", hashmap!{
              "href".to_string() => value.as_str().unwrap_or("").ensure_slashes(),
              "class".to_string() => filter_attrs("block px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-600 dark:hover:text-white", is_dark)
          }, key, vec![]).wrap_tag("li", hashmap! {})
      )
      .collect();
  let dropdown_navbar_ul = HTMLView::new("ul", hashmap! {
      "class".to_string() => filter_attrs("py-2 text-sm text-gray-700 dark:text-gray-400", is_dark),
      "aria-labelledby".to_string() => "dropdownLargeButton".to_string()  
  }, "", sub_menu_views);
  let dropdown_navbar = HTMLView::new("div", hashmap! {
      "id".to_string() => format!("dropdownNavbar:{}", title.as_str()),
      "class".to_string() => filter_attrs("z-10 hidden font-normal bg-white divide-y divide-gray-100 rounded-lg shadow-sm w-44 dark:bg-gray-700 dark:divide-gray-600", is_dark),
  }, "", vec![dropdown_navbar_ul]);
  HTMLView::new("li", hashmap! {}, "", vec![dropdown_navbar_link, dropdown_navbar])
}

pub fn make_nav(title: String, menus: Vec<String>, sub_menus: HashMap<String, Item>, is_dark: bool) -> HTMLView {
  let nav_menus = menus
      .iter()
      .map(|menu| {
          if let Some(sub) = sub_menus.get(menu) {
              if let Item::Value(value) = sub {
                  HTMLView::new("a", hashmap! {
                      "href".to_string() => value.as_str().unwrap_or("").ensure_slashes(),
                      "class".to_string() => filter_attrs("block py-2 px-3 text-gray-900 rounded-sm hover:bg-gray-100 md:hover:bg-transparent md:border-0 md:hover:text-blue-700 md:p-0 dark:text-white md:dark:hover:text-blue-500 dark:hover:bg-gray-700 dark:hover:text-white md:dark:hover:bg-transparent", is_dark),
                      "aria-current".to_string() => "page".to_string()
                  }, menu, vec![])
                      .wrap_tag("li", hashmap! {})
              } else if let Item::Table(table) = sub {
                  make_nav_sub_menus(menu.clone(), table, is_dark)
              } else {
                  HTMLView::new("a", hashmap! {}, "", vec![])
                      .wrap_tag("li", hashmap! {})
              }
          } else {
              HTMLView::new("a", hashmap! {
                  "href".to_string() => "#".to_string(),
                  "class".to_string() => filter_attrs("block py-2 px-3 text-gray-900 rounded-sm hover:bg-gray-100 md:hover:bg-transparent md:border-0 md:hover:text-blue-700 md:p-0 dark:text-white md:dark:hover:text-blue-500 dark:hover:bg-gray-700 dark:hover:text-white md:dark:hover:bg-transparent", is_dark),
                  "aria-current".to_string() => "page".to_string()
              }, menu, vec![]).wrap_tag("li", hashmap! {})
          }
      })
      .collect::<Vec<HTMLView>>();

  let ul = HTMLView::new("ul", hashmap! {
      "class".to_string() => filter_attrs("flex flex-col font-medium p-4 md:p-0 mt-4 border border-gray-100 rounded-lg bg-gray-50 md:space-x-8 rtl:space-x-reverse md:flex-row md:mt-0 md:border-0 md:bg-white dark:bg-gray-800 md:dark:bg-gray-900 dark:border-gray-700", is_dark)
  }, "", nav_menus);
  let right = HTMLView::new("div", hashmap! {
      "id".to_string() => "navbar-dropdown".to_string(),
      "class".to_string() => filter_attrs("hidden w-full md:block md:w-auto", is_dark)
  }, "", vec![ul]);

  let right_collapse_span = HTMLView::new("span", hashmap! {
      "class".to_string() => "sr-only".to_string(),
  }, "Open main menu", vec![]);
  let right_collapse = HTMLView::new("button", hashmap! {
      "data-collapse-toggle".to_string() => "navbar-dropdown".to_string(), 
      "type".to_string() => "button".to_string(), 
      "class".to_string() => filter_attrs("inline-flex items-center p-2 w-10 h-10 justify-center text-sm text-gray-500 rounded-lg md:hidden hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-gray-200 dark:text-gray-400 dark:hover:bg-gray-700 dark:focus:ring-gray-600", is_dark), 
      "aria-controls".to_string() => "navbar-dropdown".to_string(),
      "aria-expanded".to_string() => "false".to_string(),
  }, "", vec![svg_menu(), right_collapse_span]);

  let left_text = HTMLView::new("span", hashmap! {
      "class".to_string() => filter_attrs("self-center text-2xl font-semibold whitespace-nowrap dark:text-white", is_dark),
  }, title.as_str(), vec![]);
  let left = HTMLView::new("a", hashmap! {
      "class".to_string() => filter_attrs("flex items-center space-x-3 rtl:space-x-reverse", is_dark), 
      "href".to_string() => "/".to_string() 
  }, "", vec![left_text]);

  let div = HTMLView::new("div", hashmap! {"class".to_string() => filter_attrs("max-w-screen-xl flex flex-wrap items-center justify-between mx-auto p-4", is_dark) }, "", vec![left, right_collapse, right]);
  HTMLView::new("nav", hashmap! {"class".to_string() => filter_attrs("bg-white border-gray-200 dark:bg-gray-900 dark:border-gray-700", is_dark) }, "", vec![div])
}