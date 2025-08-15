use std::any::{type_name, Any};
use std::fmt::Debug;
use std::fs::File;
use std::io::{stdin, Read, Write};
use std::path::{Display, Path, PathBuf};
use tiny_http::{Header, Response, Server};
use structopt::StructOpt;
use chrono::Local;


pub fn run_server(html_path: &Path) -> Result<(), &'static str> {
    let server = Server::http("127.0.0.1:3000").unwrap();
    println!("서버 실행 중: http://127.0.0.1:3000");

    for request in server.incoming_requests() {
        println!("요청: {}", request.url());
        let url = request.url().trim_start_matches('/').trim_end_matches('/');

        let mut full_path = html_path.join(url);
        let file_extension = full_path.extension().and_then(|ext| ext.to_str());

        if let Some(ext) = file_extension {
            println!("이미지 요청: {:?}", full_path);
            if ["jpg", "jpeg", "png", "svg"].contains(&ext) {
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
                    "svg" => "image/svg+xml",
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