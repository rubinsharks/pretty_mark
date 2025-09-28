
use std::{collections::HashMap, path::Path};
use crate::page::*;


#[test]
fn test_page_recursive() {
    let path = Path::new("test");
    match read_dir_recursive(&path) {
        Ok(page) => {
            let print = page.print(0);
            let mut expected = ["test",
                            "-portfolios",
                            "--android",
                            "--ios",
                            "--python",
                            "-about_me",
                            "-blogs",
                            "-gallery",
                           ].join("\n");
            expected.push('\n');
            assert_eq!(print, expected)
        }
        Err(err) => {
            assert!(false)
        }
    }
}


#[test]
fn test_check_error() {
    // file name and dir name is same in directory
    // index.toml has no root key
    // targeted markdown or image files are not exist

}

#[test]
fn test_find_index() {

}

#[test]
fn test_directory_filtering() {

}