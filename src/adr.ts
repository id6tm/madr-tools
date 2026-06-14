import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

export const ADR_CONFIG_FILE = ".adrrc.json";

export const DEFAULT_ADR_CONFIG = {
  directory: "docs/adr",
  indexFileName: "INDEX.md",
  template: "preset:simple",
} as const;

export type AdrConfig = {
  directory: string;
  indexFileName: string;
  template: string;
};

export async function readAdrConfig(): Promise<AdrConfig> {
  try {
    const contents = await readFile(path.resolve(process.cwd(), ADR_CONFIG_FILE), "utf8");
    const config = JSON.parse(contents) as unknown;
    const values = isRecord(config) ? config : {};

    return {
      directory: stringOrDefault(values.directory, DEFAULT_ADR_CONFIG.directory),
      indexFileName: stringOrDefault(values.indexFileName, DEFAULT_ADR_CONFIG.indexFileName),
      template: stringOrDefault(values.template, DEFAULT_ADR_CONFIG.template),
    };
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return { ...DEFAULT_ADR_CONFIG };
    }

    throw error;
  }
}

export async function readMarkdownTitle(filePath: string) {
  const contents = await readFile(filePath, "utf8");
  const heading = contents.split(/\r?\n/).find((line) => line.startsWith("# "));

  return heading?.replace(/^#\s+/, "").trim();
}

function stringOrDefault(value: unknown, fallback: string) {
  return typeof value === "string" && value.trim() ? value : fallback;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
