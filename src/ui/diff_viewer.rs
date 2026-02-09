use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use similar::{ChangeTag, TextDiff};

#[derive(Clone, Debug)]
pub struct Hunk {
    pub header: String,
    pub patch: String,
    pub lines: Vec<(ChangeTag, String)>,
}

pub struct DiffState {
    pub scroll: u16,
    pub max_scroll: u16,
    pub selected_hunk: usize,
    pub hunks: Vec<Hunk>,
}

impl Default for DiffState {
    fn default() -> Self {
        Self {
            scroll: 0,
            max_scroll: 0,
            selected_hunk: 0,
            hunks: Vec::new(),
        }
    }
}

impl DiffState {
    pub fn next_hunk(&mut self) {
        if !self.hunks.is_empty() && self.selected_hunk < self.hunks.len() - 1 {
            self.selected_hunk += 1;
            // TODO: adjust scroll to show selected hunk
        }
    }
    pub fn prev_hunk(&mut self) {
        if self.selected_hunk > 0 {
            self.selected_hunk -= 1;
        }
    }
}

pub fn compute_hunks(old: &str, new: &str, file_path: &str) -> Vec<Hunk> {
    let diff = TextDiff::from_lines(old, new);
    let mut hunks = Vec::new();

    // Use unified_diff logic to get patch, but we need structured access
    // Let's create patches manually for each group
    for group in diff.grouped_ops(3).iter() {
        let mut hunk_lines = Vec::new();
        let mut patch_content = String::new();

        // Calculate header info
        // We need start/len for old and new
        // First op in group gives start
        let first_op = group.first().unwrap();
        let old_start = first_op.old_range().start;
        let new_start = first_op.new_range().start;

        let mut old_count = 0;
        let mut new_count = 0;

        for op in group {
            for change in diff.iter_changes(op) {
                let tag = change.tag();
                let sign = match tag {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };
                let line = change.to_string(); // includes newline
                patch_content.push_str(&format!("{}{}", sign, line));

                hunk_lines.push((tag, line.trim_end().to_string()));

                match tag {
                    ChangeTag::Delete => old_count += 1,
                    ChangeTag::Insert => new_count += 1,
                    ChangeTag::Equal => {
                        old_count += 1;
                        new_count += 1;
                    }
                }
            }
        }

        // Header format: @@ -old_start,old_count +new_start,new_count @@
        // Note: line numbers are 1-based in header, 0-based in similar?
        // similar ranges are 0-based.
        let header = format!(
            "@@ -{},{} +{},{} @@",
            old_start + 1,
            old_count,
            new_start + 1,
            new_count
        );

        // Add header to patch content for `git apply`
        // Also need file headers!
        // git apply needs:
        // --- a/file
        // +++ b/file
        // @@ ... @@
        // content

        let full_patch = format!(
            "--- a/{}\n+++ b/{}\n{}\n{}",
            file_path, file_path, header, patch_content
        );

        hunks.push(Hunk {
            header,
            patch: full_patch,
            lines: hunk_lines,
        });
    }

    hunks
}

pub fn render_diff(f: &mut Frame, area: Rect, state: &mut DiffState) {
    if state.hunks.is_empty() {
        let p = Paragraph::new("No changes (or binary file)")
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(p, area);
        return;
    }

    let mut lines = Vec::new();
    let mut scroll_anchor = None; // (start_line, end_line)
    let mut current_line_idx = 0;

    for (i, hunk) in state.hunks.iter().enumerate() {
        let is_selected = i == state.selected_hunk;

        // Header
        let header_style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan)
        };

        lines.push(Line::from(vec![
            Span::styled(if is_selected { "> " } else { "  " }, header_style),
            Span::styled(&hunk.header, header_style),
        ]));

        for (tag, content) in &hunk.lines {
            // ... (keep existing loop logic but update it)
            // Wait, I can't put loop inside ReplacementContent easily if it's large.
            // I'll rewrite the loop.

            let style = match tag {
                ChangeTag::Delete => Style::default().fg(Color::Red),
                ChangeTag::Insert => Style::default().fg(Color::Green),
                ChangeTag::Equal => Style::default(),
            };

            let final_style = if !is_selected && *tag == ChangeTag::Equal {
                style.add_modifier(Modifier::DIM)
            } else {
                style
            };

            lines.push(Line::from(vec![
                Span::styled(
                    match tag {
                        ChangeTag::Delete => "- ",
                        ChangeTag::Insert => "+ ",
                        ChangeTag::Equal => "  ",
                    },
                    final_style,
                ),
                Span::styled(content, final_style),
            ]));
        }

        let start_line = current_line_idx;
        // Header + lines + separator
        let hunk_height = 1 + hunk.lines.len() as u16 + 1;
        current_line_idx += hunk_height;

        if is_selected {
            scroll_anchor = Some((start_line, start_line + hunk_height));
        }

        // Separator
        lines.push(Line::from(""));
    }

    let diff_text = Text::from(lines);
    let line_count = diff_text.lines.len() as u16;
    let height = area.height.saturating_sub(2);

    state.max_scroll = if line_count > height {
        line_count - height
    } else {
        0
    };

    if let Some((start, end)) = scroll_anchor {
        // Ensure the hunk is visible
        // If hunk starts before current scroll, move scroll up to start
        if start < state.scroll {
            state.scroll = start;
        }
        // If hunk ends after current view, move scroll down so end is visible
        // But start should take precedence if hunk is larger than view
        else if end > state.scroll + height {
            state.scroll = end.saturating_sub(height);
            // Re-check start to ensure top is visible if hunk is too big
            if start < state.scroll {
                state.scroll = start;
            }
        }
    }

    if state.scroll > state.max_scroll {
        state.scroll = state.max_scroll;
    }

    let paragraph = Paragraph::new(diff_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Interactive Stage: [Up/Down] Hunk, [s] Stage, [Esc] Close "),
        )
        .scroll((state.scroll, 0));

    f.render_widget(paragraph, area);
}
