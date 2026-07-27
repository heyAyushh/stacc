import Link from "next/link";
import { WaveCanvas } from "@/components/wave-canvas";
import { ThemeToggle } from "@/components/theme-toggle";
import { InstallCopyCard } from "@/components/install-copy-card";
import { CommandSearch } from "@/components/command-search";
import { getBinaryInventory } from "@/lib/inventory";

const staccRepoUrl = "https://github.com/heyAyushh/stacc";
const staccIssueUrl = `${staccRepoUrl}/issues/new`;

const features = [
  {
    number: "01",
    titleLines: ["See", "The Plan"],
    body: "Preview every file, merge, and conflict before STACC touches your setup.",
    href: "/docs/getting-started#preview",
    panelClassName: "border-b md:border-b-0 md:border-r border-black",
    actionKind: "pill",
  },
  {
    number: "02",
    titleLines: ["Pick", "Your Stack"],
    body: "Install the skills this project needs. Keep everything else out of your agent’s context.",
    href: "/docs/skills#choose",
    panelClassName: "border-b md:border-b-0 md:border-r border-black",
    actionKind: "arrow",
  },
  {
    number: "03",
    titleLines: ["Stay", "In Control"],
    body: "Update or remove exactly what STACC installed. Your own files stay yours.",
    href: "/docs/managed-lifecycle#ownership",
    panelClassName: "bg-white",
    actionKind: "status",
  },
] as const;

const arrowAction = (
  <div className="w-8 h-8 border border-black rounded-full flex items-center justify-center group-hover:border-white">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M5 12h14M12 5l7 7-7 7" />
    </svg>
  </div>
);

const pillAction = <div className="pill-badge group-hover:border-white group-hover:text-white">/dry-run</div>;

const statusAction = (
  <div className="mt-8 w-full border-t border-black pt-4 group-hover:border-white">
    <div className="flex justify-between items-center font-mono text-xs">
      <span>YOUR FILES</span>
      <span>STAY YOURS</span>
    </div>
  </div>
);

function getFeatureAction(actionKind: (typeof features)[number]["actionKind"]) {
  if (actionKind === "pill") {
    return pillAction;
  }

  if (actionKind === "arrow") {
    return arrowAction;
  }

  return statusAction;
}

export async function LandingPage() {
  const binary = await getBinaryInventory();
  const displayVersion = `v.${binary.crateVersion}`;

  return (
    <main className="screen min-h-screen overflow-x-hidden" aria-label="STACC agent setup">
      <section className="variant-stage" aria-label="STACC product overview">
        <div className="stage-theme-toggle">
          <CommandSearch variant="compact" />
          <ThemeToggle />
        </div>
        <article className="canvas">
          <div className="board">
            <section className="board-top" aria-label="STACC product header">
              <div className="header-cell">
                <div className="logo-pill">
                  <small>OSS</small>STACC
                </div>
                <div className="system-ready">STACK READY</div>
              </div>
              <div className="header-cell">
                <h1 className="suite-title">
                  AGENT
                  <br />
                  STACK
                </h1>
                <div className="suite-kicker">• SKILLS • RULES • MCP •</div>
                <div className="spinner" aria-hidden="true" />
                <div className="mobile-right-copy">
                  One setup
                  <br />
                  Any agent
                </div>
              </div>
              <div className="header-cell">
                <div className="right-copy">
                  One setup
                  <br />
                  Any agent
                </div>
                <div className="link-stack">
                  <Link className="pill-link" href="/docs">
                    DOCS
                  </Link>
                  <CommandSearch variant="compact" />
                  <ThemeToggle />
                </div>
                <div className="curl-hint">curl -fsSL</div>
              </div>
            </section>

            <section className="wave-panel" aria-label="Set up any coding agent">
              <WaveCanvas />
              <div className="mega" aria-label="Set up any coding agent">
                <span className="mega-line">SET UP</span>
                <span className="mega-line depth">ANY AGENT</span>
              </div>
              <InstallCopyCard />
            </section>

            <section className="grid grid-cols-1 md:grid-cols-3" aria-label="Feature summary">
              {features.map((feature) => (
                <Link
                  href={feature.href}
                  key={feature.number}
                  className={`feature-panel p-8 hover-invert group min-h-[300px] flex flex-col justify-between ${feature.panelClassName}`}
                >
                  <div>
                    <h2 className="text-6xl font-compressed mb-4">#{feature.number}</h2>
                    <h3 className="text-3xl font-bold uppercase leading-none mb-4">
                      {feature.titleLines.map((line) => (
                        <span key={line}>
                          {line}
                          <br />
                        </span>
                      ))}
                    </h3>
                    <p className="text-sm font-mono leading-relaxed opacity-80 group-hover:opacity-100 group-hover:text-white max-w-[200px]">
                      {feature.body}
                    </p>
                  </div>
                  <div className="mt-8 flex gap-2">{getFeatureAction(feature.actionKind)}</div>
                </Link>
              ))}
            </section>

            <footer className="footer-grid" aria-label="Stacc links">
              <div className="footer-mark">#</div>
              <div className="version-panel">
                <span>{displayVersion}</span>
                <span className="latest-pill">LATEST STABLE</span>
              </div>
              <div className="footer-actions">
                <div className="footer-links">
                  <Link href="/docs/getting-started">GET STARTED</Link>
                  <Link href="/docs/installation">INSTALLATION</Link>
                  <Link href="/docs/configurations#catalog">CONFIG CATALOG</Link>
                  <a href={staccIssueUrl} rel="noreferrer" target="_blank">
                    OPEN ISSUE
                  </a>
                </div>
                <div className="footer-bottom">
                  <a className="open-source-badge" href={staccRepoUrl} rel="noreferrer" target="_blank">
                    GITHUB
                    <br />
                    OPEN
                    <br />
                    SOURCE
                  </a>
                  <span className="footer-brand">
                    <span>STACC.FYI</span>
                    <span>© {binary.latestCommitYear}</span>
                  </span>
                </div>
              </div>
            </footer>
          </div>
        </article>
      </section>
    </main>
  );
}
