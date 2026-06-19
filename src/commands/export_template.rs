use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::{adr, templates};

const ADR_TEMPLATE_FILE_NAME: &str = "TEMPLATE.md.tera";
const DEFAULT_TEMPLATE_NAME: &str = "standard";

pub fn run() -> Result<()> {
    println!("madr export-template");

    let config = adr::read_config()?;
    let adr_dir = std::env::current_dir()?.join(&config.directory);
    let template = templates::find(DEFAULT_TEMPLATE_NAME)?;

    fs::create_dir_all(&adr_dir)
        .with_context(|| format!("Failed to create ADR directory: {}", adr_dir.display()))?;
    fs::write(adr_dir.join(ADR_TEMPLATE_FILE_NAME), template.contents)
        .with_context(|| "Failed to write ADR template".to_string())?;

    println!(
        "Exported {}",
        Path::new(&config.directory)
            .join(ADR_TEMPLATE_FILE_NAME)
            .display()
    );
    Ok(())
}
