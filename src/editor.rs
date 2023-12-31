use crate::Terminal;
use crate::Document;
use crate::Row;
use std::env;
use std::time::{Duration, Instant};
use termion::{color,event::Key};

const STATUS_BG_COLOR: color::Rgb = color::Rgb(50,255,50);
const STATUS_FG_COLOR: color::Rgb = color::Rgb(10,10,10);
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize
}

struct StatusMessage {
    text: String,
    time: Instant
}
impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message
        }
    }
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage
}

impl Editor {
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("Ctrl-S = save | Ctrl-X = quit");
        let document = if args.len() > 1 {
            let file_name = &args[1];
            let doc = Document::open(&file_name);
            if doc.is_ok() { doc.unwrap() }
            else {
                initial_status = format!("ERR: Could not open file {}", file_name);
                Document::default()
            }
        } else {
            Document::default()
        };
        Self{
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            cursor_position: Position::default(),
            offset: Position::default(),
            document,
            status_message: StatusMessage::from(initial_status)
        }
    }
    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(&error);
            }
            if let Err(error) = self.process_keypress() {
                die(&error);
            }
            if self.should_quit {
                Terminal::cursor_position(&self.terminal.last_line());
                Terminal::clear_current_line();
                break;
            }
        }
    }
    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye.\r");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(&self.get_cursor_position());
        }
        Terminal::cursor_show();
        Terminal::flush()
    }
    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let modified_indicator = if self.document.is_dirty() {"*"} else {""};
        let mut file_name = if let Some(name) = &self.document.file_name {
            name.clone()
        } else {"[No Name]".to_string()};
        file_name.truncate(20); //todo: change this based on width
        status = format!("{}{} - {} lines", file_name, modified_indicator, self.document.len());
        let line_indicator = format!("{}/{}",self.cursor_position.y.saturating_add(1), self.document.len());
        let len = status.len() + line_indicator.len();
        if width > len {
            status.push_str(&" ".repeat(width - len));
        }
        status = format!("{}{}",status,line_indicator);
        status.truncate(width);
        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r",status);
        Terminal::reset_bg_color();
        Terminal::reset_fg_color();
    }
    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5,0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}",text);
        }
    }
    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Ctrl('x') => {
                if self.document.is_dirty() {
                    let new_name = self.prompt("Press Esc to quit without saving, Ctrl+C to cancel. Save as: ", Some(&self.document.file_name.clone().expect("Error:")));
                    if let Ok(Some(name)) = new_name {
                        self.document.file_name = Some(name);
                    //still check that it's Some bc otherwise we don't save
                        self.save();
                    }
                }
                
                //perhaps insert a way to not quit
                self.should_quit = true;
                
            },
            Key::Ctrl('s') => self.save(),
            Key::Up | Key::Down | Key::Left | Key::Right
            | Key::PageUp | Key::PageDown | Key::End
            | Key::Home => self.move_cursor(pressed_key),
            Key::Char(c) => {
                self.document.insert(&self.cursor_position, c);
                self.move_cursor(Key::Right);
            },
            Key::Delete => self.document.delete(&self.cursor_position),
            Key::Backspace | Key.Ctrl('h') => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.cursor_position);
                }
            }
            _ => ()
        }
        Ok(())
    }
    fn prompt(&mut self, prompt: &str, default: Option<&str>) -> Result<Option<String>, std::io::Error> {
        let mut result = match default {
            Some(text) => String::from(text),
            None => String::new()
        };
        loop {
            self.status_message = StatusMessage::from(format!("{} {}", prompt, result));
            self.refresh_screen()?;
            match Terminal::read_key()? {
                Key::Backspace | Key::Ctrl('h') => {
                    if !result.is_empty() {
                        result.truncate(result.len() - 1);
                    }
                },
                Key::Char('\n') => break,
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                },
                Key::Esc => return Ok(None),
                _ => ()
            }
        }
        self.status_message = StatusMessage::from(String::new());
        Ok(Some(result))
    }
    fn save(&mut self) {
        if self.document.file_name.is_none() {
            let new_name = self.prompt("Save as:", None).unwrap_or(None);
            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted.".to_string());
                return;
            }
            self.document.file_name = new_name;
        }
        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved successfully!".to_string());
        } else {
            self.status_message = StatusMessage::from("Error writing file!".to_string());
        }
    }
    fn move_cursor(&mut self, key: Key) {
        let terminal_height = self.terminal.size().height as usize;
        let Position {mut x, mut y} = self.cursor_position;
        let size = self.terminal.size();
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {0};
        match key {
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    x = if let Some(row) = self.document.row(y) {
                        row.len()
                    } else {0};
                }
            },
            Key::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            },
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            },
            Key::PageUp => {
                y = if y > terminal_height {
                    y - terminal_height
                } else {0};
            },
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y + terminal_height as usize
                } else {height};
            },
            Key::Home => x = 0,
            Key::End => x = width,
            _ => ()
        }
        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {0};
        if x > width {x = width;}
        self.cursor_position = Position {x, y};
        self.scroll();
    }
    fn scroll(&mut self) {
        let Position {x,y} = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }
    fn get_cursor_position(&self) -> Position {
        Position {
            x: self.cursor_position.x.saturating_sub(self.offset.x),
            y: self.cursor_position.y.saturating_sub(self.offset.y)
        }
    }
    fn draw_welcome_message(&self) {
        let mut welcome = format!("Hecto editor - version {}", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome = format!("~{}{}", spaces, welcome);
        welcome.truncate(width);
        println!("{}\r",welcome);
    }
    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x + width;
        let row = row.render(start,end);
        println!("{}\r", row);
    }
    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self.document.row(terminal_row as usize + self.offset.y) {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }
}
fn die(e: &std::io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}
