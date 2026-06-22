import Link from "next/link";
import { compileMDX } from "next-mdx-remote/rsc";
import type { DocsPage } from "@/lib/docs";
import { getSearchItems } from "@/lib/search";
import { CommandSearch } from "@/components/command-search";
import { DocsPageFill } from "@/components/docs-page-fill";
import { ThemeToggle } from "@/components/theme-toggle";

type DocsShellProps = {
  page: DocsPage;
  pages: DocsPage[];
};

const staticNavGroups = [
  {
    eyebrow: "00 / INTRODUCTION",
    links: [],
  },
  {
    eyebrow: "01 / CORE_SYSTEM",
    links: [
      { label: "Agent Personas", href: "#" },
      { label: "Global Rules", href: "#" },
      { label: "Active Hooks", href: "#" },
    ],
  },
  {
    eyebrow: "02 / INTERFACE",
    links: [
      { label: "CLI Reference", href: "#" },
      { label: "API Specs", href: "#" },
      { label: "Plugins", href: "#" },
    ],
  },
];

export async function DocsShell({ page, pages }: DocsShellProps) {
  const [{ content }, searchItems] = await Promise.all([
    compileMDX({
      source: page.body,
      options: {
        parseFrontmatter: false,
      },
    }),
    getSearchItems(),
  ]);
  const navGroups = [
    {
      eyebrow: "00 / INTRODUCTION",
      links: [
        ...pages.map((docsPage) => ({
          label: docsPage.frontmatter.title,
          href: `/docs/${docsPage.slug}`,
        })),
        ...staticNavGroups[0].links,
      ],
    },
    ...staticNavGroups.slice(1),
  ];

  return (
    <main className="docs-shell">
      <header className="docs-header">
        <div className="brand-block">
          <Link href="/" className="docs-logo">
            <span>STACC</span>
          </Link>
          <span className="brand-title">DOCUMENTATION</span>
        </div>

        <div className="search-block">
          <CommandSearch items={searchItems} />
        </div>

        <div className="version-block">
          <span>v1.0.4</span>
          <ThemeToggle />
          <a href="#" className="pill-badge">
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
                {group.links.map((link) => (
                  <Link
                    key={link.label}
                    href={link.href}
                    className={link.label === page.frontmatter.title ? "sidebar-link active" : "sidebar-link"}
                  >
                    {link.label}
                  </Link>
                ))}
              </nav>
            </div>
          ))}

          <div className="status-card">
            <div className="status-card-inner">
              <span className="status-label">SYSTEM STATUS</span>
              <div className="status-row">
                <div className="status-dot" />
                <span className="status-text">ALL NODES ONLINE</span>
              </div>
            </div>
          </div>
        </aside>

        <article className="docs-content">
          <div className="hero-copy">
            <span className="pill-badge">{page.frontmatter.eyebrow}</span>
            <h1 className="docs-title">{page.frontmatter.title}</h1>
            <p className="docs-lede">{page.frontmatter.description}</p>
          </div>

          <div className="docs-markdown">{content}</div>
          <div className="docs-sections">
            <DocsPageFill slug={page.slug} />
          </div>
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

          <div className="sales-card">
            <p>Need custom infrastructure for your enterprise?</p>
            <button type="button">CONTACT SALES</button>
          </div>
        </aside>
      </div>

      <footer className="docs-footer">
        <div className="footer-brand">
          <div className="footer-brand-row">
            <div className="footer-sigil">S</div>
            <div>
              <p className="footer-title">STACC Documentation</p>
              <p className="footer-subtitle">Built with stability and velocity.</p>
            </div>
          </div>
        </div>
        <div className="footer-links">
          <div className="footer-nav">
            <a href="#">Privacy</a>
            <a href="#">Terms</a>
            <a href="#">Support</a>
          </div>
          <span className="copyright">© 2024 STACC.DEV</span>
        </div>
      </footer>
    </main>
  );
}
