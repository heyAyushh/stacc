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
