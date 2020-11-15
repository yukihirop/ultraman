use crate::stream_read::{PipeStreamReader, PipedLine};
use crossbeam_channel::Select;
use std::sync::{Arc, Mutex};
use std::process::{Child};

pub fn handle_output(child: &Arc<Mutex<Child>>) {
    let mut channels: Vec<PipeStreamReader> = Vec::new();
    channels.push(PipeStreamReader::new(Box::new(
        child.lock().unwrap().stdout.take().expect("!stdout"),
    )));
    channels.push(PipeStreamReader::new(Box::new(
        child.lock().unwrap().stderr.take().expect("!stderr"),
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
                        println!("{}", line);
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

    let status = child.lock().unwrap().wait().expect("!wait");
    let child_name = String::from("child name");
    if status.success() {
        let onsucceed = String::from("on succeeed");
        let annotated_message = format!("[{}] onsucceed: {}", child_name, &onsucceed);
        println!("{}", annotated_message);
        println!("Triggering onsucceed");
    } else {
        let onfail = String::from("on failed");
        let annotated_message = format!("[{}] onfail: {}", child_name, &onfail);
        println!("{}", annotated_message);
        println!("Triggering onfail.");
    }
}
