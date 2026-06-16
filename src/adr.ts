import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { z } from "zod";

export const ADR_CONFIG_FILE = ".adrrc.json";

export const DEFAULT_ADR_CONFIG = {
  directory: "docs/adr",
} as const;

const adrConfigSchema = z.object({
  directory: z.string().trim().min(1),
});

export type AdrConfig = z.infer<typeof adrConfigSchema>;

export async function readAdrConfig(): Promise<AdrConfig> {
  const configPath = path.resolve(process.cwd(), ADR_CONFIG_FILE);
  const contents = await readFile(configPath, "utf8");
  const config = JSON.parse(contents) as unknown;
  return adrConfigSchema.parse(config);
}

export async function readMarkdownTitle(filePath: string) {
  const contents = await readFile(filePath, "utf8");
  const heading = contents.split(/\r?\n/).find((line) => line.startsWith("# "));

  return heading?.replace(/^#\s+/, "").trim();
}
