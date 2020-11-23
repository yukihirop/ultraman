use colored::*;
use std::env;

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
    "bright_green",
];

impl Log {
    pub fn new(index: usize, padding: usize) -> Self {
        match env::var("COLOR") {
            Ok(c) => {
                return Log {
                    index,
                    padding,
                    is_color: c == "true",
                }
            }
            Err(_) => {
                return Log {
                    index,
                    padding,
                    is_color: true,
                }
            }
        }
    }

    pub fn output(&self, proc_name: &str, content: &str) {
        if self.is_color {
            self.color_output(proc_name, content)
                .expect("failed color output");
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
        println!(
            "{0:1$} | {2}",
            name.color(color),
            self.padding,
            content.color(color)
        );
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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow;
    use std::env;
    use std::error::Error;
    use std::fmt;

    #[test]
    fn test_new_when_color_env_exist() -> anyhow::Result<()> {
        env::set_var("COLOR", "false");

        let result = Log::new(0, 10);
        assert_eq!(result.index, 0);
        assert_eq!(result.padding, 10);
        assert_eq!(result.is_color, false);

        Ok(())
    }

    #[test]
    fn test_new_when_color_env_do_not_exist() -> anyhow::Result<()> {
        let result = Log::new(0, 10);
        assert_eq!(result.index, 0);
        assert_eq!(result.padding, 10);
        assert_eq!(result.is_color, true);

        Ok(())
    }

    #[test]
    fn test_output_when_coloring() -> anyhow::Result<()> {
        let log = Log::new(0, 10);
        log.output("output 1", "coloring");

        Ok(())
    }

    #[test]
    fn test_output_when_not_coloring() -> anyhow::Result<()> {
        env::set_var("COLOR", "false");
        let log = Log::new(0, 10);
        log.output("output 2", "not colorinng");

        Ok(())
    }

    // https://www.366service.com/jp/qa/265b4c8f485bfeedef32947292211f12
    #[derive(Debug)]
    struct TestError<'a>(&'a str);
    impl<'a> Error for TestError<'a> {}
    impl<'a> fmt::Display for TestError<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.fmt(f)
        }
    }

    #[test]
    fn test_error() -> anyhow::Result<()> {
        let error = TestError("test error");
        let log = Log::new(0, 10);
        log.error("test_app", &error);

        Ok(())
    }
}
