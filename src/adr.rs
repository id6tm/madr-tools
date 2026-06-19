use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub const ADR_CONFIG_FILE: &str = ".madrrc.json";
pub const DEFAULT_ADR_DIRECTORY: &str = "docs/adr";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdrConfig {
    pub directory: String,
}

impl Default for AdrConfig {
    fn default() -> Self {
        Self {
            directory: DEFAULT_ADR_DIRECTORY.to_string(),
        }
    }
}

pub fn config_path() -> Result<PathBuf> {
    Ok(std::env::current_dir()?.join(ADR_CONFIG_FILE))
}

pub fn read_config() -> Result<AdrConfig> {
    let path = config_path()?;
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read ADR config: {}", path.display()))?;
    let config: AdrConfig = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse ADR config: {}", path.display()))?;

    if config.directory.trim().is_empty() {
        anyhow::bail!("ADR config field `directory` cannot be empty");
    }

    Ok(AdrConfig {
        directory: config.directory.trim().to_string(),
    })
}

pub fn read_markdown_title(path: &PathBuf) -> Result<Option<String>> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read ADR markdown file: {}", path.display()))?;

    Ok(contents
        .lines()
        .find_map(|line| line.strip_prefix("# "))
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .map(str::to_string))
}
