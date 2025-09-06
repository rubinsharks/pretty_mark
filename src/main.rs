mod cli;
mod page;
mod server;
mod file;
mod html;
mod layout;
mod markdown;
mod option;
mod common;
mod yaml;
mod test;

fn main() -> Result<(), String> {
    cli::run_cli()
}
