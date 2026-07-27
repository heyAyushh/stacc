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
        <p>
          The quick installer adds the <code className="inline-code">stacc</code> command. Run it with no arguments whenever you want the guided control
          panel.
        </p>
        <CodePanel label="Terminal" action="copy" snippet="curl -fsSL https://stacc.fyi/install.sh | bash" />
      </FillSection>

      <FillSection id="choose" number="02" title="Choose one editor and one scope">
        <p>
          STACC supports Cursor, Claude Code, Codex, OpenCode, and AMP Code. Use project scope when a setup belongs to one repository. Use global scope
          when you want it available in every project opened by that editor.
        </p>
        <ReferenceTable
          caption="A good first choice"
          rows={[
            { label: "Editor", value: "The coding agent you already use" },
            { label: "Scope", value: "project, unless the same setup should follow you everywhere" },
            { label: "Contents", value: "rules and everyday skills" },
          ]}
        />
      </FillSection>

      <FillSection id="preview" number="03" title="Preview the setup">
        <p>
          A dry run shows every destination, conflict, and file operation without changing your system. Read the plan before turning it into a real
          install.
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

      <FillSection id="apply" number="04" title="Install and confirm">
        <p>
          When the plan looks right, install it. Running the preview again should show that matching files are already in place rather than planning a
          second copy.
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

stacc install \\
  --editor codex \\
  --scope project \\
  --category rules \\
  --category skills \\
  --dry-run \\
  --print-plan`}
        />
      </FillSection>

      <FillSection id="next" number="05" title="Add one capability at a time">
        <p>Your base setup is enough to start. Add the next package only when a real task needs it.</p>
        <ReferenceTable
          caption="Common next steps"
          rows={[
            { label: "Language or framework expertise", value: "Add --category stack --stack <name>" },
            { label: "A connection to another tool", value: "Add --category mcps --mcp-server <name>" },
            { label: "Editor automation", value: "Add --category hooks --hook <name>" },
            {
              label: "See what is available",
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
      <FillSection id="methods" number="01" title="Install the app">
        <p>
          The hosted installer is the normal path on macOS or Linux. If you already manage Rust tools with Cargo, you can install directly from the
          repository instead. The local-checkout command is for people working on STACC itself.
        </p>
        <CodePanel
          label="Terminal"
          action="choose one"
          snippet={`# Recommended
curl -fsSL https://stacc.fyi/install.sh | bash

# If you already use Cargo
cargo install --git https://github.com/heyAyushh/stacc --locked --force

# STACC contributors
./install.sh`}
        />
      </FillSection>

      <FillSection id="targets" number="02" title="Choose where the setup applies">
        <p>
          Project scope keeps the setup beside one repository, which makes it easy to review and share with that project. Global scope puts it in your
          personal editor configuration so it follows you across repositories.
        </p>
        <InstallSurfaceMatrix />
      </FillSection>

      <FillSection id="conflicts" number="03" title="Protect existing configuration">
        <p>
          Existing files are never an invisible decision. STACC either backs them up, replaces them, leaves them alone, or asks about each conflict in
          an interactive terminal.
        </p>
        <ReferenceTable
          caption="What happens when a target already exists"
          rows={[
            { label: "backup (default)", value: "Keep a timestamped copy, then install the STACC version" },
            { label: "overwrite", value: "Replace the target with the STACC version" },
            { label: "skip", value: "Keep the target exactly as it is and leave it unmanaged" },
            { label: "selective", value: "Ask what to do with each conflict in an interactive terminal" },
          ]}
        />
        <p>
          Use a dry run first when you are unsure. In scripts, choose a non-interactive mode explicitly; selective mode needs a person at the terminal.
        </p>
        <CodePanel
          label="preview a replacement"
          action="preview"
          snippet="stacc install --editor codex --scope project --category skills --conflict overwrite --dry-run --print-plan"
        />
      </FillSection>

      <FillSection id="upgrade" number="04" title="Keep STACC current">
        <p>
          STACC can show the exact Cargo command it will use before upgrading itself. The preview does not download or replace anything.
        </p>
        <CodePanel label="upgrade" action="preview, then run" snippet={`stacc bootstrap --dry-run\nstacc bootstrap`} />
      </FillSection>
    </>
  );
}

function SkillsFill() {
  return (
    <>
      <FillSection id="model" number="01" title="Keep the everyday set small">
        <p>
          Everyday skills cover work that appears in almost every repository. Stacks group deeper guidance for a particular language, framework,
          platform, or workflow. An agent opens the specific instructions only when the task matches, so installing a stack does not mean every page
          of guidance is added to every prompt.
        </p>
        <SkillsOverview />
      </FillSection>

      <FillSection id="install" number="02" title="Add skills or one focused stack">
        <p>
          Install everyday skills as a base. Add a named stack when the current project needs it. Editor-specific packages stay separate so you do
          not carry Cursor or Codex workflows into tools that cannot use them.
        </p>
        <CodePanel
          label="skills"
          action="dry run"
          snippet={`stacc install --editor codex --scope global --category skills --dry-run --print-plan
