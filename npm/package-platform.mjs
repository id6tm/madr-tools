import { copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const rootPackage = JSON.parse(readFileSync(path.join(root, "package.json"), "utf8"));

const platform = requiredEnv("MADR_NPM_PLATFORM");
const arch = requiredEnv("MADR_NPM_ARCH");
const binarySource = requiredEnv("MADR_BINARY_PATH");
const packageName = `${rootPackage.name}-${platform}-${arch}`;
const packageDirectory = path.join(root, "npm-packages", `${platform}-${arch}`);
const binaryName = platform === "win32" ? "madr.exe" : "madr";

rmSync(packageDirectory, { recursive: true, force: true });
mkdirSync(path.join(packageDirectory, "bin"), { recursive: true });
copyFileSync(binarySource, path.join(packageDirectory, "bin", binaryName));

writeFileSync(
  path.join(packageDirectory, "package.json"),
  `${JSON.stringify(
    {
      name: packageName,
      version: rootPackage.version,
      description: `${rootPackage.description} (${platform}/${arch})`,
      repository: rootPackage.repository,
      license: rootPackage.license,
      os: [platform],
      cpu: [arch],
      bin: {
        madr: `bin/${binaryName}`,
      },
      files: ["bin"],
    },
    null,
    2,
  )}\n`,
);

function requiredEnv(name) {
  const value = process.env[name];
  if (!value) {
    throw new Error(`Missing required environment variable: ${name}`);
  }
  return value;
}
