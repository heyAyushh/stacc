import { copyFile, cp, mkdir, rm, stat } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const docsDir = join(scriptDir, "..");
const repoRoot = join(docsDir, "..");
const sourcePath = join(repoRoot, "install.sh");
const publicDir = join(docsDir, "public");
const targetPath = join(publicDir, "install.sh");
const sourceMirrorDir = join(docsDir, "source");
const sourceMirrorFiles = ["Cargo.toml", "package.json"];
const generatedPublicEntries = [
  "_next",
  "_not-found",
  "docs",
  "_not-found/",
  "docs/",
  "404.html",
  "index.html",
  "_not-found.html",
  "docs.html",
  "__next._full.txt",
  "__next._head.txt",
  "__next._index.txt",
  "__next._tree.txt",
  "__next.__PAGE__.txt",
  "index.txt",
  "_not-found.txt",
  "docs.txt",
];

async function assertSourceExists() {
  const source = await stat(sourcePath);
  if (!source.isFile()) {
    throw new Error(`Expected ${sourcePath} to be a file.`);
  }
}

await mkdir(publicDir, { recursive: true });
await mkdir(sourceMirrorDir, { recursive: true });
await Promise.all(
  generatedPublicEntries.map((entryName) => rm(join(publicDir, entryName), { recursive: true, force: true }))
);

try {
  await assertSourceExists();
  await copyFile(sourcePath, targetPath);
  await cp(join(repoRoot, "configs"), join(sourceMirrorDir, "configs"), {
    recursive: true,
    force: true,
  });

  await Promise.all(
    sourceMirrorFiles.map((fileName) => copyFile(join(repoRoot, fileName), join(sourceMirrorDir, fileName)))
  );
} catch (error) {
  if (error?.code !== "ENOENT") {
    throw error;
  }

  const target = await stat(targetPath);
  if (!target.isFile()) {
    throw new Error(`Expected ${targetPath} to exist when ${sourcePath} is unavailable.`);
  }
}
