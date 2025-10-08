use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use maplit::hashmap;
use serde_yaml::Value;

use crate::{layout::{common::get_tomlview_for_key, layouts_from_toml}, markdown::{markdown_wrap_to_html, markdown_wrap_to_htmlview, metas_table_from_markdown}, page::make_md_files_to_folder_except_index};


///
#[test]
fn test_frontmatter() -> Result<(), Box<dyn std::error::Error>> {
    let md_path = Path::new("test/portfolios/python/sum.md");
    let v = metas_table_from_markdown(md_path)?;

    println!("markdown {:?}", v);
    let title = v.get("title").and_then(|x| x.as_str()).ok_or("failed to get title")?;
    assert_eq!("Sum", title);
    let date = v.get("created").and_then(|x| x.as_str()).ok_or("failed to get date")?;
    assert_eq!("25-10-02", date);
    Ok(())
}

/// test/portfolios/python/index.toml 여기서 view를 읽어와서
/// view의 views를 추저하여 mdlist_row or mdlist_column이 들어있는지 확인
#[test]
fn test_mdlist() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- test_mdlist ---");
    let index_path = Path::new("test/portfolios/python/index.toml");
    let index_folder = index_path.parent().ok_or("failed to get index_folder")?;
    let layouts = layouts_from_toml(index_path).unwrap_or(hashmap! {});
    let view = get_tomlview_for_key(index_path, "root", None, None, layouts)?;
    for view in view.views() {
        if view.key() == "contents" {
            for view in view.views() {
                println!("view = {:?}", view.key());
                let value = view.value().ok_or("failed to get value")?;
                let filename = value.get("filename")
                    .ok_or("failed to get filename")?.as_str()
                    .ok_or("failed to convert as_str")?;
                if filename == "minus" {
                    let title = value.get("title")
                        .ok_or("failed to get title")?.as_str()
                        .ok_or("failed to convert as_str")?;
                    assert_eq!(title, "Python Minus");
                } else if filename == "sum" {
                    let title = value.get("title")
                        .ok_or("failed to get title")?.as_str()
                        .ok_or("failed to convert as_str")?;
                    assert_eq!(title, "Sum");
                    let date = value.get("date")
                        .ok_or("failed to get date")?.as_str()
                        .ok_or("failed to convert as_str")?;
                    assert_eq!(date, "25-10-01");    
                }
            }
        }
    }
    if view.views().len() == 0 {
        assert!(false);
    }
    Ok(())
}

/// 부모 뷰에서 지정된 layout을 하위로 정확히 가져오고 있는지.. 특히 mdlist_row, mdlist_column에서
#[test]
fn test_layout_pass_down() -> Result<(), Box<dyn std::error::Error>> {
    let index_path = Path::new("test/portfolios/index.toml");
    let layouts = layouts_from_toml(index_path).unwrap_or(hashmap! {});
    let view = get_tomlview_for_key(index_path, "root", None, None, layouts)?;
    println!("{:#?}", view);
    println!("1");
    assert_eq!(1, 1);
    Ok(())
}

/// markdown파일에 지정된 frontmatter가 제대로 전파되어 마크다운에 적용되고 있는지
#[test]
fn test_meta_layout() -> Result<(), Box<dyn std::error::Error>> {
    let index_path = Path::new("test/portfolios/index.toml");
    let layouts = layouts_from_toml(index_path).unwrap_or(hashmap! {});
    let md_path = Path::new("test/portfolios/python/minus.md");
    let view = markdown_wrap_to_htmlview(md_path, layouts);
    println!("{:?}", view);
    assert_eq!(1, 2);
    Ok(())
}