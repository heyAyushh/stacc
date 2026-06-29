import Link from "next/link";
import { getBinaryInventory, getConfigInventory, getSkillInventory, type SkillInventoryItem } from "@/lib/inventory";
import { toAnchorId, toSkillHref } from "@/lib/anchors";
import {
  displaySkillVersion,
  hasDistinctCreatorRepository,
  originalSourceLabel,
} from "@/lib/skill-display";

export { AgentDirectorySupport, CategoryRail, InstallSurfaceMatrix } from "@/components/install-surface-matrix";
export { TuiSegmentGrid } from "@/components/tui-segments";

function catalogMetadataRows(skill: SkillInventoryItem): Array<{ label: string; value: string | null }> {
  return [
    { label: "License", value: skill.licenseSpdx },
    { label: "License Source", value: skill.licenseSource },
    { label: "Original Source", value: originalSourceLabel(skill) },
  ];
}

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
                    <h4>
                      <Link href={toSkillHref(skill.collection, skill.name)}>{skill.name}</Link>
                    </h4>
                  </div>
                  <span className="version-chip">{displaySkillVersion(skill.version)}</span>
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
                  <Link href={toSkillHref(skill.collection, skill.name)}>DETAILS</Link>
                  {skill.sourceUrl ? <a href={skill.sourceUrl}>ORIGINAL SOURCE</a> : null}
                  {hasDistinctCreatorRepository(skill) ? <a href={skill.repoUrl ?? ""}>CREATOR REPO</a> : null}
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
    <section className="dependency-ledger" aria-label="Docs runtime dependencies">
      <div className="dependency-ledger-head" aria-hidden="true">
        <span>Package</span>
        <span>Version</span>
        <span>Scope</span>
      </div>
      <ul className="dependency-list">
        {binary.docsDependencies.map((dependency) => (
          <li className="dependency-item" key={`${dependency.scope}-${dependency.name}`}>
            <strong>
              <a href={dependency.packageUrl} rel="noreferrer" target="_blank">
                {dependency.name}
              </a>
            </strong>
            <code>{dependency.version}</code>
            <span className="dependency-scope">{dependency.scope}</span>
          </li>
        ))}
      </ul>
    </section>
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
                <strong data-label="Name">{item.name}</strong>
                <code data-label="Path">{item.path}</code>
                <span data-label="Detail">{item.detail ?? "—"}</span>
              </div>
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}
