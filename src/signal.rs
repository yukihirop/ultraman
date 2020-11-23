#![cfg(not(windows))]

use crate::log;
use crate::process::Process;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use signal_hook::{iterator::Signals, SIGALRM, SIGHUP, SIGINT, SIGTERM};

#[cfg(not(test))]
use std::process::exit;

use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

pub fn handle_signal(
    procs: Arc<Mutex<Vec<Arc<Mutex<Process>>>>>,
    padding: usize,
) -> JoinHandle<()> {
    let result = thread::Builder::new()
        .name(String::from("handling signal"))
        .spawn(move || trap_signal(procs, padding).expect("failed trap signals"))
        .expect("failed handle signals");

    result
}

fn trap_signal(
    procs: Arc<Mutex<Vec<Arc<Mutex<Process>>>>>,
    padding: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let signals = Signals::new(&[SIGALRM, SIGHUP, SIGINT, SIGTERM])?;

    for sig in signals.forever() {
        match sig {
            SIGINT => {
                // 2 is 「^C」 of 「^Csystem   | ctrl-c detected」
                log::output("system", "ctrl-c detected", padding - 2);
                log::output("system", "sending SIGTERM for children", padding);
                for proc in procs.lock().unwrap().iter() {
                    let proc = proc.lock().unwrap();
                    let child = &proc.child;

                    log::output(
                        "system",
                        &format!("sending SIGTERM for {0:1$} at pid {2}", &proc.name, padding, &child.id()),
                        padding,
                    );

                    if let Err(e) = signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM)
                    {
                        log::error("system", &e, padding);
                        log::output("system", "exit 1", padding);
                        
                        // https://www.reddit.com/r/rust/comments/emz456/testing_whether_functions_exit/
                        #[cfg(not(test))]
                        exit(1);
                        #[cfg(test)]
                        panic!("exit 1");
                    }
                }
                log::output("system", "exit 0", padding);
                #[cfg(not(test))]
                exit(0);
                #[cfg(test)]
                break
            }
            _ => (),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command};
    use anyhow;
    use signal_hook::{SIGINT};
    use libc;
    use std::time::Duration;
    use std::thread::sleep;

    // https://github.com/vorner/signal-hook/blob/master/tests/iterator.rs
    fn send_sigint(){
        unsafe { libc::raise(SIGINT) };
    }

    #[test]
    fn test_trap_signal() -> anyhow::Result<()> {
        let procs = Arc::new(Mutex::new(vec![
            Arc::new(Mutex::new(Process {
                name: String::from("trap-signal-1"),
                child: Command::new("./test/script/loop.sh")
                    .arg("trap_signal_1")
                    .spawn()
                    .expect("failed execute test-app-1")
            })),
            Arc::new(Mutex::new(Process {
                name: String::from("trap-signal-2"),
                child: Command::new("./test/script/loop.sh")
                    .arg("trap_signal_2")
                    .spawn()
                    .expect("failed execute test-app-2")
            }))
        ]));

        let procs2 = Arc::clone(&procs);
        let thread_trap_signal = thread::spawn(move || {
            trap_signal(procs2, 10).expect("failed trap signals")
        });

        let thread_send_sigint = thread::spawn(move || {
            sleep(Duration::from_millis(5000));
            send_sigint();
        });

        thread_trap_signal.join().expect("failed handle signals");
        thread_send_sigint.join().expect("failed send sigint");

        Ok(())
    }
}
