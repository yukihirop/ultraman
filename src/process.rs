use crate::env::read_env;
use crate::log;
use crate::output;
use crate::signal;
use nix::sys::signal::Signal;
use nix::sys::wait::WaitStatus;
use nix::{self, unistd::Pid};
use std::process::{Child, Command, Stdio};

#[cfg(not(test))]
use std::process::exit;

use std::env::{self as os_env};
use std::path::PathBuf;
use std::sync::{Arc, Barrier, Mutex};
use std::thread::{self, JoinHandle};

pub struct Process {
    pub index: usize,
    pub name: String,
    pub child: Child,
}

// https://qiita.com/quercus491/items/9f8f590c9734c81da2bd
pub fn each_handle_exec_and_output(
    procs: Arc<Mutex<Vec<Arc<Mutex<Process>>>>>,
    padding: usize,
    barrier: Arc<Barrier>,
    output: Arc<output::Output>,
) -> Box<dyn Fn(String, usize, String, PathBuf, Option<String>, usize) -> JoinHandle<()>> {
    Box::new(
        // MEMO: Refactor when you understand your lifetime
        move |process_name: String,
              con: usize,
              cmd: String,
              env_path: PathBuf,
              port: Option<String>,
              index: usize| {
            let output = output.clone();
            let procs = procs.clone();
            let barrier = barrier.clone();

            let result = thread::Builder::new()
                .name(String::from("handle exec and output"))
                .spawn(move || {
                    let mut read_env = read_env(env_path.clone()).expect("failed read .env");
                    read_env.insert(String::from("PORT"), port_for(env_path, port, index, con));
                    read_env.insert(String::from("PS"), ps_for(process_name.clone(), con + 1));
                    let shell = os_env::var("SHELL").expect("$SHELL is not set");

                    let tmp_proc = Process {
                        index,
                        name: ps_for(process_name, con + 1),
                        child: Command::new(shell)
                            .arg("-c")
                            .arg(cmd)
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .envs(read_env)
                            .spawn()
                            .expect("failed execute command"),
                    };
                    let proc = Arc::new(Mutex::new(tmp_proc));
                    let proc2 = Arc::clone(&proc);

                    let child_id = proc.lock().unwrap().child.id() as i32;
                    output.log.output(
                        "system",
                        &format!(
                            "{0:1$} start at pid: {2}",
                            &proc.lock().unwrap().name,
                            padding,
                            &child_id
                        ),
                    );

                    procs.lock().unwrap().push(proc);
                    barrier.wait();

                    output.handle_output(&proc2);
                })
                .expect("failed exec and output");
            result
        },
    )
}

pub fn check_for_child_termination_thread(
    procs: Arc<Mutex<Vec<Arc<Mutex<Process>>>>>,
    padding: usize,
) -> JoinHandle<()> {
    let result = thread::Builder::new()
        .name(String::from(format!("check child terminated")))
        .spawn(move || {
            loop {
                // Waiting for the end of any one child process
                let procs2 = Arc::clone(&procs);
                let procs3 = Arc::clone(&procs);
                if let Some((_, code)) = check_for_child_termination(procs2, padding) {
                    signal::kill_children(procs3, padding, Signal::SIGTERM, code)
                }
            }
        })
        .expect("failed check child terminated");

    result
}

pub fn check_for_child_termination(
    procs: Arc<Mutex<Vec<Arc<Mutex<Process>>>>>,
    padding: usize,
) -> Option<(Pid, i32)> {
    // Waiting for the end of any one child process
    match nix::sys::wait::waitpid(
        Pid::from_raw(-1),
        Some(nix::sys::wait::WaitPidFlag::WNOHANG),
    ) {
        Ok(exit_status) => match exit_status {
            WaitStatus::Exited(pid, code) => {
                procs.lock().unwrap().retain(|p| {
                    let child_id = p.lock().unwrap().child.id() as i32;
                    if Pid::from_raw(child_id) == pid {
                        let proc = p.lock().unwrap();
                        let proc_name = &proc.name;
                        let proc_index = proc.index;
                        log::output(
                            &proc_name,
                            &format!("exited with code {}", code),
                            padding,
                            Some(proc_index),
                        );
                    }
                    Pid::from_raw(child_id) != pid
                });
                return Some((pid, code));
            }
            WaitStatus::Signaled(pid, signal, _) => {
                procs.lock().unwrap().retain(|p| {
                    let child_id = p.lock().unwrap().child.id() as i32;
                    if Pid::from_raw(child_id) == pid {
                        let proc = p.lock().unwrap();
                        let proc_name = &proc.name;
                        let proc_index = proc.index;
                        log::output(
                            &proc_name,
                            &format!("terminated by {}", signal.as_str()),
                            padding,
                            Some(proc_index),
                        );
                    }
                    Pid::from_raw(child_id) != pid
                });
                return None;
            }
            _ => return None,
        },
        Err(e) => {
            if let nix::Error::Sys(nix::errno::Errno::ECHILD) = e {
                // close loop (thread finished)
                #[cfg(not(test))]
                exit(0);
                #[cfg(test)]
                panic!("exit 0");
            }
            return None;
        }
    };
}

fn ps_for(process_name: String, concurrency: usize) -> String {
    format!("{}.{}", process_name, concurrency)
}

fn port_for(env_path: PathBuf, port: Option<String>, index: usize, concurrency: usize) -> String {
    let result = base_port(env_path, port).parse::<usize>().unwrap() + index * 100 + concurrency - 1;
    result.to_string()
}

fn base_port(env_path: PathBuf, port: Option<String>) -> String {
    let env = read_env(env_path).unwrap();
    let default_port = String::from("5000");

    if let Some(p) = port {
        p
    } else if let Some(p) = env.get("PORT") {
        p.clone()
    } else if let Ok(p) = os_env::var("PORT") {
        p
    } else {
        default_port
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow;
    use std::sync::Barrier;

    #[test]
    fn test_each_handle_exec_and_output() -> anyhow::Result<()> {
        let procs: Arc<Mutex<Vec<Arc<Mutex<Process>>>>> = Arc::new(Mutex::new(vec![]));
        let procs2 = Arc::clone(&procs);

        let padding = 10;
        let barrier = Arc::new(Barrier::new(1));
        let output = Arc::new(output::Output::new(0, padding));

        let each_fn_thread = each_handle_exec_and_output(procs2, padding, barrier, output);
        each_fn_thread(
            String::from("each_handle_exec_and_output"),
            0,
            String::from("./test/fixtures/for.sh"),
            PathBuf::from("./test/fixtures/.env"),
            None,
            1,
        )
        .join()
        .expect("failed join each thread");

        Ok(())
    }

    #[test]
    #[should_panic(expected = "exit 1: Any")]
    fn test_check_for_child_termination_thread() {
        let procs = Arc::new(Mutex::new(vec![
            Arc::new(Mutex::new(Process {
                index: 0,
                name: String::from("check_for_child_termination_thread-1"),
                child: Command::new("./test/fixtures/exit_0.sh")
                    .spawn()
                    .expect("failed execute check_for_child_termination_thread-1"),
            })),
            Arc::new(Mutex::new(Process {
                index: 1,
                name: String::from("check_for_child_termination_thread-2"),
                child: Command::new("./test/fixtures/exit_1.sh")
                    .spawn()
                    .expect("failed execute check_for_child_termination_thread-2"),
            })),
        ]));
        let procs2 = Arc::clone(&procs);
        let padding = 10;

        check_for_child_termination_thread(procs2, padding)
            .join()
            .expect("exit 1");
    }
}
