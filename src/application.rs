use std::{
    io::{ Stdout, stdout },
    time::{ Instant, Duration },
    sync::mpsc,
    thread,
};

use tui::{
    backend::CrosstermBackend,
    Terminal,
    Frame,
    layout::{ Constraint, Direction, Layout, Alignment },
    widgets::{
        Block,
        BorderType,
        Paragraph,
        Borders
    },
    style::{ Style, Color }
};

use crossterm::{
    terminal::{
        enable_raw_mode,
        disable_raw_mode
    },
    event::{
        self,
        Event as CEvent,
        KeyCode,
        KeyEvent
    }
};

pub struct Application {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

pub enum Event<I> {
    Input(I),
    Tick,
}

impl Application {
    pub fn new() -> Application {
        let stdout = stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).expect("Can get term");

        Application {
            terminal,
        }
    }

    pub fn init(self) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let (sender, receiver) = mpsc::channel();
        let tick_rate = Duration::from_millis(200);
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).expect("Poll Works") {
                    println!("Poll");
                    if let CEvent::Key(key) = event::read().expect("Can read") {
                        println!("Sending key code");
                        sender.send(Event::Input(key)).expect("Can Send");
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if let Ok(_) = sender.send(Event::Tick) {
                        last_tick = Instant::now();
                    }
                }
            }
        });

        self.respond(receiver)
    }

    fn render(rect: &mut Frame<CrosstermBackend<Stdout>>) {
        let size = rect.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(2),
                    Constraint::Length(3),
                ].as_ref()
            )
            .split(size);

        let copyright = Paragraph::new("Test")
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("Copyright")
                    .border_type(BorderType::Plain)
            );

        rect.render_widget(copyright, chunks[2]);
    }

    fn respond(mut self: Self, receiver: mpsc::Receiver<Event<KeyEvent>>) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            self.terminal.draw(|rect| {
                Self::render(rect);
            })?;

            match receiver.recv()? {
                Event::Input(event) => match event.code {
                    KeyCode::Char('q') => {
                        disable_raw_mode()?;
                        self.terminal.show_cursor()?;
                        self.terminal.clear()?;
                        break;
                    },
                    _ => {}
                },
                Event::Tick => {}
            }
        }
       Ok(())
    }
}
