import { cache } from "react";
import { codeToHtml, type BundledLanguage } from "shiki";
import type { SpecialLanguage } from "@shikijs/types";

type CodeLanguage = BundledLanguage | SpecialLanguage;

const defaultLanguage: SpecialLanguage = "text";
const shellLanguage: BundledLanguage = "bash";
const yamlLanguage: BundledLanguage = "yaml";
const jsonLanguage: BundledLanguage = "json";
const javascriptLanguage: BundledLanguage = "javascript";
const typescriptLanguage: BundledLanguage = "typescript";
const shikiTheme = "github-light";

const yamlLabels = new Set(["agent.stacc", ".stacc", "yaml"]);
const shellActions = new Set(["cli", "copy", "dry-run", "inspect", "run", "verify"]);
const shellCommandPrefixes = ["bash", "cargo", "curl", "shellcheck", "stacc"];

function normalizeMarker(value: string): string {
  return value.trim().toLowerCase();
}

function startsWithShellCommand(source: string): boolean {
  const firstToken = source.trimStart().split(/\s+/, 1)[0]?.toLowerCase();

  return firstToken ? shellCommandPrefixes.includes(firstToken) : false;
}

export function resolveCodeLanguage(label: string, action: string, source: string): CodeLanguage {
  const normalizedLabel = normalizeMarker(label);
  const normalizedAction = normalizeMarker(action);

  if (yamlLabels.has(normalizedLabel) || normalizedAction === yamlLanguage) {
    return yamlLanguage;
  }

  if (normalizedAction === jsonLanguage || normalizedLabel.endsWith(".json")) {
    return jsonLanguage;
  }

  if (normalizedAction === typescriptLanguage || normalizedLabel.endsWith(".ts")) {
    return typescriptLanguage;
  }

  if (normalizedAction === javascriptLanguage || normalizedLabel.endsWith(".js")) {
    return javascriptLanguage;
  }

  if (shellActions.has(normalizedAction) || startsWithShellCommand(source)) {
    return shellLanguage;
  }

  return defaultLanguage;
}

export const highlightCode = cache(async function highlightCode(
  source: string,
  language: CodeLanguage
): Promise<string> {
  return codeToHtml(source, {
    lang: language,
    theme: shikiTheme,
  });
});
