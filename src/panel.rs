use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Color, Frame, Line, Modifier, Span, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap};

use crate::bootstrap::{default_bootstrap_options, BootstrapOptions};
use crate::catalog::{default_metadata_path, Catalog, Category, ConflictMode, Editor, Scope};
use crate::config::PanelConfig;
use crate::git_utils::{repository_status, RepositoryStatus};
use crate::install::InstallRequest;
use crate::metadata::{default_sync_options, SyncOptions};

const EVENT_POLL_MS: u64 = 200;
const LEFT_PANEL_PERCENT: u16 = 48;
const HEADER_HEIGHT: u16 = 3;
const FOOTER_HEIGHT: u16 = 3;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PanelOutcome {
    Quit,
    RunInstall(InstallRequest),
    SyncMetadata(SyncOptions),
    RunChecks,
    Bootstrap(BootstrapOptions),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Segment {
    Installation,
    Customization,
    VersionControl,
    Skills,
    HooksMcp,
}

impl Segment {
    const ALL: [Segment; 5] = [
        Segment::Installation,
        Segment::Customization,
        Segment::VersionControl,
        Segment::Skills,
        Segment::HooksMcp,
    ];

    fn label(self) -> &'static str {
        match self {
            Segment::Installation => "Install",
            Segment::Customization => "Customise",
            Segment::VersionControl => "Version",
            Segment::Skills => "Skills",
            Segment::HooksMcp => "Hooks/MCP",
        }
    }

    fn next(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|segment| *segment == self)
            .unwrap_or_default();
        Self::ALL[(index + 1) % Self::ALL.len()]
    }

    fn previous(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|segment| *segment == self)
            .unwrap_or_default();
        Self::ALL[(index + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PanelItemAction {
    ToggleEditor(Editor),
    CycleScope,
    CycleConflict,
    ToggleDryRun,
    RunInstall,
    ToggleCategory(Category),
    ToggleStack(String),
    ToggleMcpServer(String),
    ToggleHookPackage(String),
    RefreshStatus,
    SyncMetadata,
    RunChecks,
    Bootstrap,
    Noop,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PanelItem {
    label: String,
    action: PanelItemAction,
}

#[derive(Clone, Debug)]
struct PanelState {
    root: PathBuf,
    catalog: Catalog,
    segment: Segment,
    cursor: usize,
    editors: Vec<Editor>,
    scope: Scope,
    categories: Vec<Category>,
    stacks: Vec<String>,
    mcp_servers: Vec<String>,
    hook_packages: Vec<String>,
    conflict_mode: ConflictMode,
    dry_run: bool,
    status: RepositoryStatus,
    message: String,
}

pub fn run_panel(
    root: PathBuf,
    catalog: Catalog,
    config: PanelConfig,
    initial_message: Option<String>,
) -> Result<PanelOutcome> {
    let metadata_path = default_metadata_path(&root);
    let status = repository_status(&root, &metadata_path)?;
    let mut terminal = ratatui::init();
    let result = run_panel_loop(
        &mut PanelState {
            root,
            catalog,
            segment: Segment::Installation,
            cursor: 0,
            editors: config.default_editors,
            scope: config.default_scope,
            categories: config.default_categories,
            stacks: config.default_stacks,
            mcp_servers: config.default_mcp_servers,
            hook_packages: config.default_hook_packages,
            conflict_mode: config.conflict_mode,
            dry_run: config.dry_run,
            status,
            message: initial_message.unwrap_or_else(|| "ready".to_string()),
        },
        &mut terminal,
    );
    ratatui::restore();
    result
}

fn run_panel_loop(
    state: &mut PanelState,
    terminal: &mut ratatui::DefaultTerminal,
) -> Result<PanelOutcome> {
    loop {
        terminal.draw(|frame| render(frame, state))?;
        if !event::poll(Duration::from_millis(EVENT_POLL_MS))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(PanelOutcome::Quit),
            KeyCode::Left => switch_segment(state, state.segment.previous()),
            KeyCode::Right | KeyCode::Tab => switch_segment(state, state.segment.next()),
            KeyCode::Up => move_cursor_up(state),
            KeyCode::Down => move_cursor_down(state),
            KeyCode::Char(' ') => handle_selected_item(state)?,
            KeyCode::Enter => {
                if let Some(outcome) = handle_enter(state)? {
                    return Ok(outcome);
                }
            }
            KeyCode::Char('r') => refresh_status(state)?,
            _ => {}
        }
    }
}

fn render(frame: &mut Frame<'_>, state: &PanelState) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(HEADER_HEIGHT),
            Constraint::Min(0),
            Constraint::Length(FOOTER_HEIGHT),
        ])
        .split(frame.area());

    render_tabs(frame, state, outer[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(LEFT_PANEL_PERCENT),
            Constraint::Min(0),
        ])
        .split(outer[1]);

    render_items(frame, state, body[0]);
    render_summary(frame, state, body[1]);
    render_footer(frame, state, outer[2]);
}

