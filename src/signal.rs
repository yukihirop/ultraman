#![cfg(not(windows))]

use crate::log;
use crate::process::{self, Process};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use signal_hook::{iterator::Signals, SIGALRM, SIGHUP, SIGINT, SIGTERM};

#[cfg(not(test))]
use std::process::exit;

use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle, sleep};
use std::time::{Duration, Instant};

pub fn handle_signal_thread(
    procs: Arc<Mutex<Vec<Arc<Mutex<Process>>>>>,
    padding: usize,
    timeout: u64,
) -> JoinHandle<()> {
    let result = thread::Builder::new()
        .name(String::from("handling signal"))
        .spawn(move || trap_signal(procs, padding, timeout).expect("failed trap signals"))
        .expect("failed handle signals");

    result
}

fn trap_signal(
    procs: Arc<Mutex<Vec<Arc<Mutex<Process>>>>>,
    padding: usize,
    timeout: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let signals = Signals::new(&[SIGALRM, SIGHUP, SIGINT, SIGTERM])?;

    for sig in signals.forever() {
        match sig {
            SIGINT => {
                // 2 is 「^C」 of 「^Csystem   | ctrl-c detected」
                log::output("system", "ctrl-c detected", padding - 2);

                log::output("system", "sending SIGTERM to all processes", padding);
                terminate_gracefully(procs, padding, Signal::SIGTERM, 1, timeout);

                log::output("system", "exit 0", padding);
                #[cfg(not(test))]
                exit(0);
                #[cfg(test)]
                break;
            }
            _ => (),
        }
    }

    Ok(())
}

pub fn terminate_gracefully(
    procs: Arc<Mutex<Vec<Arc<Mutex<Process>>>>>,
    padding: usize,
    signal: Signal,
    code: i32,
    timeout: u64
){
    let procs2 = Arc::clone(&procs);
    kill_children(procs, padding, signal, code);

    // Wait for all children to stop or until the time comes to kill them all
    let start_time = Instant::now();
    while start_time.elapsed() < Duration::from_secs(timeout) {
        if procs2.lock().unwrap().len() == 0 {
            return
        }

        let procs3 = Arc::clone(&procs2);
        process::check_for_child_termination(procs3);

        // Sleep for a moment and do not blow up if more signals are coming our way
        sleep(Duration::from_millis(100));
    }

    // Ok, we have no other option than to kill all of our children
    log::output("system", "sending SIGKILL to all processes", padding);
    kill_children(procs2, padding, Signal::SIGKILL, 0);
}

pub fn kill_children(
    procs: Arc<Mutex<Vec<Arc<Mutex<Process>>>>>,
    padding: usize,
    signal: Signal,
    _code: i32,
) {
    for proc in procs.lock().unwrap().iter() {
        let proc = proc.lock().unwrap();
        let child = &proc.child;

        log::output(
            "system",
            &format!(
                "sending {3} for {0:1$} at pid {2}",
                &proc.name,
                padding,
                &child.id(),
                Signal::as_str(signal),
            ),
            padding,
        );

        if let Err(e) = signal::kill(Pid::from_raw(child.id() as i32), signal) {
            log::error("system", &e, padding);
            log::output("system", &format!("exit {}", _code), padding);
            // https://www.reddit.com/r/rust/comments/emz456/testing_whether_functions_exit/
            #[cfg(not(test))]
            exit(_code);
            #[cfg(test)]
            panic!("exit {}", _code);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libc;
    use signal_hook::SIGINT;
    use std::process::Command;
    use std::thread::sleep;
    use std::time::Duration;

    // https://github.com/vorner/signal-hook/blob/master/tests/iterator.rs
    fn send_sigint() {
        unsafe { libc::raise(SIGINT) };
    }

    #[test]
    #[should_panic(expected = "failed handle signals: Any")]
    fn test_trap_signal() {
        let procs = Arc::new(Mutex::new(vec![
            Arc::new(Mutex::new(Process {
                name: String::from("trap-signal-1"),
                child: Command::new("./test/fixtures/loop.sh")
                    .arg("trap_signal_1")
                    .spawn()
                    .expect("failed execute test-app-1"),
            })),
            Arc::new(Mutex::new(Process {
                name: String::from("trap-signal-2"),
                child: Command::new("./test/fixtures/loop.sh")
                    .arg("trap_signal_2")
                    .spawn()
                    .expect("failed execute test-app-2"),
            })),
        ]));

        let procs2 = Arc::clone(&procs);
        let thread_trap_signal =
            thread::spawn(move || trap_signal(procs2, 10, 5).expect("failed trap signals"));

        let thread_send_sigint = thread::spawn(move || {
            sleep(Duration::from_secs(5));
            send_sigint();
        });

        thread_trap_signal.join().expect("failed handle signals");
        thread_send_sigint.join().expect("failed send sigint");
    }
}
