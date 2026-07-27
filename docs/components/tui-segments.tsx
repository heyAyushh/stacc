const tuiSegments = [
  { name: "Install", detail: "Choose an editor, project or global scope, conflict behavior, and whether to preview or apply." },
  { name: "Customise", detail: "Choose the rules, skills, stacks, commands, and editor extras that belong in this setup." },
  { name: "Hooks/MCP", detail: "Add supported editor automations and connections to external tools." },
  { name: "Version", detail: "See the installed version and preview an upgrade before running it." },
  { name: "Skills", detail: "Review the available skill library and its source information." },
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
