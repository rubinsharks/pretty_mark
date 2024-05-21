use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

fn main() -> std::io::Result<()> {
    println!("Hello, world!");
    let path = Path::new("root/index.md");
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => {}
    }

    println!("{}", markdown::to_html(&s));

    let html = markdown::to_html(&s);

    let mut file = File::create("index.html")?;
    file.write_all(html.as_bytes())?;
    Ok(())
}
