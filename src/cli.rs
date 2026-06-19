use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};

use crate::commands;

#[derive(Debug, Parser)]
#[command(
    name = "madr",
    version,
    about = "Manage markdown architecture decision records"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create .madrrc.json and the ADR markdown directory
    Init,
    /// Create a new ADR markdown file in the ADR directory
    New {
        /// ADR title
        title: Vec<String>,
    },
    /// Regenerate the ADR index
    Sync,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Init) => commands::init::run(),
        Some(Command::New { title }) => commands::new::run(title.join(" ")),
        Some(Command::Sync) => commands::sync::run(),
        None => {
            Cli::command().print_help()?;
            println!();
            Ok(())
        }
    }
}
