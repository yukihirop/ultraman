use crate::log::{Log, LogOpt, Printable};
use crate::process::Process;
use crate::stream_read::{PipeError, PipeStreamReader, PipedLine};
use crate::opt::DisplayOpts;

use crossbeam_channel::Select;
use std::sync::{Arc, Mutex};

pub struct Output {
    pub log: Box<dyn Printable + Sync + Send>,
}

impl Output {
    pub fn new(index: usize, opts: DisplayOpts) -> Self {
        Output {
            log: Log::new(
                index,
                &LogOpt {
                    is_color: true,
                    padding: opts.padding,
                    is_timestamp: opts.is_timestamp,
                },
            ),
        }
    }

    pub fn handle_output(&self, proc: &Arc<Mutex<Process>>) {
        let mut channels: Vec<PipeStreamReader> = Vec::new();
        channels.push(PipeStreamReader::new(Box::new(
            proc.lock()
                .unwrap()
                .child
                .stdout
                .take()
                .expect("failed take stdout"),
        )));
        channels.push(PipeStreamReader::new(Box::new(
            proc.lock()
                .unwrap()
                .child
                .stderr
                .take()
                .expect("failed take stderr"),
        )));

        let mut select = Select::new();
        for channel in channels.iter() {
            select.recv(&channel.lines);
        }

        let mut stream_eof = false;

        while !stream_eof {
            let operation = select.select();
            let index = operation.index();
            let received = operation.recv(
                &channels
                    .get(index)
                    .expect("failed get channel at index")
                    .lines,
            );
            let log = &self.log;

            match received {
                Ok(remote_result) => match remote_result {
                    Ok(piped_line) => match piped_line {
                        PipedLine::Line(line) => {
                            log.output(&proc.lock().unwrap().name, &line);
                        }
                        PipedLine::EOF => {
                            stream_eof = true;
                            select.remove(index);
                        }
                    },
                    Err(error) => match error {
                        PipeError::IO(err) => log.error(&proc.lock().unwrap().name, &err),
                        PipeError::NotUtf8(err) => log.error(&proc.lock().unwrap().name, &err),
                    },
                },
                Err(_) => {
                    stream_eof = true;
                    select.remove(index);
                }
            }
        }

        // MEMO
        //
        // An error occurs in a child process that was terminated by sending a SIGTERM
        // It is necessary to be able to send a signal after successfully executing the termination process.
        //

        // blocking
        // let status = proc.lock().unwrap().child.wait().expect("!wait");
        // let proc_name = &proc.lock().unwrap().name;
        // if status.success() {
        //     log::output(proc_name, "onsucceed handler");
        // } else {
        //     log::output(proc_name, "onfailed handler");
        // }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow;
    use std::process::{Command, Stdio};

    #[test]
    fn test_handle_output() -> anyhow::Result<()> {
        let proc = Arc::new(Mutex::new(Process {
            index: 0,
            name: String::from("handle_output"),
            child: Command::new("./test/fixtures/for.sh")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("failed execute handle_output command"),
            opts: None,
        }));

        let proc2 = Arc::clone(&proc);
        let output = Output::new(0, DisplayOpts { padding: 10, is_timestamp: true });
        output.handle_output(&proc2);

        Ok(())
    }
}
