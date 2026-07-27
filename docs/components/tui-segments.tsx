const tuiSegments = [
  { name: "Install", detail: "Editor, scope, conflict strategy, dry-run and write execution." },
  { name: "Customise", detail: "Category and stack selection for rules, skills, stacks, MCPs, hooks." },
  { name: "Hooks/MCP", detail: "Hook package and MCP server selection before install planning." },
  { name: "Version", detail: "Git status, binary bootstrap, and the full check gate." },
  { name: "Skills", detail: "Metadata sync and skill-origin lockfile maintenance." },
];

export function TuiSegmentGrid() {
  return (
    <div className="tui-grid">
      {tuiSegments.map((segment, index) => (
        <article className="tui-card" key={segment.name}>
          <span className="section-number">#{String(index + 1).padStart(2, "0")}</span>
          <h4>{segment.name}</h4>
          <p>{segment.detail}</p>
        </article>
      ))}
    </div>
  );
}
