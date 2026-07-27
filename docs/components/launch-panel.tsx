import { CopyPanel } from "@/components/copy-panel";

type LaunchPanelProps = {
  label: string;
  command: string;
};

export function LaunchPanel({ label, command }: LaunchPanelProps) {
  return (
    <CopyPanel ariaLabel={`Copy command: ${command}`} className="launch-button" value={command}>
      <span className="launch-row">
        <span className="launch-label">{label}</span>
        <span className="launch-command">{command}</span>
      </span>
    </CopyPanel>
  );
}
