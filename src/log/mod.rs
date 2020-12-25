use crate::opt::DisplayOpts;
use chrono::Local;

pub mod color;
pub mod plain;

pub trait Printable {
    fn output(&self, proc_name: &str, content: &str);
    fn error(&self, proc_name: &str, err: &dyn std::error::Error);
}

pub struct Log;

#[derive(Clone)]
pub struct LogOpt {
    pub is_color: bool,
    pub padding: usize,
    pub is_timestamp: bool,
}

impl Log {
    pub fn new(index: usize, opt: &LogOpt) -> Box<dyn Printable + Sync + Send> {
        if opt.is_color {
            let mut color = color::Log::boxed_new();
            color.index = index;
            color.opts = Self::display_opts(opt);
            color
        } else {
            let mut plain = plain::Log::boxed_new();
            plain.index = index;
            plain.opts = Self::display_opts(opt);
            plain
        }
    }

    fn display_opts(opt: &LogOpt) -> DisplayOpts {
        DisplayOpts {
            padding: opt.padding,
            is_timestamp: opt.is_timestamp,
        }
    }
}

pub fn output(proc_name: &str, content: &str, index: Option<usize>, opt: &LogOpt) {
    let index = index.unwrap_or_else(|| 0);
    let log = Log::new(index, opt);
    log.output(proc_name, content)
}

pub fn error(proc_name: &str, err: &dyn std::error::Error, padding: Option<usize>, opt: &LogOpt) {
    let content = &format!("error: {:?}", err);
    if let Some(p) = padding {
        let remake_opt = LogOpt {
            is_color: opt.is_color,
            padding: p,
            is_timestamp: opt.is_timestamp,
        };
        output(proc_name, content, None, &remake_opt);
    } else {
        let remake_opt = LogOpt {
            is_color: opt.is_color,
            padding: proc_name.len() + 1,
            is_timestamp: opt.is_timestamp,
        };
        output(proc_name, content, None, &remake_opt);
    }
}

pub fn now() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow;
    use std::error::Error;
    use std::fmt;

    #[test]
    fn test_output_when_coloring() -> anyhow::Result<()> {
        let log = Log::new(
            0,
            &LogOpt {
                is_color: true,
                padding: 10,
                is_timestamp: true,
            },
        );
        log.output("output 1", "coloring");

        Ok(())
    }

    #[test]
    fn test_output_when_not_coloring() -> anyhow::Result<()> {
        let log = Log::new(
            0,
            &LogOpt {
                is_color: false,
                padding: 10,
                is_timestamp: true,
            },
        );
        log.output("output 1", "not coloring");

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
        let log = Log::new(
            0,
            &LogOpt {
                is_color: true,
                padding: 10,
                is_timestamp: true,
            },
        );
        log.error("test_app", &error);

        Ok(())
    }
}
