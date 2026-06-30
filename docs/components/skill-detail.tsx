import Link from "next/link";
import type { SkillInventoryItem } from "@/lib/inventory";
import { toSkillHref } from "@/lib/anchors";
import { CopyPanel } from "@/components/copy-panel";
import {
  hasDistinctCreatorRepository,
  originalSourceLabel,
  creatorRepositoryLabel,
  sourceDisplayLabel,
} from "@/lib/skill-display";

type SkillDetailProps = {
  includedSkills?: SkillInventoryItem[];
  siblingSkills?: SkillInventoryItem[];
  skill: SkillInventoryItem;
  skillMarkdown: string;
};

const emptySkills: SkillInventoryItem[] = [];

function metadataRows(skill: SkillInventoryItem): Array<{ label: string; value: string | null }> {
  return [
    { label: "Config Path", value: skill.localPath },
    { label: "License", value: skill.licenseSpdx },
    { label: "License Source", value: skill.licenseSource },
    { label: "Origin Error", value: skill.headError },
  ];
}

function sourceRows(skill: SkillInventoryItem): Array<{ label: string; value: string | null }> {
  return [
    { label: "Docs Path", value: toSkillHref(skill.localPath) },
    { label: "Original Source", value: originalSourceLabel(skill) },
    { label: "Creator Repository", value: hasDistinctCreatorRepository(skill) ? creatorRepositoryLabel(skill) : null },
    { label: "Declared Commit", value: skill.declaredCommit },
    { label: "Head Commit", value: skill.headCommit },
  ];
}

function CopyValue({ label, value }: { label: string; value: string }) {
  return (
    <CopyPanel ariaLabel={`Copy ${label}`} className="copy-value" value={value}>
      <span>{value}</span>
      <span className="copy-value-action" aria-hidden="true">
        <svg viewBox="0 0 24 24" focusable="false">
          <rect x="8" y="8" width="11" height="11" rx="1.5" />
          <path d="M5 15V5h10" />
        </svg>
      </span>
    </CopyPanel>
  );
}

function SkillSourceChip({ skill }: { skill: SkillInventoryItem }) {
  const source = originalSourceLabel(skill);

  return source ? <span className="source-chip">{source}</span> : null;
}

export function SkillDetail({ includedSkills = emptySkills, siblingSkills = emptySkills, skill, skillMarkdown }: SkillDetailProps) {
  const heroChipLabel = sourceDisplayLabel(skill);

  return (
    <div className="skill-detail">
      <section className="skill-detail-hero" id="overview">
        <div>
          <span className="metric-label">{skill.collection}</span>
          <h2>{skill.name}</h2>
        </div>
        <span className="source-chip">{heroChipLabel}</span>
      </section>

      <section className="skill-detail-section" id="metadata">
        <h3>Metadata</h3>
        <dl className="skill-detail-grid">
          {metadataRows(skill).map((row) =>
            row.value ? (
              <div key={row.label}>
                <dt>{row.label}</dt>
                <dd>
                  <CopyValue label={row.label} value={row.value} />
                </dd>
              </div>
            ) : null
          )}
        </dl>
      </section>

      <section className="skill-detail-section" id="description">
        <h3>Description</h3>
        <p>{skill.description}</p>
      </section>

      {includedSkills.length > 0 ? (
        <section className="skill-detail-section stack-contents" id="included-skills">
          <div className="stack-contents-heading">
            <h3>Included Skills</h3>
            <span>{includedSkills.length} skills</span>
          </div>
          <div className="stack-skill-list">
            {includedSkills.map((includedSkill) => (
              <article className="stack-skill-card" key={includedSkill.localPath}>
                <div className="stack-skill-card-top">
                  <div>
                    <span className="metric-label">{includedSkill.localPath.replace(`${skill.localPath}/`, "")}</span>
                    <h4>
                      <Link href={toSkillHref(includedSkill.localPath)}>{includedSkill.name}</Link>
                    </h4>
                  </div>
                  <SkillSourceChip skill={includedSkill} />
                </div>
                <p>{includedSkill.description}</p>
                <dl className="skill-meta">
                  <div>
                    <dt>License</dt>
                    <dd>{includedSkill.licenseSpdx}</dd>
                  </div>
                </dl>
              </article>
            ))}
          </div>
        </section>
      ) : null}

      <section className="skill-detail-section" id="source">
        <h3>Source</h3>
        <dl className="skill-detail-grid">
          {sourceRows(skill).map((row) =>
            row.value ? (
              <div key={row.label}>
                <dt>{row.label}</dt>
                <dd>
                  <CopyValue label={row.label} value={row.value} />
                </dd>
              </div>
            ) : null
          )}
        </dl>
      </section>

      {siblingSkills.length > 0 ? (
        <section className="skill-detail-section stack-contents" id="stack-context">
          <div className="stack-contents-heading">
            <h3>Stack Context</h3>
            <span>{siblingSkills.length} skills in stack</span>
          </div>
          <div className="stack-skill-list">
            {siblingSkills.map((siblingSkill) => {
              const isCurrentSkill = siblingSkill.localPath === skill.localPath;
              const sourceLabel = originalSourceLabel(siblingSkill);

              return (
                <article className={isCurrentSkill ? "stack-skill-card current" : "stack-skill-card"} key={siblingSkill.localPath}>
                  <div className="stack-skill-card-top">
                    <div>
                      <span className="metric-label">{siblingSkill.localPath.split("/").at(-1)}</span>
                      <h4>
                        <Link aria-current={isCurrentSkill ? "page" : undefined} href={toSkillHref(siblingSkill.localPath)}>
                          {siblingSkill.name}
                        </Link>
                      </h4>
                    </div>
                    {isCurrentSkill || sourceLabel ? (
                      <span className={isCurrentSkill ? "source-chip current" : "source-chip"}>{isCurrentSkill ? "Current skill" : sourceLabel}</span>
                    ) : null}
                  </div>
                  <p>{siblingSkill.description}</p>
                </article>
              );
            })}
          </div>
        </section>
      ) : null}

      <section className="skill-detail-section" id="markdown">
        <h3>Markdown</h3>
        <CopyPanel ariaLabel={`Copy ${skill.name} Markdown`} className="code-block copy-panel skill-markdown-panel" mode="overlay" value={skillMarkdown}>
          <div className="code-meta">
            <span>{skill.localPath}/SKILL.md</span>
            <span>Markdown</span>
          </div>
          <pre className="max-w-full whitespace-pre-wrap break-words">
            <code className="block max-w-full break-words [overflow-wrap:anywhere]">{skillMarkdown}</code>
          </pre>
        </CopyPanel>
      </section>

      <section className="skill-detail-section" id="links">
        <h3>Links</h3>
        <div className="skill-links">
          <Link href="/docs/skills">ALL SKILLS</Link>
          {skill.sourceUrl ? (
            <a href={skill.sourceUrl} rel="noreferrer" target="_blank">
              {originalSourceLabel(skill)}
            </a>
          ) : null}
          {hasDistinctCreatorRepository(skill) ? (
            <a href={skill.repoUrl ?? ""} rel="noreferrer" target="_blank">
              {creatorRepositoryLabel(skill)}
            </a>
          ) : null}
        </div>
      </section>
    </div>
  );
}
