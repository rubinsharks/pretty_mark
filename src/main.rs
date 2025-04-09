
mod parser;

use std::env;
use std::fs;

use std::any::{Any, type_name};
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Display, Path};

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        return Err("too few arguments");
    }
    if args[1] == "html" {
        if args.len() < 4 {
            return Err("too few arguments");
        }
        let md_path = Path::new(&args[2]);
        let html_path = Path::new(&args[3]);
        make_htmls(md_path, html_path)?;
        return Ok(())
    } else if args[1] == "new" {
        if args.len() < 3 {
            return Err("too few arguments");
        }
        let name = &args[2];

        let md_path = Path::new(".");
        make_new_page(md_path, name)?;
        return Ok(())
    }

    let md_path = Path::new("root");
    let html_path = Path::new("html");
    make_htmls(md_path, html_path)?;
    make_new_page(md_path, "test")?;
    Ok(())
}

fn make_htmls(md: &Path, html: &Path) -> Result<(), &'static str> {
    parser::make_pages(md, html)
}

/// ### make new page
/// make {name} directory
///
/// make {name}.md in directory
///
/// make option.toml in directory
fn make_new_page(path: &Path, name: &str) -> Result<(), &'static str> {
    let dir_path = path.join(name);
    if dir_path.exists() {
        println!("Directory '{}' already exists, returning.", name);
        return Err("Directory already exists")  // Return early if the directory exists
    }

    fs::create_dir_all(&dir_path).ok().ok_or("faild to create directory")?;

    // Create the .md file in the new directory
    if let Some(last_name) = dir_path.file_name() {
        let last_name = last_name.to_str().unwrap();
        
        let md_path = dir_path.join(format!("{}.md", last_name));
        let mut md_file = fs::File::create(md_path).ok().ok_or("faild to create file")?;
        
        writeln!(md_file, "# {} Page", last_name).ok().ok_or("faild to write to file")?;

        // Create the option.toml file in the new directory
        let toml_path = dir_path.join("option.toml");
        let mut toml_file = fs::File::create(toml_path).ok().ok_or("faild to create file")?;

        // Optionally, write a basic config to the toml file
        writeln!(toml_file, "[settings]").ok().ok_or("")?;
        writeln!(toml_file, "name = \"{}\"", last_name).ok().ok_or("")?;

        Ok(())
    } else {
        return Err("faild to create file")
    }
}