import type { ReactNode } from "react";

type DocStepProps = {
  id: string;
  number: string;
  title: string;
  children: ReactNode;
};

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
