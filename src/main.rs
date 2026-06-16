mod adr;
mod cli;
mod commands;
mod templates;

use anyhow::Result;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    cli::run()
}
