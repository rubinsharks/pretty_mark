
use crate::server;
use crate::page;
use crate::file;
use crate::server::run_server;

use std::env::{args};
use std::fs;

use std::any::{type_name, Any};
use std::fmt::Debug;
use std::fs::File;
use std::io::{stdin, Read, Write};
use std::path::{Display, Path, PathBuf};
use maplit::hashmap;
use structopt::StructOpt;
use chrono::Local;

#[derive(Debug, StructOpt)]
#[structopt(name = "prema", about = "A CLI tool")]
enum Cli {
    #[structopt(name = "new", about = "Make new markdown folder and files")]
    New(NewCommand),
    #[structopt(name = "html", about = "Converting markdown files to html files")]
    Html(HtmlCommand),
}

#[derive(Debug, StructOpt)]
struct NewCommand {
    name: String,

    #[structopt(long, use_delimiter = true)]
    tags: Vec<String>,
}

#[derive(Debug, StructOpt)]
struct HtmlCommand {
    md_path: String,
    html_path: String,

    #[structopt(long)]
    server: bool,
}

pub fn run_cli() -> Result<(), String> {
    // Match on subcommands and handle errors
    let cli = Cli::from_iter_safe(args()).unwrap_or_else(|e| {
        println!("{e}");
        println!("EXAMPLE:");
        println!("    prema new {{name}}");
        println!("    prema new {{name}} --tags \"ios, android\"");
        println!("    prema html {{md_path}} {{html_path}}");
        std::process::exit(1); // Exit with an error code
    });

    match cli {
        Cli::New(cmd) => {
            println!("With tags: {:?}", cmd.tags);
            let name = cmd.name.as_str();

            let md_path = Path::new(".");
            make_new_page(md_path, name, cmd.tags)?;
            Ok(())
        },
        Cli::Html(cmd) => {
            let md_root_path = Path::new(cmd.md_path.as_str());
            let html_root_path = Path::new(cmd.html_path.as_str());
            generate_html(md_root_path, html_root_path)?;
            
            if cmd.server {
                server::run_server(html_root_path)?;
            }

            let mut input = String::new();
            println!("텍스트를 입력하세요:");

            stdin()
                .read_line(&mut input)
                .expect("입력을 읽는 데 실패했습니다");

            println!("입력한 내용: {}", input);
            Ok(())
        }
    }
}

/// ### generate html
/// make page tree
/// validate pages and update
/// make html files
pub fn generate_html(md_root_path: &Path, html_root_path: &Path) -> Result<(), String> {
    let mut page = page::read_dir_recursive(md_root_path)?;
    page.inflate_html(hashmap! {})?;
    page.make_html_file(html_root_path)?;
    run_server(html_root_path)?;
    Ok(())
}

/// ### make new page
/// make {name} directory
///
/// make {name}.md in directory
///
/// make option.toml in directory
fn make_new_page(path: &Path, name: &str, tags: Vec<String>) -> Result<(), &'static str> {
    let dir_path = path.join(name);
    if dir_path.exists() {
        println!("Directory '{}' already exists, returning.", name);
        return Err("Directory already exists"); // Return early if the directory exists
    }

    fs::create_dir_all(&dir_path)
        .ok()
        .ok_or("faild to create directory")?;

    // Create the .md file in the new directory
    if let Some(last_name) = dir_path.file_name() {
        let last_name = last_name.to_str().unwrap();

        let md_path = dir_path.join(format!("{}.md", last_name));
        let mut md_file = fs::File::create(md_path)
            .ok()
            .ok_or("faild to create file")?;

        writeln!(md_file, "# {} Page", last_name)
            .ok()
            .ok_or("faild to write to file")?;

        let toml_path = dir_path.join("option.toml");
        let mut toml_file = fs::File::create(toml_path)
            .ok()
            .ok_or("faild to create file")?;

        if !tags.is_empty() {
            writeln!(toml_file, "[basic]").ok().ok_or("")?;
            let tags_str = tags.iter()
                  .map(|s| format!("\"{}\"", s.trim())) 
                  .collect::<Vec<String>>()
                .join(",");
            writeln!(toml_file, "tags = [{}]", tags_str)
                .ok()
                .ok_or("")?;

            let now = Local::now();
            let formatted_time = now.format("%Y-%m-%d %H:%M:%S").to_string();
            writeln!(toml_file, "created = \"{}\"", formatted_time)
                .ok()
                .ok_or("")?;
        }

        Ok(())
    } else {
        return Err("faild to create file");
    }
}