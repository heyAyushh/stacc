import { notFound } from "next/navigation";
import { DocsShell } from "@/components/docs-shell";
import { SkillDetail } from "@/components/skill-detail";
import { getAllDocsPages } from "@/lib/docs";
import { getAllSkillItems, getSkillBySlug } from "@/lib/inventory";
import { toSkillSlug } from "@/lib/anchors";

type SkillPageProps = {
  params: Promise<{
    skillSlug: string;
  }>;
};

const skillSections = [
  { id: "overview", label: "Overview" },
  { id: "metadata", label: "Metadata" },
  { id: "description", label: "Description" },
  { id: "links", label: "Links" },
];

export async function generateStaticParams() {
  const skills = await getAllSkillItems();

  return skills.map((skill) => ({
    skillSlug: toSkillSlug(skill.collection, skill.name),
  }));
}

export async function generateMetadata({ params }: SkillPageProps) {
  const { skillSlug } = await params;
  const skill = await getSkillBySlug(skillSlug);

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
  const { skillSlug } = await params;
  const [pages, skill] = await Promise.all([getAllDocsPages(), getSkillBySlug(skillSlug)]);

  if (!skill) {
    notFound();
  }

  return (
    <div className="docs-route">
      <DocsShell
        page={{
          slug: `skills/${skillSlug}`,
          frontmatter: {
            title: skill.name,
            eyebrow: `${skill.collection} / ${skill.licenseSpdx}`,
            description: skill.description,
            order: 999,
            sections: skillSections,
          },
          body: "",
        }}
        pages={pages}
      >
        <SkillDetail skill={skill} />
      </DocsShell>
    </div>
  );
}
