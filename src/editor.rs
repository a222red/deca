use std::io::{self, Write, Stdout};

use crossterm::{
    execute,
    queue,
    terminal::{enable_raw_mode, disable_raw_mode, size, SetSize, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
    cursor,
    style,
    event::{read, Event, KeyModifiers, KeyCode}
};

pub struct Editor {
    stdout: Stdout,
    size: (u16, u16),
    should_quit: bool,
    cursor: (u16, u16),
    buffer: Vec<(String, bool)>
}

//todo: Implement line wrap / cut-off
//todo: Implement highlight selection
//todo: Implement cut-copy-paste

impl Editor {
    pub fn new() -> Editor {
        Editor { stdout: io::stdout(), size: size().unwrap(), should_quit: false, cursor: (0, 0), buffer: vec![(String::new(), false)] }
    }

    pub fn run(&mut self) {
        enable_raw_mode().unwrap();
        execute!(self.stdout, cursor::Hide, EnterAlternateScreen).unwrap();

        while !self.should_quit {
            self.process_key();
            self.refresh();
        }

        self.cleanup();
    }

    fn refresh(&mut self) {
        self.draw_rows();
        self.draw_cursor();
        self.stdout.flush().unwrap();
    }

    fn clear_current_line(&mut self) {
        queue!(self.stdout, Clear(ClearType::CurrentLine)).unwrap()
    }

    fn draw_cursor(&mut self) {
        queue!(self.stdout, cursor::MoveTo(self.cursor.0, self.cursor.1), style::Print('█')).unwrap()
    }

    fn draw_rows(&mut self) {
        for i in 0..self.size.1 {
            queue!(self.stdout, cursor::MoveTo(0, i)).unwrap();

            if (i as usize) < self.buffer.len() {
                if self.buffer[i as usize].1 {
                    self.clear_current_line();
                    self.buffer[i as usize].1 = false;
                }

                if self.buffer[i as usize].0.len() > 0 {
                    queue!(self.stdout, style::Print(&self.buffer[i as usize].0)).unwrap()
                }
            } else {
                queue!(self.stdout, style::Print('~')).unwrap()
            }
        }
    }

    fn handle_backspace(&mut self) {
        let row = &mut self.buffer[self.cursor.1 as usize];
        
        if row.0.len() > 0 {
            row.0.pop();
            row.1 = true;

            self.cursor.0 -= 1;
        }
    }

    fn handle_enter(&mut self) {
        self.buffer[self.cursor.1 as usize].1 = true;

        let slice = self.buffer[self.cursor.1 as usize].0[(self.cursor.0 as usize)..].to_owned();
        self.cursor.1 += 1;
        self.buffer.insert(self.cursor.1 as usize, (slice, false));
        self.buffer[(self.cursor.1 - 1) as usize].0.truncate(self.cursor.0 as usize);

        self.cursor.0 = 0;
        self.buffer[(self.cursor.1 as usize)..].iter_mut().for_each(|(_, m)| *m = true);
    }

    fn handle_left(&mut self) {
        if self.cursor.0 == 0 {
            if self.cursor.1 > 0 {
                self.buffer[self.cursor.1 as usize].1 = true;
                self.cursor.1 -= 1;
                self.buffer[self.cursor.1 as usize].1 = true;
                self.cursor.0 = self.buffer[self.cursor.1 as usize].0.len() as u16;
            }
        } else {
            self.buffer[self.cursor.1 as usize].1 = true;
            self.cursor.0 -= 1;
        }
    }

    fn handle_right(&mut self) {
        if self.cursor.0 as usize == self.buffer[self.cursor.1 as usize].0.len() {
            if self.cursor.1 as usize + 1 < self.buffer.len() {
                self.buffer[self.cursor.1 as usize].1 = true;
                self.cursor.1 += 1;
                self.buffer[self.cursor.1 as usize].1 = true;
                self.cursor.0 = 0;
            }
        } else {
            self.buffer[self.cursor.1 as usize].1 = true;
            self.cursor.0 += 1;
        }
    }

    fn type_char(&mut self, c: char) {
        let row = &mut self.buffer[self.cursor.1 as usize];

        if (self.cursor.0 as usize) < row.0.len() {
            row.0.insert(self.cursor.0 as usize, c);
        } else {
            assert_eq!(self.cursor.0 as usize, row.0.len());
            row.0.push(c);
        }

        self.cursor.0 += 1;
    }

    fn type_str(&mut self, s: &str) {
        let row = &mut self.buffer[self.cursor.1 as usize];

        if (self.cursor.0 as usize) < row.0.len() {
            row.0.insert_str(self.cursor.0 as usize, s);
        } else {
            assert_eq!(self.cursor.0 as usize, row.0.len());
            row.0.push_str(s);
        }
        
        self.cursor.0 += s.len() as u16;
    }

    fn process_key(&mut self) {
        match read().unwrap() {
            Event::Key(k) => {
                if k.modifiers == KeyModifiers::NONE {
                    match k.code {
                        KeyCode::Char(c) => self.type_char(c),
                        KeyCode::Backspace => self.handle_backspace(),
                        KeyCode::Enter => self.handle_enter(),
                        KeyCode::Left => self.handle_left(),
                        KeyCode::Right => self.handle_right(),
                        _ => ()
                    }
                } else if k.modifiers == KeyModifiers::SHIFT {
                    match k.code {
                        KeyCode::Char(c) => {
                            self.type_char(c);
                        },
                        _ => ()
                    }
                } else if k.modifiers == KeyModifiers::CONTROL {
                    match k.code {
                        KeyCode::Char('v') => self.type_str("foo bar"),
                        KeyCode::Char('q') => self.should_quit = true,
                        _ => ()
                    }
                }
            },
            _ => ()
        }
    }

    pub fn cleanup(&mut self) {
        execute!(self.stdout, SetSize(self.size.0, self.size.1), LeaveAlternateScreen, cursor::Show).unwrap();
        disable_raw_mode().unwrap();
    }
}
