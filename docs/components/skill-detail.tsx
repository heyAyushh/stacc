import Link from "next/link";
import type { SkillInventoryItem } from "@/lib/inventory";
import {
  creatorRepositoryLabel,
  displaySkillVersion,
  hasDistinctCreatorRepository,
  originalSourceLabel,
} from "@/lib/skill-display";

type SkillDetailProps = {
  skill: SkillInventoryItem;
};

function metadataRows(skill: SkillInventoryItem): Array<{ label: string; value: string | null }> {
  return [
    { label: "Collection", value: skill.collection },
    { label: "Version", value: displaySkillVersion(skill.version) },
    { label: "Original Source", value: originalSourceLabel(skill) },
    { label: "License", value: skill.licenseSpdx },
    { label: "License Source", value: skill.licenseSource },
    { label: "Creator Repository", value: creatorRepositoryLabel(skill) },
    { label: "Origin Error", value: skill.headError },
  ];
}

export function SkillDetail({ skill }: SkillDetailProps) {
  return (
    <div className="skill-detail">
      <section className="skill-detail-hero" id="overview">
        <div>
          <span className="metric-label">{skill.collection}</span>
          <h2>{skill.name}</h2>
        </div>
        <span className="version-chip">{displaySkillVersion(skill.version)}</span>
      </section>

      <section className="skill-detail-section" id="metadata">
        <h3>Metadata</h3>
        <dl className="skill-detail-grid">
          {metadataRows(skill).map((row) =>
            row.value ? (
              <div key={row.label}>
                <dt>{row.label}</dt>
                <dd>{row.value}</dd>
              </div>
            ) : null
          )}
        </dl>
      </section>

      <section className="skill-detail-section" id="description">
        <h3>Description</h3>
        <p>{skill.description}</p>
      </section>

      <section className="skill-detail-section" id="links">
        <h3>Links</h3>
        <div className="skill-links">
          <Link href="/docs/skills">ALL SKILLS</Link>
          {skill.sourceUrl ? <a href={skill.sourceUrl}>ORIGINAL SOURCE</a> : null}
          {hasDistinctCreatorRepository(skill) ? <a href={skill.repoUrl ?? ""}>CREATOR REPOSITORY</a> : null}
        </div>
      </section>
    </div>
  );
}
