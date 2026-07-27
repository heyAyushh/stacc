import fs from "node:fs/promises";
import { execFile } from "node:child_process";
import path from "node:path";
import { cache } from "react";
import { promisify } from "node:util";
import { toSkillPath } from "@/lib/anchors";
import {
  asNullableString,
  asRecord,
  asString,
  optionalString,
  readJsonFile,
  repoRoot,
} from "@/lib/inventory-shared";

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
  latestCommitYear: string;
};

export type ConfigInventoryGroup = {
  name: string;
  description: string;
  count: number;
  items: Array<{ name: string; path: string; href: string; hrefLabel: string; isExternal: boolean }>;
};

export type ConfigInventory = {
  totalItems: number;
  groups: ConfigInventoryGroup[];
};

const skillLockPath = path.join(repoRoot, "configs", "metadata", "skills.lock.json");
const cargoManifestPath = path.join(repoRoot, "Cargo.toml");
const execFileAsync = promisify(execFile);

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

async function latestCommitYear(): Promise<string> {
  try {
    const { stdout } = await execFileAsync("git", ["-C", repoRoot, "log", "-1", "--format=%cd", "--date=format:%Y"]);
    const year = stdout.trim();

    if (/^\d{4}$/.test(year)) {
      return year;
    }
  } catch (error: unknown) {
    if (!(error instanceof Error)) {
      throw error;
    }

    return currentCalendarYear();
  }

  return currentCalendarYear();
}

function currentCalendarYear(): string {
  return String(new Date().getFullYear());
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

export const getAllSkillItems = cache(async function getAllSkillItems(): Promise<SkillInventoryItem[]> {
  const inventory = await getSkillInventory();

  return inventory.collections.flatMap((collection) => collection.skills);
});

export const getSkillByPath = cache(async function getSkillByPath(skillPath: string): Promise<SkillInventoryItem | null> {
  const skills = await getAllSkillItems();

  return skills.find((skill) => toSkillPath(skill.localPath).join("/") === skillPath) ?? null;
});

export const getSkillMarkdown = cache(async function getSkillMarkdown(skill: SkillInventoryItem): Promise<string> {
  return fs.readFile(path.join(repoRoot, skill.localPath, "SKILL.md"), "utf8");
});

export const getBinaryInventory = cache(async function getBinaryInventory(): Promise<BinaryInventory> {
  const [cargoSource, commitYear] = await Promise.all([
    fs.readFile(cargoManifestPath, "utf8"),
    latestCommitYear(),
  ]);

  return {
    crateName: readTomlString(cargoSource, "name"),
    crateVersion: readTomlString(cargoSource, "version"),
    crateLicense: readTomlString(cargoSource, "license"),
    crateDescription: readTomlString(cargoSource, "description"),
    binaryName: firstBinName(cargoSource),
    latestCommitYear: commitYear,
  };
});

export { getConfigInventory } from "@/lib/inventory-config";
