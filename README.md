# madr-tools

Small CLI for managing Markdown ADRs (Architectural Decision Records).

It creates a local ADR directory, writes new numbered records from a template, and regenerates a `README.md` index for the ADR folder.

## Install

```sh
npm install --save-dev @id6tm/madr-tools
```

## Usage

### Initialize ADRs

```sh
madr init
```

This creates:

- `.madrrc.json` with the ADR directory path
- `docs/adr/TEMPLATE.md.tera`
- `docs/adr/README.md`

### Create an ADR

```sh
madr new "Use Postgres for relational data"
```

This creates the next numbered Markdown file, renders the template values, and refreshes the ADR index.
When existing ADRs are present, the CLI asks whether the new ADR supersedes any of them.

To skip the prompt and supersede ADRs directly, pass one or more ADR numbers:

```sh
madr new --supersede 1,2 "Replace database decision"
```

This updates the superseded ADR status lines to link to the new ADR.

### Regenerate the Index

```sh
madr sync
```

Use this after manually editing ADR files. The index is already regenerated automatically when you run `madr new`.

### Export the Bundled Template

```sh
madr export-template
```

This overwrites `TEMPLATE.md.tera` with the template shipped in the current `madr` version.