fn render_tabs(frame: &mut Frame<'_>, state: &PanelState, area: ratatui::layout::Rect) {
    let titles = Segment::ALL
        .iter()
        .map(|segment| {
            Line::from(Span::styled(
                segment.label(),
                Style::default().fg(Color::White),
            ))
        })
        .collect::<Vec<_>>();
    let selected = Segment::ALL
        .iter()
        .position(|segment| *segment == state.segment)
        .unwrap_or_default();
    let tabs = Tabs::new(titles)
        .select(selected)
        .block(Block::default().borders(Borders::ALL).title("stacc"))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, area);
}

fn render_items(frame: &mut Frame<'_>, state: &PanelState, area: ratatui::layout::Rect) {
    let items = state
        .items()
        .into_iter()
        .map(|item| ListItem::new(item.label))
        .collect::<Vec<_>>();
    let mut list_state = ListState::default();
    if !items.is_empty() {
        list_state.select(Some(state.cursor.min(items.len() - 1)));
    }
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(state.segment.label()),
        )
        .highlight_symbol("> ")
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_summary(frame: &mut Frame<'_>, state: &PanelState, area: ratatui::layout::Rect) {
    let summary = state.summary_lines().join("\n");
    let paragraph = Paragraph::new(summary)
        .block(Block::default().borders(Borders::ALL).title("Summary"))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn render_footer(frame: &mut Frame<'_>, state: &PanelState, area: ratatui::layout::Rect) {
    let footer = Paragraph::new(format!(
        "←/→ segment  ↑/↓ move  Space toggle  Enter run  r refresh  q quit\n{}",
        state.message
    ))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, area);
}

impl PanelState {
    fn items(&self) -> Vec<PanelItem> {
        match self.segment {
            Segment::Installation => self.installation_items(),
            Segment::Customization => self.customization_items(),
            Segment::VersionControl => self.version_items(),
            Segment::Skills => self.skill_items(),
            Segment::HooksMcp => self.hook_mcp_items(),
        }
    }

    fn installation_items(&self) -> Vec<PanelItem> {
        let mut items = Editor::ALL
            .iter()
            .map(|editor| PanelItem {
                label: format!(
                    "{} {}",
                    marker(self.editors.contains(editor)),
                    editor.label()
                ),
                action: PanelItemAction::ToggleEditor(*editor),
            })
            .collect::<Vec<_>>();
        items.push(PanelItem {
            label: format!("Scope: {}", self.scope),
            action: PanelItemAction::CycleScope,
        });
        items.push(PanelItem {
            label: format!("Conflict: {}", self.conflict_mode),
            action: PanelItemAction::CycleConflict,
        });
        items.push(PanelItem {
            label: format!("{} Dry-run", marker(self.dry_run)),
            action: PanelItemAction::ToggleDryRun,
        });
        items.push(PanelItem {
            label: "Run install plan".to_string(),
            action: PanelItemAction::RunInstall,
        });
        items
    }

