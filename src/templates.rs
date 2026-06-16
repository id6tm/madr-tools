use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Copy)]
pub struct AdrTemplate {
    pub name: &'static str,
    pub description: &'static str,
    pub contents: &'static str,
}

pub const ADR_TEMPLATES: &[AdrTemplate] = &[AdrTemplate {
    name: "standard",
    description: "Context, Options Considered, Decision Outcome, Consequences, and Links",
    contents: include_str!("templates/standard.md"),
}];

pub fn find(name: &str) -> Result<AdrTemplate> {
    ADR_TEMPLATES
        .iter()
        .copied()
        .find(|template| template.name == name)
        .ok_or_else(|| anyhow!("Unknown ADR template: {name}"))
}
