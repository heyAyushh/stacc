import Link from "next/link";
import { getBinaryInventory, getConfigInventory, getSkillInventory, type SkillInventoryItem } from "@/lib/inventory";
import { toAnchorId, toSkillHref } from "@/lib/anchors";
import {
  hasDistinctCreatorRepository,
  sourceDisplayLabel,
} from "@/lib/skill-display";

export { AgentDirectorySupport } from "@/components/agent-directory-support";
export { CategoryRail } from "@/components/category-rail";
export { InstallSurfaceMatrix } from "@/components/install-surface-matrix";
export { TuiSegmentGrid } from "@/components/tui-segments";

const skillCollectionDetails: Record<string, { label: string; order: number; description: string }> = {
  skills: {
    label: "Everyday Skills",
    order: 0,
    description: "Broad help for tasks that appear across most projects.",
  },
  stack: {
    label: "Focused Stacks",
    order: 1,
    description: "Deeper expertise grouped by language, framework, platform, or workflow.",
  },
  "command-skills": {
    label: "Workflow Commands",
    order: 2,
    description: "Named workflows to invoke when you want a specific engineering routine.",
  },
  "codex-skills": {
    label: "Codex Skills",
    order: 3,
    description: "Optional workflows designed specifically for Codex.",
  },
  "cursor-plugins": {
    label: "Cursor Plugins",
    order: 4,
    description: "Optional workflows and automations designed specifically for Cursor.",
  },
};

function skillCollectionDetailsFor(name: string) {
  return (
    skillCollectionDetails[name] ?? {
      label: name,
      order: Number.MAX_SAFE_INTEGER,
      description: "Additional agent guidance available through STACC.",
    }
  );
}

function catalogMetadataRows(skill: SkillInventoryItem): Array<{ label: string; value: string | null }> {
  return [
    { label: "License", value: skill.licenseSpdx },
  ];
}

function configInstallHint(groupName: string, itemName: string): string {
  switch (groupName) {
    case "Stacks":
      return `--category stack --stack ${itemName}`;
    case "Hooks":
      return `--category hooks --hook ${itemName}`;
    case "MCP Servers":
      return `--category mcps --mcp-server ${itemName}`;
    case "Cursor Plugin Skills":
    case "Cursor Plugin Hooks":
    case "Cursor Plugin Agents":
      return "--category cursor-plugins";
    case "Codex Skill Imports":
      return "--category codex-skills";
    default:
      return `--category ${groupName.toLowerCase()}`;
  }
}

export async function SkillsOverview() {
  const inventory = await getSkillInventory();
  const everydaySkills = inventory.collections.find((collection) => collection.name === "skills")?.count ?? 0;
  const focusedStackSkills = inventory.collections.find((collection) => collection.name === "stack")?.count ?? 0;
  const workflowPackages = inventory.totalSkills - everydaySkills - focusedStackSkills;

  return (
    <div className="inventory-grid">
      <div className="metric-card metric-card-hero">
        <span className="metric-label">USE EVERYWHERE</span>
        <strong>EVERYDAY</strong>
        <p>{everydaySkills} skills for recurring work across projects.</p>
      </div>
      <div className="metric-card">
        <span className="metric-label">ADD WHEN NEEDED</span>
        <strong>FOCUSED</strong>
        <p>{focusedStackSkills} skills grouped into task-specific stacks.</p>
      </div>
      <div className="metric-card">
        <span className="metric-label">EDITOR-SPECIFIC</span>
        <strong>EXTRAS</strong>
        <p>{workflowPackages} command and editor packages you opt into.</p>
      </div>
    </div>
  );
}

