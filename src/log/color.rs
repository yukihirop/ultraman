use crate::log::{now, Printable};
use crate::opt::DisplayOpts;
use colored::*;

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
    "bright_green",
];

#[derive(Default)]
pub struct Log {
    pub index: usize,
    pub opts: DisplayOpts,
}

unsafe impl Sync for Log {}
unsafe impl Send for Log {}

// https://teratail.com/questions/244925
impl Log {
    fn boxed(self) -> Box<Self> {
        Box::new(self)
    }

    pub fn boxed_new() -> Box<Self> {
        Self::default().boxed()
    }
}

impl Printable for Log {
    fn output(&self, proc_name: &str, content: &str) {
        let color = COLORS[self.index % COLORS.len()];

        if self.opts.is_timestamp {
            println!(
                "{3} {0:1$} | {2}",
                proc_name.color(color),
                self.opts.padding,
                content.color(color),
                now().color(color)
            )
        } else {
            println!(
                "{0:1$} | {2}",
                proc_name.color(color),
                self.opts.padding,
                content.color(color),
            )
        }
    }

    fn error(&self, proc_name: &str, err: &dyn std::error::Error) {
        let content = &format!("error: {:?}", err);
        self.output(proc_name, content);
    }
}
