use std::fs;

use anyhow::{Context, Result};
use dialoguer::{Input, Select, theme::ColorfulTheme};

use crate::{
    adr::{self, AdrConfig, DEFAULT_ADR_DIRECTORY},
    commands::sync,
    templates::{self, ADR_TEMPLATES},
};

pub fn run() -> Result<()> {
    println!("madr init");

    let theme = ColorfulTheme::default();
    let directory: String = Input::with_theme(&theme)
        .with_prompt("Where should ADR files be stored?")
        .default(DEFAULT_ADR_DIRECTORY.to_string())
        .interact_text()?;

    let template_options: Vec<String> = ADR_TEMPLATES
        .iter()
        .map(|template| format!("{} - {}", template.name, template.description))
        .collect();
    let selection = Select::with_theme(&theme)
        .with_prompt("Which ADR template should be used?")
        .items(&template_options)
        .default(0)
        .interact()?;
    let template = templates::find(ADR_TEMPLATES[selection].name)?;

    let config = AdrConfig {
        directory: normalized_directory(&directory),
    };
    let adr_dir = std::env::current_dir()?.join(&config.directory);

    fs::create_dir_all(&adr_dir)
        .with_context(|| format!("Failed to create ADR directory: {}", adr_dir.display()))?;
    fs::write(adr_dir.join("TEMPLATE.md"), template.contents)
        .with_context(|| "Failed to write ADR template".to_string())?;
    fs::write(
        adr::config_path()?,
        format!("{}\n", serde_json::to_string_pretty(&config)?),
    )
    .with_context(|| format!("Failed to write {}", adr::ADR_CONFIG_FILE))?;

    sync::run()?;
    println!("ADR config ready: {}", adr::ADR_CONFIG_FILE);
    Ok(())
}

fn normalized_directory(directory: &str) -> String {
    let trimmed = directory.trim();
    if trimmed.is_empty() {
        DEFAULT_ADR_DIRECTORY.to_string()
    } else {
        trimmed.to_string()
    }
}
