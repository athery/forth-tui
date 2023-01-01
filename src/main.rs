use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use forth_tui::{Forth, ForthResult};
use std::io::{self, StdoutLock};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::Spans;
use tui::widgets::{Block, Borders, Paragraph, Widget};
use tui::Frame;
use tui::Terminal;
use tui_textarea::TextArea;

/// App holds the state of the application
struct App {
    /// Forth evaluator
    pub forth: Forth,
    pub code_status: ForthResult,
    pub input_mode: InputMode,
}

impl Default for App {
    fn default() -> App {
        App {
            forth: Forth::new(),
            code_status: Ok(()),
            input_mode: InputMode::Edit,
        }
    }
}

impl App {
    fn toggle_input_mode(&mut self) {
        self.input_mode = match self.input_mode {
            InputMode::Edit => InputMode::Menu,
            InputMode::Menu => InputMode::Edit,
        }
    }
}

enum InputMode {
    Edit,
    Menu,
}

fn main() -> io::Result<()> {
    let mut terminal = init_terminal()?;

    // create app and run it
    let mut app = App::default();
    let res = run_app(&mut terminal, &mut app);

    // handle program exit
    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error : {:?}", err)
    }
    Ok(())
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<StdoutLock<'static>>>> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let mut textarea = TextArea::default();

    loop {
        terminal.draw(|f| ui(f, &mut textarea, &app))?;

        if let Event::Key(key) = crossterm::event::read()? {
            if key.code == KeyCode::Esc {
                app.toggle_input_mode();
            }

            if let InputMode::Menu = app.input_mode {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            } else {
                textarea.input(key);
                app.forth = Forth::new();
                app.code_status = app.forth.eval(&textarea.lines().join("\t"));
            };
        }
    }
    Ok(())
}

fn ui<B: Backend>(f: &mut Frame<B>, textarea: &mut TextArea, app: &App) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(f.size());

    let body_columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(40),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(sections[1]);

    let footer_columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(sections[2]);

    f.render_widget(title_widget(), sections[0]);
    f.render_widget(editor_widget(textarea, app), body_columns[0]);
    f.render_widget(definitions_widget(app), body_columns[1]);
    f.render_widget(stack_widget(app), body_columns[2]);
    f.render_widget(editor_message_widget(app), footer_columns[0]);
    f.render_widget(menu_widget(app), footer_columns[1])
}

fn status_color(app: &App) -> Color {
    match app.code_status {
        Err(forth_tui::Error::UnknownWord) => Color::Rgb(255, 164, 76),
        Err(_) => Color::LightRed,
        Ok(_) => Color::White,
    }
}

fn title_widget<'a>() -> Paragraph<'a> {
    Paragraph::new("Forth TUI")
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default().borders(Borders::ALL).style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        )
}

fn editor_widget<'a>(textarea: &'a mut TextArea, app: &App) -> impl Widget + 'a {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Editor")
        .border_style(Style::default().fg(status_color(app)));
    textarea.set_block(block);
    textarea.widget()
}

fn editor_message_widget(app: &App) -> Paragraph {
    let message = match app.code_status {
        Err(forth_tui::Error::DivisionByZero) => "Error: Cannot divide by 0",
        Err(forth_tui::Error::InvalidWord) => "Error: Invalid word definition",
        Err(forth_tui::Error::StackUnderflow) => "Error: Stack underflow",
        Err(forth_tui::Error::UnknownWord) => "Unknown word, type on :)",
        Ok(_) => "",
    };

    Paragraph::new(message)
        .style(Style::default().fg(status_color(app)))
        .alignment(Alignment::Left)
}

fn menu_widget(app: &App) -> Paragraph {
    let text = match app.input_mode {
        InputMode::Edit => "[ESC] Access menu",
        InputMode::Menu => "[q] Quit , [ESC] Resume editing",
    };
    Paragraph::new(text).alignment(Alignment::Right)
}

fn definitions_widget(app: &App) -> Paragraph {
    let definition_items: Vec<Spans> = app
        .forth
        .definitions
        .iter()
        .map(|d| Spans::from(format!("{} : {}", d.name, d.instructions.join(" "))))
        .collect();
    Paragraph::new(definition_items)
        .block(Block::default().title("Definitions").borders(Borders::ALL))
}

fn stack_widget(app: &App) -> Paragraph {
    let stack_items: Vec<Spans> = app
        .forth
        .stack
        .iter()
        .map(|v| Spans::from(format!("{}", v)))
        .collect();
    Paragraph::new(stack_items).block(Block::default().title("Stack").borders(Borders::ALL))
}
