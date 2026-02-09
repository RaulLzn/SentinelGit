use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use tui_textarea::{Input, Key, TextArea};

#[derive(Debug, Clone, PartialEq)]
pub enum WizardStep {
    Type,
    Scope,
    Description,
    Body,
    Footer,
    Confirmation,
}

pub struct CommitWizardState<'a> {
    pub step: WizardStep,
    pub type_input: TextArea<'a>,
    pub scope_input: TextArea<'a>,
    pub desc_input: TextArea<'a>,
    pub body_input: TextArea<'a>,
    pub footer_input: TextArea<'a>,
}

impl<'a> Default for CommitWizardState<'a> {
    fn default() -> Self {
        let mut type_input = TextArea::default();
        type_input.set_placeholder_text("feat");
        type_input.set_block(Block::default().borders(Borders::ALL).title(" Type "));

        let mut scope_input = TextArea::default();
        scope_input.set_placeholder_text("core");
        scope_input.set_block(Block::default().borders(Borders::ALL).title(" Scope "));

        let mut desc_input = TextArea::default();
        desc_input.set_placeholder_text("Add new feature");
        desc_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Description "),
        );

        let mut body_input = TextArea::default();
        body_input.set_placeholder_text("Detailed description...");
        body_input.set_block(Block::default().borders(Borders::ALL).title(" Body "));

        let mut footer_input = TextArea::default();
        footer_input.set_placeholder_text("Closes #123");
        footer_input.set_block(Block::default().borders(Borders::ALL).title(" Footer "));

        Self {
            step: WizardStep::Type,
            type_input,
            scope_input,
            desc_input,
            body_input,
            footer_input,
        }
    }
}

impl<'a> CommitWizardState<'a> {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn handle_input(&mut self, input: Input) -> bool {
        // Returns true if specific step action handled (like next/prev), false if text input handled
        match input {
            Input {
                key: Key::Enter, ..
            } => {
                self.next_step();
                true
            }
            Input { key: Key::Esc, .. } => {
                // Let parent handle close
                true
            }
            _ => {
                match self.step {
                    WizardStep::Type => {
                        self.type_input.input(input);
                    }
                    WizardStep::Scope => {
                        self.scope_input.input(input);
                    }
                    WizardStep::Description => {
                        self.desc_input.input(input);
                    }
                    WizardStep::Body => {
                        self.body_input.input(input);
                    }
                    WizardStep::Footer => {
                        self.footer_input.input(input);
                    }
                    WizardStep::Confirmation => {}
                }
                false
            }
        }
    }

    pub fn next_step(&mut self) {
        self.step = match self.step {
            WizardStep::Type => WizardStep::Scope,
            WizardStep::Scope => WizardStep::Description,
            WizardStep::Description => WizardStep::Body,
            WizardStep::Body => WizardStep::Footer,
            WizardStep::Footer => WizardStep::Confirmation,
            WizardStep::Confirmation => WizardStep::Confirmation, // Handled by App commit
        };
    }

    pub fn format_commit_message(&self) -> String {
        let type_ = self.type_input.lines().first().cloned().unwrap_or_default();
        let scope = self
            .scope_input
            .lines()
            .first()
            .cloned()
            .unwrap_or_default();
        let desc = self.desc_input.lines().first().cloned().unwrap_or_default();
        let body = self.body_input.lines().join("\n");
        let footer = self.footer_input.lines().join("\n");

        let header = if scope.is_empty() {
            format!("{}: {}", type_, desc)
        } else {
            format!("{}({}): {}", type_, scope, desc)
        };

        let mut msg = header;
        if !body.trim().is_empty() {
            msg.push_str("\n\n");
            msg.push_str(&body);
        }
        if !footer.trim().is_empty() {
            msg.push_str("\n\n");
            msg.push_str(&footer);
        }
        msg
    }
}

pub fn render(f: &mut Frame, area: Rect, state: &mut CommitWizardState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3), // Title
                Constraint::Length(3), // Input
                Constraint::Min(5),    // Preview/Help
            ]
            .as_ref(),
        )
        .split(area);

    let title_text = match state.step {
        WizardStep::Type => "Step 1/6: Commit Type",
        WizardStep::Scope => "Step 2/6: Scope (Optional)",
        WizardStep::Description => "Step 3/6: Short Description",
        WizardStep::Body => "Step 4/6: Detailed Body (Optional)",
        WizardStep::Footer => "Step 5/6: Footer/Breaking Changes (Optional)",
        WizardStep::Confirmation => "Step 6/6: Confirm Commit",
    };

    let title = Paragraph::new(title_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    match state.step {
        WizardStep::Type => f.render_widget(state.type_input.widget(), chunks[1]),
        WizardStep::Scope => f.render_widget(state.scope_input.widget(), chunks[1]),
        WizardStep::Description => f.render_widget(state.desc_input.widget(), chunks[1]),
        WizardStep::Body => f.render_widget(state.body_input.widget(), chunks[1]),
        WizardStep::Footer => f.render_widget(state.footer_input.widget(), chunks[1]),
        WizardStep::Confirmation => {
            let msg = state.format_commit_message();
            let p = Paragraph::new(msg)
                .block(Block::default().borders(Borders::ALL).title(" Preview "))
                .wrap(Wrap { trim: false });
            f.render_widget(p, chunks[1]);
        }
    }

    // Help / Context
    let help_text = match state.step {
        WizardStep::Type => {
            "Enter the type of change (feat, fix, docs, style, refactor, perf, test, chore)."
        }
        WizardStep::Scope => {
            "Enter the scope of this change (e.g., login, core, ui). Leave empty if global."
        }
        WizardStep::Description => {
            "Enter a short, imperative summary of the change (e.g., 'add login button')."
        }
        WizardStep::Body => "Enter a detailed description of the motivation and changes.",
        WizardStep::Footer => {
            "Enter any footer information, such as 'BREAKING CHANGE: ...' or 'Closes #123'."
        }
        WizardStep::Confirmation => "Press Enter to COMMIT. Press Esc to cancel.",
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    f.render_widget(help, chunks[2]);
}
