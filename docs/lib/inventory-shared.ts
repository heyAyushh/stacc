import fs from "node:fs/promises";
import { existsSync } from "node:fs";
import path from "node:path";

function resolveRepoRoot(): string {
  const parentRoot = path.join(process.cwd(), "..");
  if (existsSync(path.join(parentRoot, "configs"))) {
    return parentRoot;
  }

  return path.join(process.cwd(), "source");
}

export const repoRoot = resolveRepoRoot();
export const configsRoot = path.join(repoRoot, "configs");

export function asRecord(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`Expected object for ${label}`);
  }

  return value as Record<string, unknown>;
}

export function asString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`Expected string for ${label}`);
  }

  return value;
}

export function asNullableString(value: unknown): string | null {
  return typeof value === "string" && value.trim().length > 0 ? value : null;
}

export function optionalString(value: unknown, fallback: string): string {
  return typeof value === "string" && value.trim().length > 0 ? value : fallback;
}

export async function readJsonFile(filePath: string): Promise<Record<string, unknown>> {
  const source = await fs.readFile(filePath, "utf8");

  return asRecord(JSON.parse(source), filePath);
}
