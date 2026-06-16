#!/usr/bin/env node

const { spawnSync } = require("node:child_process");
const path = require("node:path");

const packages = {
  "darwin arm64": ["@id6tm/madr-tools-darwin-arm64", "madr"],
  "linux arm64": ["@id6tm/madr-tools-linux-arm64", "madr"],
  "linux x64": ["@id6tm/madr-tools-linux-x64", "madr"],
  "win32 x64": ["@id6tm/madr-tools-win32-x64", "madr.exe"],
};

const key = `${process.platform} ${process.arch}`;
const target = packages[key];

if (!target) {
  console.error(`madr is not available for ${process.platform}/${process.arch}.`);
  process.exit(1);
}

const [packageName, binaryName] = target;
let binaryPath;

try {
  binaryPath = require.resolve(path.join(packageName, "bin", binaryName));
} catch {
  console.error(
    `Missing native madr package ${packageName}. Reinstall @id6tm/madr-tools and ensure optional dependencies are enabled.`,
  );
  process.exit(1);
}

const result = spawnSync(binaryPath, process.argv.slice(2), { stdio: "inherit" });

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);
