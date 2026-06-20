use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use chrono::Local;
use dialoguer::{Confirm, Input, MultiSelect, theme::ColorfulTheme};
use serde::Serialize;
use tera::{Context as TeraContext, Tera};

use crate::{adr, commands::sync};

const ADR_TEMPLATE_FILE_NAME: &str = "TEMPLATE.md.tera";

pub fn run(title_from_argument: String, supersede: Vec<String>) -> Result<()> {
    println!("madr new");

    let config = adr::read_config()?;
    let title = resolve_title(title_from_argument)?;
    let adr_dir = std::env::current_dir()?.join(&config.directory);
    fs::create_dir_all(&adr_dir)
        .with_context(|| format!("Failed to create ADR directory: {}", adr_dir.display()))?;

    let adr_files = adr_files(&adr_dir)?;
    let file_numbers = adr_files
        .iter()
        .map(|adr_file| adr_file.number.clone())
        .collect::<Vec<_>>();
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
    let superseded_adrs = resolve_superseded_adrs(&adr_files, &supersede)?;
    let contents = render_template(
        &adr_dir.join(ADR_TEMPLATE_FILE_NAME),
        &TemplateValues {
            counter: adr_number.clone(),
            date: Local::now().date_naive().to_string(),
            number: adr_number.clone(),
            id: format!("ADR-{adr_number}"),
            title: title.trim().to_string(),
            supersedes: superseded_adrs.clone(),
        },
    )?;

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(adr_dir.join(&file_name))
        .with_context(|| format!("Failed to create ADR file: {}", relative_path.display()))?;
    file.write_all(contents.as_bytes())?;

    update_superseded_adrs(
        &superseded_adrs,
        &format!(
            "Superseded by [ADR-{}]({})",
            adr_number,
            urlencoding::encode(&file_name)
        ),
    )?;
    sync::run()?;
    println!("Created {}", relative_path.display());
    Ok(())
}

fn resolve_superseded_adrs(
    adr_files: &[AdrFile],
    supersede_values: &[String],
) -> Result<Vec<SupersededAdr>> {
    if !supersede_values.is_empty() {
        return superseded_adrs(adr_files, supersede_values);
    }

    prompt_for_superseded_adrs(adr_files)
}

fn prompt_for_superseded_adrs(adr_files: &[AdrFile]) -> Result<Vec<SupersededAdr>> {
    if adr_files.is_empty() {
        return Ok(Vec::new());
    }

    let theme = ColorfulTheme::default();
    let supersedes = Confirm::with_theme(&theme)
        .with_prompt("Does this ADR supersede any prior one?")
        .default(false)
        .interact()?;
    if !supersedes {
        return Ok(Vec::new());
    }

    let options = adr_files
        .iter()
        .map(adr_selection_label)
        .collect::<Result<Vec<_>>>()?;
    let selections = MultiSelect::with_theme(&theme)
        .with_prompt("Which ADRs does this supersede?")
        .items(&options)
        .interact()?;
    let selected_numbers = selections
        .iter()
        .map(|index| adr_files[*index].number.clone())
        .collect::<Vec<_>>();

    superseded_adrs(adr_files, &selected_numbers)
}

fn adr_selection_label(adr_file: &AdrFile) -> Result<String> {
    let title = adr::read_markdown_title(&adr_file.path)?.unwrap_or_else(|| {
        adr_file
            .path
            .file_name()
            .map(|file_name| file_name.to_string_lossy().to_string())
            .unwrap_or_else(|| adr_file.path.display().to_string())
    });

    Ok(format!("ADR-{} - {}", adr_file.number, title))
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

#[derive(Debug, Clone)]
struct AdrFile {
    number: String,
    path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
struct SupersededAdr {
    id: String,
    link: String,
    #[serde(skip)]
    path: PathBuf,
}

fn adr_files(adr_dir: &Path) -> Result<Vec<AdrFile>> {
    let mut adr_files = Vec::new();

    for entry in fs::read_dir(adr_dir)
        .with_context(|| format!("Failed to read ADR directory: {}", adr_dir.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }

        if let Some(number) = leading_adr_number(&entry.file_name().to_string_lossy()) {
            adr_files.push(AdrFile {
                number: number.to_string(),
                path: entry.path(),
            });
        }
    }

    adr_files.sort_by(|left, right| {
        left.number
            .parse::<u32>()
            .ok()
            .cmp(&right.number.parse::<u32>().ok())
            .then_with(|| left.number.cmp(&right.number))
    });

    Ok(adr_files)
}

fn superseded_adrs(adr_files: &[AdrFile], values: &[String]) -> Result<Vec<SupersededAdr>> {
    let requested_numbers = parse_superseded_numbers(values)?;
    if requested_numbers.is_empty() {
        return Ok(Vec::new());
    }

    let mut files_by_number = BTreeMap::new();
    for adr_file in adr_files {
        let number = adr_file
            .number
            .parse::<u32>()
            .with_context(|| format!("Invalid ADR number: {}", adr_file.number))?;
        files_by_number.insert(number, adr_file.clone());
    }

    let mut superseded_adrs = Vec::new();
    let mut missing = Vec::new();
    for number in requested_numbers {
        if let Some(adr_file) = files_by_number.get(&number) {
            let file_name = adr_file
                .path
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .with_context(|| {
                    format!(
                        "Invalid ADR file name for path: {}",
                        adr_file.path.display()
                    )
                })?;
            superseded_adrs.push(SupersededAdr {
                id: format!("ADR-{}", adr_file.number),
                link: urlencoding::encode(file_name).to_string(),
                path: adr_file.path.clone(),
            });
        } else {
            missing.push(number);
        }
    }

    if !missing.is_empty() {
        anyhow::bail!(
            "Cannot supersede unknown ADR number(s): {}",
            join_numbers(&missing)
        );
    }

    Ok(superseded_adrs)
}

fn parse_superseded_numbers(values: &[String]) -> Result<Vec<u32>> {
    let mut numbers = Vec::new();

    for value in values {
        for part in value.split(',') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }

            let number = trimmed
                .parse::<u32>()
                .map_err(|_| anyhow!("Invalid ADR number for --supersede: {trimmed}"))?;
            if !numbers.contains(&number) {
                numbers.push(number);
            }
        }
    }

    Ok(numbers)
}

