use ratatui::{
    layout::{Constraint, Direction, Layout, Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Clear},
    Frame,
};
use crate::app::App;

pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(f.size());

    // Search box only if search_mode is active
    if app.search_mode {
        let search_text = format!("/{}", app.search);
        let search = Paragraph::new(search_text)
            .block(Block::default().borders(Borders::ALL).title(" Search "))
            .style(Style::default().fg(app.theme.foreground).bg(app.theme.background));
        f.render_widget(search, chunks[0]);
    } else {
        let search_text = if app.search.is_empty() { "Type / to search...".into() } else { format!("Filter: {}", app.search) };
        let search = Paragraph::new(search_text)
            .block(Block::default().borders(Borders::ALL).title(" Search "))
            .style(Style::default().fg(app.theme.foreground).bg(app.theme.background));
        f.render_widget(search, chunks[0]);
    }

    // Table header
    let header = Row::new(vec![
        Cell::from("Local Address"),
        Cell::from("Remote Address"),
        Cell::from("State"),
        Cell::from("PID"),
        Cell::from("Process"),
    ]).style(Style::default().fg(app.theme.header_fg).add_modifier(Modifier::BOLD));

    // Table rows
    let selected_idx = app.selected_index();
    let rows: Vec<Row> = app.filtered_ports.iter().enumerate().map(|(i, p)| {
        let style = if i == selected_idx {
            Style::default().bg(app.theme.highlight_bg).fg(app.theme.highlight_fg)
        } else {
            Style::default().fg(app.theme.foreground).bg(app.theme.background)
        };
        let state_color = match p.state.as_str() {
            "LISTEN" => Color::Green,
            "ESTABLISHED" => Color::Magenta,
            "TIME_WAIT" => Color::Yellow,
            _ => app.theme.foreground,
        };
        let reserved = if let Some(port) = p.local_addr.split(':').last() {
            if let Ok(port_num) = port.parse::<u16>() {
                if app.reservation_manager.is_reserved(port_num) {
                    "(reserved)"
                } else {
                    ""
                }
            } else { "" }
        } else { "" };
        Row::new(vec![
            Cell::from(format!("{} {}", p.local_addr.clone(), reserved)),
            Cell::from(p.remote_addr.clone()),
            Cell::from(p.state.clone()).style(Style::default().fg(state_color)),
            Cell::from(p.pid.map_or("-".into(), |pid| pid.to_string())),
            Cell::from(p.process.clone().unwrap_or_else(|| "-".into())),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Percentage(20),
    ])
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Open Ports "))
    .highlight_style(Style::default().fg(Color::Yellow).bg(Color::Blue));

    f.render_widget(table, chunks[1]);

    // Status bar
    let status_text = format!("j/k: move  |  c: kill  |  r: reserve  |  u: unreserve  |  q: quit  |  Filter: {}", app.search);
    let status = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));
    f.render_widget(status, chunks[2]);

    // Reservation popup
    if let Some((port, service)) = &app.reservation_popup {
        let area = centered_rect(50, 20, f.size());
        let msg = format!("Port {} reserved for '{}'. Press any key to continue.", port, service);
        let popup = Paragraph::new(msg)
            .block(Block::default().borders(Borders::ALL).title(" Reservation "))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
        f.render_widget(Clear, area);
        f.render_widget(popup, area);
    }
    if let Some(err) = &app.reservation_error {
        let area = centered_rect(50, 20, f.size());
        let popup = Paragraph::new(err.clone())
            .block(Block::default().borders(Borders::ALL).title(" Error "))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
        f.render_widget(Clear, area);
        f.render_widget(popup, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y)/2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y)/2)
        ])
        .split(r);
    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x)/2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x)/2)
        ])
        .split(vertical_layout[1]);
    horizontal_layout[1]
}

