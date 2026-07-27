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

export function CategoryRail() {
  return (
    <div className="category-rail">
      {installCategories.map((category) => (
        <span key={category}>{category}</span>
      ))}
    </div>
  );
}
