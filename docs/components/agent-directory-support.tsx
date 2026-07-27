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
