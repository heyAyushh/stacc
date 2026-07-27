const editorTargets = [
  { tool: "Cursor", global: "~/.cursor/", project: ".cursor/" },
  { tool: "Claude Code", global: "~/.claude/", project: ".claude/" },
  { tool: "Codex", global: "~/.codex/", project: ".codex/" },
  { tool: "OpenCode", global: "~/.config/opencode/", project: ".opencode/" },
  { tool: "AMP Code", global: "~/.config/amp/", project: ".agents/" },
];

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
