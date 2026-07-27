import fs from "node:fs/promises";
import type { Dirent } from "node:fs";
import path from "node:path";
import { cache } from "react";
import type { ConfigInventory, ConfigInventoryGroup } from "@/lib/inventory";
import { toSkillHref } from "@/lib/anchors";
import { asRecord, configsRoot, readJsonFile } from "@/lib/inventory-shared";

type ConfigItem = { name: string; path: string };

const staccRepositoryUrl = "https://github.com/heyAyushh/stacc";
const docBackedGroups = new Set(["Skills", "Stacks", "Cursor Plugin Skills", "Codex Skill Imports"]);

function isNodeError(error: unknown): error is NodeJS.ErrnoException {
  return error instanceof Error;
}

async function safeReadDir(directoryPath: string): Promise<Dirent[]> {
  try {
    return await fs.readdir(directoryPath, { withFileTypes: true });
  } catch (error: unknown) {
    if (isNodeError(error) && error.code === "ENOENT") {
      return [];
    }

    throw error;
  }
}

async function filesWithExtension(directory: string, extension: string): Promise<Array<{ name: string; path: string }>> {
  const entries = await safeReadDir(path.join(configsRoot, directory));

  return entries
    .reduce<Array<{ name: string; path: string }>>((items, entry) => {
      if (entry.isFile() && entry.name.endsWith(extension)) {
        items.push({
          name: entry.name.replace(extension, ""),
          path: path.join("configs", directory, entry.name),
        });
      }

      return items;
    }, [])
    .toSorted((left, right) => left.name.localeCompare(right.name));
}

async function childDirectories(directory: string): Promise<Array<{ name: string; path: string }>> {
  const entries = await safeReadDir(path.join(configsRoot, directory));

  return entries
    .reduce<Array<{ name: string; path: string }>>((items, entry) => {
      if (entry.isDirectory()) {
        items.push({
          name: entry.name,
          path: path.join("configs", directory, entry.name),
        });
      }

      return items;
    }, [])
    .toSorted((left, right) => left.name.localeCompare(right.name));
}

async function mcpServers(): Promise<Array<{ name: string; path: string }>> {
  const source = await readJsonFile(path.join(configsRoot, "mcps", "mcp.json"));
  const servers = asRecord(source.mcpServers, "mcpServers");

  return Object.keys(servers)
    .toSorted()
    .map((name) => ({
      name,
      path: "configs/mcps/mcp.json",
    }));
}

function sourceHref(configPath: string): string {
  const sourceKind = path.extname(configPath) ? "blob" : "tree";

  return `${staccRepositoryUrl}/${sourceKind}/main/${configPath}`;
}

function configItemLink(groupName: string, configPath: string): Pick<ConfigInventoryGroup["items"][number], "href" | "hrefLabel" | "isExternal"> {
  if (docBackedGroups.has(groupName)) {
    return {
      href: toSkillHref(configPath),
      hrefLabel: "Docs",
      isExternal: false,
    };
  }

  return {
    href: sourceHref(configPath),
    hrefLabel: "Source",
    isExternal: true,
  };
}

function group(name: string, description: string, items: ConfigItem[]): ConfigInventoryGroup {
  return {
    name,
    description,
    count: items.length,
    items: items.map((item) => {
      const link = configItemLink(name, item.path);

      return {
        name: item.name,
        path: item.path,
        ...link,
      };
    }),
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
    childDirectories(path.join("plugins", "cursor", "skills")),
    childDirectories(path.join("plugins", "cursor", "hooks")),
    childDirectories(path.join("plugins", "cursor", "agents")),
    childDirectories(path.join("plugins", "codex", "skills")),
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
