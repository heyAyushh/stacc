import fs from "node:fs/promises";
import type { Dirent } from "node:fs";
import path from "node:path";
import { cache } from "react";

export type SkillInventoryItem = {
  name: string;
  description: string;
  localPath: string;
  collection: string;
  licenseSpdx: string;
  licenseSource: string;
  licenseFile: string | null;
  version: string;
  versionSource: string;
  sourceUrl: string | null;
  repoUrl: string | null;
  declaredCommit: string | null;
  headCommit: string | null;
  headError: string | null;
};

export type SkillCollection = {
  name: string;
  count: number;
  licenses: string[];
  skills: SkillInventoryItem[];
};

export type SkillInventory = {
  generatedAt: string;
  sourceRepo: string;
  totalSkills: number;
  collections: SkillCollection[];
};

export type BinaryInventory = {
  crateName: string;
  crateVersion: string;
  crateLicense: string;
  crateDescription: string;
  binaryName: string;
  docsVersion: string;
  rootVersion: string;
  docsDependencies: Array<{ name: string; version: string; scope: "runtime" | "dev" }>;
};

export type ConfigInventoryGroup = {
  name: string;
  description: string;
  count: number;
  items: Array<{ name: string; path: string; detail: string | null }>;
};

export type ConfigInventory = {
  totalItems: number;
  groups: ConfigInventoryGroup[];
};

const repoRoot = path.join(process.cwd(), "..");
const skillLockPath = path.join(repoRoot, "configs", "metadata", "skills.lock.json");
const cargoManifestPath = path.join(repoRoot, "Cargo.toml");
const docsPackagePath = path.join(process.cwd(), "package.json");
const rootPackagePath = path.join(repoRoot, "package.json");
const configsRoot = path.join(repoRoot, "configs");

function asRecord(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`Expected object for ${label}`);
  }

  return value as Record<string, unknown>;
}

function asString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`Expected string for ${label}`);
  }

  return value;
}

function asNullableString(value: unknown): string | null {
  return typeof value === "string" && value.trim().length > 0 ? value : null;
}

function optionalString(value: unknown, fallback: string): string {
  return typeof value === "string" && value.trim().length > 0 ? value : fallback;
}

function readTomlString(source: string, key: string): string {
  const match = source.match(new RegExp(`^${key}\\s*=\\s*"([^"]+)"`, "m"));

  if (!match) {
    throw new Error(`Missing Cargo field: ${key}`);
  }

  return match[1];
}

function firstBinName(source: string): string {
  const binSection = source.match(/\[\[bin\]\][\s\S]*?name\s*=\s*"([^"]+)"/);

  return binSection?.[1] ?? readTomlString(source, "name");
}

async function readJsonFile(filePath: string): Promise<Record<string, unknown>> {
  const source = await fs.readFile(filePath, "utf8");

  return asRecord(JSON.parse(source), filePath);
}

function parseSkill(value: unknown): SkillInventoryItem {
  const skill = asRecord(value, "skill");
  const license = asRecord(skill.license, "skill.license");
  const version = asRecord(skill.version, "skill.version");
  const origin = asRecord(skill.origin, "skill.origin");

  return {
    name: asString(skill.name, "skill.name"),
    description: optionalString(skill.description, "No description declared in skill metadata."),
    localPath: asString(skill.local_path, "skill.local_path"),
    collection: asString(skill.collection, "skill.collection"),
    licenseSpdx: asString(license.spdx, "skill.license.spdx"),
    licenseSource: asString(license.source, "skill.license.source"),
    licenseFile: asNullableString(license.file),
    version: asString(version.value, "skill.version.value"),
    versionSource: asString(version.source, "skill.version.source"),
    sourceUrl: asNullableString(origin.source_url),
    repoUrl: asNullableString(origin.repo_url),
    declaredCommit: asNullableString(origin.declared_commit),
    headCommit: asNullableString(origin.head_commit),
    headError: asNullableString(origin.head_error),
  };
}

function groupSkills(skills: SkillInventoryItem[]): SkillCollection[] {
  const groups = new Map<string, SkillInventoryItem[]>();

  for (const skill of skills) {
    const items = groups.get(skill.collection) ?? [];
    items.push(skill);
    groups.set(skill.collection, items);
  }

  return Array.from(groups.entries())
    .map(([name, items]) => ({
      name,
      count: items.length,
      licenses: Array.from(new Set(items.map((item) => item.licenseSpdx))).toSorted(),
      skills: items.toSorted((left, right) => left.name.localeCompare(right.name)),
    }))
    .toSorted((left, right) => left.name.localeCompare(right.name));
}

export const getSkillInventory = cache(async function getSkillInventory(): Promise<SkillInventory> {
  const lockfile = await readJsonFile(skillLockPath);
  const rawSkills = lockfile.skills;

  if (!Array.isArray(rawSkills)) {
    throw new Error("skills.lock.json is missing skills array");
  }

  const skills = rawSkills.map(parseSkill).toSorted((left, right) => left.name.localeCompare(right.name));
  const generatedSeconds = Number(lockfile.generated_unix_seconds);

  if (!Number.isFinite(generatedSeconds)) {
    throw new Error("skills.lock.json is missing generated_unix_seconds");
  }

  return {
    generatedAt: new Date(generatedSeconds * 1000).toISOString(),
    sourceRepo: asString(lockfile.source_repo, "source_repo"),
    totalSkills: skills.length,
    collections: groupSkills(skills),
  };
});

