import { cp, mkdir, readdir } from "node:fs/promises";
import path from "node:path";

const sourceDir = path.resolve("src/templates");
const targetDir = path.resolve("dist/templates");

await mkdir(targetDir, { recursive: true });

const entries = await readdir(sourceDir, { withFileTypes: true });

await Promise.all(
  entries
    .filter((entry) => entry.isFile() && entry.name.endsWith(".md"))
    .map((entry) => cp(path.join(sourceDir, entry.name), path.join(targetDir, entry.name))),
);
