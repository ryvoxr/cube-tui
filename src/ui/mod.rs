use super::app::*;
use crossterm::event::{self, Event, KeyCode};
use std::{
    error::Error,
    time::{Duration, Instant},
};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame, Terminal,
};

pub fn run<B: Backend>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
    let mut app = App::new(Duration::from_millis(100));
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = app
            .tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char(' ') => match app.timer.space_press() {
                        Some(mut t) => {
                            t.gen_stats(&app.times);
                            app.times.push(t);
                        }
                        None => (),
                    },
                    _ => (),
                }
            }
        }
        if last_tick.elapsed() >= app.tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // define chunks
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(40), Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(7),
                Constraint::Percentage(100),
            ]
            .as_ref(),
        )
        .split(chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Percentage(100)].as_ref())
        .split(chunks[1]);

    // render left side
    render_help_and_tools(f, app, left_chunks[0]);
    render_timer(f, app, left_chunks[1]);
    render_times(f, app, left_chunks[2]);

    // render right side
    let block = Block::default().title("Scramble").borders(Borders::ALL);
    f.render_widget(block, right_chunks[0]);
    let block = Block::default().title("Main").borders(Borders::ALL);
    f.render_widget(block, right_chunks[1]);
}

fn render_help_and_tools<B: Backend>(f: &mut Frame<B>, app: &mut App, layout_chunk: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(layout_chunk);

    let style = Style::default().fg(get_color_from_id(&app, ActiveBlock::Tools));
    let block = Block::default()
        .title("Tools")
        .borders(Borders::ALL)
        .style(style);
    f.render_widget(block, chunks[0]);

    let style = Style::default().fg(get_color_from_id(&app, ActiveBlock::Help));
    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .style(style);
    f.render_widget(block, chunks[1]);
}

fn render_timer<B: Backend>(f: &mut Frame<B>, app: &mut App, layout_chunk: Rect) {
    let text = format!("\n\n{}", app.timer.text());
    let style = Style::default().fg(get_color_from_id(&app, ActiveBlock::Timer));
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("Timer")
                .borders(Borders::ALL)
                .border_style(style),
        )
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, layout_chunk);
}

pub fn render_times<B: Backend>(f: &mut Frame<B>, app: &mut App, layout_chunk: Rect) {
    let selected_style = Style::default().add_modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(Color::White);
    let header_cells = ["i", "time", "ao5", "ao12"].iter().map(|h| Cell::from(*h));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.times.iter().enumerate().map(|(i, t)| {
        let ao5 = match t.ao5 {
            Some(v) => format!("{:.2}", v),
            None => "-".to_string(),
        };
        let ao12 = match t.ao12 {
            Some(v) => format!("{:.2}", v),
            None => "-".to_string(),
        };
        let cells = vec![
            i.to_string(),
            format!("{:.2}", t.time),
            format!("{}", ao5),
            format!("{}", ao12),
        ];
        Row::new(cells)
    });
    let table = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Table"))
        .highlight_style(selected_style)
        .widths(&[
            Constraint::Ratio(1, 10),
            Constraint::Ratio(3, 10),
            Constraint::Ratio(3, 10),
            Constraint::Ratio(3, 10),
        ]);
    f.render_stateful_widget(table, layout_chunk, &mut app.times_state);
}

pub fn get_color_from_id(app: &App, id: ActiveBlock) -> Color {
    let color;
    if let ActiveBlock::Timer = app.route.active_block {
        color = Color::LightGreen;
    } else if let ActiveBlock::Timer = app.route.selected_block {
        color = Color::Magenta;
    } else {
        color = Color::LightBlue;
    }

    color
}
