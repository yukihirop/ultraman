use colored::*;

pub struct Log {
    index: usize,
    padding: usize,
    is_color: bool,
}

const COLORS: [&str; 12] = [
    "yellow",
    "cyan",
    "magenta",
    "white",
    "red",
    "green",
    "bright_yellow",
    "bright_magenta",
    "bright_cyan",
    "bright_white",
    "bright_red",
    "bright_green"
];

impl Log {
    pub fn new(index: usize, padding: usize, is_color: bool) -> Self {
        Log {
            index,
            padding,
            is_color
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
        println!("{0:1$} | {2}", name.color(color), self.padding, content.color(color));
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
