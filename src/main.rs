use nix::sys::signal::{self as nix_signal, Signal};
use nix::sys::wait::WaitStatus;
use nix::{self, unistd::Pid};
use std::collections::HashMap;
use std::process::{exit, Command, Stdio};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

mod log;
mod output;
mod process;
mod signal;
mod stream_read;

struct Script {
    cmd: String,
    concurrency: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut proc_handles = vec![];
    let procs: Arc<Mutex<Vec<Arc<Mutex<process::Process>>>>> = Arc::new(Mutex::new(vec![]));

    let mut scripts = HashMap::<&str, Script>::new();
    scripts.insert(
        "loop",
        Script {
            cmd: String::from("./bin/loop.sh"),
            concurrency: 2,
        },
    );
    scripts.insert(
        "exit_1",
        Script {
            cmd: String::from("./bin/exit_1.sh"),
            concurrency: 1,
        },
    );
    scripts.insert(
        "exit_0",
        Script {
            cmd: String::from("./bin/exit_0.sh"),
            concurrency: 1,
        },
    );

    let mut concurrencies = vec![];
    for (_, script) in &scripts {
        concurrencies.push(script.concurrency);
    }
    let process_len = concurrencies.iter().fold(0, |sum, a| sum + a);

    let barrier = Arc::new(Barrier::new(process_len + 1));
    // e.g) <name>.<concurrency> |
    let padding = scripts.keys().map(|name| name.len()).max().unwrap() + 3;
    let mut index = 0;

    for (key, script) in scripts {
        let con = script.concurrency;
        let script = Arc::new(script);
        let output = Arc::new(output::Output::new(index, padding, true));

        for n in 0..con {
            let barrier = barrier.clone();
            let script = script.clone();
            let procs = procs.clone();
            let output = output.clone();

            let handle_output = thread::Builder::new()
                .name(String::from("handling output"))
                .spawn(move || {
                    let tmp_proc = process::Process {
                        name: String::from(format!("{}.{}", key, n + 1)),
                        child: Command::new(&script.cmd)
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()
                            .unwrap(),
                    };
                    let proc = Arc::new(Mutex::new(tmp_proc));
                    let proc2 = Arc::clone(&proc);

                    let child_id = proc.lock().unwrap().child.id() as i32;
                    output.log.output(
                        "system",
                        &format!("{0:1$} start at pid: {2}", &proc.lock().unwrap().name, padding, &child_id),
                    );

                    procs.lock().unwrap().push(proc);
                    barrier.wait();

                    output.handle_output(&proc2);
                })?;

            proc_handles.push(handle_output);
        }

        index += 1;
    }

    barrier.wait();

    // use handle_signal
    let procs_2 = Arc::clone(&procs);

    let check_child_terminated = thread::Builder::new()
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
                                nix_signal::kill(Pid::from_raw(child_id as i32), Signal::SIGTERM)
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
        })?;
    proc_handles.push(check_child_terminated);

    let procs = Arc::clone(&procs_2);
    let handle_signal = thread::Builder::new()
        .name(String::from("handling signal"))
        .spawn(move || signal::handle_signal(procs, padding).expect("fail to handle signal"))?;
    proc_handles.push(handle_signal);

    for handle in proc_handles {
        handle.join().expect("failed join");
    }

    Ok(())
}
