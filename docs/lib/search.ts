import { cache } from "react";
import { toAnchorId } from "@/lib/anchors";
import { getAllDocsPages } from "@/lib/docs";
import { getSkillInventory } from "@/lib/inventory";

export type SearchItemKind = "doc" | "section" | "skill";

export type SearchItem = {
  id: string;
  kind: SearchItemKind;
  title: string;
  eyebrow: string;
  description: string;
  href: string;
  keywords: string[];
};

function compactKeywords(values: Array<string | null | undefined>): string[] {
  return Array.from(
    new Set(
      values
        .flatMap((value) => value?.split(/[\s/|,.-]+/) ?? [])
        .map((value) => value.trim().toLowerCase())
        .filter(Boolean)
    )
  );
}

export const getSearchItems = cache(async function getSearchItems(): Promise<SearchItem[]> {
  const [pages, skillInventory] = await Promise.all([getAllDocsPages(), getSkillInventory()]);
  const docItems = pages.flatMap<SearchItem>((page) => {
    const pageHref = `/docs/${page.slug}`;
    const pageItem: SearchItem = {
      id: `doc-${page.slug}`,
      kind: "doc",
      title: page.frontmatter.title,
      eyebrow: page.frontmatter.eyebrow,
      description: page.frontmatter.description,
      href: pageHref,
      keywords: compactKeywords([page.slug, page.frontmatter.title, page.frontmatter.eyebrow]),
    };
    const sectionItems = page.frontmatter.sections.map<SearchItem>((section) => ({
      id: `section-${page.slug}-${section.id}`,
      kind: "section",
      title: section.label,
      eyebrow: page.frontmatter.title,
      description: page.frontmatter.description,
      href: `${pageHref}#${section.id}`,
      keywords: compactKeywords([page.slug, page.frontmatter.title, section.label, section.id]),
    }));

    return [pageItem, ...sectionItems];
  });
  const skillItems = skillInventory.collections.flatMap<SearchItem>((collection) =>
    collection.skills.map((skill) => ({
      id: `skill-${toAnchorId([skill.collection, skill.name])}`,
      kind: "skill",
      title: skill.name,
      eyebrow: `${skill.collection} / ${skill.version} / ${skill.licenseSpdx}`,
      description: skill.description,
      href: `/docs/skills#${toAnchorId([skill.collection, skill.name])}`,
      keywords: compactKeywords([
        skill.name,
        skill.collection,
        skill.description,
        skill.version,
        skill.versionSource,
        skill.licenseSpdx,
        skill.licenseSource,
        skill.localPath,
        skill.repoUrl,
        skill.sourceUrl,
      ]),
    }))
  );

  return [...docItems, ...skillItems];
});
