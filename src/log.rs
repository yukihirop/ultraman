use term;
use std::sync::{Arc, Mutex};

pub struct Log {
    index: usize,
    padding: usize,
    is_color: bool,
    terminal: Arc<Mutex<Box<term::StdoutTerminal>>>,
}

const COLORS: [u32; 12] = [
    term::color::MAGENTA,
    term::color::CYAN,
    term::color::YELLOW,
    term::color::GREEN,
    term::color::BLUE,
    term::color::RED,
    term::color::BRIGHT_CYAN,
    term::color::BRIGHT_YELLOW,
    term::color::BRIGHT_GREEN,
    term::color::BRIGHT_MAGENTA,
    term::color::BRIGHT_RED,
    term::color::BRIGHT_BLUE
];

impl Log {
    pub fn new(index: usize, padding: usize, is_color: bool) -> Self {
        Log {
            index,
            padding,
            is_color,
            terminal: Arc::new(Mutex::new(term::stdout().unwrap())),
        }
    }

    pub fn output(&self, proc_name: &str, content: &str) {
        if self.is_color {
            self.color_output(proc_name, content).expect("failed color output");
        } else {
            println!("{0:1$} | {2}", proc_name, self.padding, content);
        }
    }

    pub fn error(&self, proc_name: &str, err: &dyn std::error::Error) {
        let content = &format!("error: {:?}", err);
        self.output(proc_name, content);
    }

    fn color_output(&self, name: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let color = COLORS[self.index % COLORS.len()];
        let mut terminal = self.terminal.lock().unwrap();
        terminal.fg(color)?;
        write!(terminal, "{0:1$} | {2}\n", name, self.padding, content)?;
        terminal.reset()?;
        terminal.flush()?;
        Ok(())
    }
}

pub fn output(proc_name: &str, content: &str, padding: usize) {
    println!("{0:1$} | {2}", proc_name, padding, content);
}

pub fn error(proc_name: &str, err: &dyn std::error::Error, padding: usize) {
    let content = &format!("error: {:?}", err);
    if padding == 0 {
        output(proc_name, content, proc_name.len() + 1);
    } else {
        output(proc_name, content, padding);
    }
}
