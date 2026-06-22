import { getBinaryInventory, getConfigInventory, getSkillInventory } from "@/lib/inventory";
import { toAnchorId } from "@/lib/anchors";

const editorTargets = [
  { tool: "Cursor", global: "~/.cursor/", project: ".cursor/" },
  { tool: "Claude Code", global: "~/.claude/", project: ".claude/" },
  { tool: "Codex", global: "~/.codex/", project: ".codex/" },
  { tool: "OpenCode", global: "~/.config/opencode/", project: ".opencode/" },
  { tool: "AMP Code", global: "~/.config/amp/", project: ".agents/" },
];

const installCategories = [
  "commands",
  "rules",
  "agents",
  "skills",
  "stack",
  "hooks",
  "mcps",
  "cursor-plugins",
  "codex-skills",
];

const tuiSegments = [
  { name: "Install", detail: "Editor, scope, conflict strategy, dry-run and write execution." },
  { name: "Customise", detail: "Category and stack selection for rules, skills, stacks, MCPs, hooks." },
  { name: "Hooks/MCP", detail: "Hook package and MCP server selection before install planning." },
  { name: "Version", detail: "Git status, binary bootstrap, and the full check gate." },
  { name: "Skills", detail: "Metadata sync and skill-origin lockfile maintenance." },
];

export async function SkillsOverview() {
  const inventory = await getSkillInventory();

  return (
    <div className="inventory-grid">
      <div className="metric-card metric-card-hero">
        <span className="metric-label">TOTAL SKILLS</span>
        <strong>{inventory.totalSkills}</strong>
        <p>Loaded from {inventory.sourceRepo} metadata lockfile.</p>
      </div>
      <div className="metric-card">
        <span className="metric-label">COLLECTIONS</span>
        <strong>{inventory.collections.length}</strong>
        <p>{inventory.collections.map((collection) => collection.name).join(" / ")}</p>
      </div>
      <div className="metric-card">
        <span className="metric-label">GENERATED</span>
        <strong>{inventory.generatedAt.slice(0, 10)}</strong>
        <p>{inventory.generatedAt.slice(11, 19)} UTC</p>
      </div>
    </div>
  );
}

export async function SkillsCatalog() {
  const inventory = await getSkillInventory();

  return (
    <div className="catalog-stack">
      {inventory.collections.map((collection) => (
        <section className="catalog-section" id={`collection-${collection.name}`} key={collection.name}>
          <div className="catalog-heading">
            <div>
              <span className="metric-label">{collection.count} ITEMS</span>
              <h3>{collection.name}</h3>
            </div>
            <p>{collection.licenses.join(" / ")}</p>
          </div>

          <div className="skill-list">
            {collection.skills.map((skill) => (
              <article className="skill-card" id={toAnchorId([skill.collection, skill.name])} key={`${skill.collection}-${skill.name}`}>
                <div className="skill-card-top">
                  <div>
                    <span className="metric-label">{skill.collection}</span>
                    <h4>{skill.name}</h4>
                  </div>
                  <span className="version-chip">{skill.version}</span>
                </div>

                <p>{skill.description}</p>

                <dl className="skill-meta">
                  <div>
                    <dt>License</dt>
                    <dd>{skill.licenseSpdx}</dd>
                  </div>
                  <div>
                    <dt>License Source</dt>
                    <dd>{skill.licenseSource}</dd>
                  </div>
                  <div>
                    <dt>Version Source</dt>
                    <dd>{skill.versionSource}</dd>
                  </div>
                  <div>
                    <dt>Path</dt>
                    <dd>{skill.localPath}</dd>
                  </div>
                  {skill.declaredCommit ? (
                    <div>
                      <dt>Declared Commit</dt>
                      <dd>{skill.declaredCommit}</dd>
                    </div>
                  ) : null}
                  {skill.headCommit ? (
                    <div>
                      <dt>Origin Head</dt>
                      <dd>{skill.headCommit.slice(0, 12)}</dd>
                    </div>
                  ) : null}
                  {skill.licenseFile ? (
                    <div>
                      <dt>License File</dt>
                      <dd>{skill.licenseFile}</dd>
                    </div>
                  ) : null}
                </dl>

                <div className="skill-links">
                  {skill.sourceUrl ? <a href={skill.sourceUrl}>SOURCE</a> : <span>LOCAL SOURCE</span>}
                  {skill.repoUrl ? <a href={skill.repoUrl}>REPOSITORY</a> : null}
                  {skill.headError ? <span>ORIGIN ERROR: {skill.headError}</span> : null}
                </div>
              </article>
            ))}
          </div>
        </section>
      ))}
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
        <span className="metric-label">DOCS APP</span>
        <strong>v{binary.docsVersion}</strong>
        <p>Root workspace v{binary.rootVersion}</p>
      </div>
      <div className="metric-card">
        <span className="metric-label">DESCRIPTION</span>
        <strong>CLI/TUI</strong>
        <p>{binary.crateDescription}</p>
      </div>
    </div>
  );
}

export async function RuntimeDependencyGrid() {
  const binary = await getBinaryInventory();

  return (
    <div className="dependency-grid">
      {binary.docsDependencies.map((dependency) => (
        <div className="dependency-row" key={`${dependency.scope}-${dependency.name}`}>
          <span>{dependency.name}</span>
          <code>{dependency.version}</code>
          <em>{dependency.scope}</em>
        </div>
      ))}
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
        <span className="metric-label">CONFIG ITEMS</span>
        <strong>{inventory.totalItems}</strong>
        <p>Installable assets under configs/.</p>
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
            {group.items.map((item) => (
              <div className="config-row" key={`${group.name}-${item.path}-${item.name}`}>
                <strong>{item.name}</strong>
                <code>{item.path}</code>
                {item.detail ? <span>{item.detail}</span> : null}
              </div>
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}

export function InstallSurfaceMatrix() {
  return (
    <div className="matrix-table">
      <div className="matrix-row matrix-head">
        <span>Tool</span>
        <span>Global</span>
        <span>Project</span>
      </div>
      {editorTargets.map((target) => (
        <div className="matrix-row" key={target.tool}>
          <strong>{target.tool}</strong>
          <code>{target.global}</code>
          <code>{target.project}</code>
        </div>
      ))}
    </div>
  );
}

export function CategoryRail() {
  return (
    <div className="category-rail">
      {installCategories.map((category) => (
        <span key={category}>{category}</span>
      ))}
    </div>
  );
}

export function TuiSegmentGrid() {
  return (
    <div className="tui-grid">
      {tuiSegments.map((segment, index) => (
        <article className="tui-card" key={segment.name}>
          <span className="section-number">#{String(index + 1).padStart(2, "0")}</span>
          <h4>{segment.name}</h4>
          <p>{segment.detail}</p>
        </article>
      ))}
    </div>
  );
}
