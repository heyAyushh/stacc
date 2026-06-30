const unsafeAnchorCharacters = /[^a-z0-9]+/g;
const edgeDashes = /(^-|-$)/g;

export function toAnchorId(parts: string[]): string {
  const anchor = parts
    .join("-")
    .toLowerCase()
    .replace(unsafeAnchorCharacters, "-")
    .replace(edgeDashes, "");

  return anchor.length > 0 ? anchor : "item";
}

export function toSkillSlug(collection: string, name: string): string {
  return toAnchorId([collection, name]);
}

export function toSkillPath(localPath: string): string[] {
  return localPath.split("/").reduce<string[]>((segments, segment) => {
    const trimmedSegment = segment.trim();

    if (trimmedSegment) {
      segments.push(trimmedSegment);
    }

    return segments;
  }, []);
}

export function toSkillHref(localPath: string): string {
  return `/docs/skills/${toSkillPath(localPath).map(encodeURIComponent).join("/")}`;
}
