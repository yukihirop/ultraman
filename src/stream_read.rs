use crossbeam::channel::{unbounded, Receiver};
use std::io;
use std::string::FromUtf8Error;
use std::thread::spawn;

#[derive(Debug)]
pub enum PipeError {
    IO(io::Error),
    NotUtf8(FromUtf8Error),
}

#[derive(Debug)]
pub enum PipedLine {
    Line(String),
    EOF,
}

#[derive(Debug)]
pub struct PipeStreamReader {
    pub lines: Receiver<Result<PipedLine, PipeError>>,
}

impl PipeStreamReader {
    pub fn new(mut stream: Box<dyn io::Read + Send>) -> PipeStreamReader {
        PipeStreamReader {
            lines: {
                let (tx, rx) = unbounded();

                spawn(move || {
                    let mut buf = Vec::new();
                    let mut byte = [0u8];
                    loop {
                        match stream.read(&mut byte) {
                            Ok(0) => {
                                let _ = tx.send(Ok(PipedLine::EOF));
                                break;
                            }
                            Ok(_) => {
                                if byte[0] == 0x0A {
                                    match tx.send(match String::from_utf8(buf.clone()) {
                                        Ok(line) => Ok(PipedLine::Line(line)),
                                        Err(err) => Err(PipeError::NotUtf8(err)),
                                    }) {
                                        Ok(_) => {}
                                        Err(_) => {
                                            break;
                                        }
                                    }

                                    buf.clear()
                                } else {
                                    buf.push(byte[0])
                                }
                            }
                            Err(error) => {
                                if let Err(_) = tx.send(Err(PipeError::IO(error))) {
                                    break;
                                }
                            }
                        }
                    }
                });

                rx
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow;
    use std::process::{Command, Stdio};

    #[test]
    fn test_new() -> anyhow::Result<()> {
        let mut child = Command::new("echo")
            .arg("Test")
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed execute command");
        let stream = Box::new(child.stdout.take().unwrap());
        let result = PipeStreamReader::new(stream);

        match result.lines.recv().unwrap() {
            Ok(piped_line) => match piped_line {
                PipedLine::Line(line) => assert_eq!(line, "Test"),
                PipedLine::EOF => println!("EOF"),
            },
            Err(error) => match error {
                PipeError::IO(err) => println!("{}", err),
                PipeError::NotUtf8(err) => println!("{}", err),
            },
        }

        Ok(())
    }

    #[test]
    fn test_early_process_termination() -> anyhow::Result<()> {
        let mut child = Command::new("seq")
            .arg("3")
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed execute command");
        let stream = Box::new(child.stdout.take().unwrap());
        let reader = PipeStreamReader::new(stream);

        let mut lines_received = 0;
        loop {
            match reader.lines.recv() {
                Ok(Ok(PipedLine::Line(_))) => {
                    lines_received += 1;
                }
                Ok(Ok(PipedLine::EOF)) => {
                    break;
                }
                Ok(Err(_)) => {
                    break;
                }
                Err(_) => {
                    break;
                }
            }
        }

        assert!(lines_received <= 3);
        Ok(())
    }

    #[test]
    fn test_graceful_channel_disconnection() -> anyhow::Result<()> {
        use std::sync::Arc;
        use std::thread;
        use std::time::Duration;

        let mut child = Command::new("yes")
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed execute command");
        let stream = Box::new(child.stdout.take().unwrap());
        let reader = Arc::new(PipeStreamReader::new(stream));

        let reader_clone = Arc::clone(&reader);
        let handle = thread::spawn(move || {
            let mut count = 0;
            while count < 5 {
                match reader_clone.lines.recv() {
                    Ok(Ok(PipedLine::Line(_))) => count += 1,
                    Ok(Ok(PipedLine::EOF)) => break,
                    Ok(Err(_)) => break,
                    Err(_) => break,
                }
            }
        });

        thread::sleep(Duration::from_millis(100));
        child.kill().expect("failed to kill child");
        
        handle.join().expect("thread panicked");
        Ok(())
    }
}