fn join_numbers(numbers: &[u32]) -> String {
    numbers
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn update_superseded_adrs(superseded_adrs: &[SupersededAdr], status: &str) -> Result<()> {
    for adr in superseded_adrs {
        let contents = fs::read_to_string(&adr.path)
            .with_context(|| format!("Failed to read superseded ADR: {}", adr.path.display()))?;
        let updated = replace_status(&contents, status)
            .with_context(|| format!("Failed to update ADR status: {}", adr.path.display()))?;
        fs::write(&adr.path, updated)
            .with_context(|| format!("Failed to write superseded ADR: {}", adr.path.display()))?;
    }

    Ok(())
}

fn replace_status(contents: &str, status: &str) -> Result<String> {
    let mut replaced = false;
    let mut lines = Vec::new();

    for line in contents.lines() {
        if !replaced && line.trim_start().starts_with("- **Status:**") {
            let indent = &line[..line.len() - line.trim_start().len()];
            lines.push(format!("{indent}- **Status:** {status}"));
            replaced = true;
        } else {
            lines.push(line.to_string());
        }
    }

    if !replaced {
        anyhow::bail!("ADR does not contain a `- **Status:**` line");
    }

    let mut updated = lines.join("\n");
    if contents.ends_with('\n') {
        updated.push('\n');
    }

    Ok(updated)
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
    use std::path::PathBuf;

    use super::{
        SupersededAdr, TemplateValues, leading_adr_number, parse_superseded_numbers,
        render_template_contents, replace_status, title_slug,
    };

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

    #[test]
    fn parses_comma_separated_and_repeated_superseded_numbers() {
        let numbers =
            parse_superseded_numbers(&["1, 0002".to_string(), "2".to_string(), "3".to_string()])
                .unwrap();

        assert_eq!(numbers, vec![1, 2, 3]);
    }

    #[test]
    fn replaces_first_status_line_with_superseded_link() {
        let contents = "# Old ADR\n\n- **Status:** Accepted\n- **Date:** 2026-06-19\n";
        let updated = replace_status(contents, "Superseded by [ADR-0002](0002-new.md)").unwrap();

        assert_eq!(
            updated,
            "# Old ADR\n\n- **Status:** Superseded by [ADR-0002](0002-new.md)\n- **Date:** 2026-06-19\n"
        );
    }

    #[test]
    fn renders_template_without_supersedes_line_when_none_are_provided() {
        let rendered = render_template_contents(
            include_str!("../templates/standard.md.tera"),
            &TemplateValues {
                counter: "0002".to_string(),
                date: "2026-06-19".to_string(),
                number: "0002".to_string(),
                id: "ADR-0002".to_string(),
                title: "New ADR".to_string(),
                supersedes: Vec::new(),
            },
        )
        .unwrap();

        assert!(rendered.contains("# New ADR"));
        assert!(!rendered.contains("**Supersedes:**"));
    }

    #[test]
    fn renders_template_with_superseded_adr_links() {
        let rendered = render_template_contents(
            include_str!("../templates/standard.md.tera"),
            &TemplateValues {
                counter: "0003".to_string(),
                date: "2026-06-19".to_string(),
                number: "0003".to_string(),
                id: "ADR-0003".to_string(),
                title: "New ADR".to_string(),
                supersedes: vec![
                    SupersededAdr {
                        id: "ADR-0001".to_string(),
                        link: "0001-old.md".to_string(),
                        path: PathBuf::from("0001-old.md"),
                    },
                    SupersededAdr {
                        id: "ADR-0002".to_string(),
                        link: "0002-other.md".to_string(),
                        path: PathBuf::from("0002-other.md"),
                    },
                ],
            },
        )
        .unwrap();

        assert!(
            rendered
                .contains("- **Supersedes:** [ADR-0001](0001-old.md), [ADR-0002](0002-other.md)")
        );
    }

    #[test]
    fn renders_legacy_template_variables_with_tera() {
        let rendered = render_template_contents(
            "{{ id }} {{ counter }} {{ date }} {{ title }}",
            &TemplateValues {
                counter: "0004".to_string(),
                date: "2026-06-19".to_string(),
                number: "0004".to_string(),
                id: "ADR-0004".to_string(),
                title: "Legacy Values".to_string(),
                supersedes: Vec::new(),
            },
        )
        .unwrap();

        assert_eq!(rendered, "ADR-0004 0004 2026-06-19 Legacy Values");
    }
}

#[derive(Serialize)]
struct TemplateValues {
    counter: String,
    date: String,
    number: String,
    id: String,
    title: String,
    supersedes: Vec<SupersededAdr>,
}

fn render_template(template_path: &Path, values: &TemplateValues) -> Result<String> {
    let template = fs::read_to_string(template_path)
        .with_context(|| format!("Failed to read ADR template: {}", template_path.display()))?;

    render_template_contents(&template, values)
}

fn render_template_contents(template: &str, values: &TemplateValues) -> Result<String> {
    let context = TeraContext::from_serialize(values)?;
    Tera::one_off(template, &context, false).with_context(|| "Failed to render ADR template")
}
