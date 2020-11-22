use std::process::{Child, exit};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use nix::sys::signal::{self, Signal};
use nix::sys::wait::WaitStatus;
use nix::{self, unistd::Pid};
use crate::log;

pub struct Process {
    pub name: String,
    pub child: Child,
}

pub fn check_child_terminated(procs: Arc<Mutex<Vec<Arc<Mutex<Process>>>>>, padding: usize) -> JoinHandle<()> {
    let result = thread::Builder::new()
        .name(String::from(format!("check child terminated")))
        .spawn(move || {
            loop {
                // Waiting for the end of any one child process
                match nix::sys::wait::waitpid(
                    Pid::from_raw(-1),
                    Some(nix::sys::wait::WaitPidFlag::WNOHANG),
                ) {
                    Ok(exit_status) => match exit_status {
                        WaitStatus::Exited(pid, code) => {
                            procs.lock().unwrap().retain(|p| {
                                let child_id = p.lock().unwrap().child.id() as i32;
                                Pid::from_raw(child_id) != pid
                            });

                            // If the child process dies, send SIGTERM to all child processes
                            for proc in procs.lock().unwrap().iter() {
                                let proc = proc.lock().unwrap();
                                let child_id = proc.child.id();

                                log::output(
                                    "system",
                                    &format!(
                                        "sending SIGTERM for {} at pid {}",
                                        &proc.name, &child_id
                                    ),
                                    padding
                                );
                                signal::kill(Pid::from_raw(child_id as i32), Signal::SIGTERM)
                                    .unwrap();
                            }
                            log::output("system", &format!("exit {}", &code), padding);
                            // close loop (thread finished)
                            exit(code);
                        }
                        _ => (),
                    },
                    Err(e) => {
                        if let nix::Error::Sys(nix::errno::Errno::ECHILD) = e {
                            // close loop (thread finished)
                            exit(0);
                        }
                    }
                };
            }
        }).expect("failed check child terminated");

    result
}
