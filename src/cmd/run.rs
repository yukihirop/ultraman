use crate::config::{read_config, DEFAULT_ENV, DEFAULT_PROCFILE};
use crate::env::read_env;
use crate::procfile::read_procfile;

use nix::sys::wait::WaitStatus;
use nix::unistd::{fork, pause, ForkResult};
use nix::{self};
use std::env::{self as std_env};
use std::path::PathBuf;
use std::process::{exit, Command};
use std::thread;
use structopt::{clap, StructOpt};
use yaml_rust::Yaml;

#[derive(StructOpt, Debug)]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct RunOpts {
    /// App Name
    #[structopt(name = "APP_NAME")]
    pub app_name: String,

    /// Specify an environment file to load
    #[structopt(name = "ENV", short = "e", long = "env", parse(from_os_str))]
    pub env_path: Option<PathBuf>,

    /// Specify an Procfile to load
    #[structopt(name = "PROCFILE", short = "f", long = "procfile", parse(from_os_str))]
    pub procfile_path: Option<PathBuf>,
}

pub fn run(input_opts: RunOpts) {
    let dotconfig = read_config(PathBuf::from(".ultraman")).unwrap();
    let opts = merged_opts(&input_opts, &dotconfig);

    let app_name = opts.app_name;
    let procfile_path = opts.procfile_path.unwrap();
    let env_path = opts.env_path.unwrap();

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

                    check_for_child_termination_thread
                        .join()
                        .expect("failed join handle_signal_thread");
                }
            },
            Err(e) => {
                println!("failed ultraman run");
                println!("error: {}", &e)
            }
        }
    }
}

fn merged_opts(input_opts: &RunOpts, dotconfig: &Yaml) -> RunOpts {
    RunOpts {
        app_name: input_opts.app_name.to_string(),
        env_path: match &input_opts.env_path {
            Some(r) => Some(PathBuf::from(r)),
            None => Some(
                dotconfig["env"]
                    .as_str()
                    .map(|r| PathBuf::from(r))
                    .unwrap_or(PathBuf::from(DEFAULT_ENV)),
            ),
        },
        procfile_path: match &input_opts.procfile_path {
            Some(r) => Some(PathBuf::from(r)),
            None => Some(
                dotconfig["procfile"]
                    .as_str()
                    .map(|r| PathBuf::from(r))
                    .unwrap_or(PathBuf::from(DEFAULT_PROCFILE)),
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn prepare_dotconfig() -> Yaml {
        let dir = tempdir().ok().unwrap();
        let file_path = dir.path().join(".ultraman");
        let mut file = File::create(file_path.clone()).ok().unwrap();
        // Writing a comment causes a parse error
        writeln!(
            file,
            r#"
procfile: ./Procfile
env: .env

formation: app=1,web=2
port: 6000
timeout: 5000

no-timestamp: true

app: app-for-runit
log: /var/app/log/ultraman.log
run: /tmp/pids/ultraman.pid
template: ../../src/cmd/export/templates/supervisord
user: root
root: /home/app

hoge: hogehoge
      "#
        )
        .unwrap();

        let dotconfig = read_config(file_path).expect("failed read .ultraman");
        dotconfig
    }

    #[test]
    fn test_merged_opts_when_prefer_dotconfig() -> anyhow::Result<()> {
        let input_opts = RunOpts {
            app_name: String::from("web"),
            env_path: None,
            procfile_path: None,
        };

        let dotconfig = prepare_dotconfig();
        let result = merged_opts(&input_opts, &dotconfig);

        assert_eq!(result.app_name, String::from("web"));
        assert_eq!(result.env_path.unwrap(), PathBuf::from(".env"));
        assert_eq!(result.procfile_path.unwrap(), PathBuf::from("./Procfile"));

        Ok(())
    }

    #[test]
    fn test_merged_opts_when_prefer_input_opts() -> anyhow::Result<()> {
        let input_opts = RunOpts {
            app_name: String::from("web"),
            env_path: Some(PathBuf::from("./tmp/.env")),
            procfile_path: Some(PathBuf::from("./tmp/Procfile")),
        };

        let dotconfig = prepare_dotconfig();
        let result = merged_opts(&input_opts, &dotconfig);

        assert_eq!(result.app_name, String::from("web"));
        assert_eq!(result.env_path.unwrap(), PathBuf::from("./tmp/.env"));
        assert_eq!(
            result.procfile_path.unwrap(),
            PathBuf::from("./tmp/Procfile")
        );

        Ok(())
    }
}
