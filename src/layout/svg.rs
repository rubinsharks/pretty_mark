use maplit::hashmap;
use crate::html::HTMLView;

pub fn svg_menu() -> HTMLView {
  let path = HTMLView::new("path", hashmap! {
      "stroke".to_string() => "currentColor".to_string(),
      "stroke-linecap".to_string() => "round".to_string(),
      "stroke-linejoin".to_string() => "round".to_string(),
      "stroke-width".to_string() => "2".to_string(),
      "d".to_string() => "M1 1h15M1 7h15M1 13h15".to_string()
  }, "", vec![]);
  HTMLView::new("svg", hashmap! {
      "class".to_string() => "w-5 h-5".to_string(),
      "aria-hidden".to_string() => "true".to_string(),
      "xmlns".to_string() => "http://www.w3.org/2000/svg".to_string(),
      "fill".to_string() => "none".to_string(),
      "viewBox".to_string() => "0 0 17 14".to_string()
  }, "", vec![path])
}

pub fn svg_dropdown() -> HTMLView {
  let path = HTMLView::new("path", hashmap! {
      "stroke".to_string() => "currentColor".to_string(),
      "stroke-linecap".to_string() => "round".to_string(),
      "stroke-linejoin".to_string() => "round".to_string(),
      "stroke-width".to_string() => "2".to_string(),
      "d".to_string() => "m1 1 4 4 4-4".to_string()
  }, "", vec![]);
  HTMLView::new("svg", hashmap! {
      "class".to_string() => "w-2.5 h-2.5 ms-2.5".to_string(),
      "aria-hidden".to_string() => "true".to_string(),
      "xmlns".to_string() => "http://www.w3.org/2000/svg".to_string(),
      "fill".to_string() => "none".to_string(),
      "viewBox".to_string() => "0 0 10 6".to_string()
  }, "", vec![path])
}