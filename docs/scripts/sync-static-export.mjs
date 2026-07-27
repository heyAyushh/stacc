import { cp, stat } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const docsDir = join(scriptDir, "..");
const exportDir = join(docsDir, "out");
const publicDir = join(docsDir, "public");

const output = await stat(exportDir);
if (!output.isDirectory()) {
  throw new Error(`Expected ${exportDir} to be a directory.`);
}

await cp(exportDir, publicDir, {
  recursive: true,
  force: true,
});
