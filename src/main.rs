use std::process::{Command, Stdio, exit};
use std::sync::{self, Arc, Mutex, Barrier};
use std::thread;
use std::collections::HashMap;
use nix::{self, unistd::Pid};
use nix::sys::signal::{self as nix_signal, Signal};
use std::time;

mod output;
mod stream_read;
mod process;
mod log;
mod signal;

struct Script {
    cmd: String,
    concurrency: usize
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut proc_handles = vec![];
    let procs: Arc<Mutex<Vec<Arc<Mutex<process::Process>>>>> = Arc::new(Mutex::new(vec![]));
    
    let mut scripts = HashMap::<&str, Script>::new();
    scripts.insert("loop", Script {
        cmd: String::from("./bin/loop.sh"),
        concurrency: 2
    });
    scripts.insert("exit_1", Script {
        cmd: String::from("./bin/exit_1.sh"),
        concurrency: 1
    });
    scripts.insert("exit_0", Script {
        cmd: String::from("./bin/exit_0.sh"),
        concurrency: 1
    });

    let mut concurrencies = vec![];
    for (_, script) in &scripts {
        concurrencies.push(script.concurrency);
    }
    let process_len = concurrencies.iter().fold(0, |sum, a| sum + a);
    
    let barrier = Arc::new(Barrier::new(process_len + 1));
    let (tx, rx) = sync::mpsc::channel();

    for (key, script) in scripts {
        let con = script.concurrency;
        let script = Arc::new(script);

        for n in 0..con {
            let tx = tx.clone();
            let barrier = barrier.clone();
            let script = script.clone();
            let procs = procs.clone();

            let handle_output = thread::Builder::new()
                .name(String::from("handling output"))
                .spawn(move || {
                    let tmp_proc = process::Process {
                        name: String::from(format!("{}.{}", key, n+1)),
                        child: Command::new(&script.cmd)
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn().unwrap(),
                    };
                    let proc = Arc::new(Mutex::new(tmp_proc));
                    let proc2 = Arc::clone(&proc);

                    let child_id = proc.lock().unwrap().child.id() as i32;
                    log::output(&proc.lock().unwrap().name, &format!("start at pid: {}", &child_id));

                    tx.send(child_id).unwrap();
                    procs.lock().unwrap().push(proc);
                    barrier.wait();

                    output::handle_output(&proc2);
                })?;
        
            proc_handles.push(handle_output);
        }
    }

    barrier.wait();
    for (idx, pid) in rx.iter().enumerate() {
        let procs = procs.clone();
        let check_child_terminated = thread::Builder::new()
            .name(String::from(format!("check child terminated: {}", idx)))
            .spawn(move || {
                match nix::sys::wait::waitpid(Pid::from_raw(pid), None) {
                    Ok(_) => {
                        procs.lock().unwrap().retain(|p| p.lock().unwrap().child.id() != pid as u32);

                        // Wait a moment because there may be a process that ended at the same timing as the terminated child process
                        thread::sleep(time::Duration::from_millis(500));

                        let procs_len = procs.lock().unwrap().len();
                        log::output("system", &procs_len.to_string());

                        // If the child process dies, send SIGTERM to all child processes
                        for proc in procs.lock().unwrap().iter() {
                            let proc = proc.lock().unwrap();
                            let child_id = proc.child.id();
                            
                            log::output("system", &format!("sending SIGTERM to {} at pid {}", &proc.name, &child_id));
                            nix_signal::kill(
                                Pid::from_raw(child_id as i32),
                                Signal::SIGTERM,
                            )
                            .unwrap();
                        }
                        log::output("system", "exit 0");
                        exit(0);
                    },
                    Err(e) => log::error("system", &e)
                };
            })?;
        proc_handles.push(check_child_terminated);
    }

    // let procs = Arc::clone(&procs);
    // let handle_signal = thread::Builder::new()
    //     .name(String::from("handling signal"))
    //     .spawn(move || {
    //         signal::handle_signal(procs).expect("fail to handle signal")
    //     })?;
    // proc_handles.push(handle_signal);

    for handle in proc_handles {
        handle.join().expect("failed join");
    }

    Ok(())
}
