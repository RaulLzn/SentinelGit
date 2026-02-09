use crate::chronos::storage::{ChronosStore, SnapshotInfo}; // Import Chronos types
use crate::config::Config;
use crate::core::GitRepository;
use crate::features::impact_radar::{self, ImpactScore};
use crate::features::interactive_rebase::{self, RebaseEntry};
use crate::features::smart_context;
use crate::sentinel::Sentinel;
use crate::ui::commit_wizard::CommitWizardState;
use crate::ui::diff_viewer::{self, DiffState};
use crate::ui::shelf::ShelfState;
use crate::ui::zen_mode::ZenState;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph}, // Added ListState
    Terminal,
};
use std::path::Path;
use std::{io, time::Duration};
use tui_textarea::{Input, Key, TextArea}; // <--- Nueva Importación

pub fn run(config: Config, store: ChronosStore) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(config, store);
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

struct FileItem {
    path: String,
    status: String,
    issues: Vec<String>,
}

struct App<'a> {
    // Agregamos lifetime para el TextArea
    repo: Option<GitRepository>,
    files: Vec<FileItem>,
    logs: Vec<String>,
    selected_index: usize,

    // Core components
    sentinel: Sentinel,
    #[allow(dead_code)] // Keep config for future use if needed
    config: Config,
    chronos: ChronosStore, // New field

    // Features
    zen_mode: ZenState,
    shelf: ShelfState,
    impact_score: Option<ImpactScore>,
    smart_prefix: String,
    rebase_commits: Vec<RebaseEntry>,

    // Commit Modal State
    // Commit Wizard State
    commit_wizard_active: bool,
    commit_wizard_state: crate::ui::commit_wizard::CommitWizardState<'a>,

    // History Modal State
    show_history_modal: bool,
    history_items: Vec<SnapshotInfo>,
    history_state: ListState,

    // Diff Modal State
    show_diff_modal: bool,
    diff_state: DiffState,
    diff_old: String,
    diff_new: String,

    // Time Machine Modal State
    show_time_machine_modal: bool,
    time_machine_events: Vec<(i64, String)>,
    time_machine_state: ListState,

    // Help Modal State
    show_help_modal: bool,
}

impl<'a> App<'a> {
    fn new(config: Config, store: ChronosStore) -> App<'a> {
        let mut files = vec![];
        let mut logs = vec!["Welcome to SentinelGit v0.1.0".to_string()];
        let mut shelf = ShelfState::new();
        let mut impact_score = None;
        let mut smart_prefix = String::new();
        let mut rebase_commits = vec![];
        let mut repo_opt = None;

        // Config is now passed in, no need to load it here.
        let sentinel = Sentinel::new(&config);

        match GitRepository::open(".") {
            Ok(mut repo) => {
                logs.push("Repository connected.".to_string());

                // Cargar Status
                match repo.status() {
                    Ok(statuses) => {
                        for (path, status) in statuses {
                            files.push(FileItem {
                                path,
                                status,
                                issues: vec![],
                            });
                        }
                    }
                    Err(e) => logs.push(format!("Status error: {}", e)),
                }

                // Cargar Features
                shelf.refresh(&mut repo);
                impact_score = impact_radar::scan_changes(&repo);
                smart_prefix = smart_context::suggest_prefix(&repo);
                rebase_commits = interactive_rebase::load_commits(&repo);

                repo_opt = Some(repo);
            }
            Err(e) => logs.push(format!("Failed to open repo: {}", e)),
        }

