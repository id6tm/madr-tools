use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use anyhow::{Context, Result};
use chrono::Local;
use dialoguer::{Input, theme::ColorfulTheme};

use crate::{adr, commands::sync};

const ADR_TEMPLATE_FILE_NAME: &str = "TEMPLATE.md";

pub fn run(title_from_argument: String) -> Result<()> {
    println!("madr new");

    let config = adr::read_config()?;
    let title = resolve_title(title_from_argument)?;
    let adr_dir = std::env::current_dir()?.join(&config.directory);
    fs::create_dir_all(&adr_dir)
        .with_context(|| format!("Failed to create ADR directory: {}", adr_dir.display()))?;

    let file_numbers = adr_file_numbers(&adr_dir)?;
    let width = file_numbers
        .iter()
        .map(|number| number.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let next_number = file_numbers
        .iter()
        .filter_map(|number| number.parse::<u32>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    let adr_number = format!("{next_number:0width$}");
    let file_name = format!("{}-{}.md", adr_number, title_slug(&title));
    let relative_path = Path::new(&config.directory).join(&file_name);
    let contents = render_template(
        &adr_dir.join(ADR_TEMPLATE_FILE_NAME),
        &TemplateValues {
            date: Local::now().date_naive().to_string(),
            number: adr_number.clone(),
            title: title.trim().to_string(),
        },
    )?;

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(adr_dir.join(&file_name))
        .with_context(|| format!("Failed to create ADR file: {}", relative_path.display()))?;
    file.write_all(contents.as_bytes())?;

    sync::run()?;
    println!("Created {}", relative_path.display());
    Ok(())
}

fn resolve_title(title_from_argument: String) -> Result<String> {
    let title = title_from_argument.trim();
    if !title.is_empty() {
        return Ok(title.to_string());
    }

    let theme = ColorfulTheme::default();
    Ok(Input::with_theme(&theme)
        .with_prompt("What is the ADR title?")
        .validate_with(|value: &String| -> Result<(), &str> {
            if value.trim().is_empty() {
                Err("Enter an ADR title.")
            } else {
                Ok(())
            }
        })
        .interact_text()?)
}

fn adr_file_numbers(adr_dir: &Path) -> Result<Vec<String>> {
    let mut numbers = Vec::new();

    for entry in fs::read_dir(adr_dir)
        .with_context(|| format!("Failed to read ADR directory: {}", adr_dir.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }

        if let Some(number) = leading_adr_number(&entry.file_name().to_string_lossy()) {
            numbers.push(number.to_string());
        }
    }

    Ok(numbers)
}

fn title_slug(title: &str) -> String {
    let slug = slug::slugify(title);
    if slug.is_empty() {
        "adr".to_string()
    } else {
        slug
    }
}

fn leading_adr_number(file_name: &str) -> Option<&str> {
    let stem = file_name
        .strip_suffix(".md")
        .or_else(|| file_name.strip_suffix(".MD"))?;
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
    use super::{leading_adr_number, title_slug};

    #[test]
    fn extracts_adr_number_from_supported_file_names() {
        assert_eq!(leading_adr_number("0001-first.md"), Some("0001"));
        assert_eq!(leading_adr_number("12_first.md"), Some("12"));
        assert_eq!(leading_adr_number("9 title.MD"), Some("9"));
        assert_eq!(leading_adr_number("123.md"), Some("123"));
    }

    #[test]
    fn ignores_markdown_files_without_adr_number_prefixes() {
        assert_eq!(leading_adr_number("README.md"), None);
        assert_eq!(leading_adr_number("123abc.md"), None);
        assert_eq!(leading_adr_number("123.txt"), None);
    }

    #[test]
    fn title_slug_falls_back_when_title_has_no_slug_characters() {
        assert_eq!(title_slug("Use Postgres"), "use-postgres");
        assert_eq!(title_slug("!!!"), "adr");
    }
}

struct TemplateValues {
    date: String,
    number: String,
    title: String,
}

fn render_template(template_path: &Path, values: &TemplateValues) -> Result<String> {
    let template = fs::read_to_string(template_path)
        .with_context(|| format!("Failed to read ADR template: {}", template_path.display()))?;

    Ok(template
        .replace("{{counter}}", &values.number)
        .replace("{{date}}", &values.date)
        .replace("{{id}}", &format!("ADR-{}", values.number))
        .replace("{{title}}", &values.title))
}
