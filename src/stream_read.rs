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
                                    tx.send(match String::from_utf8(buf.clone()) {
                                        Ok(line) => Ok(PipedLine::Line(line)),
                                        Err(err) => Err(PipeError::NotUtf8(err)),
                                    })
                                    .unwrap();
                                    buf.clear()
                                } else {
                                    buf.push(byte[0])
                                }
                            }
                            Err(error) => {
                                tx.send(Err(PipeError::IO(error))).unwrap();
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
}
