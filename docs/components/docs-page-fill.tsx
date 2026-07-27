import {
  AgentDirectorySupport,
  BinaryOverview,
  CategoryRail,
  ConfigInventoryCatalog,
  ConfigInventoryOverview,
  InstallSurfaceMatrix,
  RuntimeDependencyGrid,
  SkillsCatalog,
  SkillsOverview,
  TuiSegmentGrid,
} from "@/components/inventory-panels";
import { CodePanel, LaunchPanel } from "@/components/mdx-components";

type PageFillProps = {
  slug: string;
};

type FillSectionProps = {
  id: string;
  number: string;
  title: string;
  children: React.ReactNode;
};

function FillSection({ id, number, title, children }: FillSectionProps) {
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

function GettingStartedFill() {
  return (
    <>
      <FillSection id="installation" number="01" title="Installation">
        <p>Deploy the STACC binary to your local environment using the universal installer.</p>
        <CodePanel label="Terminal" action="Copy" snippet="curl -fsSL https://stacc.fyi/install.sh | bash" />
      </FillSection>

      <FillSection id="defining-agents" number="02" title="Define Your Agent">
        <p>
          Agents are defined via <code className="inline-code">.stacc</code> files in your project root. They govern how the LLM
          interacts with your codebase.
        </p>
        <CodePanel
          label="agent.stacc"
          action="yaml"
          snippet={`agent:
  name: "TS_ARCHITECT"
  role: "Strict TypeScript development"
  constraints:
    - "No 'any' types allowed"
    - "Prefer functional patterns"
    - "Auto-document exported functions"
  hooks:
    - pre-commit: "stacc validate"`}
        />
      </FillSection>

      <FillSection id="execution-loop" number="03" title="Execution">
        <p>Run the daemon to start monitoring files and enforcing rulesets in real-time.</p>
        <LaunchPanel label="Launch Daemon" command="stacc --watch ./src" />
      </FillSection>
    </>
  );
}

function ArchitectureFill() {
  return (
    <>
      <FillSection id="config-surface" number="01" title="Config Surface">
        <p>
          STACC ships rules, commands, skills, agents, hooks, stacks, and MCP configuration as source-controlled assets under{" "}
          <code className="inline-code">configs/</code>.
        </p>
        <CodePanel label="tree" action="source" snippet={`configs/
  rules/
  commands/
  skills/
  agents/
  hooks/
  stacks/
  mcps/`} />
      </FillSection>

      <FillSection id="install-planner" number="02" title="Install Planner">
        <p>
          The Rust CLI builds an install plan before writing files. Planning keeps dry runs, conflict detection, backups, and
          selective installs aligned across supported editors.
        </p>
        <CodePanel label="dry run" action="inspect" snippet={`cargo run -- install \\
  --editor codex \\
  --scope global \\
  --category rules \\
  --dry-run \\
  --print-plan`} />
      </FillSection>

      <FillSection id="merge-boundaries" number="03" title="Merge Boundaries">
        <p>
          MCP files are merged by target format: JSON targets use recursive object merge, Codex targets use TOML tables, and AMP
          settings are wrapped under <code className="inline-code">amp.mcpServers</code>.
        </p>
        <LaunchPanel label="Validate Config" command="cargo run -- check" />
      </FillSection>
    </>
  );
}

function InstallationFill() {
  return (
    <>
      <FillSection id="local-checkout" number="01" title="Local Checkout">
        <p>Run the Rust CLI directly when developing from this repository.</p>
        <CodePanel label="local" action="run" snippet="cargo run -- install --dry-run --print-plan" />
      </FillSection>

      <FillSection id="bootstrap" number="02" title="Bootstrap">
        <p>The shell bootstrap keeps legacy flags working while forwarding install behavior to the Rust binary.</p>
        <CodePanel label="bootstrap" action="verify" snippet={`bash -n install.sh
shellcheck -x install.sh`} />
      </FillSection>

      <FillSection id="remote-install" number="03" title="Remote Install">
        <p>The universal installer is the user-facing path for remote setup. Keep it display-only inside docs.</p>
        <CodePanel label="remote" action="copy" snippet="curl -fsSL https://stacc.fyi/install.sh | bash" />
      </FillSection>
    </>
  );
}

function SkillsFill() {
  return (
    <>
      <FillSection id="inventory" number="01" title="Inventory Snapshot">
        <p>Counts, collections, and timestamps come from the checked-in metadata lockfile.</p>
        <SkillsOverview />
      </FillSection>

      <FillSection id="catalog" number="02" title="All Skills">
        <p>Browse each skill package with version, license, and original source metadata.</p>
        <SkillsCatalog />
      </FillSection>

      <FillSection id="install" number="03" title="Install Skills">
        <CodePanel label="skills" action="dry-run" snippet="cargo run -- install --editor codex --scope global --category skills --dry-run --print-plan" />
        <CodePanel label="stacks" action="dry-run" snippet="cargo run -- install --editor codex --scope global --category stack --stack nextjs --dry-run --print-plan" />
        <CodePanel label="codex imports" action="dry-run" snippet="cargo run -- install --editor codex --scope global --category codex-skills --dry-run --print-plan" />
      </FillSection>
    </>
  );
}

function BinaryTuiFill() {
  return (
    <>
      <FillSection id="binary" number="01" title="Binary Surface">
        <BinaryOverview />
        <CodePanel label="help" action="cli" snippet={`stacc
stacc --panel
stacc status --json
stacc install --editor codex --scope global --category rules --category skills --dry-run
stacc bootstrap --dry-run
stacc check`} />
      </FillSection>

      <FillSection id="tui" number="02" title="TUI Segments">
        <TuiSegmentGrid />
      </FillSection>

      <FillSection id="installer" number="03" title="Installer Surface">
        <CategoryRail />
        <AgentDirectorySupport />
        <InstallSurfaceMatrix />
        <CodePanel label="install help" action="cli" snippet={`stacc install --editor ampcode --scope project --category rules --category skills --dry-run
stacc install --editor cursor --scope project --category rules --category skills --dry-run
stacc install --editor codex --scope global --category rules --category skills --category mcps --mcp-server github --yes
stacc install --editor cursor --scope project --category hooks --hook continual-learning --dry-run`} />
      </FillSection>

      <FillSection id="dependencies" number="04" title="Docs Runtime">
        <RuntimeDependencyGrid />
      </FillSection>
    </>
  );
}

function ConfigurationsFill() {
  return (
    <>
      <FillSection id="payload" number="01" title="Payload Snapshot">
        <ConfigInventoryOverview />
      </FillSection>

      <FillSection id="groups" number="02" title="Installable Groups">
        <ConfigInventoryCatalog />
      </FillSection>

      <FillSection id="install" number="03" title="Install Payload">
        <CodePanel label="rules + skills" action="dry-run" snippet="cargo run -- install --editor codex --scope global --category rules --category skills --dry-run --print-plan" />
        <CodePanel label="hooks" action="dry-run" snippet="cargo run -- install --editor cursor --scope project --category hooks --hook continual-learning --dry-run --print-plan" />
        <CodePanel label="mcps" action="dry-run" snippet="cargo run -- install --editor codex --scope global --category mcps --mcp-server github --dry-run --print-plan" />
      </FillSection>
    </>
  );
}

function LazyCodexFill() {
  return (
    <>
      <FillSection id="package" number="01" title="Package">
        <p>
          LazyCodex is published as <code className="inline-code">lazycodex-ai</code>. The package exposes both{" "}
          <code className="inline-code">lazycodex-ai</code> and <code className="inline-code">lazycodex</code> command names.
        </p>
        <CodePanel label="npm" action="inspect" snippet="npm view lazycodex-ai name version description bin license --json" />
      </FillSection>

      <FillSection id="install" number="02" title="Install">
        <p>Use the upstream package installer when you want the LazyCodex harness itself.</p>
        <CodePanel label="lazycodex" action="install" snippet="npx lazycodex-ai install" />
      </FillSection>

      <FillSection id="source" number="03" title="Source">
        <p>
          The npm package points at <code className="inline-code">code-yeongyu/oh-my-openagent</code> and declares the{" "}
          <code className="inline-code">SUL-1.0</code> license.
        </p>
        <CodePanel label="source" action="github" snippet="https://github.com/code-yeongyu/oh-my-openagent" />
      </FillSection>
    </>
  );
}

export function DocsPageFill({ slug }: PageFillProps) {
  switch (slug) {
    case "getting-started":
      return <GettingStartedFill />;
    case "architecture":
      return <ArchitectureFill />;
    case "installation":
      return <InstallationFill />;
    case "skills":
      return <SkillsFill />;
    case "binary-tui":
      return <BinaryTuiFill />;
    case "configurations":
      return <ConfigurationsFill />;
    case "lazycodex":
      return <LazyCodexFill />;
    default:
      return null;
  }
}