    fn customization_items(&self) -> Vec<PanelItem> {
        let mut items = Vec::new();
        for category in &self.catalog.categories {
            items.push(PanelItem {
                label: format!(
                    "{} {}",
                    marker(self.categories.contains(category)),
                    category.label()
                ),
                action: PanelItemAction::ToggleCategory(*category),
            });
        }
        for stack in &self.catalog.stacks {
            items.push(PanelItem {
                label: format!("{} stack: {}", marker(self.stacks.contains(stack)), stack),
                action: PanelItemAction::ToggleStack(stack.clone()),
            });
        }
        items
    }

    fn version_items(&self) -> Vec<PanelItem> {
        vec![
            PanelItem {
                label: "Refresh git status".to_string(),
                action: PanelItemAction::RefreshStatus,
            },
            PanelItem {
                label: "Sync skills metadata".to_string(),
                action: PanelItemAction::SyncMetadata,
            },
            PanelItem {
                label: "Run repository checks".to_string(),
                action: PanelItemAction::RunChecks,
            },
            PanelItem {
                label: "Install/upgrade stacc binary".to_string(),
                action: PanelItemAction::Bootstrap,
            },
        ]
    }

    fn skill_items(&self) -> Vec<PanelItem> {
        vec![
            PanelItem {
                label: format!("{} skills discovered", self.catalog.skill_count),
                action: PanelItemAction::Noop,
            },
            PanelItem {
                label: format!(
                    "lockfile: {}",
                    lockfile_label(self.status.metadata_lock_exists)
                ),
                action: PanelItemAction::Noop,
            },
        ]
    }

    fn hook_mcp_items(&self) -> Vec<PanelItem> {
        let mut items = Vec::new();
        for hook in &self.catalog.hook_packages {
            items.push(PanelItem {
                label: format!(
                    "{} hook: {}",
                    marker(self.hook_packages.contains(&hook.name)),
                    hook.name
                ),
                action: PanelItemAction::ToggleHookPackage(hook.name.clone()),
            });
        }
        for server in &self.catalog.mcp_servers {
            items.push(PanelItem {
                label: format!(
                    "{} mcp: {}",
                    marker(self.mcp_servers.contains(server)),
                    server
                ),
                action: PanelItemAction::ToggleMcpServer(server.clone()),
            });
        }
        if items.is_empty() {
            items.push(PanelItem {
                label: "no hooks or MCP servers found".to_string(),
                action: PanelItemAction::Noop,
            });
        }
        items
    }

    fn summary_lines(&self) -> Vec<String> {
        vec![
            format!("root: {}", self.root.display()),
            format!("branch: {} @ {}", self.status.branch, self.status.head),
            format!("changed paths: {}", self.status.changed_paths),
            format!("editors: {}", join_display(&self.editors)),
            format!("scope: {}", self.scope),
            format!("categories: {}", join_display(&self.categories)),
            format!("stacks: {}", join_strings(&self.stacks)),
            format!("mcp: {}", join_strings(&self.mcp_servers)),
            format!("hooks: {}", join_strings(&self.hook_packages)),
            format!("conflict: {}", self.conflict_mode),
            format!("dry-run: {}", self.dry_run),
        ]
    }
}