export async function SkillsCatalog() {
  const inventory = await getSkillInventory();
  const collections = inventory.collections.toSorted(
    (left, right) => skillCollectionDetailsFor(left.name).order - skillCollectionDetailsFor(right.name).order
  );

  return (
    <div className="catalog-stack">
      {collections.map((collection) => {
        const collectionDetails = skillCollectionDetailsFor(collection.name);

        return (
          <section className="catalog-section" id={`collection-${collection.name}`} key={collection.name}>
          <div className="catalog-heading">
            <div>
              <span className="metric-label">{collection.count} ITEMS</span>
              <h3>{collectionDetails.label}</h3>
            </div>
            <p>{collectionDetails.description}</p>
          </div>

          <div className="skill-list">
            {collection.skills.map((skill) => (
              <article className="skill-card" id={toAnchorId([skill.collection, skill.name])} key={`${skill.collection}-${skill.name}`}>
                <div className="skill-card-top">
                  <div>
                    <span className="metric-label">{collectionDetails.label}</span>
                    <h4>
                      <Link href={toSkillHref(skill.localPath)}>{skill.name}</Link>
                    </h4>
                  </div>
                  <span className="source-chip">{sourceDisplayLabel(skill)}</span>
                </div>

                <p>{skill.description}</p>

                <dl className="skill-meta">
                  {catalogMetadataRows(skill).map((row) =>
                    row.value ? (
                      <div key={row.label}>
                        <dt>{row.label}</dt>
                        <dd>{row.value}</dd>
                      </div>
                    ) : null
                  )}
                </dl>

                <div className="skill-links">
                  <Link href={toSkillHref(skill.localPath)}>DETAILS</Link>
                  {skill.sourceUrl ? (
                    <a href={skill.sourceUrl} rel="noreferrer" target="_blank">
                      ORIGINAL SOURCE
                    </a>
                  ) : null}
                  {hasDistinctCreatorRepository(skill) ? (
                    <a href={skill.repoUrl ?? ""} rel="noreferrer" target="_blank">
                      CREATOR REPO
                    </a>
                  ) : null}
                </div>
              </article>
            ))}
          </div>
        </section>
        );
      })}
    </div>
  );
}

export async function BinaryOverview() {
  const binary = await getBinaryInventory();

  return (
    <div className="inventory-grid">
      <div className="metric-card metric-card-hero">
        <span className="metric-label">BINARY</span>
        <strong>{binary.binaryName}</strong>
        <p>
          {binary.crateName} v{binary.crateVersion} / {binary.crateLicense}
        </p>
      </div>
      <div className="metric-card">
        <span className="metric-label">CHECK GATE</span>
        <strong>stacc check</strong>
        <p>Full local validation for the binary, installer, and payload.</p>
      </div>
      <div className="metric-card">
        <span className="metric-label">DESCRIPTION</span>
        <strong>CLI/TUI</strong>
        <p>{binary.crateDescription}</p>
      </div>
    </div>
  );
}

export async function ConfigInventoryOverview() {
  const inventory = await getConfigInventory();
  const largestGroups = inventory.groups
    .filter((group) => group.count > 0)
    .toSorted((left, right) => right.count - left.count)
    .slice(0, 3);

  return (
    <div className="inventory-grid">
      <div className="metric-card metric-card-hero">
        <span className="metric-label">AVAILABLE</span>
        <strong>{inventory.totalItems}</strong>
        <p>Rules, skills, tools, and integrations you can choose from.</p>
      </div>
      {largestGroups.map((group) => (
        <div className="metric-card" key={group.name}>
          <span className="metric-label">{group.name}</span>
          <strong>{group.count}</strong>
          <p>{group.description}</p>
        </div>
      ))}
    </div>
  );
}

export async function ConfigInventoryCatalog() {
  const inventory = await getConfigInventory();

  return (
    <div className="catalog-stack">
      {inventory.groups.map((group) => (
        <section className="catalog-section" id={`config-${group.name.toLowerCase().replaceAll(" ", "-")}`} key={group.name}>
          <div className="catalog-heading">
            <div>
              <span className="metric-label">{group.count} ITEMS</span>
              <h3>{group.name}</h3>
            </div>
            <p>{group.description}</p>
          </div>

          <div className="config-list">
            {group.items.map((item) =>
              item.isExternal ? (
                <a className="config-row" href={item.href} key={`${group.name}-${item.path}-${item.name}`} rel="noreferrer" target="_blank">
                  <strong data-label="Name">{item.name}</strong>
                  <code data-label="Install">{configInstallHint(group.name, item.name)}</code>
                  <span className="config-row-action" data-label="Open">{item.hrefLabel}</span>
                </a>
              ) : (
                <Link className="config-row" href={item.href} key={`${group.name}-${item.path}-${item.name}`}>
                  <strong data-label="Name">{item.name}</strong>
                  <code data-label="Install">{configInstallHint(group.name, item.name)}</code>
                  <span className="config-row-action" data-label="Open">{item.hrefLabel}</span>
                </Link>
              )
            )}
          </div>
        </section>
      ))}
    </div>
  );
}
