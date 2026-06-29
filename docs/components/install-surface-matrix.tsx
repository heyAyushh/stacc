const editorTargets = [
  { tool: "Cursor", global: "~/.cursor/", project: ".cursor/" },
  { tool: "Claude Code", global: "~/.claude/", project: ".claude/" },
  { tool: "Codex", global: "~/.codex/", project: ".codex/" },
  { tool: "OpenCode", global: "~/.config/opencode/", project: ".opencode/" },
  { tool: "AMP Code", global: "~/.config/amp/", project: ".agents/" },
];

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

const agentDirectorySupport = {
  label: ".agents/ supported",
  tool: "AMP Code",
  scope: "Project",
  path: ".agents/",
  categories: ["commands", "rules", "skills", "stack"],
  note: "MCP configuration remains global in ~/.config/amp/settings.json.",
};

export function AgentDirectorySupport() {
  return (
    <aside className="support-callout" aria-label=".agents project support">
      <div>
        <span className="metric-label">{agentDirectorySupport.label}</span>
        <strong>
          {agentDirectorySupport.tool} {agentDirectorySupport.scope.toLowerCase()} installs write to{" "}
          <code>{agentDirectorySupport.path}</code>
        </strong>
      </div>
      <div className="support-meta">
        <span>{agentDirectorySupport.categories.join(" / ")}</span>
        <small>{agentDirectorySupport.note}</small>
      </div>
    </aside>
  );
}

export function InstallSurfaceMatrix() {
  return (
    <div className="matrix-table-wrap">
      <table className="matrix-table">
        <caption>Install target directories by editor and scope</caption>
        <thead>
          <tr>
            <th scope="col">Tool</th>
            <th scope="col">Global</th>
            <th scope="col">Project</th>
          </tr>
        </thead>
        <tbody>
          {editorTargets.map((target) => (
            <tr key={target.tool}>
              <th scope="row" data-label="Tool">
                {target.tool}
              </th>
              <td data-label="Global">
                <code>{target.global}</code>
              </td>
              <td data-label="Project">
                <code>{target.project}</code>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function CategoryRail() {
  return (
    <div className="category-rail">
      {installCategories.map((category) => (
        <span key={category}>{category}</span>
      ))}
    </div>
  );
}
