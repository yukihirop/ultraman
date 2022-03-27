use crate::log::{self, LogOpt};
use crate::opt::DisplayOpts;
use crate::process::{self, Process};

use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use signal_hook::{iterator::Signals};
use signal_hook::consts::signal::{SIGALRM, SIGHUP, SIGINT, SIGTERM};
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep, JoinHandle};
use std::time::{Duration, Instant};

#[cfg(not(test))]
use std::process::exit;

pub fn handle_signal_thread(
    procs: Arc<Mutex<Vec<Arc<Mutex<Process>>>>>,
    timeout: u64,
    opts: DisplayOpts,
) -> JoinHandle<()> {
    let result = thread::Builder::new()
        .name(String::from("handling signal"))
        .spawn(move || {
            trap_signal_at_multithred(procs, timeout, opts).expect("failed trap signals")
        })
        .expect("failed handle signals");

    result
}

fn trap_signal_at_multithred(
    procs: Arc<Mutex<Vec<Arc<Mutex<Process>>>>>,
    timeout: u64,
    opts: DisplayOpts,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut signals = Signals::new(&[SIGALRM, SIGHUP, SIGINT, SIGTERM])?;

    for sig in signals.forever() {
        match sig {
            SIGINT => {
                // 2 is 「^C」 of 「^Csystem   | SIGINT received, starting shutdown」
                log::output(
                    "system",
                    "SIGINT received, starting shutdown",
                    None,
                    &LogOpt {
                        is_color: false,
                        padding: opts.padding - 2,
                        is_timestamp: opts.is_timestamp,
                    },
                );

                log::output(
                    "system",
                    "sending SIGTERM to all processes",
                    None,
                    &LogOpt {
                        is_color: false,
                        padding: opts.padding,
                        is_timestamp: opts.is_timestamp,
                    },
                );

                terminate_gracefully(procs, Signal::SIGTERM, 1, timeout, opts.clone());

                log::output(
                    "system",
                    "exit 0",
                    None,
                    &LogOpt {
                        is_color: false,
                        padding: opts.padding,
                        is_timestamp: opts.is_timestamp,
                    },
                );
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
    signal: Signal,
    code: i32,
    timeout: u64,
    opts: DisplayOpts,
) {
    let procs2 = Arc::clone(&procs);
    kill_children(procs, signal, code, opts.clone());

    // Wait for all children to stop or until the time comes to kill them all
    let start_time = Instant::now();
    while start_time.elapsed() < Duration::from_secs(timeout) {
        if procs2.lock().unwrap().len() == 0 {
            return;
        }

        let procs3 = Arc::clone(&procs2);
        process::check_for_child_termination(procs3, opts.clone());

        // Sleep for a moment and do not blow up if more signals are coming our way
        sleep(Duration::from_millis(100));
    }

    // Ok, we have no other option than to kill all of our children
    log::output(
        "system",
        "sending SIGKILL to all processes",
        None,
        &LogOpt {
            is_color: false,
            padding: opts.padding,
            is_timestamp: opts.is_timestamp,
        },
    );

    kill_children(procs2, Signal::SIGKILL, 0, opts);
}

pub fn kill_children(
    procs: Arc<Mutex<Vec<Arc<Mutex<Process>>>>>,
    signal: Signal,
    _code: i32,
    opts: DisplayOpts,
) {
    for proc in procs.lock().unwrap().iter() {
        let proc = proc.lock().unwrap();
        let child = &proc.child;

        log::output(
            "system",
            &format!(
                "sending {3} for {0:1$} at pid {2}",
                &proc.name,
                opts.padding,
                &child.id(),
                Signal::as_str(signal),
            ),
            None,
            &LogOpt {
                is_color: false,
                padding: opts.padding,
                is_timestamp: opts.is_timestamp,
            },
        );

        if let Err(e) = signal::kill(Pid::from_raw(child.id() as i32), signal) {
            log::error(
                "system",
                &e,
                true,
                &LogOpt {
                    is_color: false,
                    padding: opts.padding,
                    is_timestamp: opts.is_timestamp,
                },
            );
            log::output(
                "system",
                &format!("exit {}", _code),
                None,
                &LogOpt {
                    is_color: false,
                    padding: opts.padding,
                    is_timestamp: opts.is_timestamp,
                },
            );
            // https://www.reddit.com/r/rust/comments/emz456/testing_whether_functions_exit/
            #[cfg(not(test))]
            exit(_code);
            #[cfg(test)]
            panic!("exit {}", _code);
        }
    }
}

// Tests that need to send a SIGINT to kill the process can interrupt other tests and are usually ignored
#[cfg(test)]
mod tests {
    use super::*;
    use libc;
    use signal_hook::consts::signal::SIGINT;
    use std::process::Command;
    use std::thread::sleep;
    use std::time::Duration;

    // https://github.com/vorner/signal-hook/blob/master/tests/iterator.rs
    fn send_sigint() {
        unsafe { libc::raise(SIGINT) };
    }

    #[test]
    #[ignore]
    fn test_trap_signal_at_multithred() {
        let procs = Arc::new(Mutex::new(vec![
            Arc::new(Mutex::new(Process {
                index: 0,
                name: String::from("trap_signal_at_multithred-1"),
                child: Command::new("./test/fixtures/loop.sh")
                    .arg("trap_signal_at_multithred_1")
                    .spawn()
                    .expect("failed execute test-app-1"),
                opts: None,
            })),
            Arc::new(Mutex::new(Process {
                index: 1,
                name: String::from("trap_signal_at_multithred-2"),
                child: Command::new("./test/fixtures/loop.sh")
                    .arg("trap_signal_at_multithred_2")
                    .spawn()
                    .expect("failed execute test-app-2"),
                opts: None,
            })),
        ]));

        let procs2 = Arc::clone(&procs);
        let thread_trap_signal = thread::spawn(move || {
            trap_signal_at_multithred(
                procs2,
                5,
                DisplayOpts {
                    padding: 10,
                    is_timestamp: true,
                },
            )
            .expect("failed trap_signal_at_multithred")
        });

        let thread_send_sigint = thread::spawn(move || {
            sleep(Duration::from_secs(5));
            send_sigint();
        });

        thread_trap_signal.join().expect("failed handle signals");
        thread_send_sigint.join().expect("failed send sigint");
    }
}