stacc install --editor codex --scope global --category stack --stack rust --dry-run --print-plan
stacc install --editor codex --scope project --category codex-skills --dry-run --print-plan`}
        />
      </FillSection>

      <FillSection id="catalog" number="03" title="Find the right skill">
        <div className="catalog-discovery">
          <div>
            <span className="metric-label">SEARCH THE LIBRARY</span>
            <p>Search for the task, language, framework, platform, or tool you are working with. Source and license remain visible on every detail page.</p>
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
      <FillSection id="choose" number="01" title="Choose the interface that fits the job">
        <ReferenceTable
          caption="Control panel or commands"
          rows={[
            { label: "Setting up by hand", value: "Run stacc and use the control panel" },
            { label: "Repeating the same setup", value: "Use a complete stacc install command" },
            { label: "Running in automation", value: "Use --dry-run, --print-plan, --json, and an explicit --yes gate" },
          ]}
        />
      </FillSection>

      <FillSection id="tui" number="02" title="Use the guided control panel">
        <p>
          Run <code className="inline-code">stacc</code> with no subcommand. Choose the editor, project or global scope, what you want to add, and how to
          handle conflicts. Dry run starts on so you can inspect the result before writing.
        </p>
        <TuiSegmentGrid />
      </FillSection>

      <FillSection id="commands" number="03" title="Use commands for repeatable work">
        <ReferenceTable
          caption="Commands most users need"
          rows={[
            { label: "stacc", value: "Open the guided control panel" },
            { label: "stacc install", value: "Preview or install a selected setup" },
            { label: "stacc sync", value: "Bring an older STACC install under management" },
            { label: "stacc update", value: "Refresh a managed skill, stack, or Codex plugin" },
            { label: "stacc uninstall", value: "Remove a managed skill, stack, or Codex plugin" },
            { label: "stacc bootstrap", value: "Install or upgrade the STACC command" },
          ]}
        />
        <CodePanel label="help" action="see every option" snippet={`stacc --help\nstacc install --help\nstacc update --help`} />
      </FillSection>

      <FillSection id="scripts" number="04" title="Automate without hiding changes">
        <p>
          Machine-readable status tells a script whether the STACC bundle and catalog are available. Printed plans make file operations reviewable.
          Add <code className="inline-code">--yes</code> only at the point where the workflow is allowed to write.
        </p>
        <CodePanel
          label="automation"
          action="inspect first"
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
      <FillSection id="base" number="01" title="Start with rules and everyday skills">
        <p>
          Rules keep expectations consistent across an editor. Everyday skills teach common workflows such as diagnosis, testing, handoff, and browser
          work. Together they make a useful first setup without committing you to a particular framework or toolchain.
        </p>
        <ConfigInventoryOverview />
      </FillSection>

      <FillSection id="categories" number="02" title="Add capabilities when they solve a real need">
        <ReferenceTable
          caption="What each category gives you"
          rows={[
            { label: "Commands", value: "Named workflows you invoke on demand" },
            { label: "Agents", value: "Reusable specialist roles for tools that support them" },
            { label: "Stacks", value: "Focused expertise for one language, framework, platform, or workflow" },
            { label: "Hooks", value: "Automations that react to supported editor events" },
            { label: "MCP servers", value: "Connections that let an agent use another tool or data source" },
            { label: "Editor extras", value: "Packages designed specifically for Cursor or Codex" },
          ]}
        />
      </FillSection>

      <FillSection id="compatibility" number="03" title="Check compatibility before you install">
        <p>
          STACC filters choices for the selected editor, but the command line still expects explicit selectors for focused or external capabilities.
        </p>
        <ReferenceTable
          caption="Selections that need extra detail"
          rows={[
            { label: "Stack", value: "Add --stack <name>; use --stack all only when you truly want every stack" },
            { label: "MCP server", value: "Add --mcp-server <name>; availability depends on the editor and scope" },
            { label: "Hook", value: "Add --hook <name>; hooks are currently available for Cursor and Claude Code" },
            { label: "Codex plugin", value: "Add --codex-plugin <name>; Codex plugins are installed globally" },
          ]}
        />
      </FillSection>

      <FillSection id="catalog" number="04" title="Browse everything available">
        <p>
          Open a skill or stack for its full instructions, source, and license. Other entries link to their tracked source so you can inspect exactly
          what STACC installs.
        </p>
        <ConfigInventoryCatalog />
      </FillSection>
    </>
  );
}

function ManagedLifecycleFill() {
  return (
    <>
      <FillSection id="ownership" number="01" title="STACC remembers what it installed">
        <p>
          A successful install records the selected skills, stacks, command packages, and Codex plugins in a small manifest inside the target. That
          record is what gives STACC permission to update or remove them later. Skipped files and unrelated editor configuration are not claimed.
        </p>
      </FillSection>

      <FillSection id="adopt" number="02" title="Bring an older STACC install under management">
        <p>
          If STACC packages were copied before ownership manifests existed, <code className="inline-code">sync</code> can recognize matching installed
          packages. Preview the exact entries before writing the manifest.
        </p>
        <CodePanel
          label="sync"
          action="preview"
          snippet={`stacc sync --editor codex --scope project --dry-run --print-plan
