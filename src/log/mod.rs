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
    pub is_timestamp: bool,
}

impl Log {
    pub fn new(index: usize, padding: usize, opt: &LogOpt) -> Box<dyn Printable + Sync + Send> {
        if opt.is_color {
            let mut color = color::Log::boxed_new();
            color.index = index;
            color.padding = padding;
            color.is_timestamp = opt.is_timestamp;
            color
        } else {
            let mut plain = plain::Log::boxed_new();
            plain.index = index;
            plain.padding = padding;
            plain.is_timestamp = opt.is_timestamp;
            plain
        }
    }
}

pub fn output(proc_name: &str, content: &str, padding: usize, index: Option<usize>, opt: &LogOpt) {
    let index = index.unwrap_or_else(|| 0);
    let log = Log::new(index, padding, opt);
    log.output(proc_name, content)
}

pub fn error(proc_name: &str, err: &dyn std::error::Error, padding: Option<usize>, opt: &LogOpt) {
    let content = &format!("error: {:?}", err);
    if let Some(p) = padding {
        output(proc_name, content, p, None, opt);
    } else {
        output(proc_name, content, proc_name.len() + 1, None, opt);
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
            10,
            &LogOpt {
                is_color: true,
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
            10,
            &LogOpt {
                is_color: false,
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
            10,
            &LogOpt {
                is_color: true,
                is_timestamp: true,
            },
        );
        log.error("test_app", &error);

        Ok(())
    }
}
