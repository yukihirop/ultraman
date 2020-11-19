use crate::stream_read::{PipeStreamReader, PipedLine};
use crate::process::Process;
use crate::log;
use crossbeam_channel::Select;
use std::sync::{Arc, Mutex};

pub fn handle_output(proc: &Arc<Mutex<Process>>) {
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

        match received {
            Ok(remote_result) => match remote_result {
                Ok(piped_line) => match piped_line {
                    PipedLine::Line(line) => {
                        log::output(&proc.lock().unwrap().name, &line);
                    }
                    PipedLine::EOF => {
                        stream_eof = true;
                        select.remove(index);
                    }
                },
                Err(error) => {
                    let err = format!("error: {:?}", error);
                    println!("{}", err);
                },
            },
            Err(_) => {
                stream_eof = true;
                select.remove(index);
            }
        }
    }

    // let status = proc.lock().unwrap().child.wait().expect("!wait");
    // let proc_name = &proc.lock().unwrap().name;
    // if status.success() {
    //     log::output(proc_name, "onsucceed handler");
    // } else {
    //     log::output(proc_name, "onfailed handler");
    // }
}
