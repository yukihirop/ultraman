use crate::log;
use crate::process::Process;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use signal_hook::{iterator::Signals, SIGALRM, SIGHUP, SIGINT, SIGTERM};
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

pub fn handle_signal(procs: Arc<Mutex<Vec<Arc<Mutex<Process>>>>>, padding: usize) -> JoinHandle<()> {
    let result = thread::Builder::new()
        .name(String::from("handling signal"))
        .spawn(move || trap_signal(procs, padding)
        .expect("failed trap signals")
    ).expect("failed handle signals");

    result
}

pub fn trap_signal(
    procs: Arc<Mutex<Vec<Arc<Mutex<Process>>>>>,
    padding: usize
) -> Result<(), Box<dyn std::error::Error>> {
    let signals = Signals::new(&[SIGALRM, SIGHUP, SIGINT, SIGTERM])?;

    for sig in signals.forever() {
        match sig {
            SIGINT => {
                log::output("system", "ctrl-c detected", padding);
                log::output("system", "sending SIGTERM for children", padding);
                for proc in procs.lock().unwrap().iter() {
                    let proc = proc.lock().unwrap();
                    let child = &proc.child;

                    log::output(
                        "system",
                        &format!("sending SIGTERM for {} at pid {}", &proc.name, &child.id()),
                        0
                    );

                    if let Err(e) = signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM)
                    {
                        log::error("system", &e, padding);
                        log::output("system", "exit 1", padding);
                        exit(1);
                    }
                }
                log::output("system", "exit 0", padding);
                exit(0)
            }
            _ => (),
        }
    }

    Ok(())
}
