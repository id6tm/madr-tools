import type { Command } from "commander";
import { readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { readAdrConfig, readMarkdownTitle } from "../adr.ts";

const ADR_INDEX_FILE_NAME = "README.md";
const ADR_TEMPLATE_FILE_NAME = "TEMPLATE.md";

export function registerSyncCommand(program: Command) {
  program.command("sync").description(`Regenerate the ADR index`).action(syncAdrIndex);
}

export async function syncAdrIndex() {
  const config = await readAdrConfig();
  const resolvedDir = path.resolve(process.cwd(), config.directory);
  const entries = await readdir(resolvedDir, { withFileTypes: true });
  const indexFileName = ADR_INDEX_FILE_NAME.toLowerCase();
  const templateFileName = ADR_TEMPLATE_FILE_NAME.toLowerCase();

  const adrFiles = entries
    .filter((entry) => entry.isFile())
    .map((entry) => entry.name)
    .filter((name) => name.toLowerCase().endsWith(".md"))
    .filter((name) => name.toLowerCase() !== indexFileName)
    .filter((name) => name.toLowerCase() !== templateFileName)
    .sort((a, b) => a.localeCompare(b));

  const items = await Promise.all(
    adrFiles.map(async (fileName) => {
      const title = await readMarkdownTitle(path.join(resolvedDir, fileName));
      return `- [${adrId(fileName)}](${encodeURI(fileName)}) — ${title}`;
    }),
  );

  const index = [
    "# Architecture Decision Records (ADR)",
    "",
    "This directory tracks the key architectural decisions shaping our system. Each record documents the underlying rationale, trade-offs, and final decisions to provide a transparent history of our technical choices.",
    "",
    "## Workflow",
    "",
    "We manage our ADRs using [@id6tm/madr-tools](https://github.com/id6tm/madr-tools). Do not edit this index file directly.",
    "",
    '- **Create a new ADR:** Run `madr new "Title of your decision"`',
    "- **Regenerate this index:** Run `madr sync`",
    "",
    "## Documented Decisions",
    "",
    items.length > 0 ? items.join("\n") : "**No ADR issued at the moment.**",
    "",
  ].join("\n");

  await writeFile(path.join(resolvedDir, ADR_INDEX_FILE_NAME), index, "utf8");
  console.log(`Generated ${path.join(config.directory, ADR_INDEX_FILE_NAME)}`);
}

function adrId(fileName: string) {
  const number = /^(\d+)(?:[-_ ].*)?\.md$/i.exec(fileName)?.[1];
  return number ? `ADR-${number}` : "ADR";
}
