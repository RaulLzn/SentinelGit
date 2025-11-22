use crate::core::GitRepository;
use crate::features::impact_radar::{self, ImpactScore};
use crate::features::interactive_rebase::{self, RebaseEntry};
use crate::features::smart_context;
use crate::sentinel::Sentinel;
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
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph}, // Importamos Clear para el popup
    Terminal,
};
use std::path::Path;
use std::{io, time::Duration};
use tui_textarea::{Input, Key, TextArea}; // <--- Nueva Importación

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
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

    // Features
    zen_mode: ZenState,
    shelf: ShelfState,
    impact_score: Option<ImpactScore>,
    smart_prefix: String,
    rebase_commits: Vec<RebaseEntry>,

    // Commit Modal State
    show_commit_modal: bool,
    commit_input: TextArea<'a>,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        let mut files = vec![];
        let mut logs = vec!["Welcome to SentinelGit v0.1.0".to_string()];
        let mut shelf = ShelfState::new();
        let mut impact_score = None;
        let mut smart_prefix = String::new();
        let mut rebase_commits = vec![];
        let mut repo_opt = None;

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
            zen_mode: ZenState::new(),
            shelf,
            impact_score,
            smart_prefix,
            rebase_commits,
            show_commit_modal: false,
            commit_input: textarea,
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
            match Sentinel::scan_file(path) {
                Ok(issues) => {
                    file.issues = issues.clone();
                }
                Err(_) => {}
            }
        }
    }

    fn open_commit_modal(&mut self) {
        self.show_commit_modal = true;
        // Pre-rellenar con Smart Context (ej: "feat: ")
        self.commit_input = TextArea::default();
        self.commit_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Commit Message (Enter to Submit, Esc to Cancel) "),
        );
        self.commit_input.insert_str(&self.smart_prefix);
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
            let lines = self.commit_input.lines();
            let message = lines.join("\n");

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
                        self.show_commit_modal = false;
                        self.refresh_status(); // Recargar status completo
                    }
                    Err(e) => self.logs.push(format!("❌ Error en commit: {}", e)),
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
                } if !app.show_commit_modal => return Ok(()),

                // Lógica del Modal de Commit
                input if app.show_commit_modal => {
                    match input {
                        Input { key: Key::Esc, .. } => app.show_commit_modal = false,
                        Input {
                            key: Key::Enter, ..
                        } => app.perform_commit(),
                        _ => {
                            app.commit_input.input(input);
                        } // Escribir en el cuadro
                    }
                }

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
                } => app.open_commit_modal(), // <--- ABRIR MODAL

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

    // 2. Renderizar Modal encima si está activo
    if app.show_commit_modal {
        let area = centered_rect(60, 20, f.size());
        f.render_widget(Clear, area); // Limpiar lo de abajo
        f.render_widget(app.commit_input.widget(), area); // Pintar el input
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