stacc sync --editor codex --scope project --skill ultragoal --dry-run --print-plan`}
        />
      </FillSection>

      <FillSection id="update" number="03" title="Update only what you name">
        <p>
          An update must match an owned entry and stays inside the same editor and scope. Existing conflicts still follow the conflict mode you choose.
        </p>
        <CodePanel label="update" action="preview" snippet="stacc update --editor codex --scope project --skill ultragoal --dry-run --print-plan" />
      </FillSection>

      <FillSection id="uninstall" number="04" title="Remove only what STACC owns">
        <p>
          Uninstall refuses to infer ownership from filenames or folders. It removes only the managed entry you name. Preview the plan, then repeat
          with <code className="inline-code">--yes</code> when the target is correct.
        </p>
        <CodePanel label="uninstall" action="preview" snippet="stacc uninstall --editor codex --scope project --skill ultragoal --dry-run --print-plan" />
      </FillSection>
    </>
  );
}

function TroubleshootingFill() {
  return (
    <>
      <FillSection id="nothing-written" number="01" title="The command finished, but nothing changed">
        <p>
          Check whether the command included <code className="inline-code">--dry-run</code>. A preview never writes. To apply the same plan, remove that
          flag and confirm interactively, or add <code className="inline-code">--yes</code> in a workflow that is already allowed to write.
        </p>
      </FillSection>

      <FillSection id="conflict" number="02" title="A configuration file already exists">
        <p>
          Run the install again as a dry run and read the conflict in the printed plan. Keep the default backup mode if you want the safest reversible
          choice. Use skip to preserve the existing file, overwrite only when replacement is intentional, or selective mode when you are at an
          interactive terminal.
        </p>
        <CodePanel
          label="conflict"
          action="inspect"
          snippet="stacc install --editor codex --scope project --category skills --dry-run --print-plan"
        />
      </FillSection>

      <FillSection id="invalid-selection" number="03" title="The selected combination is not supported">
        <p>
          Read the error together with the editor, scope, and category you selected. Stacks need a named <code className="inline-code">--stack</code>;
          MCP support varies by editor and scope; hooks are limited to Cursor and Claude Code; Codex plugins install globally.
        </p>
        <CodePanel label="inspect" action="help" snippet={`stacc install --help\nstacc status`} />
      </FillSection>

      <FillSection id="manifest" number="04" title="Update or removal says the package is unmanaged">
        <p>
          STACC will not update or delete a folder merely because its name looks familiar. If it came from an earlier STACC install, use
          <code className="inline-code"> sync</code> to preview an ownership record. If it came from somewhere else, leave it unmanaged.
        </p>
        <CodePanel label="ownership" action="preview" snippet="stacc sync --editor codex --scope project --dry-run --print-plan" />
      </FillSection>

      <FillSection id="runtime" number="05" title="An MCP server or plugin still is not available">
        <p>
          First confirm that the install plan targeted the editor and scope you are actually using, then restart that editor so it reloads its
          configuration. STACC writes the connection or invokes the selected plugin marketplace; it does not provide third-party credentials. Check
          the tool&apos;s own login, environment variables, and runtime requirements if the entry is present but cannot connect.
        </p>
        <CodePanel label="verify target" action="inspect" snippet="stacc install --editor codex --scope global --category mcps --mcp-server github --dry-run --print-plan" />
      </FillSection>
    </>
  );
}

function HowItWorksFill() {
  return (
    <>
      <FillSection id="destinations" number="01" title="Files go only to the editor and scope you choose">
        <p>
          Project installs stay beside the current repository. Global installs go to that editor&apos;s user configuration. STACC filters out categories
          the selected editor or scope cannot use, and a dry run shows the resolved destinations before any write.
        </p>
        <InstallSurfaceMatrix />
      </FillSection>

      <FillSection id="existing" number="02" title="Existing configuration is an explicit choice">
        <p>
          The default is to keep a timestamped backup before replacing a conflicting target. You can instead skip it, overwrite it, or decide one
          conflict at a time. The dry-run plan uses the same decisions as the real install but applies none of them.
        </p>
        <CodePanel label="preview" action="inspect" snippet="stacc install --editor codex --scope project --category skills --dry-run --print-plan" />
      </FillSection>

      <FillSection id="structured" number="03" title="MCP settings are merged, not flattened">
        <p>
          Adding one MCP server keeps unrelated settings in the same configuration file. STACC writes the structure each editor expects, including
          Codex TOML and AMP&apos;s nested MCP settings. The plan shows the target file before the merge is applied.
        </p>
      </FillSection>

      <FillSection id="ownership" number="04" title="Updates and removal stop at the ownership boundary">
        <p>
          STACC records the packages it successfully installed. Later updates and uninstalls require those records; they do not scan arbitrary editor
          folders and guess what belongs to STACC. Skipped conflicts are not marked as owned.
        </p>
      </FillSection>
    </>
  );
}

function LazyCodexFill() {
  return (
    <>
      <FillSection id="about" number="01" title="Add the OmO workflow to Codex">
        <p>
          LazyCodex is the marketplace source for the <code className="inline-code">omo@sisyphuslabs</code> Codex plugin. STACC keeps it optional and
          installs it only when you name the <code className="inline-code">lazycodex</code> plugin key.
        </p>
      </FillSection>

      <FillSection id="before" number="02" title="Make sure Codex plugins work first">
        <p>
          You need a working local Codex CLI with plugin marketplace support. Codex plugins are global, so this is an editor-wide add-on rather than a
          package copied into one project.
        </p>
      </FillSection>

      <FillSection id="install" number="03" title="Preview the marketplace commands, then install">
        <p>
          The preview shows the marketplace and plugin commands without running them. The plugin key already selects Codex, the plugin category, and
          global scope.
        </p>
        <CodePanel
          label="lazycodex"
          action="preview"
          snippet={`stacc install --editor codex --codex-plugin lazycodex --dry-run --print-plan
stacc install --editor codex --codex-plugin lazycodex --yes`}
        />
      </FillSection>

      <FillSection id="manage" number="04" title="Update or remove the managed plugin">
        <p>
          STACC can update or remove LazyCodex only after it has recorded the successful install. Preview either action before applying it.
        </p>
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
