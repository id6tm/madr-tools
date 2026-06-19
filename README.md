# madr-tools

Small CLI for managing Markdown ADRs (Architectural Decision Records).

It creates a local ADR directory, writes new numbered records from a template, and regenerates a `README.md` index for the ADR folder.

## Install

```sh
npm install --save-dev @id6tm/madr-tools
```

## Usage

Initialize ADRs in a repository:

```sh
madr init
```

This creates:

- `.madrrc.json` with the ADR directory path
- `docs/adr/TEMPLATE.md.tera`
- `docs/adr/README.md`

Create a new ADR:

```sh
madr new "Use Postgres for relational data"
```

This creates the next numbered Markdown file, renders the template values, and refreshes the ADR index.
When existing ADRs are present, the CLI asks whether the new ADR supersedes any of them.

Create a new ADR that supersedes existing ADRs:

```sh
madr new --supersede 1,2 "Replace database decision"
```

This updates the superseded ADR status lines to link to the new ADR.

Regenerate the index after manual file changes:

```sh
madr sync
```

## Config

`.madrrc.json` currently supports:

```json
{
  "directory": "docs/adr"
}
```
