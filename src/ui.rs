use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Focus, InputMode};
use crate::metadata::TagField;

pub fn draw(f: &mut Frame, app: &mut App) {
    if app.show_help {
        draw_help(f);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(f.area());

    draw_file_browser(f, app, chunks[0]);
    draw_metadata_view(f, app, chunks[1]);
}

fn draw_file_browser(f: &mut Frame, app: &mut App, area: Rect) {
    let border_style = if app.focus == Focus::FileBrowser && app.input_mode == InputMode::Normal {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    // Calculate scroll offset to keep selected item visible (edge scrolling)
    // Account for borders (2 lines: top and bottom)
    let visible_height = area.height.saturating_sub(2) as usize;
    let selected = app.file_browser.selected;
    let total_items = app.file_browser.entries.len();

    // Edge scrolling: cursor reaches top/bottom before scrolling
    if total_items <= visible_height {
        // All items fit, no scrolling needed
        app.file_browser.scroll_offset = 0;
    } else if selected < app.file_browser.scroll_offset {
        // Cursor moved above visible area, scroll up
        app.file_browser.scroll_offset = selected;
    } else if selected >= app.file_browser.scroll_offset + visible_height {
        // Cursor moved below visible area, scroll down
        app.file_browser.scroll_offset = selected.saturating_sub(visible_height - 1);
    }
    // Otherwise, keep scroll_offset unchanged (edge scrolling behavior)

    let scroll_offset = app.file_browser.scroll_offset;

    let items: Vec<ListItem> = app
        .file_browser
        .entries
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(i, entry)| {
            let icon = if entry.is_dir { "📁" } else { "🎵" };
            let content = format!("{} {}", icon, entry.name);

            let style = if i == app.file_browser.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let current_path = app.file_browser.current_dir.to_string_lossy();
    let title = format!(" File Browser - {} ", current_path);

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        );

    f.render_widget(list, area);
}

fn draw_metadata_view(f: &mut Frame, app: &mut App, area: Rect) {
    let border_style = if app.focus == Focus::MetadataView && app.input_mode == InputMode::Normal {
        Style::default().fg(Color::Cyan)
    } else if app.input_mode == InputMode::Editing {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    // Update scroll offset if in editing mode
    if app.input_mode == InputMode::Editing {
        // Account for borders (2 chars: left and right)
        let text_area_width = area.width.saturating_sub(2) as usize;
        app.update_scroll(text_area_width);
    }

    if let Some(metadata) = &app.metadata {
        let fields = TagField::all();

        let items: Vec<ListItem> = fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let label = field.label();
                let value = metadata.get_field_value(i);

                let is_current = app.input_mode == InputMode::Editing && i == app.editing_field;

                let content = if is_current {
                    let mut spans = vec![
                        Span::styled(
                            format!("{:12}: ", label),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                    ];

                    // Show cursor in editing mode with scrolling
                    let char_count = value.chars().count();
                    if app.cursor_position <= char_count {
                        // Apply scroll offset to show only the visible portion
                        let visible_value: String = value.chars().skip(app.scroll_offset).collect();
                        let cursor_pos_in_view = app.cursor_position.saturating_sub(app.scroll_offset);

                        let before: String = visible_value.chars().take(cursor_pos_in_view).collect();
                        let after: String = visible_value.chars().skip(cursor_pos_in_view).collect();
                        spans.push(Span::raw(before));
                        spans.push(Span::styled(
                            "█",
                            Style::default().fg(Color::Yellow),
                        ));
                        spans.push(Span::raw(after));
                    } else {
                        let visible_value: String = value.chars().skip(app.scroll_offset).collect();
                        spans.push(Span::raw(visible_value));
                    }

                    Line::from(spans)
                } else {
                    Line::from(vec![
                        Span::styled(
                            format!("{:12}: ", label),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(value),
                    ])
                };

                let style = if is_current {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };

                ListItem::new(content).style(style)
            })
            .collect();

        let mode_indicator = match app.input_mode {
            InputMode::Normal => " [NORMAL] ",
            InputMode::Editing => " [EDITING - Press Enter to save, Esc to cancel] ",
        };

        let filename = app
            .current_file
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let title = format!(" {} - {}", filename, mode_indicator);

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(border_style),
            );

        f.render_widget(list, area);
    } else {
        let help_text = vec![
            Line::from(""),
            Line::from("No file selected"),
            Line::from(""),
            Line::from("Select an audio file from the file browser"),
            Line::from("to view and edit its metadata."),
            Line::from(""),
            Line::from("Supported formats:"),
            Line::from("  • MP3 (.mp3)"),
            Line::from("  • FLAC (.flac)"),
            Line::from("  • M4A/MP4/AAC (.m4a, .mp4, .aac)"),
        ];

        let paragraph = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Metadata ")
                    .border_style(border_style),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }
}

fn draw_help(f: &mut Frame) {
    let area = centered_rect(80, 80, f.area());

    let help_text = vec![
        Line::from(Span::styled(
            "ID3 Tagger TUI - Help",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  ↑/k         - Move up"),
        Line::from("  ↓/j         - Move down"),
        Line::from("  Enter       - Select file/directory"),
        Line::from("  Backspace   - Go to parent directory"),
        Line::from("  Tab         - Switch between file browser and metadata view"),
        Line::from(""),
        Line::from("Editing:"),
        Line::from("  e           - Enter edit mode (when metadata is focused)"),
        Line::from("  ↑/↓         - Move between fields (in edit mode)"),
        Line::from("  ←/→         - Move cursor (in edit mode)"),
        Line::from("  Enter       - Save changes (in edit mode)"),
        Line::from("  Esc         - Cancel editing (in edit mode)"),
        Line::from(""),
        Line::from("General:"),
        Line::from("  h           - Toggle this help screen"),
        Line::from("  q           - Quit application"),
        Line::from(""),
        Line::from("Supported Formats:"),
        Line::from("  • MP3  - ID3v2 tags"),
        Line::from("  • FLAC - Vorbis comments"),
        Line::from("  • M4A  - iTunes-style metadata"),
        Line::from(""),
        Line::from(Span::styled(
            "Press 'h' to close this help screen",
            Style::default().fg(Color::Yellow),
        )),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
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
