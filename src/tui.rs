use crate::history;
use crate::storage::{Workflow, WorkflowStore};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::io::stdout;

// ── Shared types ──────────────────────────────────────────────────────

#[derive(PartialEq, Clone, Copy)]
enum InputMode { Normal, Typing }

#[derive(PartialEq, Clone, Copy)]
enum CreateField { Name, Description, Command }

// ── Terminal helpers ──────────────────────────────────────────────────

fn init_terminal() -> std::io::Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = stdout().execute(LeaveAlternateScreen);
}

// ── CREATE / EDIT WORKFLOW TUI ────────────────────────────────────────

pub fn create_workflow_tui(existing: Option<&Workflow>) -> Option<(String, String, Vec<String>)> {
    let mut terminal = match init_terminal() {
        Ok(t) => t,
        Err(_) => return None,
    };

    let mut name = existing.map(|w| w.name.clone()).unwrap_or_default();
    let mut desc = existing.map(|w| w.description.clone()).unwrap_or_default();
    let mut commands: Vec<String> = existing.map(|w| w.commands.clone()).unwrap_or_default();
    let mut input = String::new();
    let mut mode = InputMode::Normal;
    let mut field = CreateField::Name;
    let mut cmd_selected: usize = 0;
    let mut show_history = false;
    let mut history_items: Vec<String> = Vec::new();
    let mut history_filtered: Vec<String> = Vec::new();
    let mut history_selected: usize = 0;
    let mut history_filter = String::new();
    let mut status_msg = String::new();

    let is_edit = existing.is_some();

    // If editing, start on command field
    if is_edit {
        field = CreateField::Command;
    }

    let result = loop {
        terminal.draw(|f| {
            let area = f.area();
            let title = if is_edit { " Edit Workflow " } else { " Create New Workflow " };
            let outer = Block::default()
                .title(title)
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan));
            let inner = outer.inner(area);
            f.render_widget(outer, area);

            let chunks = Layout::vertical([
                Constraint::Length(3), // name
                Constraint::Length(3), // desc
                Constraint::Min(6),   // commands / history
                Constraint::Length(2), // help
            ]).split(inner);

            // Name field
            let name_style = if field == CreateField::Name && mode == InputMode::Typing {
                Style::default().fg(Color::Yellow)
            } else { Style::default().fg(Color::White) };
            let name_block = Block::default().borders(Borders::ALL).title(" Name ")
                .border_style(if field == CreateField::Name { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) });
            let name_display = if field == CreateField::Name && mode == InputMode::Typing {
                format!("{}_", name)
            } else { name.clone() };
            f.render_widget(Paragraph::new(name_display).style(name_style).block(name_block), chunks[0]);

            // Description field
            let desc_style = if field == CreateField::Description && mode == InputMode::Typing {
                Style::default().fg(Color::Yellow)
            } else { Style::default().fg(Color::White) };
            let desc_block = Block::default().borders(Borders::ALL).title(" Description ")
                .border_style(if field == CreateField::Description { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) });
            let desc_display = if field == CreateField::Description && mode == InputMode::Typing {
                format!("{}_", desc)
            } else { desc.clone() };
            f.render_widget(Paragraph::new(desc_display).style(desc_style).block(desc_block), chunks[1]);

            // Commands area
            if show_history {
                // Split into commands list (left) and history browser (right)
                let hsplit = Layout::horizontal([
                    Constraint::Percentage(45),
                    Constraint::Percentage(55),
                ]).split(chunks[2]);

                render_command_list(f, &commands, cmd_selected, hsplit[0]);
                render_history_browser(f, &history_filtered, history_selected, &history_filter, hsplit[1]);
            } else {
                // Split into commands list and input
                let vsplit = Layout::vertical([
                    Constraint::Min(3),
                    Constraint::Length(3),
                ]).split(chunks[2]);

                render_command_list(f, &commands, cmd_selected, vsplit[0]);

                // Input field
                let input_block = Block::default().borders(Borders::ALL).title(" Add Command ")
                    .border_style(if field == CreateField::Command { Style::default().fg(Color::Green) } else { Style::default().fg(Color::DarkGray) });
                let input_display = if field == CreateField::Command && mode == InputMode::Typing {
                    format!("{}_", input)
                } else if field == CreateField::Command {
                    "Press Enter to type, ↑ to browse history".to_string()
                } else { input.clone() };
                let input_style = if field == CreateField::Command && mode == InputMode::Typing {
                    Style::default().fg(Color::Green)
                } else { Style::default().fg(Color::DarkGray) };
                f.render_widget(Paragraph::new(input_display).style(input_style).block(input_block), vsplit[1]);
            }

            // Help bar
            let help = if show_history {
                " ↑↓ Navigate │ Enter Select │ Type to filter │ Esc Back "
            } else if mode == InputMode::Typing && field == CreateField::Command {
                " Enter Add cmd │ ↑ History │ Esc Cancel "
            } else if mode == InputMode::Typing {
                " Enter Confirm │ Esc Cancel "
            } else if field == CreateField::Command {
                " Enter Type │ ↑ History │ Tab Switch field │ Del Remove cmd │ Ctrl+S Save │ Esc Quit "
            } else {
                " Enter Edit │ Tab Next field │ Ctrl+S Save │ Esc Quit "
            };
            let help_with_status = if !status_msg.is_empty() {
                format!("  {}  │{}", status_msg, help)
            } else { help.to_string() };
            f.render_widget(
                Paragraph::new(help_with_status)
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Center),
                chunks[3],
            );
        }).ok();

        if let Ok(Event::Key(key)) = event::read() {
            if key.kind != KeyEventKind::Press { continue; }
            status_msg.clear();

            // Ctrl+S saves from anywhere
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
                if name.trim().is_empty() {
                    status_msg = "⚠ Name required!".to_string();
                    continue;
                }
                if commands.is_empty() {
                    status_msg = "⚠ Add at least one command!".to_string();
                    continue;
                }
                break Some((name.clone(), desc.clone(), commands.clone()));
            }

            if show_history {
                match key.code {
                    KeyCode::Esc => { show_history = false; }
                    KeyCode::Up => { if history_selected > 0 { history_selected -= 1; } }
                    KeyCode::Down => { if !history_filtered.is_empty() && history_selected < history_filtered.len() - 1 { history_selected += 1; } }
                    KeyCode::Enter => {
                        if !history_filtered.is_empty() {
                            let cmd = history_filtered[history_selected].clone();
                            commands.push(cmd);
                            cmd_selected = commands.len().saturating_sub(1);
                        }
                        show_history = false;
                        history_filter.clear();
                    }
                    KeyCode::Char(c) => {
                        history_filter.push(c);
                        history_filtered = history::filter_history(&history_items, &history_filter);
                        history_selected = 0;
                    }
                    KeyCode::Backspace => {
                        history_filter.pop();
                        history_filtered = history::filter_history(&history_items, &history_filter);
                        history_selected = 0;
                    }
                    _ => {}
                }
                continue;
            }

            match mode {
                InputMode::Typing => match key.code {
                    KeyCode::Esc => { mode = InputMode::Normal; }
                    KeyCode::Enter => {
                        match field {
                            CreateField::Name => { mode = InputMode::Normal; }
                            CreateField::Description => { mode = InputMode::Normal; }
                            CreateField::Command => {
                                let trimmed = input.trim().to_string();
                                if !trimmed.is_empty() {
                                    commands.push(trimmed);
                                    cmd_selected = commands.len().saturating_sub(1);
                                    input.clear();
                                }
                            }
                        }
                    }
                    KeyCode::Up if field == CreateField::Command => {
                        mode = InputMode::Normal;
                        show_history = true;
                        history_items = history::read_shell_history();
                        history_filtered = history_items.clone();
                        history_selected = 0;
                        history_filter.clear();
                    }
                    KeyCode::Backspace => {
                        match field {
                            CreateField::Name => { name.pop(); }
                            CreateField::Description => { desc.pop(); }
                            CreateField::Command => { input.pop(); }
                        }
                    }
                    KeyCode::Char(c) => {
                        match field {
                            CreateField::Name => name.push(c),
                            CreateField::Description => desc.push(c),
                            CreateField::Command => input.push(c),
                        }
                    }
                    _ => {}
                },
                InputMode::Normal => match key.code {
                    KeyCode::Esc => { break None; }
                    KeyCode::Enter => { mode = InputMode::Typing; }
                    KeyCode::Tab => {
                        field = match field {
                            CreateField::Name => CreateField::Description,
                            CreateField::Description => CreateField::Command,
                            CreateField::Command => CreateField::Name,
                        };
                    }
                    KeyCode::BackTab => {
                        field = match field {
                            CreateField::Name => CreateField::Command,
                            CreateField::Description => CreateField::Name,
                            CreateField::Command => CreateField::Description,
                        };
                    }
                    KeyCode::Up if field == CreateField::Command => {
                        show_history = true;
                        history_items = history::read_shell_history();
                        history_filtered = history_items.clone();
                        history_selected = 0;
                        history_filter.clear();
                    }
                    KeyCode::Delete | KeyCode::Char('d') if field == CreateField::Command => {
                        if !commands.is_empty() {
                            commands.remove(cmd_selected);
                            if cmd_selected > 0 && cmd_selected >= commands.len() {
                                cmd_selected = commands.len().saturating_sub(1);
                            }
                        }
                    }
                    KeyCode::Char('j') if field == CreateField::Command => {
                        if !commands.is_empty() && cmd_selected < commands.len() - 1 {
                            cmd_selected += 1;
                        }
                    }
                    KeyCode::Char('k') if field == CreateField::Command => {
                        if cmd_selected > 0 { cmd_selected -= 1; }
                    }
                    // Move command up
                    KeyCode::Char('K') if field == CreateField::Command => {
                        if cmd_selected > 0 {
                            commands.swap(cmd_selected, cmd_selected - 1);
                            cmd_selected -= 1;
                        }
                    }
                    // Move command down
                    KeyCode::Char('J') if field == CreateField::Command => {
                        if !commands.is_empty() && cmd_selected < commands.len() - 1 {
                            commands.swap(cmd_selected, cmd_selected + 1);
                            cmd_selected += 1;
                        }
                    }
                    _ => {}
                },
            }
        }
    };

    restore_terminal();
    result
}

