import type { SkillInventoryItem } from "@/lib/inventory";

const gitVersionPrefix = "git:";
const localGitVersionPrefix = "local-git:";
const githubHost = "github.com";

type SkillSourceFields = Pick<SkillInventoryItem, "sourceUrl" | "repoUrl">;

function parseHttpUrl(value: string | null): URL | null {
  if (!value) {
    return null;
  }

  try {
    return new URL(value);
  } catch (error: unknown) {
    if (!(error instanceof TypeError)) {
      throw error;
    }

    return null;
  }
}

function githubOwnerRepo(url: URL): string | null {
  if (url.hostname !== githubHost) {
    return null;
  }

  const [owner, repo] = url.pathname.split("/").filter(Boolean);
  if (!owner || !repo) {
    return null;
  }

  return `${owner}/${repo}`;
}

function sourceLabel(value: string | null): string | null {
  const url = parseHttpUrl(value);
  if (!url) {
    return value;
  }

  return githubOwnerRepo(url) ?? url.hostname.replace(/^www\./, "");
}

export function displaySkillVersion(version: string): string {
  if (version.startsWith(localGitVersionPrefix)) {
    return "Local bundle";
  }

  if (version.startsWith(gitVersionPrefix)) {
    return "Pinned git";
  }

  return version;
}

export function originalSourceLabel(skill: SkillSourceFields): string | null {
  return sourceLabel(skill.sourceUrl ?? skill.repoUrl);
}

export function creatorRepositoryLabel(skill: Pick<SkillInventoryItem, "repoUrl">): string | null {
  return sourceLabel(skill.repoUrl);
}

export function hasDistinctCreatorRepository(skill: SkillSourceFields): boolean {
  return Boolean(skill.repoUrl && skill.repoUrl !== skill.sourceUrl && creatorRepositoryLabel(skill) !== originalSourceLabel(skill));
}

export function sourceDisplayLabel(skill: SkillSourceFields & Pick<SkillInventoryItem, "version">): string {
  return originalSourceLabel(skill) ?? creatorRepositoryLabel(skill) ?? displaySkillVersion(skill.version);
}
