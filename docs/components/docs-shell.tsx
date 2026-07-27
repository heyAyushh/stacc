import Link from "next/link";
import type { ReactNode } from "react";
import { markdownToHtml } from "satteri";
import type { DocsPage } from "@/lib/docs";
import { CommandSearch } from "@/components/command-search";
import { DocsPageFill } from "@/components/docs-page-fill";
import { ThemeToggle } from "@/components/theme-toggle";
import { getBinaryInventory } from "@/lib/inventory";

type DocsShellProps = {
  page: DocsPage;
  pages: DocsPage[];
  children?: ReactNode;
};

const staccRepoUrl = "https://github.com/heyAyushh/stacc";
const staccIssueUrl = `${staccRepoUrl}/issues/new`;
const markdownOptions = {
  features: {
    gfm: true,
  },
} as const;

const docsNavGroups = [
  {
    eyebrow: "Start",
    slugs: ["getting-started", "installation"],
  },
  {
    eyebrow: "Use STACC",
    slugs: ["configurations", "skills", "binary-tui", "managed-lifecycle"],
  },
  {
    eyebrow: "Reference",
    slugs: ["troubleshooting", "architecture", "lazycodex"],
  },
];

export async function DocsShell({ page, pages, children }: DocsShellProps) {
  const isSkillDetailPage = page.slug.startsWith("skills/");
  const [renderedMarkdown, binary] = await Promise.all([
    children ? Promise.resolve(null) : Promise.resolve(markdownToHtml(page.body, markdownOptions).html),
    getBinaryInventory(),
  ]);
  const navGroups = docsNavGroups.map((group) => ({
    eyebrow: group.eyebrow,
    links: group.slugs.flatMap((slug) => {
      const docsPage = pages.find((candidate) => candidate.slug === slug);

      return docsPage
        ? [{ label: docsPage.frontmatter.title, href: `/docs/${docsPage.slug}` }]
        : [];
    }),
  }));

  return (
    <main className={isSkillDetailPage ? "docs-shell skill-detail-shell" : "docs-shell"}>
      <header className="docs-header">
        <div className="brand-block">
          <Link href="/" className="docs-logo">
            <span>STACC</span>
          </Link>
          <span className="brand-title">DOCUMENTATION</span>
        </div>

        <div className="search-block">
          <CommandSearch />
        </div>

        <div className="version-block">
          <span>v{binary.crateVersion}</span>
          <ThemeToggle variant="icon" />
          <a href={staccRepoUrl} className="pill-badge" rel="noreferrer" target="_blank">
            GITHUB
          </a>
        </div>
      </header>

      <div className="docs-layout">
        <aside className="left-sidebar">
          {navGroups.map((group) => (
            <div className="nav-group" key={group.eyebrow}>
              <h4 className="nav-eyebrow">{group.eyebrow}</h4>
              <nav className="nav-stack">
                {group.links.map((link) => {
                  const isActivePage = link.href === `/docs/${page.slug}`;

                  return (
                    <Link
                      aria-current={isActivePage ? "page" : undefined}
                      key={link.label}
                      href={link.href}
                      className={isActivePage ? "sidebar-link active" : "sidebar-link"}
                    >
                      {link.label}
                    </Link>
                  );
                })}
              </nav>
            </div>
          ))}
        </aside>

        <article className="docs-content">
          <div className="hero-copy">
            <span className="pill-badge">{page.frontmatter.eyebrow}</span>
            <h1 className="docs-title">{page.frontmatter.title}</h1>
            <p className="docs-lede">{page.frontmatter.description}</p>
          </div>

          <nav className="mobile-section-rail" aria-label="Sections on this page">
            {page.frontmatter.sections.map((section) => (
              <a href={`#${section.id}`} key={section.id}>
                {section.label}
              </a>
            ))}
          </nav>

          {children ? (
            children
          ) : (
            <>
              <div className="docs-markdown" dangerouslySetInnerHTML={{ __html: renderedMarkdown ?? "" }} />
              <div className="docs-sections">
                <DocsPageFill slug={page.slug} />
              </div>
            </>
          )}
        </article>

        <aside className="right-sidebar">
          <h4 className="nav-eyebrow">ON THIS PAGE</h4>
          <nav className="nav-stack">
            {page.frontmatter.sections.map((section) => (
              <a href={`#${section.id}`} key={section.id}>
                {section.label}
              </a>
            ))}
          </nav>
        </aside>
      </div>

      <footer className="docs-footer">
        <div className="footer-brand">
          <div className="footer-brand-row">
            <div className="footer-sigil">S</div>
            <div>
              <p className="footer-title">STACC Documentation</p>
              <p className="footer-subtitle">Install agent configuration safely across supported editors.</p>
            </div>
          </div>
        </div>
        <div className="footer-links">
          <div className="footer-nav">
            <Link href="/docs">Documentation</Link>
            <a href={staccRepoUrl} rel="noreferrer" target="_blank">
              GitHub
            </a>
            <a href={staccIssueUrl} rel="noreferrer" target="_blank">
              Support
            </a>
          </div>
          <span className="copyright">© {binary.latestCommitYear} STACC.FYI</span>
        </div>
      </footer>
    </main>
  );
}
