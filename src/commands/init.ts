import type { Command } from "commander";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { ADR_CONFIG_FILE, DEFAULT_ADR_CONFIG } from "../adr.ts";

export function registerInitCommand(program: Command) {
  program
    .command("init")
    .description(`Create ${ADR_CONFIG_FILE} and the ADR markdown directory`)
    .action(init);
}

async function init() {
  const { cancel, intro, isCancel, outro, select, text } = await import("@clack/prompts");

  intro("madr-toolkit init");

  const directory = await text({
    message: "Where should ADR files be stored?",
    placeholder: DEFAULT_ADR_CONFIG.directory,
    defaultValue: DEFAULT_ADR_CONFIG.directory,
  });

  if (isCancel(directory)) {
    cancel("ADR initialization cancelled.");
    process.exit(1);
  }

  const indexFileName = await text({
    message: "What should the generated index filename be?",
    placeholder: DEFAULT_ADR_CONFIG.indexFileName,
    defaultValue: DEFAULT_ADR_CONFIG.indexFileName,
  });

  if (isCancel(indexFileName)) {
    cancel("ADR initialization cancelled.");
    process.exit(1);
  }

  const template = await select({
    message: "Which ADR template should be used?",
    options: [
      {
        label: "simple",
        value: "preset:simple",
        hint: "Context and Problem Statement, Options COnsidered, DEcision Outcome, Consequences, Links",
      },
    ],
  });

  if (isCancel(template)) {
    cancel("ADR initialization cancelled.");
    process.exit(1);
  }

  const config = {
    directory: directory.trim() || DEFAULT_ADR_CONFIG.directory,
    indexFileName: indexFileName.trim() || DEFAULT_ADR_CONFIG.indexFileName,
    template,
  };
  const resolvedDir = path.resolve(process.cwd(), config.directory);

  await mkdir(resolvedDir, { recursive: true });
  await writeFile(
    path.resolve(process.cwd(), ADR_CONFIG_FILE),
    `${JSON.stringify(config, null, 2)}\n`,
    "utf8",
  );

  outro(`ADR config ready: ${ADR_CONFIG_FILE}`);
}
