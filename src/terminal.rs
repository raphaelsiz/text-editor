use crate::Position;
use std::io::{stdin, stdout, Write};
use termion::{
    color,
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal}
};

pub struct Size {
    pub width: u16,
    pub height: u16
}
pub struct Terminal {
    size: Size,
    _stdout: RawTerminal<std::io::Stdout>
}
impl Terminal {
    pub fn default() -> Result<Self, std::io::Error> {
        let size = termion::terminal_size()?;
        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1.saturating_sub(2)
            },
            _stdout: stdout().into_raw_mode()?
        })
    }
    pub fn size(&self) -> &Size {
        &self.size
        //we don't want callers to modify terminal size so we don't mark it as public bc then it would be changeable from the outside
        //but we want to be able to read it
    }
    pub fn last_line(&self) -> Position {
        Position{x: 0,y:1+self.size().height as usize}
    }
    pub fn clear_screen() {
        print!("{}", termion::clear::All);
        //print!("\x1b[2J"); // \x1b = escape (27) J = erase in display. argument 2 = clear entire screen
        // \x1b[1J would clear up to cursor, \x1b[J would clear from cursor to end
    }
    pub fn clear_current_line() {
        print!("{}", termion::clear::CurrentLine);
    }
    pub fn set_bg_color(color: color::Rgb) {
        print!("{}", color::Bg(color));
    }
    pub fn reset_bg_color() {
        print!("{}", color::Bg(color::Reset));
    }
    pub fn set_fg_color(color: color::Rgb) {
        print!("{}", color::Fg(color));
    }
    pub fn reset_fg_color() {
        print!("{}", color::Fg(color::Reset));
    }
    #[allow(clippy::cast_possible_truncation)]
    pub fn cursor_position(position: &Position) {
        let Position{mut x, mut y} = position; //destructuring, creates let mut x and let mut y
        x = x.saturating_add(1); //prevents overflow but makes it so i can say 0,0. if i said 65000,65000 it would not add. but like what monitor could even have a terminal that big lmao
        y = y.saturating_add(1);
        let x = x as u16;
        let y = y as u16;
        print!("{}", termion::cursor::Goto(x,y));
    }
    pub fn cursor_hide(){
        print!("{}", termion::cursor::Hide);
    }
    pub fn cursor_show(){
        print!("{}", termion::cursor::Show);
    }
    pub fn flush() -> Result<(), std::io::Error> {
        stdout().flush()
    }
    pub fn read_key() -> Result<Key, std::io::Error> {
        loop {
            if let Some(key) = stdin().lock().keys().next() {
                return key;
            }
        }
    }
}