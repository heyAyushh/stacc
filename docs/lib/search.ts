import { cache } from "react";
import { toSkillHref } from "@/lib/anchors";
import { getAllDocsPages } from "@/lib/docs";
import { getSkillInventory } from "@/lib/inventory";
import { getConfigInventory } from "@/lib/inventory-config";
import { creatorRepositoryLabel, displaySkillVersion, originalSourceLabel, sourceDisplayLabel } from "@/lib/skill-display";

export type SearchItemKind = "doc" | "section" | "skill" | "mcp";

export type SearchItem = {
  id: string;
  kind: SearchItemKind;
  title: string;
  eyebrow: string;
  detail: string;
  description: string;
  href: string;
  keywords: string[];
};

const maxDescriptionLength = 180;
const maxKeywordCount = 48;

function compactDescription(value: string): string {
  if (value.length <= maxDescriptionLength) {
    return value;
  }

  return `${value.slice(0, maxDescriptionLength - 1).trimEnd()}…`;
}

function compactKeywords(values: Array<string | null | undefined>): string[] {
  return Array.from(
    new Set(
      values.flatMap((value) =>
        (value?.split(/[\s/|,.-]+/) ?? []).flatMap((segment) => {
          const normalizedSegment = segment.trim().toLowerCase();

          return normalizedSegment ? [normalizedSegment] : [];
        })
      )
    )
  ).slice(0, maxKeywordCount);
}

export const getSearchItems = cache(async function getSearchItems(): Promise<SearchItem[]> {
  const [pages, skillInventory, configInventory] = await Promise.all([
    getAllDocsPages(),
    getSkillInventory(),
    getConfigInventory(),
  ]);
  const docItems = pages.flatMap<SearchItem>((page) => {
    const pageHref = `/docs/${page.slug}`;
    const pageItem: SearchItem = {
      id: `doc-${page.slug}`,
      kind: "doc",
      title: page.frontmatter.title,
      eyebrow: page.frontmatter.eyebrow,
      detail: `/docs/${page.slug}`,
      description: compactDescription(page.frontmatter.description),
      href: pageHref,
      keywords: compactKeywords([page.slug, page.frontmatter.title, page.frontmatter.eyebrow]),
    };
    const sectionItems = page.frontmatter.sections.map<SearchItem>((section) => ({
      id: `section-${page.slug}-${section.id}`,
      kind: "section",
      title: section.label,
      eyebrow: page.frontmatter.title,
      detail: `/docs/${page.slug}#${section.id}`,
      description: `Jump to ${section.label} inside ${page.frontmatter.title}.`,
      href: `${pageHref}#${section.id}`,
      keywords: compactKeywords([page.slug, page.frontmatter.title, section.label, section.id]),
    }));

    return [pageItem, ...sectionItems];
  });
  const skillItems = skillInventory.collections.flatMap<SearchItem>((collection) =>
    collection.skills.map((skill) => ({
      id: `skill-${skill.localPath}`,
      kind: "skill",
      title: skill.name,
      eyebrow: `${skill.collection} / ${creatorRepositoryLabel(skill)}`,
      detail: `${sourceDisplayLabel(skill)} / ${displaySkillVersion(skill.version)} / ${skill.licenseSpdx}`,
      description: compactDescription(skill.description),
      href: toSkillHref(skill.localPath),
      keywords: compactKeywords([
        skill.name,
        skill.collection,
        skill.description,
        displaySkillVersion(skill.version),
        skill.licenseSpdx,
        skill.licenseSource,
        originalSourceLabel(skill),
        creatorRepositoryLabel(skill),
        skill.repoUrl,
        skill.sourceUrl,
      ]),
    }))
  );
  const mcpItems = (configInventory.groups.find((group) => group.name === "MCP Servers")?.items ?? []).map<SearchItem>(
    (server) => ({
      id: `mcp-${server.name}`,
      kind: "mcp",
      title: server.name,
      eyebrow: "MCP Server",
      detail: `--mcp-server ${server.name}`,
      description: `Install the ${server.name} MCP server from the STACC catalog.`,
      href: "/docs/configurations#catalog",
      keywords: compactKeywords(["mcp", "server", server.name, server.path, `--mcp-server ${server.name}`]),
    })
  );

  return [...docItems, ...skillItems, ...mcpItems];
});
