mod parser;

use std::env;
use std::fs;

use std::any::{type_name, Any};
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Display, Path, PathBuf};
use tiny_http::{Header, Response, Server};
use structopt::StructOpt;
use chrono::Local;

#[derive(Debug, StructOpt)]
#[structopt(name = "prema", about = "A CLI tool")]
enum Cli {
    New(NewCommand),
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
}

fn main() -> Result<(), String> {
    // Match on subcommands and handle errors
    let cli = Cli::from_iter_safe(std::env::args()).unwrap_or_else(|e| {
        println!("{e}");
        println!("EXAMPLE:");
        println!("    prema new {{name}}");
        println!("    prema html {{md_path}} {{html_path}}");
        println!("    prema html {{md_path}} {{html_path}} --tags \"ios, android\"");
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
            make_htmls(md_root_path, html_root_path)?;
            run_server(html_root_path)?;
            Ok(())
        }
    }
}

fn run_server(html_path: &Path) -> Result<(), &'static str> {
    let server = Server::http("127.0.0.1:3000").unwrap();
    println!("서버 실행 중: http://127.0.0.1:3000");

    for request in server.incoming_requests() {
        println!("요청: {}", request.url());
        let url = request.url().trim_start_matches('/').trim_end_matches('/');

        let mut full_path = html_path.join(url);
        let file_extension = full_path.extension().and_then(|ext| ext.to_str());
        
        if let Some(ext) = file_extension {
            println!("이미지 요청: {:?}", full_path);
            if ["jpg", "jpeg", "png"].contains(&ext) {
                let mut file = match File::open(&full_path) {
                    Ok(f) => f,
                    Err(_) => {
                        let response =
                            Response::from_string("파일을 찾을 수 없습니다").with_status_code(404);
                        request.respond(response).unwrap();
                        continue;
                    }
                };

                let mut contents = Vec::new();
                file.read_to_end(&mut contents).unwrap();

                // MIME 타입 추론 (jpg, jpeg, png)
                let mime = match ext {
                    "jpg" | "jpeg" => "image/jpeg",
                    "png" => "image/png",
                    _ => "application/octet-stream", // 기본 값
                };

                let header = Header::from_bytes(&b"Content-Type"[..], mime.as_bytes()).unwrap();

                let response = Response::from_data(contents).with_header(header);

                request.respond(response).unwrap();
            }
        } else {
            if url.is_empty() {
                full_path = html_path.join("index.html");
            } else {
                full_path = html_path.join(url).join("index.html");
            }

            println!("정상 요청: {:?}", full_path);
            let mut file = match File::open(full_path.clone()) {
                Ok(f) => f,
                Err(_) => {
                    let response =
                        Response::from_string("index.html not found").with_status_code(404);
                    request.respond(response).unwrap();
                    continue;
                }
            };

            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap();

            let response = Response::from_string(contents).with_header(header);

            request.respond(response).unwrap();
        }
    }
    Ok(())
}

fn make_htmls(md: &Path, html: &Path) -> Result<(), String> {
    parser::make_pages(md, html)
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