function dependencyRows(
  dependencies: Record<string, unknown> | undefined,
  scope: "runtime" | "dev"
): Array<{ name: string; version: string; scope: "runtime" | "dev" }> {
  if (!dependencies) {
    return [];
  }

  return Object.entries(dependencies).map(([name, version]) => ({
    name,
    version: asString(version, `dependency ${name}`),
    scope,
  }));
}

export const getBinaryInventory = cache(async function getBinaryInventory(): Promise<BinaryInventory> {
  const [cargoSource, docsPackage, rootPackage] = await Promise.all([
    fs.readFile(cargoManifestPath, "utf8"),
    readJsonFile(docsPackagePath),
    readJsonFile(rootPackagePath),
  ]);
  const docsDependencies = asRecord(docsPackage.dependencies, "docs.dependencies");
  const docsDevDependencies = asRecord(docsPackage.devDependencies, "docs.devDependencies");

  return {
    crateName: readTomlString(cargoSource, "name"),
    crateVersion: readTomlString(cargoSource, "version"),
    crateLicense: readTomlString(cargoSource, "license"),
    crateDescription: readTomlString(cargoSource, "description"),
    binaryName: firstBinName(cargoSource),
    docsVersion: asString(docsPackage.version, "docs.version"),
    rootVersion: asString(rootPackage.version, "root.version"),
    docsDependencies: [
      ...dependencyRows(docsDependencies, "runtime"),
      ...dependencyRows(docsDevDependencies, "dev"),
    ].toSorted((left, right) => left.name.localeCompare(right.name)),
  };
});

async function safeReadDir(directoryPath: string): Promise<Dirent[]> {
  try {
    return await fs.readdir(directoryPath, { withFileTypes: true });
  } catch (error) {
    const nodeError = error as NodeJS.ErrnoException;

    if (nodeError.code === "ENOENT") {
      return [];
    }

    throw error;
  }
}

async function filesWithExtension(directory: string, extension: string): Promise<Array<{ name: string; path: string }>> {
  const entries = await safeReadDir(path.join(configsRoot, directory));

  return entries
    .filter((entry) => entry.isFile() && entry.name.endsWith(extension))
    .map((entry) => ({
      name: entry.name.replace(extension, ""),
      path: path.join("configs", directory, entry.name),
    }))
    .toSorted((left, right) => left.name.localeCompare(right.name));
}

async function childDirectories(directory: string): Promise<Array<{ name: string; path: string }>> {
  const entries = await safeReadDir(path.join(configsRoot, directory));

  return entries
    .filter((entry) => entry.isDirectory())
    .map((entry) => ({
      name: entry.name,
      path: path.join("configs", directory, entry.name),
    }))
    .toSorted((left, right) => left.name.localeCompare(right.name));
}

async function mcpServers(): Promise<Array<{ name: string; path: string }>> {
  const mcpPath = path.join(configsRoot, "mcps", "mcp.json");
  const source = await readJsonFile(mcpPath);
  const servers = asRecord(source.mcpServers, "mcpServers");

  return Object.keys(servers)
    .toSorted()
    .map((name) => ({
      name,
      path: "configs/mcps/mcp.json",
    }));
}

function group(
  name: string,
  description: string,
  items: Array<{ name: string; path: string; detail?: string | null }>
): ConfigInventoryGroup {
  return {
    name,
    description,
    count: items.length,
    items: items.map((item) => ({
      name: item.name,
      path: item.path,
      detail: item.detail ?? null,
    })),
  };
}

export const getConfigInventory = cache(async function getConfigInventory(): Promise<ConfigInventory> {
  const [
    agents,
    commands,
    rules,
    skills,
    stacks,
    hooks,
    cursorPluginSkills,
    cursorPluginHooks,
    cursorPluginAgents,
    codexSkillGroups,
    mcps,
  ] = await Promise.all([
    filesWithExtension("agents", ".md"),
    filesWithExtension("commands", ".md"),
    filesWithExtension("rules", ".mdc"),
    childDirectories("skills"),
    childDirectories("stacks"),
    childDirectories("hooks"),
    childDirectories(path.join("cursor-plugins", "skills")),
    childDirectories(path.join("cursor-plugins", "hooks")),
    childDirectories(path.join("cursor-plugins", "agents")),
    childDirectories(path.join("codex-skills", "skills")),
    mcpServers(),
  ]);
  const groups = [
    group("Agents", "Prompted agent definitions used by supported tools.", agents),
    group("Commands", "Slash-command prompt files installed into command-capable editors.", commands),
    group("Rules", "Always-applied Cursor MDC rules and repo policy files.", rules),
    group("Skills", "General STACC skill packages with SKILL.md entrypoints.", skills),
    group("Stacks", "Framework and language stack bundles installable via --category stack.", stacks),
    group("Hooks", "Optional hook packages selected with --hook.", hooks),
    group("MCP Servers", "Server keys from configs/mcps/mcp.json.", mcps),
    group("Cursor Plugin Skills", "Imported Cursor plugin skills kept in a separate category.", cursorPluginSkills),
    group("Cursor Plugin Hooks", "Imported Cursor plugin hook packages.", cursorPluginHooks),
    group("Cursor Plugin Agents", "Imported Cursor plugin agent definitions.", cursorPluginAgents),
    group("Codex Skill Imports", "Codex-specific imported skills.", codexSkillGroups),
  ];

  return {
    groups,
    totalItems: groups.reduce((total, current) => total + current.count, 0),
  };
});