// ── RUN WORKFLOW SELECTOR TUI ─────────────────────────────────────────

pub fn run_selector_tui(store: &WorkflowStore) -> Option<String> {
    if store.workflows.is_empty() {
        println!("  No workflows saved. Create one with: cmdflow new");
        return None;
    }

    let mut terminal = match init_terminal() {
        Ok(t) => t,
        Err(_) => return None,
    };

    let mut selected: usize = 0;
    let workflows = &store.workflows;

    let result = loop {
        terminal.draw(|f| {
            let area = f.area();
            let outer = Block::default()
                .title(" Select Workflow ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan));
            let inner = outer.inner(area);
            f.render_widget(outer, area);

            let chunks = Layout::horizontal([
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ]).split(inner);

            // Workflow list
            let items: Vec<ListItem> = workflows.iter().enumerate().map(|(i, w)| {
                let style = if i == selected {
                    Style::default().fg(Color::Cyan).bold()
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(format!("  {}", w.name)).style(style)
            }).collect();

            let mut list_state = ListState::default().with_selected(Some(selected));
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(" Workflows ").border_style(Style::default().fg(Color::Blue)))
                .highlight_symbol("▶ ")
                .highlight_style(Style::default().fg(Color::Cyan).bold());
            f.render_stateful_widget(list, chunks[0], &mut list_state);

            // Preview panel
            let wf = &workflows[selected];
            let mut preview_lines: Vec<Line> = Vec::new();
            preview_lines.push(Line::from(vec![
                Span::styled(" Name: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&wf.name, Style::default().fg(Color::White).bold()),
            ]));
            if !wf.description.is_empty() {
                preview_lines.push(Line::from(vec![
                    Span::styled(" Desc: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(&wf.description, Style::default().fg(Color::White)),
                ]));
            }
            preview_lines.push(Line::from(""));
            preview_lines.push(Line::from(Span::styled(" Commands:", Style::default().fg(Color::Green).bold())));

            for (i, cmd) in wf.commands.iter().enumerate() {
                preview_lines.push(Line::from(vec![
                    Span::styled(format!("  {}. ", i + 1), Style::default().fg(Color::DarkGray)),
                    Span::styled(cmd, Style::default().fg(Color::White)),
                ]));
            }

            preview_lines.push(Line::from(""));
            preview_lines.push(Line::from(Span::styled(
                format!(" Created: {}", wf.created_at.format("%Y-%m-%d %H:%M")),
                Style::default().fg(Color::DarkGray),
            )));

            let preview = Paragraph::new(preview_lines)
                .block(Block::default().borders(Borders::ALL).title(" Preview ").border_style(Style::default().fg(Color::Green)))
                .wrap(Wrap { trim: false });
            f.render_widget(preview, chunks[1]);
        }).ok();

        if let Ok(Event::Key(key)) = event::read() {
            if key.kind != KeyEventKind::Press { continue; }
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => { if selected > 0 { selected -= 1; } }
                KeyCode::Down | KeyCode::Char('j') => { if selected < workflows.len() - 1 { selected += 1; } }
                KeyCode::Enter => { break Some(workflows[selected].name.clone()); }
                KeyCode::Esc | KeyCode::Char('q') => { break None; }
                _ => {}
            }
        }
    };

    restore_terminal();
    result
}

// ── DELETE CONFIRMATION TUI ───────────────────────────────────────────

pub fn delete_confirm_tui(store: &WorkflowStore) -> Option<String> {
    if store.workflows.is_empty() {
        println!("  No workflows to delete.");
        return None;
    }

    let mut terminal = match init_terminal() {
        Ok(t) => t,
        Err(_) => return None,
    };

    let mut selected: usize = 0;
    let mut confirming = false;

    let result = loop {
        let workflows = &store.workflows;
        terminal.draw(|f| {
            let area = f.area();
            let outer = Block::default()
                .title(" Delete Workflow ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Red));
            let inner = outer.inner(area);
            f.render_widget(outer, area);

            if confirming {
                let wf = &workflows[selected];
                let confirm_area = centered_rect(50, 30, inner);
                f.render_widget(Clear, confirm_area);
                let block = Block::default().borders(Borders::ALL).title(" Confirm Delete ")
                    .border_style(Style::default().fg(Color::Red));
                let text = vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("  Delete workflow \"{}\"?", wf.name),
                        Style::default().fg(Color::White).bold(),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("  ({} commands will be removed)", wf.commands.len()),
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(""),
                    Line::from(Span::styled("  [Y] Yes  [N] No", Style::default().fg(Color::Yellow))),
                ];
                f.render_widget(Paragraph::new(text).block(block), confirm_area);
            } else {
                let items: Vec<ListItem> = workflows.iter().map(|w| {
                    ListItem::new(format!("  {} — {} cmd(s)", w.name, w.commands.len()))
                }).collect();
                let mut state = ListState::default().with_selected(Some(selected));
                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title(" Select workflow to delete ").border_style(Style::default().fg(Color::Red)))
                    .highlight_symbol("▶ ")
                    .highlight_style(Style::default().fg(Color::Red).bold());
                f.render_stateful_widget(list, inner, &mut state);
            }
        }).ok();

        if let Ok(Event::Key(key)) = event::read() {
            if key.kind != KeyEventKind::Press { continue; }
            if confirming {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        break Some(store.workflows[selected].name.clone());
                    }
                    _ => { confirming = false; }
                }
            } else {
                match key.code {
                    KeyCode::Up => { if selected > 0 { selected -= 1; } }
                    KeyCode::Down => { if selected < store.workflows.len() - 1 { selected += 1; } }
                    KeyCode::Enter => { confirming = true; }
                    KeyCode::Esc | KeyCode::Char('q') => { break None; }
                    _ => {}
                }
            }
        }
    };

    restore_terminal();
    result
}