        // Inicializar TextArea vacío
        let mut textarea = TextArea::default();
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Commit Message "),
        );

        App {
            repo: repo_opt,
            files,
            logs,
            selected_index: 0,
            sentinel,
            config,
            chronos: store,
            zen_mode: ZenState::new(),
            shelf,
            impact_score,
            smart_prefix,
            rebase_commits,
            commit_wizard_active: false,
            commit_wizard_state: CommitWizardState::default(),
            show_history_modal: false,
            history_items: vec![],
            history_state: ListState::default(),
            show_diff_modal: false,
            diff_state: DiffState::default(),
            diff_old: String::new(),
            diff_new: String::new(),
            show_time_machine_modal: false,
            time_machine_events: vec![],
            time_machine_state: ListState::default(),
            show_help_modal: false,
        }
    }

    fn next(&mut self) {
        if !self.files.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.files.len();
            self.scan_selected();
        }
    }

    fn previous(&mut self) {
        if !self.files.is_empty() {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = self.files.len() - 1;
            }
            self.scan_selected();
        }
    }

    fn scan_selected(&mut self) {
        if let Some(file) = self.files.get_mut(self.selected_index) {
            let path = Path::new(&file.path);
            match self.sentinel.scan_file(path) {
                Ok(issues) => {
                    file.issues = issues.clone();
                }
                Err(_) => {}
            }
        }
    }

    fn open_commit_modal(&mut self) {
        self.commit_wizard_active = true;
        self.commit_wizard_state.reset();

        // Try to pre-fill type from smart_prefix (e.g. "feat:")
        let clean_prefix = self.smart_prefix.replace(":", "");
        let type_val = clean_prefix.trim();
        if !type_val.is_empty() {
            // Create new TextArea to reset content
            use tui_textarea::TextArea;
            let mut new_input = TextArea::default();
            new_input.set_block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title(" Type "),
            );
            new_input.insert_str(type_val);
            self.commit_wizard_state.type_input = new_input;
        }
    }

    fn refresh_status(&mut self) {
        if let Some(repo) = &self.repo {
            if let Ok(statuses) = repo.status() {
                self.files.clear();
                for (path, status) in statuses {
                    self.files.push(FileItem {
                        path,
                        status,
                        issues: vec![],
                    });
                }
            }
        }
    }

    fn perform_commit(&mut self) {
        if let Some(repo) = &self.repo {
            let message = self.commit_wizard_state.format_commit_message();

            if message.trim().is_empty() {
                self.logs
                    .push("❌ Commit abortado: Mensaje vacío.".to_string());
            } else {
                match repo.commit(&message) {
                    Ok(oid) => {
                        self.logs.push(format!(
                            "🚀 Commit exitoso: {} - {}",
                            &oid.to_string()[..7],
                            message
                        ));
                        self.commit_wizard_active = false;
                        self.commit_wizard_state.reset();
                        self.refresh_status(); // Recargar status completo
                    }
                    Err(e) => self.logs.push(format!("❌ Error en commit: {}", e)),
                }
            }
        }
    }

    fn load_history(&mut self) {
        if let Some(file) = self.files.get(self.selected_index) {
            match self.chronos.get_history(&file.path) {
                Ok(snapshots) => {
                    self.history_items = snapshots;
                    self.show_history_modal = true;
                    self.history_state = ListState::default();
                    if !self.history_items.is_empty() {
                        self.history_state.select(Some(0));
                    }
                    self.logs.push(format!(
                        "Loaded {} snapshots for {}",
                        self.history_items.len(),
                        file.path
                    ));
                }
                Err(e) => {
                    self.logs.push(format!("Error loading history: {}", e));
                }
            }
        }
    }

    fn restore_snapshot(&mut self) {
        if let Some(selected) = self.history_state.selected() {
            if let Some(snapshot) = self.history_items.get(selected) {
                if let Some(file) = self.files.get(self.selected_index) {
                    match self.chronos.get_snapshot(&file.path, snapshot.timestamp) {
                        Ok(Some(content)) => {
                            // Write content back to file
                            if let Err(e) = std::fs::write(&file.path, content) {
                                self.logs.push(format!("Error restoring file: {}", e));
                            } else {
                                self.logs
                                    .push(format!("Restored snapshot from {}", snapshot.timestamp));
                                self.show_history_modal = false;
                                self.refresh_status(); // File is now modified
                            }
                        }
                        Ok(None) => self.logs.push("Snapshot content not found.".to_string()),
                        Err(e) => self.logs.push(format!("Error retrieving snapshot: {}", e)),
                    }
                }
            }
        }
    }

    fn open_time_machine(&mut self) {
        match self.chronos.get_global_timeline(50) {
            Ok(events) => {
                self.time_machine_events = events;
                self.show_time_machine_modal = true;
                self.time_machine_state = ListState::default();
                if !self.time_machine_events.is_empty() {
                    self.time_machine_state.select(Some(0));
                }
                self.logs.push(format!(
                    "Loaded {} global timeline events. Welcome to Ghost Branches.",
                    self.time_machine_events.len()
                ));
            }
            Err(e) => {
                self.logs.push(format!("Error loading Time Machine: {}", e));
            }
        }
    }

    fn restore_time_machine(&mut self) {
        if let Some(selected) = self.time_machine_state.selected() {
            if let Some((timestamp, _)) = self.time_machine_events.get(selected) {
                // Restore ALL files to this point
                match self.chronos.get_checkpoint_state(*timestamp) {
                    Ok(files) => {
                        let count = files.len();
                        for (path, content) in files {
                            if let Err(e) = std::fs::write(&path, content) {
                                self.logs.push(format!("Failed to restore {}: {}", path, e));
                            }
                        }
                        self.logs.push(format!(
                            "👻 GHOST BRANCH ACTIVE: Restored {} files to state at {}",
                            count, timestamp
                        ));
                        self.show_time_machine_modal = false;
                        self.refresh_status();
                    }
                    Err(e) => self.logs.push(format!("Checkpoint restore failed: {}", e)),
                }
            }
        }
    }

    fn load_diff(&mut self) {
        if let Some(file) = self.files.get(self.selected_index) {
            let path = &file.path;

            // Get New Content (from working directory)
            let new_content = std::fs::read_to_string(path).unwrap_or_default();

            // Get Old Content (from HEAD)
            let old_content = if let Some(repo) = &self.repo {
                match repo.get_file_content_at_head(path) {
                    Ok(Some(content)) => content,
                    _ => String::new(),
                }
            } else {
                String::new()
            };

            self.diff_old = old_content;
            self.diff_new = new_content;
            self.diff_state.hunks =
                crate::ui::diff_viewer::compute_hunks(&self.diff_old, &self.diff_new, path);
            self.diff_state.selected_hunk = 0;
            self.diff_state.scroll = 0;
            self.show_diff_modal = true;
        }
    }

    fn stage_selected_hunk(&mut self) {
        if self.diff_state.hunks.is_empty() {
            return;
        }

        // Ensure selected_hunk is valid
        if self.diff_state.selected_hunk >= self.diff_state.hunks.len() {
            self.diff_state.selected_hunk = 0;
        }

        let hunk = &self.diff_state.hunks[self.diff_state.selected_hunk];
        // We need to apply this hunk.
        // We rely on the `apply_patch` method in `GitRepository`.
        // However, `GitRepository` is owned by `App` inside `Option<GitRepository>`.

        if let Some(repo) = &self.repo {
            match repo.apply_patch(&hunk.patch) {
                Ok(_) => {
                    self.logs.push("Hunk staged successfully.".to_string());
                    // Refresh diff to remove the staged hunk from view (or update it)
                    self.load_diff();
                }
                Err(e) => {
                    self.logs.push(format!("Failed to stage hunk: {}", e));
                }
            }
        }
    }
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if crossterm::event::poll(Duration::from_millis(250))? {
            match event::read()?.into() {
                Input {
                    key: Key::Char('q'),
                    ..
                } if !app.commit_wizard_active
                    && !app.show_history_modal
                    && !app.show_diff_modal
                    && !app.show_time_machine_modal =>
                {
                    return Ok(())
                }

                // Lógica del Modal de Commit
                // Lógica del Commit Wizard
                input if app.commit_wizard_active => {
                    use crate::ui::commit_wizard::WizardStep;

                    if input.key == Key::Esc {
                        app.commit_wizard_active = false;
                        app.logs.push("Commit wizard cancelled.".to_string());
                    } else {
                        let key = input.key;
                        if app.commit_wizard_state.handle_input(input) {
                            // Check ONLY if we just pressed Enter on Confirmation
                            // Note: handle_input returns true if it handled a navigation key (like Enter)
                            if key == Key::Enter
                                && app.commit_wizard_state.step == WizardStep::Confirmation
                            {
                                app.perform_commit();
                            }
                        }
                    }
                }

                // Lógica del Modal de Historial
                Input { key: Key::Esc, .. } if app.show_history_modal => {
                    app.show_history_modal = false;
                }
                Input {
                    key: Key::Enter, ..
                } if app.show_history_modal => {
                    app.restore_snapshot();
                }
                Input { key: Key::Down, .. } if app.show_history_modal => {
                    if !app.history_items.is_empty() {
                        let i = match app.history_state.selected() {
                            Some(i) => {
                                if i >= app.history_items.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        app.history_state.select(Some(i));
                    }
                }
                Input { key: Key::Up, .. } if app.show_history_modal => {
                    if !app.history_items.is_empty() {
                        let i = match app.history_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    app.history_items.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        app.history_state.select(Some(i));
                    }
                }

                // Lógica del Modal de Diff
                Input { key: Key::Esc, .. } if app.show_diff_modal => {
                    app.show_diff_modal = false;
                }
                Input { key: Key::Up, .. } if app.show_diff_modal => {
                    app.diff_state.prev_hunk();
                }
                Input { key: Key::Down, .. } if app.show_diff_modal => {
                    app.diff_state.next_hunk();
                }
                Input {
                    key: Key::Char('s'),
                    ..
                } if app.show_diff_modal => {
                    app.stage_selected_hunk();
                }
                _ if app.show_diff_modal => {}

                // Lógica del Time Machine Modal
                Input { key: Key::Esc, .. } if app.show_time_machine_modal => {
                    app.show_time_machine_modal = false;
                }
                Input {
                    key: Key::Enter, ..
                } if app.show_time_machine_modal => {
                    app.restore_time_machine();
                }
                Input { key: Key::Down, .. } if app.show_time_machine_modal => {
                    if !app.time_machine_events.is_empty() {
                        let i = match app.time_machine_state.selected() {
                            Some(i) => {
                                if i >= app.time_machine_events.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        app.time_machine_state.select(Some(i));
                    }
                }
                Input { key: Key::Up, .. } if app.show_time_machine_modal => {
                    if !app.time_machine_events.is_empty() {
                        let i = match app.time_machine_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    app.time_machine_events.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        app.time_machine_state.select(Some(i));
                    }
                }
                _ if app.show_time_machine_modal => {}

                // Block other inputs when history/diff modal is open
                _ if app.show_history_modal => {}

                // Lógica Normal (Navegación)
                Input { key: Key::Down, .. } => app.next(),
                Input { key: Key::Up, .. } => app.previous(),
                Input {
                    key: Key::Char('z'),
                    ..
                } => app.zen_mode.toggle(),
                Input {
                    key: Key::Char('c'),
                    ..
                } => app.open_commit_modal(),
                Input {
                    key: Key::Char('h'),
                    ..
                } => app.load_history(), // <--- NEW KEYBINDING
                Input {
                    key: Key::Char('d'),
                    ..
                } => app.load_diff(),
                Input {
                    key: Key::Char('t'),
                    ..
                } => app.open_time_machine(),

                Input {
                    key: Key::Char(' '),
                    ..
                } => {
                    // STAGE/UNSTAGE INTELIGENTE
                    if let Some(repo) = &app.repo {
                        if let Some(file) = app.files.get_mut(app.selected_index) {
                            if file.status.contains("Index") || file.status == "Staged" {
                                // UNSTAGE
                                if let Err(e) = repo.unstage(&file.path) {
                                    app.logs.push(format!("Error unstaging: {}", e));
                                } else {
                                    file.status = "Modified".to_string(); // Visual update (will be refreshed properly on next loop if we wanted, but immediate feedback is good)
                                    app.logs.push(format!("🔙 Unstaged: {}", file.path));
                                }
                            } else {
                                // STAGE
                                if !file.issues.is_empty() {
                                    app.logs.push(format!(
                                        "🚫 BLOQUEADO: {} tiene riesgos de seguridad.",
                                        file.path
                                    ));
                                } else {
                                    if let Err(e) = repo.add(&[&file.path]) {
                                        app.logs.push(format!("Error: {}", e));
                                    } else {
                                        file.status = "Staged".to_string();
                                        app.logs.push(format!("✅ Staged: {}", file.path));
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    // 1. Renderizar UI Base (Igual que antes)
    if app.zen_mode.active {
        render_zen_mode(f, app);
    } else {
        render_dashboard(f, app);
    }

    // 2. Renderizar Modal de Commit Wizard
    if app.commit_wizard_active {
        let area = centered_rect(70, 60, f.size());
        f.render_widget(Clear, area); // Limpiar lo de abajo
        crate::ui::commit_wizard::render(f, area, &mut app.commit_wizard_state);
    }

    // 3. Renderizar Modal de Historial
    if app.show_history_modal {
        let area = centered_rect(60, 60, f.size());
        f.render_widget(Clear, area);

        let items: Vec<ListItem> = app
            .history_items
            .iter()
            .map(|snap| {
                let dt = chrono::DateTime::from_timestamp_millis(snap.timestamp);
                let time_str = dt
                    .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                ListItem::new(format!("{} - Size: {} bytes", time_str, snap.size))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" File History (Enter to Restore / Esc to Close) "),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, area, &mut app.history_state);
    }

    // 5. Renderizar Time Machine Modal
    if app.show_time_machine_modal {
        let area = centered_rect(70, 70, f.size());
        f.render_widget(Clear, area);

        let items: Vec<ListItem> = app
            .time_machine_events
            .iter()
            .map(|(ts, path)| {
                let dt = chrono::DateTime::from_timestamp_millis(*ts);
                let time_str = dt
                    .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                ListItem::new(format!("{} - Modified: {}", time_str, path))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 👻 Ghost Branches (Time Machine) - Enter to Restore All "),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, area, &mut app.time_machine_state);
    }

    // 4. Renderizar Modal de Diff
    if app.show_diff_modal {
        let area = centered_rect(80, 80, f.size());
        f.render_widget(Clear, area);
        diff_viewer::render_diff(f, area, &mut app.diff_state);
    }

    if app.show_help_modal {
        let area = centered_rect(60, 60, f.size());
        f.render_widget(Clear, area);

        let help_text = vec![
            Line::from(Span::styled(
                "Keyboard Shortcuts",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Navigation:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
            Line::from("  ↑/↓    : Navigate files"),
            Line::from("  Space  : Stage/Unstage file"),
            Line::from(""),
            Line::from(Span::styled(
                "Features:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
            Line::from("  d      : View Diff (Interactive Staging)"),
            Line::from("    Use ↑/↓ to select hunk, 's' to stage hunk"),
            Line::from("  c      : Commit (Wizard)"),
            Line::from("  h      : File History"),
            Line::from("  t      : Time Machine (Ghost Branches)"),
            Line::from("  z      : Toggle Zen Mode"),
            Line::from(""),
            Line::from(Span::styled(
                "General:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
            Line::from("  ?      : Toggle Help"),
            Line::from("  q      : Quit"),
            Line::from("  Esc    : Close Modal / Cancel"),
        ];

        let p = Paragraph::new(Text::from(help_text)).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" SentinelGit Help "),
        );
        f.render_widget(p, area);
    }
}

// Funciones auxiliares de renderizado para mantener el código limpio
fn render_zen_mode(f: &mut ratatui::Frame, app: &mut App) {
    let items: Vec<ListItem> = app
        .files
        .iter()
        .map(|i| {
            let style = if !i.issues.is_empty() {
                Style::default().fg(Color::Red)
            } else {
                Style::default()
            };
            ListItem::new(format!("{} [{}]", i.path, i.status)).style(style)
        })
        .collect();
    f.render_stateful_widget(
        List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Zen Mode (Focus View - Press 'z' to exit)"),
        ),
        f.size(),
        &mut ratatui::widgets::ListState::default(),
    );
}

fn render_dashboard(f: &mut ratatui::Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.size());

    // Lista de archivos
    let items: Vec<ListItem> = app
        .files
        .iter()
        .map(|i| {
            let style = if !i.issues.is_empty() {
                Style::default().fg(Color::Red)
            } else {
                Style::default()
            };
            ListItem::new(format!("{} [{}]", i.path, i.status)).style(style)
        })
        .collect();

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.selected_index));
    f.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Files"))
            .highlight_symbol(">> "),
        chunks[0],
        &mut state,
    );

    // Panel Derecho
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(chunks[1]);

    f.render_widget(
        Paragraph::new(app.logs.join("\n"))
            .block(Block::default().borders(Borders::ALL).title("Logs")),
        right_chunks[0],
    );

    let info = format!(
        "Impact: {:?}\nScope: {}",
        app.impact_score.as_ref().map(|s| s.score).unwrap_or(0.0),
        app.smart_prefix
    );
    f.render_widget(
        Paragraph::new(info).block(Block::default().borders(Borders::ALL).title("Analysis")),
        right_chunks[1],
    );

    // Shelf
    let shelf_items: Vec<ListItem> = app
        .shelf
        .stashes
        .iter()
        .map(|s| ListItem::new(s.as_str()))
        .collect();
    let shelf_list =
        List::new(shelf_items).block(Block::default().borders(Borders::ALL).title("Shelf"));
    f.render_widget(shelf_list, right_chunks[2]);

    // Rebase
    let rebase_items: Vec<ListItem> = app
        .rebase_commits
        .iter()
        .map(|c| ListItem::new(format!("{} {}", c.id, c.message)))
        .collect();
    let rebase_list = List::new(rebase_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Recent Commits"),
    );
    f.render_widget(rebase_list, right_chunks[3]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
