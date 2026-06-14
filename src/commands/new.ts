import type { Command } from "commander";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { DEFAULT_ADR_CONFIG, readAdrConfig } from "../adr.ts";
import { syncAdrIndex } from "./sync.ts";

const TEMPLATE_PATH = path.join(path.dirname(fileURLToPath(import.meta.url)), "../template.md");

export function registerNewCommand(program: Command) {
  program
    .command("new")
    .description(
      `Create a new ADR markdown file in the ADR directory (${DEFAULT_ADR_CONFIG.directory})`,
    )
    .action(newAdr);
}

async function newAdr() {
  const { cancel, intro, isCancel, outro, text } = await import("@clack/prompts");

  intro("madr-toolkit new");

  const title = await text({
    message: "What is the ADR title?",
    validate(value = "") {
      if (!value.trim()) {
        return "Enter an ADR title.";
      }
    },
  });

  if (isCancel(title)) {
    cancel("ADR creation cancelled.");
    process.exit(1);
  }

  const config = await readAdrConfig();
  const resolvedDir = path.resolve(process.cwd(), config.directory);
  await mkdir(resolvedDir, { recursive: true });

  const entries = await readdir(resolvedDir, { withFileTypes: true });
  const fileNumbers = entries
    .filter((entry) => entry.isFile())
    .map((entry) => /^(\d+)(?:[-_ ].*)?\.md$/i.exec(entry.name)?.[1])
    .filter((number): number is string => number !== undefined);

  const width = Math.max(4, ...fileNumbers.map((number) => number.length));
  const nextNumber = Math.max(0, ...fileNumbers.map((number) => Number.parseInt(number, 10))) + 1;
  const adrNumber = String(nextNumber).padStart(width, "0");
  const fileName = `${adrNumber}-${slugify(title)}.md`;
  const relativePath = path.join(config.directory, fileName);

  const contents = await renderAdrTemplate({
    date: currentDate(),
    id: `ADR-${adrNumber}`,
    number: adrNumber,
    title,
  });

  await writeFile(path.join(resolvedDir, fileName), contents, {
    encoding: "utf8",
    flag: "wx",
  });
  await syncAdrIndex();

  outro(`Created ${relativePath}`);
}

async function renderAdrTemplate(values: {
  date: string;
  id: string;
  number: string;
  title: string;
}) {
  const template = await readFile(TEMPLATE_PATH, "utf8");
  return Object.entries({
    "{{counter}}": values.number,
    "{{date}}": values.date,
    "{{id}}": values.id,
    "{{title}}": values.title.trim(),
  }).reduce((contents, [placeholder, value]) => contents.replaceAll(placeholder, value), template);
}

function currentDate() {
  return new Date().toISOString().slice(0, 10);
}

function slugify(value: string) {
  const slug = value
    .trim()
    .toLowerCase()
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");

  return slug || "adr";
}
