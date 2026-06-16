import type { Command } from "commander";
import { copyFile, mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { ADR_CONFIG_FILE, DEFAULT_ADR_CONFIG } from "../adr.ts";
import { ADR_TEMPLATES, findAdrTemplate } from "../templates/index.ts";
import { syncAdrIndex } from "./sync.ts";

export function registerInitCommand(program: Command) {
  program
    .command("init")
    .description(`Create ${ADR_CONFIG_FILE} and the ADR markdown directory`)
    .action(init);
}

async function init() {
  const { cancel, intro, isCancel, outro, select, text } = await import("@clack/prompts");

  intro("madr init");

  const directory = await text({
    message: "Where should ADR files be stored?",
    placeholder: DEFAULT_ADR_CONFIG.directory,
    defaultValue: DEFAULT_ADR_CONFIG.directory,
  });

  if (isCancel(directory)) {
    cancel("ADR initialization cancelled.");
    process.exit(1);
  }

  const selectedTemplate = await select({
    message: "Which ADR template should be used?",
    options: ADR_TEMPLATES.map((template) => ({
      label: template.name,
      value: template.name,
      hint: template.description,
    })),
  });

  if (isCancel(selectedTemplate)) {
    cancel("ADR initialization cancelled.");
    process.exit(1);
  }

  const config = {
    directory: directory.trim() || DEFAULT_ADR_CONFIG.directory,
  };
  const resolvedDir = path.resolve(process.cwd(), config.directory);
  const template = findAdrTemplate(selectedTemplate);

  if (!template) {
    throw new Error(`Unknown ADR template: ${selectedTemplate}`);
  }

  await mkdir(resolvedDir, { recursive: true });
  await copyFile(template.path, path.join(resolvedDir, "TEMPLATE.md"));
  await writeFile(
    path.resolve(process.cwd(), ADR_CONFIG_FILE),
    `${JSON.stringify(config, null, 2)}\n`,
    "utf8",
  );
  await syncAdrIndex();

  outro(`ADR config ready: ${ADR_CONFIG_FILE}`);
}
