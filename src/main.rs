
mod parser;

use std::any::{Any, type_name};
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Display, Path};

fn main() -> Result<(), &'static str> {
    let path = Path::new("root");
    parser::make_pages(path)?;
    Ok(())
}