// ── HELPER: render command list widget ────────────────────────────────

fn render_command_list(f: &mut Frame, commands: &[String], selected: usize, area: Rect) {
    let items: Vec<ListItem> = commands.iter().enumerate().map(|(i, cmd)| {
        let style = if i == selected {
            Style::default().fg(Color::Cyan).bold()
        } else {
            Style::default().fg(Color::White)
        };
        ListItem::new(format!(" {}. {}", i + 1, cmd)).style(style)
    }).collect();

    let mut state = ListState::default();
    if !commands.is_empty() {
        state.select(Some(selected));
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Commands ")
            .border_style(Style::default().fg(Color::Magenta)))
        .highlight_symbol("▶ ")
        .highlight_style(Style::default().fg(Color::Cyan).bold());
    f.render_stateful_widget(list, area, &mut state);
}

// ── HELPER: render history browser widget ─────────────────────────────

fn render_history_browser(f: &mut Frame, items: &[String], selected: usize, filter: &str, area: Rect) {
    let vsplit = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(3),
    ]).split(area);

    // Filter input
    let filter_display = if filter.is_empty() {
        "Type to filter...".to_string()
    } else {
        format!("{}_", filter)
    };
    let filter_style = if filter.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Yellow)
    };
    f.render_widget(
        Paragraph::new(filter_display).style(filter_style)
            .block(Block::default().borders(Borders::ALL).title(" Search History ")
                .border_style(Style::default().fg(Color::Yellow))),
        vsplit[0],
    );

    // History list
    let list_items: Vec<ListItem> = items.iter().enumerate().map(|(i, cmd)| {
        let style = if i == selected {
            Style::default().fg(Color::Yellow).bold()
        } else {
            Style::default().fg(Color::White)
        };
        ListItem::new(format!(" {}", cmd)).style(style)
    }).collect();

    let mut state = ListState::default();
    if !items.is_empty() {
        state.select(Some(selected));
    }

    let count = items.len();
    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL)
            .title(format!(" History ({}) ", count))
            .border_style(Style::default().fg(Color::Yellow)))
        .highlight_symbol("▶ ")
        .highlight_style(Style::default().fg(Color::Yellow).bold());
    f.render_stateful_widget(list, vsplit[1], &mut state);
}

// ── HELPER: centered rect ─────────────────────────────────────────────

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ]).split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ]).split(popup_layout[1])[1]
}
