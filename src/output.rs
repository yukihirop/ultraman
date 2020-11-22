use crate::log::Log;
use crate::process::Process;
use crate::stream_read::{PipeStreamReader, PipedLine, PipeError};
use crossbeam_channel::Select;
use std::sync::{Arc, Mutex};

pub struct Output {
    pub log: Log,
}

impl Output {
    pub fn new(index: usize, padding: usize) -> Self {
        Output {
            log: Log::new(index, padding)
        }
    }

    pub fn handle_output(&self, proc: &Arc<Mutex<Process>>) {
        let mut channels: Vec<PipeStreamReader> = Vec::new();
        channels.push(PipeStreamReader::new(Box::new(
            proc.lock().unwrap().child.stdout.take().expect("!stdout"),
        )));
        channels.push(PipeStreamReader::new(Box::new(
            proc.lock().unwrap().child.stderr.take().expect("!stderr"),
        )));

        let mut select = Select::new();
        for channel in channels.iter() {
            select.recv(&channel.lines);
        }

        let mut stream_eof = false;

        while !stream_eof {
            let operation = select.select();
            let index = operation.index();
            let received = operation.recv(&channels.get(index).expect("!channel").lines);
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
                    Err(error) => {
                        match error {
                            PipeError::IO(err) => log.error(&proc.lock().unwrap().name, &err),
                            PipeError::NotUtf8(err) => log.error(&proc.lock().unwrap().name, &err),
                        }
                    }
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
