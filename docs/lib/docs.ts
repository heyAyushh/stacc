import fs from "node:fs/promises";
import path from "node:path";
import { cache } from "react";
import { parse as parseYaml } from "yaml";

export type DocsSection = {
  id: string;
  label: string;
};

export type DocsFrontmatter = {
  title: string;
  eyebrow: string;
  description: string;
  order: number;
  sections: DocsSection[];
};

export type DocsPage = {
  slug: string;
  frontmatter: DocsFrontmatter;
  body: string;
};

const docsRoot = path.join(process.cwd(), "content");
const frontmatterDelimiter = "---";
const markdownExtension = ".md";
const markdownExtensionPattern = /\.md$/;

function splitFrontmatter(source: string): {
  data: Record<string, unknown>;
  content: string;
} {
  if (!source.startsWith(frontmatterDelimiter)) {
    throw new Error("Docs file is missing frontmatter");
  }

  const frontmatterEnd = source.indexOf(`\n${frontmatterDelimiter}`, frontmatterDelimiter.length);

  if (frontmatterEnd === -1) {
    throw new Error("Docs file frontmatter is not closed");
  }

  const rawFrontmatter = source.slice(frontmatterDelimiter.length, frontmatterEnd).trim();
  const content = source.slice(frontmatterEnd + frontmatterDelimiter.length + 1).trimStart();
  const data = parseYaml(rawFrontmatter);

  if (typeof data !== "object" || data === null || Array.isArray(data)) {
    throw new Error("Docs file frontmatter must be an object");
  }

  return {
    data: data as Record<string, unknown>,
    content,
  };
}

function assertString(value: unknown, fieldName: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`Invalid docs frontmatter field: ${fieldName}`);
  }

  return value;
}

function assertNumber(value: unknown, fieldName: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new Error(`Invalid docs frontmatter field: ${fieldName}`);
  }

  return value;
}

function parseSections(value: unknown): DocsSection[] {
  if (!Array.isArray(value)) {
    throw new Error("Invalid docs frontmatter field: sections");
  }

  return value.map((section) => {
    if (typeof section !== "object" || section === null) {
      throw new Error("Invalid docs section entry");
    }

    const candidate = section as Record<string, unknown>;

    return {
      id: assertString(candidate.id, "sections.id"),
      label: assertString(candidate.label, "sections.label"),
    };
  });
}

function parseFrontmatter(data: Record<string, unknown>): DocsFrontmatter {
  return {
    title: assertString(data.title, "title"),
    eyebrow: assertString(data.eyebrow, "eyebrow"),
    description: assertString(data.description, "description"),
    order: assertNumber(data.order, "order"),
    sections: parseSections(data.sections),
  };
}

const getDocsPage = cache(async function getDocsPage(slug: string): Promise<DocsPage> {
  const filePath = await resolveDocsFile(slug);
  const rawSource = await fs.readFile(filePath, "utf8");
  const parsed = splitFrontmatter(rawSource);

  return {
    slug,
    frontmatter: parseFrontmatter(parsed.data),
    body: parsed.content,
  };
});

async function resolveDocsFile(slug: string): Promise<string> {
  const filePath = path.join(docsRoot, `${slug}${markdownExtension}`);

  try {
    await fs.access(filePath);
    return filePath;
  } catch (error) {
    const nodeError = error as NodeJS.ErrnoException;

    if (nodeError.code !== "ENOENT") {
      throw error;
    }
  }

  throw new Error(`Markdown docs file not found for slug: ${slug}`);
}

export const getAllDocsPages = cache(async function getAllDocsPages(): Promise<DocsPage[]> {
  const entries = await fs.readdir(docsRoot, { withFileTypes: true });
  const slugs = entries.reduce<string[]>((currentSlugs, entry) => {
    if (entry.isFile() && entry.name.endsWith(markdownExtension)) {
      currentSlugs.push(entry.name.replace(markdownExtensionPattern, ""));
    }

    return currentSlugs;
  }, []);
  const pages = await Promise.all(slugs.map((slug) => getDocsPage(slug)));

  return pages.toSorted((left, right) => left.frontmatter.order - right.frontmatter.order);
});
