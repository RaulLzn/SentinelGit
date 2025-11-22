use crate::core::GitRepository;
use crate::features::impact_radar::{self, ImpactScore};
use crate::features::interactive_rebase::{self, RebaseEntry};
use crate::features::smart_context;
use crate::sentinel::Sentinel;
use crate::ui::shelf::ShelfState;
use crate::ui::zen_mode::ZenState;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::path::Path;
use std::{io, time::Duration};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Run app loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
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

struct App {
    repo: Option<GitRepository>,
    files: Vec<FileItem>,
    logs: Vec<String>,
    selected_index: usize,
    // New Features
    zen_mode: ZenState,
    shelf: ShelfState,
    impact_score: Option<ImpactScore>,
    smart_prefix: String,
    rebase_commits: Vec<RebaseEntry>,
}

impl App {
    fn new() -> App {
        let mut files = vec![];
        let mut logs = vec!["Welcome to SentinelGit".to_string()];
        let mut shelf = ShelfState::new();
        let mut impact_score = None;
        let mut smart_prefix = String::new();
        let mut rebase_commits = vec![];
        let mut repo_opt = None;

        // Initialize Git Repository
        match GitRepository::open(".") {
            Ok(mut repo) => {
                logs.push("Repository opened successfully.".to_string());
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
                    Err(e) => logs.push(format!("Error fetching status: {}", e)),
                }

                // Initialize Features
                shelf.refresh(&mut repo);
                impact_score = impact_radar::scan_changes(&repo);
                smart_prefix = smart_context::suggest_prefix(&repo);
                rebase_commits = interactive_rebase::load_commits(&repo);

                repo_opt = Some(repo);
            }
            Err(e) => logs.push(format!("Failed to open repository: {}", e)),
        }

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
                    if !issues.is_empty() {
                        self.logs
                            .push(format!("Issues found in {}: {:?}", file.path, issues));
                    }
                }
                Err(e) => {
                    self.logs
                        .push(format!("Error scanning {}: {}", file.path, e));
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
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('z') => app.zen_mode.toggle(),
                    KeyCode::Char(' ') => {
                        if let Some(repo) = &app.repo {
                            if let Some(file) = app.files.get_mut(app.selected_index) {
                                // Sentinel Check
                                if !file.issues.is_empty() {
                                    app.logs.push(format!("🚫 BLOCKED: {} has security issues. Fix them before staging.", file.path));
                                } else {
                                    // Stage
                                    if let Err(e) = repo.add(&[&file.path]) {
                                        app.logs
                                            .push(format!("Error staging {}: {}", file.path, e));
                                    } else {
                                        file.status = "Staged".to_string(); // Visual update
                                        app.logs.push(format!("✅ Staged: {}", file.path));
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    if app.zen_mode.active {
        // Zen Mode: Only show file list
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

        let files_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Files (Zen Mode)"),
            )
            .highlight_symbol(">> ")
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        f.render_stateful_widget(
            files_list,
            f.size(),
            &mut ratatui::widgets::ListState::default(),
        );
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

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

    let files_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Files"))
        .highlight_symbol(">> ")
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.selected_index));
    f.render_stateful_widget(files_list, chunks[0], &mut state);

    // Right Panel: Logs + Features
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(40), // Logs
                Constraint::Percentage(20), // Impact & Smart Context
                Constraint::Percentage(20), // Shelf
                Constraint::Percentage(20), // Rebase
            ]
            .as_ref(),
        )
        .split(chunks[1]);

    let logs_paragraph = Paragraph::new(app.logs.join("\n"))
        .block(Block::default().borders(Borders::ALL).title("Logs"));
    f.render_widget(logs_paragraph, right_chunks[0]);

    // Impact & Smart Context
    let impact_text = if let Some(score) = &app.impact_score {
        format!("Impact: {} ({:.1})", score.level, score.score)
    } else {
        "Impact: N/A".to_string()
    };
    let context_text = format!("Suggested Scope: {}", app.smart_prefix);
    let info_paragraph = Paragraph::new(format!("{}\n{}", impact_text, context_text))
        .block(Block::default().borders(Borders::ALL).title("Analysis"));
    f.render_widget(info_paragraph, right_chunks[1]);

    // Shelf
    let shelf_items: Vec<ListItem> = app
        .shelf
        .stashes
        .iter()
        .map(|s| ListItem::new(s.as_str()))
        .collect();
    let shelf_list = List::new(shelf_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Shelf (Stash)"),
    );
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
