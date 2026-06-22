import type { ReactNode } from "react";
import { highlightCode, resolveCodeLanguage } from "@/lib/highlight";

type DocStepProps = {
  id: string;
  number: string;
  title: string;
  children: ReactNode;
};

type CodePanelProps = {
  label: string;
  action: string;
  snippet?: string;
  children?: ReactNode;
};

type LaunchPanelProps = {
  label: string;
  command: string;
};

function sourceFromCodeContent(snippet: string | undefined, children: ReactNode): string {
  if (typeof children === "string" || typeof children === "number") {
    return String(children);
  }

  return snippet ?? "";
}

export function DocStep({ id, number, title, children }: DocStepProps) {
  return (
    <section className="docs-section" id={id}>
      <div className="section-heading">
        <span className="section-number">#{number}</span>
        <h2>{title}</h2>
      </div>
      <div className="section-body">{children}</div>
    </section>
  );
}

export async function CodePanel({ label, action, snippet, children }: CodePanelProps) {
  const source = sourceFromCodeContent(snippet, children);
  const language = resolveCodeLanguage(label, action, source);
  const highlightedCode = await highlightCode(source, language);

  return (
    <div className="code-block" data-language={language}>
      <div className="code-meta">
        <span>{label}</span>
        <span>{action}</span>
      </div>
      <div className="code-highlight" dangerouslySetInnerHTML={{ __html: highlightedCode }} />
    </div>
  );
}

export function LaunchPanel({ label, command }: LaunchPanelProps) {
  return (
    <button className="launch-button" type="button">
      <span className="launch-row">
        <span className="launch-label">{label}</span>
        <span className="launch-command">{command}</span>
      </span>
    </button>
  );
}
