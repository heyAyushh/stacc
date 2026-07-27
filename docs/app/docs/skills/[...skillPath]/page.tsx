import { notFound } from "next/navigation";
import { DocsShell } from "@/components/docs-shell";
import { SkillDetail } from "@/components/skill-detail";
import { getAllDocsPages } from "@/lib/docs";
import { getAllSkillItems, getSkillByPath, getSkillMarkdown, type SkillInventoryItem } from "@/lib/inventory";
import { toSkillPath } from "@/lib/anchors";

type SkillPageProps = {
  params: Promise<{
    skillPath: string[];
  }>;
};

const baseSkillSections = [
  { id: "description", label: "What it helps with" },
  { id: "source", label: "Source and license" },
  { id: "instructions", label: "Instructions" },
  { id: "links", label: "Explore more" },
];

const stackSkillSections = [
  { id: "description", label: "What it helps with" },
  { id: "included-skills", label: "Included skills" },
  { id: "source", label: "Source and license" },
  { id: "instructions", label: "Instructions" },
  { id: "links", label: "Explore more" },
];

const skillCollectionLabels: Record<string, string> = {
  skills: "Everyday skill",
  stack: "Focused skill",
  "command-skills": "Workflow command",
  "codex-skills": "Codex skill",
  "cursor-plugins": "Cursor plugin",
};

function includedStackSkills(stack: SkillInventoryItem, skills: SkillInventoryItem[]): SkillInventoryItem[] {
  const stackPath = `${stack.localPath}/`;

  if (stack.collection !== "stack") {
    return [];
  }

  return skills
    .filter((skill) => skill.localPath.startsWith(stackPath))
    .toSorted((left, right) => left.name.localeCompare(right.name));
}

function siblingStackSkills(skill: SkillInventoryItem, skills: SkillInventoryItem[]): SkillInventoryItem[] {
  const [configsSegment, stacksSegment, stackName] = skill.localPath.split("/");

  if (configsSegment !== "configs" || stacksSegment !== "stacks" || !stackName || skill.localPath === `configs/stacks/${stackName}`) {
    return [];
  }

  return skills
    .filter((candidate) => candidate.localPath.startsWith(`configs/stacks/${stackName}/`))
    .toSorted((left, right) => left.name.localeCompare(right.name));
}

export async function generateStaticParams() {
  const skills = await getAllSkillItems();

  return skills.map((skill) => ({
    skillPath: toSkillPath(skill.localPath),
  }));
}

export async function generateMetadata({ params }: SkillPageProps) {
  const { skillPath } = await params;
  const skill = await getSkillByPath(skillPath.join("/"));

  if (!skill) {
    return {
      title: "Skill | STACC Documentation",
    };
  }

  return {
    title: `${skill.name} | STACC Skill`,
    description: skill.description,
  };
}

export default async function SkillPage({ params }: SkillPageProps) {
  const { skillPath } = await params;
  const normalizedSkillPath = skillPath.join("/");
  const [pages, skills, skill] = await Promise.all([getAllDocsPages(), getAllSkillItems(), getSkillByPath(normalizedSkillPath)]);

  if (!skill) {
    notFound();
  }

  const stackSkills = includedStackSkills(skill, skills);
  const siblingSkills = stackSkills.length > 0 ? [] : siblingStackSkills(skill, skills);
  const markdown = await getSkillMarkdown(skill);
  const sections = stackSkills.length > 0
    ? stackSkillSections
    : siblingSkills.length > 0
      ? [
          ...baseSkillSections.slice(0, 2),
          { id: "stack-context", label: "Related skills" },
          ...baseSkillSections.slice(2),
        ]
      : baseSkillSections;

  return (
    <div className="docs-route">
      <DocsShell
        page={{
          slug: `skills/${normalizedSkillPath}`,
          frontmatter: {
            title: skill.name,
            eyebrow: skillCollectionLabels[skill.collection] ?? "Skill",
            description: skill.description,
            order: 999,
            sections,
          },
          body: "",
        }}
        pages={pages}
      >
        <SkillDetail includedSkills={stackSkills} siblingSkills={siblingSkills} skill={skill} skillMarkdown={markdown} />
      </DocsShell>
    </div>
  );
}
