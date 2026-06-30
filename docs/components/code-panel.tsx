import { CopyPanel } from "@/components/copy-panel";
import { highlightCode, resolveCodeLanguage } from "@/lib/highlight";

type CodePanelProps = {
  label: string;
  action: string;
  snippet: string;
};

export async function CodePanel({ label, action, snippet }: CodePanelProps) {
  const language = resolveCodeLanguage(label, action, snippet);
  const highlightedCode = await highlightCode(snippet, language);

  return (
    <CopyPanel ariaLabel={`Copy ${label} command`} className="code-block copy-panel" mode="overlay" value={snippet}>
      <div className="code-meta">
        <span>{label}</span>
        <span className="copy-status" data-label={action}>
          {action}
        </span>
      </div>
      <div className="code-highlight" dangerouslySetInnerHTML={{ __html: highlightedCode }} />
    </CopyPanel>
  );
}
