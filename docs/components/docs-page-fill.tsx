import Link from "next/link";
import { CommandSearch } from "@/components/command-search";
import {
  ConfigInventoryCatalog,
  ConfigInventoryOverview,
  InstallSurfaceMatrix,
  SkillsCatalog,
  SkillsOverview,
  TuiSegmentGrid,
} from "@/components/inventory-panels";
import { CodePanel } from "@/components/mdx-components";

type PageFillProps = {
  slug: string;
};

type FillSectionProps = {
  id: string;
  number: string;
  title: string;
  children: React.ReactNode;
};

type ReferenceRow = {
  label: string;
  value: React.ReactNode;
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

function ReferenceTable({ caption, rows }: { caption: string; rows: ReferenceRow[] }) {
  return (
    <div className="matrix-table-wrap">
      <table className="matrix-table reference-table">
        <caption>{caption}</caption>
        <tbody>
          {rows.map((row) => (
            <tr key={row.label}>
              <th scope="row">{row.label}</th>
              <td>{row.value}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function GettingStartedFill() {
  return (
    <>
      <FillSection id="install" number="01" title="Install STACC">
        <p>Install the binary with the hosted bootstrap. Running <code className="inline-code">stacc</code> then opens the interactive control panel.</p>
        <CodePanel label="Terminal" action="copy" snippet="curl -fsSL https://stacc.fyi/install.sh | bash" />
      </FillSection>

      <FillSection id="preview" number="02" title="Preview an install">
        <p>
          Pick an editor, scope, and payload. Start with a dry run so the exact destinations, conflicts, and file operations are visible before anything changes.
        </p>
        <CodePanel
          label="dry run"
          action="inspect"
          snippet={`stacc install \\
  --editor codex \\
  --scope project \\
  --category rules \\
  --category skills \\
  --dry-run \\
  --print-plan`}
        />
      </FillSection>

      <FillSection id="apply" number="03" title="Apply and verify">
        <p>
          Remove <code className="inline-code">--dry-run</code> and add <code className="inline-code">--yes</code> when the plan is correct. Backup is the default conflict mode.
        </p>
        <CodePanel
          label="install"
          action="apply"
          snippet={`stacc install \\
  --editor codex \\
  --scope project \\
  --category rules \\
  --category skills \\
  --yes

stacc status`}
        />
      </FillSection>

      <FillSection id="next" number="04" title="Add what you need">
        <ReferenceTable
          caption="Common next steps"
          rows={[
            { label: "Framework or language", value: "Add --category stack --stack <name>" },
            { label: "One MCP server", value: "Add --category mcps --mcp-server <name>" },
            { label: "Cursor hook", value: "Add --category hooks --hook <name>" },
            {
              label: "Browse available packages",
              value: (
                <>
                  Open <Link href="/docs/skills">Skills &amp; Stacks</Link> or{" "}
                  <Link href="/docs/configurations">What You Can Install</Link>
                </>
              ),
            },
          ]}
        />
      </FillSection>
    </>
  );
}

function InstallationFill() {
  return (
    <>
      <FillSection id="methods" number="01" title="Install methods">
        <p>Use the hosted bootstrap for normal use, Cargo for a direct binary install, or the repository script while developing STACC.</p>
        <CodePanel
          label="Terminal"
          action="choose one"
          snippet={`# Hosted bootstrap
curl -fsSL https://stacc.fyi/install.sh | bash

# Direct from GitHub
cargo install --git https://github.com/heyAyushh/stacc --locked --force

# From a local checkout
./install.sh`}
        />
      </FillSection>

      <FillSection id="targets" number="02" title="Choose a scope">
        <p>
          Project scope writes configuration beside the current project. Global scope writes to the editor&apos;s user configuration directory.
        </p>
        <InstallSurfaceMatrix />
      </FillSection>

      <FillSection id="conflicts" number="03" title="Handle conflicts">
        <ReferenceTable
          caption="Conflict modes"
          rows={[
            { label: "backup (default)", value: "Move the existing target to .bak.<timestamp>, then install" },
            { label: "overwrite", value: "Replace the existing target" },
            { label: "skip", value: "Leave the existing target unchanged and do not claim ownership" },
            { label: "selective", value: "Prompt for each conflict in an interactive terminal" },
          ]}
        />
        <p>For scripts and agents, prefer backup, overwrite, skip, or a dry run. Selective mode expects an interactive terminal.</p>
        <CodePanel
          label="conflict strategy"
          action="preview"
          snippet="stacc install --editor codex --scope project --category skills --conflict overwrite --dry-run --print-plan"
        />
      </FillSection>

      <FillSection id="upgrade" number="04" title="Upgrade the binary">
        <p>Preview the Cargo command first, then run the upgrade when the source and flags look correct.</p>
        <CodePanel label="bootstrap" action="preview" snippet={`stacc bootstrap --dry-run\nstacc bootstrap`} />
      </FillSection>
    </>
  );
}

function SkillsFill() {
  return (
    <>
      <FillSection id="choose" number="01" title="Choose a package">
        <p>
          Keep the everyday set small, then add focused stacks for the work you actually do. Agents see the lightweight router first and load deeper guidance only when a task matches.
          Every detail page keeps the original source and license visible.
        </p>
        <SkillsOverview />
      </FillSection>

      <FillSection id="install" number="02" title="Install skills">
        <CodePanel
          label="skills"
          action="dry run"
          snippet={`stacc install --editor codex --scope global --category skills --dry-run --print-plan
stacc install --editor codex --scope global --category stack --stack rust --dry-run --print-plan
stacc install --editor codex --scope project --category codex-skills --dry-run --print-plan`}
        />
      </FillSection>

      <FillSection id="catalog" number="03" title="Browse the catalog">
        <div className="catalog-discovery">
          <div>
            <span className="metric-label">FIND ONE FAST</span>
            <p>Search by skill, stack, framework, source, or license instead of scanning the full catalog.</p>
          </div>
          <CommandSearch variant="compact" />
        </div>
        <SkillsCatalog />
      </FillSection>
    </>
  );
}

function CliTuiFill() {
  return (
    <>
      <FillSection id="commands" number="01" title="Command reference">
        <ReferenceTable
          caption="STACC commands"
          rows={[
            { label: "stacc", value: "Open the terminal control panel" },
            { label: "status", value: "Show repository, bundle, and catalog status" },
            { label: "install", value: "Plan and copy selected configuration" },
            { label: "sync", value: "Record ownership for existing STACC installs" },
            { label: "update", value: "Refresh entries already owned by STACC" },
            { label: "uninstall", value: "Remove entries already owned by STACC" },
            { label: "sync-metadata", value: "Audit or refresh source and license metadata" },
            { label: "bootstrap", value: "Install or upgrade the STACC binary" },
            { label: "check", value: "Run the repository validation gate" },
          ]}
        />
        <CodePanel label="help" action="reference" snippet={`stacc --help\nstacc install --help\nstacc update --help`} />
      </FillSection>

      <FillSection id="tui" number="02" title="Control panel">
        <p>
          The panel covers editor, scope, conflicts, categories, stacks, hooks, MCPs, plugin selection, metadata, bootstrap, and checks.
          Dry-run starts enabled in the shipped defaults.
        </p>
        <TuiSegmentGrid />
      </FillSection>

      <FillSection id="automation" number="03" title="Automation">
        <p>Use JSON status and printed dry-run plans in non-interactive workflows. Add <code className="inline-code">--yes</code> only after reviewing a plan.</p>
        <CodePanel
          label="agent workflow"
          action="safe"
          snippet={`stacc status --json
stacc install --editor cursor --editor codex --scope project \\
  --category rules --category skills --dry-run --print-plan`}
        />
      </FillSection>
    </>
  );
}

function ConfigurationsFill() {
  return (
    <>
      <FillSection id="categories" number="01" title="Installable categories">
        <p>
          Rules and skills are the common baseline. Add commands, agents, stacks, hooks, MCP servers, or editor-specific adapters only when the selected editor supports them.
        </p>
        <ConfigInventoryOverview />
      </FillSection>

      <FillSection id="selection" number="02" title="Selection rules">
        <ReferenceTable
          caption="Category selectors"
          rows={[
            { label: "stack", value: "Requires --stack <name> or --stack all" },
            { label: "mcps", value: "Use --mcp-server <name> to select individual servers" },
            { label: "hooks", value: "Use --hook <name>; available for Cursor and Claude Code" },
            { label: "cursor-plugins", value: "Cursor-only adapters" },
            { label: "codex-skills", value: "Codex skills at project or global scope" },
            { label: "codex-plugins", value: "Codex global only; --codex-plugin selects a catalog entry" },
          ]}
        />
      </FillSection>

      <FillSection id="defaults" number="03" title="Panel defaults">
        <p>
          Pass <code className="inline-code">--config path/to/config.json</code> to load another defaults file. Unknown fields are rejected instead of silently ignored.
        </p>
        <CodePanel
          label="stacc-panel.json"
          action="example"
          snippet={`{
  "default_editors": ["cursor", "codex"],
  "default_scope": "project",
  "default_categories": ["rules", "skills"],
  "default_stacks": ["rust"],
  "default_mcp_servers": [],
  "default_hook_packages": [],
  "default_codex_plugins": [],
  "conflict_mode": "backup",
  "dry_run": true
}`}
        />
      </FillSection>

      <FillSection id="catalog" number="04" title="Payload catalog">
        <ConfigInventoryCatalog />
      </FillSection>
    </>
  );
}

function ManagedLifecycleFill() {
  return (
    <>
      <FillSection id="ownership" number="01" title="Managed ownership">
        <p>
          Successful installs record owned skills, stacks, command-as-skill packages, and Codex plugins in
          <code className="inline-code"> &lt;target-root&gt;/.stacc/manifest.json</code>. STACC does not claim skipped content or arbitrary files already in an editor directory.
        </p>
      </FillSection>

      <FillSection id="adopt" number="02" title="Adopt an existing install">
        <p>Use sync when STACC content was installed before manifests existed. Preview the entries before writing the manifest.</p>
        <CodePanel
          label="sync"
          action="preview"
          snippet={`stacc sync --editor codex --scope project --dry-run --print-plan
stacc sync --editor codex --scope project --skill ultragoal --dry-run --print-plan`}
        />
      </FillSection>

      <FillSection id="update" number="03" title="Update managed content">
        <p>Update requires a matching manifest entry and keeps the same editor and scope boundary.</p>
        <CodePanel label="update" action="preview" snippet="stacc update --editor codex --scope project --skill ultragoal --dry-run --print-plan" />
      </FillSection>

      <FillSection id="uninstall" number="04" title="Uninstall safely">
        <p>Uninstall removes only matching managed entries. Preview the plan, then repeat with <code className="inline-code">--yes</code>.</p>
        <CodePanel label="uninstall" action="preview" snippet="stacc uninstall --editor codex --scope project --skill ultragoal --dry-run --print-plan" />
      </FillSection>
    </>
  );
}

function TroubleshootingFill() {
  return (
    <>
      <FillSection id="nothing-written" number="01" title="Nothing was written">
        <p>
          Dry-run is intentionally non-mutating. Remove <code className="inline-code">--dry-run</code> and confirm interactively, or add
          <code className="inline-code"> --yes</code> in a reviewed non-interactive workflow.
        </p>
      </FillSection>

      <FillSection id="invalid-selection" number="02" title="Selection rejected">
        <p>
          Check the editor, scope, and category combination. Hooks are limited to Cursor and Claude Code, MCP support varies by scope, stacks need
          <code className="inline-code"> --stack</code>, and Codex plugins install globally.
        </p>
        <CodePanel label="inspect" action="help" snippet={`stacc install --help\nstacc status`} />
      </FillSection>

      <FillSection id="manifest" number="03" title="Manifest missing">
        <p>
          Update and uninstall refuse unmanaged content. If the files came from STACC, use <code className="inline-code">sync</code> to preview and backfill ownership.
          Otherwise, leave them unmanaged.
        </p>
        <CodePanel label="ownership" action="preview" snippet="stacc sync --editor codex --scope project --dry-run --print-plan" />
      </FillSection>

      <FillSection id="checks" number="04" title="Run diagnostics">
        <p>
          Repository maintainers can run the full format, test, lint, installer, JSON, offline install, and binary smoke gate. If a gate fails,
          fix the named source input and rerun the check. Use <code className="inline-code">--require-shellcheck</code> only when ShellCheck is
          expected in the environment.
        </p>
        <CodePanel label="repository" action="validate" snippet={`stacc check\nstacc check --require-shellcheck`} />
      </FillSection>
    </>
  );
}

function HowItWorksFill() {
  return (
    <>
      <FillSection id="payload" number="01" title="Source payload">
        <p>
          STACC bundles the canonical configuration under <code className="inline-code">configs/</code>. The binary discovers what is available and filters it for the selected editor and scope.
        </p>
      </FillSection>

      <FillSection id="planning" number="02" title="Plan before write">
        <p>
          Install, sync, update, and uninstall build deterministic plans first. Dry runs use the same planner as real execution, without applying operations.
        </p>
        <CodePanel label="plan" action="inspect" snippet="stacc install --editor codex --scope project --category skills --dry-run --print-plan" />
      </FillSection>

      <FillSection id="merging" number="03" title="Merge configuration">
        <p>
          MCP JSON targets merge recursively. Codex MCP servers are written as TOML tables. AMP servers remain under
          <code className="inline-code"> amp.mcpServers</code>. File and directory conflicts follow the selected conflict mode.
        </p>
      </FillSection>

      <FillSection id="ownership" number="04" title="Track ownership">
        <p>
          The managed manifest is the boundary for later updates and uninstalls. This prevents STACC from scanning and modifying unrelated editor configuration.
        </p>
      </FillSection>
    </>
  );
}

function LazyCodexFill() {
  return (
    <>
      <FillSection id="catalog" number="01" title="Optional Codex plugin">
        <p>
          LazyCodex is an opt-in entry in STACC&apos;s Codex plugin catalog. It requires a working local Codex CLI because STACC delegates installation
          to the Codex plugin marketplace and does not vendor the plugin payload.
        </p>
      </FillSection>

      <FillSection id="install" number="02" title="Preview and install">
        <p>The plugin key implies Codex, the plugin category, and global scope. Apply only after reviewing the fixed marketplace commands in the plan.</p>
        <CodePanel
          label="lazycodex"
          action="preview"
          snippet={`stacc install --editor codex --codex-plugin lazycodex --dry-run --print-plan
stacc install --editor codex --codex-plugin lazycodex --yes`}
        />
      </FillSection>

      <FillSection id="manage" number="03" title="Update or remove">
        <p>Both actions require the managed plugin entry written by STACC.</p>
        <CodePanel
          label="managed plugin"
          action="preview"
          snippet={`stacc update --editor codex --codex-plugin lazycodex --dry-run --print-plan
stacc uninstall --editor codex --codex-plugin lazycodex --dry-run --print-plan`}
        />
      </FillSection>
    </>
  );
}

export function DocsPageFill({ slug }: PageFillProps) {
  switch (slug) {
    case "getting-started":
      return <GettingStartedFill />;
    case "installation":
      return <InstallationFill />;
    case "skills":
      return <SkillsFill />;
    case "binary-tui":
      return <CliTuiFill />;
    case "configurations":
      return <ConfigurationsFill />;
    case "managed-lifecycle":
      return <ManagedLifecycleFill />;
    case "troubleshooting":
      return <TroubleshootingFill />;
    case "architecture":
      return <HowItWorksFill />;
    case "lazycodex":
      return <LazyCodexFill />;
    default:
      return null;
  }
}
