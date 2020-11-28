use crate::env::read_env;
use crate::procfile::read_procfile;
use crate::signal;

use nix::sys::wait::WaitStatus;
use nix::unistd::{fork, pause, ForkResult};
use nix::{self};
use std::env::{self as std_env};
use std::path::PathBuf;
use std::process::{exit, Command};
use std::thread;
use structopt::{clap, StructOpt};

#[derive(StructOpt, Debug)]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct RunOpts {
    /// App Name
    #[structopt(name = "APP_NAME")]
    pub app_name: String,

    /// Specify an environment file to load
    #[structopt(
        name = "ENV",
        short = "e",
        long = "env",
        parse(from_os_str),
        default_value = ".env"
    )]
    pub env_path: PathBuf,

    /// Specify an Procfile to load
    #[structopt(
        name = "PROCFILE",
        short = "f",
        long = "procfile",
        parse(from_os_str),
        default_value = "Procfile"
    )]
    pub procfile_path: PathBuf,
}

pub fn run(opts: RunOpts) {
    let app_name = opts.app_name;
    let procfile_path = opts.procfile_path;
    let env_path = opts.env_path;

    let procfile = read_procfile(procfile_path).expect("failed read Procfile");
    let pe = procfile.find_by(&app_name);

    let mut read_env = read_env(env_path).expect("failed read .env");
    read_env.insert(String::from("PORT"), String::from("5000"));
    read_env.insert(String::from("PS"), String::from(&app_name));

    let shell = std_env::var("SHELL").expect("$SHELL is not set");

    unsafe {
        match fork() {
            Ok(fork_result) => match fork_result {
                ForkResult::Child => {
                    let _ = Command::new(shell)
                        .arg("-c")
                        .arg(&pe.command)
                        .envs(read_env)
                        .spawn()
                        .expect("failed execute command");
                    // we need the child to stay alive until the parent calls read
                    pause();
                }
                ForkResult::Parent { child } => {
                    let handle_signal_thread = thread::Builder::new()
                        .name(String::from("handle_signal_thread"))
                        .spawn(move || {
                            signal::trap_signal(child).expect("failed trap signal");
                        })
                        .expect("failed spawn handle_signal");

                    let check_for_child_termination_thread = thread::Builder::new()
                        .name(String::from("check_for_child_termination_thread"))
                        .spawn(move || {
                            if let Ok(exit_status) = nix::sys::wait::waitpid(child, None) {
                                match exit_status {
                                    WaitStatus::Exited(_, code) => exit(code),
                                    _ => (),
                                }
                            }
                        })
                        .expect("failed spawn check_for_child_termination");

                    handle_signal_thread
                        .join()
                        .expect("failed join handle_signal_thread");
                    check_for_child_termination_thread
                        .join()
                        .expect("failed join handle_signal_thread");
                }
            },
            Err(e) => {
                println!("failed rustman run");
                println!("error: {}", &e)
            }
        }
    }
}
