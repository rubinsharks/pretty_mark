mod cli;
mod page;
mod server;
mod file;
mod html;
mod layout;
mod markdown;
mod option;
mod common;

fn main() -> Result<(), String> {
    cli::run_cli()
}
