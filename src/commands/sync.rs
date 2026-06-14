use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::adr;

const ADR_INDEX_FILE_NAME: &str = "README.md";
const ADR_TEMPLATE_FILE_NAME: &str = "TEMPLATE.md";

pub fn run() -> Result<()> {
    let config = adr::read_config()?;
    let adr_dir = std::env::current_dir()?.join(&config.directory);
    let mut adr_files = Vec::new();

    for entry in fs::read_dir(&adr_dir)
        .with_context(|| format!("Failed to read ADR directory: {}", adr_dir.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy().to_string();
        if is_adr_markdown_file(&file_name) {
            adr_files.push(file_name);
        }
    }

    adr_files.sort();

    let items = adr_files
        .iter()
        .map(|file_name| {
            let path = adr_dir.join(file_name);
            let title = adr::read_markdown_title(&path)?.unwrap_or_default();
            Ok(format!(
                "- [{}]({}) — {}",
                adr_id(file_name),
                urlencoding::encode(file_name),
                title
            ))
        })
        .collect::<Result<Vec<_>>>()?;

    let documented_decisions = if items.is_empty() {
        "**No ADR issued at the moment.**".to_string()
    } else {
        items.join("\n")
    };

    let index = [
        "# Architecture Decision Records (ADR)",
        "",
        "This directory tracks the key architectural decisions shaping our system. Each record documents the underlying rationale, trade-offs, and final decisions to provide a transparent history of our technical choices.",
        "",
        "## Workflow",
        "",
        "We manage our ADRs using [@id6tm/madr-tools](https://github.com/id6tm/madr-tools). Do not edit this index file directly.",
        "",
        "- **Create a new ADR:** Run `madr new \"Title of your decision\"`",
        "- **Regenerate this index:** Run `madr sync`",
        "",
        "## Documented Decisions",
        "",
        &documented_decisions,
        "",
    ]
    .join("\n");

    fs::write(adr_dir.join(ADR_INDEX_FILE_NAME), index)
        .with_context(|| "Failed to write ADR index".to_string())?;
    println!(
        "Generated {}",
        Path::new(&config.directory)
            .join(ADR_INDEX_FILE_NAME)
            .display()
    );
    Ok(())
}

fn is_adr_markdown_file(file_name: &str) -> bool {
    let lower = file_name.to_lowercase();
    lower.ends_with(".md")
        && lower != ADR_INDEX_FILE_NAME.to_lowercase()
        && lower != ADR_TEMPLATE_FILE_NAME.to_lowercase()
}

fn adr_id(file_name: &str) -> String {
    leading_adr_number(file_name)
        .map(|number| format!("ADR-{number}"))
        .unwrap_or_else(|| "ADR".to_string())
}

fn leading_adr_number(file_name: &str) -> Option<&str> {
    let lower = file_name.to_lowercase();
    if !lower.ends_with(".md") {
        return None;
    }

    let stem = &file_name[..file_name.len() - 3];
    let number_end = stem
        .char_indices()
        .take_while(|(_, character)| character.is_ascii_digit())
        .last()
        .map(|(index, character)| index + character.len_utf8())?;

    if number_end == 0 {
        return None;
    }

    let separator = stem[number_end..].chars().next();
    if separator.is_none() || matches!(separator, Some('-' | '_' | ' ')) {
        Some(&stem[..number_end])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{adr_id, is_adr_markdown_file};

    #[test]
    fn excludes_index_and_template_from_adr_markdown_files() {
        assert!(!is_adr_markdown_file("README.md"));
        assert!(!is_adr_markdown_file("template.md"));
        assert!(is_adr_markdown_file("0001-first.md"));
    }

    #[test]
    fn derives_adr_id_from_file_name_prefix() {
        assert_eq!(adr_id("0001-first.md"), "ADR-0001");
        assert_eq!(adr_id("manual-note.md"), "ADR");
    }
}
