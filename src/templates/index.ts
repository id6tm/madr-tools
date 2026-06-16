import path from "node:path";
import { fileURLToPath } from "node:url";

const TEMPLATES_DIR = path.dirname(fileURLToPath(import.meta.url));

export const ADR_TEMPLATES = [
  {
    name: "standard",
    description: "Context, Options Considered, Decision Outcome, Consequences, and Links",
    path: path.join(TEMPLATES_DIR, "standard.md"),
  },
] as const;

export function findAdrTemplate(name: string) {
  return ADR_TEMPLATES.find((template) => template.name === name);
}
