#!/usr/bin/env node

import { Command } from "commander";
import process from "node:process";
import { registerInitCommand } from "./commands/init.ts";
import { registerNewCommand } from "./commands/new.ts";
import { registerSyncCommand } from "./commands/sync.ts";
import pc from "picocolors";

async function main() {
  const program = new Command();

  program
    .name("madr-toolkit")
    .description("Manage markdown architecture decision records")
    .version("1.0.0");

  program.configureHelp({
    sortSubcommands: true,

    // Clean, high-contrast typography
    styleTitle: (str) => pc.bold(pc.white(str)),
    styleDescriptionText: (str) => pc.dim(str),

    // Commands & Subcommands (Unified under cool tones)
    styleCommandText: (str) => pc.cyan(str),
    styleSubcommandText: (str) => pc.cyan(str),
    styleCommandDescription: (str) => pc.gray(str),

    // Options & Arguments (High readability inputs)
    styleOptionText: (str) => pc.green(str),
    styleArgumentText: (str) => pc.yellow(str),
  });

  registerInitCommand(program);
  registerNewCommand(program);
  registerSyncCommand(program);

  if (process.argv.length <= 2) {
    program.outputHelp();
    return;
  }

  await program.parseAsync(process.argv);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exitCode = 1;
});
