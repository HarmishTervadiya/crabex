use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    symbols::{block, border},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use serde::Deserialize;
use std::{
    io,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::io::AsyncBufReadExt;
use tokio::net::TcpStream;

#[derive(Debug, Deserialize, Clone)]
pub struct MarketData {
    pub bids: Vec<(u64, u64)>,
    pub asks: Vec<(u64, u64)>,
}

struct AppState {
    market_data: MarketData,
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen);

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app_state = Arc::new(Mutex::new(AppState {
        market_data: MarketData {
            bids: Vec::new(),
            asks: Vec::new(),
        },
    }));

    let network_state = Arc::clone(&app_state);

    tokio::spawn(async move {
        if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8000").await {
            let (reader, _) = stream.split();
            let mut buf_reader = tokio::io::BufReader::new(reader);
            let mut line = String::new();

            while let Ok(bytes) = buf_reader.read_line(&mut line).await {
                if bytes == 0 {
                    break;
                };
                if let Ok(parsed_data) = serde_json::from_str::<MarketData>(&line) {
                    let mut state = network_state.lock().unwrap();
                    state.market_data = parsed_data;
                }

                line.clear();
            }
        }
    });

    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| {
            let state = app_state.lock().unwrap();
            let data = &state.market_data;

            let bid_items: Vec<ListItem> = data
                .bids
                .iter()
                .map(|(price, qty)| ListItem::new(format!(" ${:<5} | Vol: {:>4}", price, qty)))
                .collect();

            let ask_items: Vec<ListItem> = data
                .asks
                .iter()
                .map(|(price, qty)| ListItem::new(format!(" ${:<5} | Vol: {:>4}", price, qty)))
                .collect();

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(f.size());

            let bid_list = List::new(bid_items)
                .block(
                    Block::default()
                        .title(" BIDS (Buyers) ")
                        .borders(Borders::ALL),
                )
                .style(Style::default().fg(Color::Green));

            let ask_list = List::new(ask_items)
                .block(
                    Block::default()
                        .title(" ASKS (Sellers) ")
                        .borders(Borders::ALL),
                )
                .style(Style::default().fg(Color::Red));

            f.render_widget(bid_list, chunks[0]);
            f.render_widget(ask_list, chunks[1]);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char(q) = key.code {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
