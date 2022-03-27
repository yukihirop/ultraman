use crate::config::{read_config, Config};
use crate::opt::DisplayOpts;
use crate::output;
use crate::process::{self, Process};
use crate::procfile::read_procfile;
use crate::signal;

use std::path::PathBuf;
use std::sync::{Arc, Barrier, Mutex};
use structopt::{clap, StructOpt};

#[derive(StructOpt, Debug)]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct StartOpts {
    /// Specify the number of each process type to run. The value passed in should be in the format process=num,process=num
    #[structopt(name = "APP=NUMBER", short = "m", long = "formation")]
    pub formation: Option<String>,

    /// Specify an environment file to load
    #[structopt(name = "ENV", short = "e", long = "env", parse(from_os_str))]
    pub env_path: Option<PathBuf>,

    /// Specify an Procfile to load
    #[structopt(name = "PROCFILE", short = "f", long = "procfile", parse(from_os_str))]
    pub procfile_path: Option<PathBuf>,

    /// Specify the amount of time (in seconds) processes have to shutdown gracefully before receiving a SIGTERM
    #[structopt(name = "TIMEOUT (sec)", short = "t", long = "timeout")]
    pub timeout: Option<String>,

    /// Specify which port to use as the base for this application. Should be a multiple of 1000
    #[structopt(name = "PORT", short = "p", long = "port")]
    pub port: Option<String>,

    /// Include timestamp in output
    #[structopt(name = "NOTIMESTAMP", short = "n", long = "no-timestamp")]
    pub is_no_timestamp: Option<bool>,
}

pub fn run(input_opts: StartOpts) -> Result<(), Box<dyn std::error::Error>> {
    let mut proc_handles = vec![];
    let procs: Arc<Mutex<Vec<Arc<Mutex<process::Process>>>>> = Arc::new(Mutex::new(vec![]));
    let dotconfig = read_config(PathBuf::from(".ultraman")).unwrap();
    let opts = merged_opts(&input_opts, dotconfig);

    let procfile = read_procfile(opts.procfile_path.unwrap()).expect("failed read Procfile");
    // Read the formation from the command line option and always call it before process_len for the convenience of setting concurrency
    procfile.set_concurrency(&opts.formation.unwrap());

    let process_len = procfile.process_len();
    let padding = procfile.padding();

    let barrier = Arc::new(Barrier::new(process_len + 1));
    let mut total = 0;
    let is_timestamp = !opts.is_no_timestamp.unwrap();
    let display_opts = DisplayOpts {
        padding,
        is_timestamp,
    };

    for (name, pe) in procfile.data.iter() {
        let con = pe.concurrency.get();
        let index = total;
        let output = Arc::new(output::Output::new(index, display_opts.clone()));
        total += 1;

        for n in 0..con {
            let barrier = barrier.clone();
            let procs = procs.clone();
            let output = output.clone();
            let process_name = name.clone();
            let cmd = pe.command.clone();
            let env_path = opts.env_path.clone();
            let port = opts.port.clone();
            let opts = display_opts.clone();

            let exec_and_output_thread = process::build_exec_and_output_thread(move || {
                let proc = Process::new(
                    process_name,
                    cmd,
                    env_path.unwrap(),
                    port,
                    n,
                    index,
                    Some(opts),
                );
                let proc2 = Arc::new(Mutex::new(proc));
                let proc3 = Arc::clone(&proc2);
                let child_id = proc2.lock().unwrap().child.id() as i32;

                output.log.output(
                    "system",
                    &format!(
                        "{0:1$} start at pid: {2}",
                        &proc2.lock().unwrap().name,
                        padding,
                        &child_id
                    ),
                );

                procs.lock().unwrap().push(proc2);
                barrier.wait();

                output.handle_output(&proc3);
            });
            proc_handles.push(exec_and_output_thread);
        }
    }

    barrier.wait();

    // use handle_signal
    let procs2 = Arc::clone(&procs);
    let check_for_child_termination_thread =
        process::build_check_for_child_termination_thread(procs2, display_opts.clone());
    proc_handles.push(check_for_child_termination_thread);

    let procs = Arc::clone(&procs);
    proc_handles.push(signal::handle_signal_thread(
        procs,
        opts.timeout.unwrap().parse::<u64>().unwrap(),
        display_opts,
    ));

    for handle in proc_handles {
        handle.join().expect("failed join");
    }

    Ok(())
}

fn merged_opts(input_opts: &StartOpts, dotconfig: Config) -> StartOpts {
    StartOpts {
        formation: match &input_opts.formation {
            Some(r) => Some(r.to_string()),
            None => dotconfig.formation,
        },
        env_path: match &input_opts.env_path {
            Some(r) => Some(PathBuf::from(r)),
            None => dotconfig.env_path,
        },
        procfile_path: match &input_opts.procfile_path {
            Some(r) => Some(PathBuf::from(r)),
            None => dotconfig.procfile_path,
        },
        port: match &input_opts.port {
            Some(r) => Some(r.to_string()),
            None => dotconfig.port.map(|r| r.to_string()),
        },
        timeout: match &input_opts.timeout {
            Some(r) => Some(r.to_string()),
            None => dotconfig.timeout.map(|r| r.to_string()),
        },
        is_no_timestamp: match &input_opts.is_no_timestamp {
            Some(r) => Some(r.clone()),
            None => dotconfig.is_no_timestamp,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn prepare_dotconfig() -> Config {
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
        let input_opts = StartOpts {
            formation: None,
            env_path: None,
            procfile_path: None,
            port: None,
            timeout: None,
            is_no_timestamp: None,
        };

        let dotconfig = prepare_dotconfig();
        let result = merged_opts(&input_opts, dotconfig);

        assert_eq!(result.formation.unwrap(), "app=1,web=2");
        assert_eq!(result.env_path.unwrap(), PathBuf::from(".env"));
        assert_eq!(result.procfile_path.unwrap(), PathBuf::from("./Procfile"));
        assert_eq!(result.port.unwrap(), "6000");
        assert_eq!(result.timeout.unwrap(), "5000");
        assert_eq!(result.is_no_timestamp.unwrap(), true);

        Ok(())
    }

    #[test]
    fn test_merged_opts_when_prefer_input_opts() -> anyhow::Result<()> {
        let input_opts = StartOpts {
            formation: Some("app=2,web=2,server=2".to_string()),
            env_path: Some(PathBuf::from("./tmp/.env")),
            procfile_path: Some(PathBuf::from("./tmp/Procfile")),
            port: Some("9999".to_string()),
            timeout: Some("1".to_string()),
            is_no_timestamp: Some(false),
        };

        let dotconfig = prepare_dotconfig();
        let result = merged_opts(&input_opts, dotconfig);

        assert_eq!(result.formation.unwrap(), "app=2,web=2,server=2");
        assert_eq!(result.env_path.unwrap(), PathBuf::from("./tmp/.env"));
        assert_eq!(
            result.procfile_path.unwrap(),
            PathBuf::from("./tmp/Procfile")
        );
        assert_eq!(result.port.unwrap(), "9999");
        assert_eq!(result.timeout.unwrap(), "1");
        assert_eq!(result.is_no_timestamp.unwrap(), false);

        Ok(())
    }
}
