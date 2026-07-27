import Link from "next/link";
import type { SkillInventoryItem } from "@/lib/inventory";
import { toSkillHref } from "@/lib/anchors";
import { CopyPanel } from "@/components/copy-panel";
import {
  hasDistinctCreatorRepository,
  originalSourceLabel,
  creatorRepositoryLabel,
} from "@/lib/skill-display";

type SkillDetailProps = {
  includedSkills?: SkillInventoryItem[];
  siblingSkills?: SkillInventoryItem[];
  skill: SkillInventoryItem;
  skillMarkdown: string;
};

const emptySkills: SkillInventoryItem[] = [];

function sourceRows(skill: SkillInventoryItem): Array<{ label: string; value: string | null }> {
  return [
    { label: "License", value: skill.licenseSpdx },
    { label: "Original Source", value: originalSourceLabel(skill) },
    { label: "Creator Repository", value: hasDistinctCreatorRepository(skill) ? creatorRepositoryLabel(skill) : null },
    { label: "Imported Revision", value: skill.declaredCommit },
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

function SkillSection({
  children,
  className,
  id,
  number,
  title,
}: {
  children: React.ReactNode;
  className?: string;
  id: string;
  number: string;
  title: string;
}) {
  return (
    <section className={className ? `skill-detail-section ${className}` : "skill-detail-section"} id={id}>
      <div className="section-heading">
        <span className="section-number">#{number}</span>
        <h3>{title}</h3>
      </div>
      {children}
    </section>
  );
}

export function SkillDetail({ includedSkills = emptySkills, siblingSkills = emptySkills, skill, skillMarkdown }: SkillDetailProps) {
  let sectionCounter = 1;
  const nextSectionNumber = () => String(sectionCounter++).padStart(2, "0");

  return (
    <div className="skill-detail">
      <SkillSection id="description" number={nextSectionNumber()} title="What it helps with">
        <p>{skill.description}</p>
      </SkillSection>

      {includedSkills.length > 0 ? (
        <SkillSection className="stack-contents" id="included-skills" number={nextSectionNumber()} title="What this stack includes">
          <p className="section-summary">{includedSkills.length} focused skills are available when a matching task needs them.</p>
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
        </SkillSection>
      ) : null}

      <SkillSection id="source" number={nextSectionNumber()} title="Source and license">
        <p className="section-summary">
          STACC keeps the original author, license, and imported revision visible so you can review where this guidance came from.
        </p>
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
      </SkillSection>

      {siblingSkills.length > 0 ? (
        <SkillSection className="stack-contents" id="stack-context" number={nextSectionNumber()} title="Related skills in this stack">
          <p className="section-summary">{siblingSkills.length} skills cover nearby tasks without loading all of their instructions at once.</p>
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
        </SkillSection>
      ) : null}

      <SkillSection id="instructions" number={nextSectionNumber()} title="Read the instructions">
        <p className="section-summary">This is the complete guidance an agent can load when the skill matches a task.</p>
        <CopyPanel ariaLabel={`Copy ${skill.name} instructions`} className="code-block copy-panel skill-markdown-panel" mode="overlay" value={skillMarkdown}>
          <div className="code-meta">
            <span>{skill.localPath}/SKILL.md</span>
            <span>Skill instructions</span>
          </div>
          <pre className="max-w-full whitespace-pre-wrap break-words">
            <code className="block max-w-full break-words [overflow-wrap:anywhere]">{skillMarkdown}</code>
          </pre>
        </CopyPanel>
      </SkillSection>

      <SkillSection id="links" number={nextSectionNumber()} title="Explore more">
        <div className="skill-links">
          <Link href="/docs/skills">BROWSE ALL SKILLS</Link>
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
      </SkillSection>
    </div>
  );
}