fn handle_enter(state: &mut PanelState) -> Result<Option<PanelOutcome>> {
    let items = state.items();
    let Some(item) = items.get(state.cursor) else {
        return Ok(None);
    };
    match &item.action {
        PanelItemAction::RunInstall => {
            return Ok(Some(PanelOutcome::RunInstall(InstallRequest {
                root: state.root.clone(),
                editors: state.editors.clone(),
                scope: state.scope,
                categories: state.categories.clone(),
                stacks: state.stacks.clone(),
                mcp_servers: state.mcp_servers.clone(),
                hook_packages: state.hook_packages.clone(),
                conflict_mode: state.conflict_mode,
                yes: true,
                dry_run: state.dry_run,
            })));
        }
        PanelItemAction::SyncMetadata => {
            let mut options = default_sync_options(state.root.clone());
            options.refresh_origin = true;
            return Ok(Some(PanelOutcome::SyncMetadata(options)));
        }
        PanelItemAction::RunChecks => return Ok(Some(PanelOutcome::RunChecks)),
        PanelItemAction::Bootstrap => {
            let mut options = default_bootstrap_options();
            options.dry_run = state.dry_run;
            return Ok(Some(PanelOutcome::Bootstrap(options)));
        }
        PanelItemAction::RefreshStatus => refresh_status(state)?,
        _ => handle_item_action(state, &item.action),
    }
    Ok(None)
}

fn handle_selected_item(state: &mut PanelState) -> Result<()> {
    let items = state.items();
    let Some(item) = items.get(state.cursor) else {
        return Ok(());
    };
    handle_item_action(state, &item.action);
    Ok(())
}

fn handle_item_action(state: &mut PanelState, action: &PanelItemAction) {
    match action {
        PanelItemAction::ToggleEditor(editor) => toggle_value(&mut state.editors, *editor, true),
        PanelItemAction::CycleScope => {
            state.scope = match state.scope {
                Scope::Global => Scope::Project,
                Scope::Project => Scope::Global,
            };
        }
        PanelItemAction::CycleConflict => state.conflict_mode = state.conflict_mode.next(),
        PanelItemAction::ToggleDryRun => state.dry_run = !state.dry_run,
        PanelItemAction::ToggleCategory(category) => {
            toggle_value(&mut state.categories, *category, true)
        }
        PanelItemAction::ToggleStack(stack) => toggle_string(&mut state.stacks, stack),
        PanelItemAction::ToggleMcpServer(server) => toggle_string(&mut state.mcp_servers, server),
        PanelItemAction::ToggleHookPackage(hook) => toggle_string(&mut state.hook_packages, hook),
        PanelItemAction::Noop
        | PanelItemAction::RunInstall
        | PanelItemAction::RefreshStatus
        | PanelItemAction::SyncMetadata
        | PanelItemAction::RunChecks
        | PanelItemAction::Bootstrap => {}
    }
}

fn refresh_status(state: &mut PanelState) -> Result<()> {
    state.status = repository_status(&state.root, &default_metadata_path(&state.root))?;
    state.message = "status refreshed".to_string();
    Ok(())
}

fn switch_segment(state: &mut PanelState, segment: Segment) {
    state.segment = segment;
    state.cursor = 0;
}

fn move_cursor_up(state: &mut PanelState) {
    if state.cursor > 0 {
        state.cursor -= 1;
    }
}

fn move_cursor_down(state: &mut PanelState) {
    let item_count = state.items().len();
    if item_count > 0 {
        state.cursor = (state.cursor + 1).min(item_count - 1);
    }
}

fn toggle_value<T: Eq + Copy>(values: &mut Vec<T>, value: T, keep_one: bool) {
    if let Some(index) = values.iter().position(|existing| *existing == value) {
        if !keep_one || values.len() > 1 {
            values.remove(index);
        }
    } else {
        values.push(value);
    }
}

fn toggle_string(values: &mut Vec<String>, value: &str) {
    if let Some(index) = values.iter().position(|existing| existing == value) {
        values.remove(index);
    } else {
        values.push(value.to_string());
    }
}

fn marker(selected: bool) -> &'static str {
    if selected {
        "[x]"
    } else {
        "[ ]"
    }
}

fn lockfile_label(exists: bool) -> &'static str {
    if exists {
        "present"
    } else {
        "missing"
    }
}

fn join_display<T: ToString>(values: &[T]) -> String {
    if values.is_empty() {
        return "-".to_string();
    }
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn join_strings(values: &[String]) -> String {
    if values.is_empty() {
        "-".to_string()
    } else {
        values.join(",")
    }
}
