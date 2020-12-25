use crate::output;
use crate::process::{self, Process, ProcessOpts};
use crate::procfile::read_procfile;
use crate::signal;

use std::path::PathBuf;
use std::sync::{Arc, Barrier, Mutex};
use structopt::{clap, StructOpt};

#[derive(StructOpt, Debug)]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct StartOpts {
    /// Specify the number of each process type to run. The value passed in should be in the format process=num,process=num
    #[structopt(
        name = "APP=NUMBER",
        short = "m",
        long = "formation",
        default_value = "all=1"
    )]
    pub formation: String,

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

    /// Specify the amount of time (in seconds) processes have to shutdown gracefully before receiving a SIGTERM
    #[structopt(
        name = "TIMEOUT (sec)",
        short = "t",
        long = "timeout",
        default_value = "5"
    )]
    pub timeout: String,

    /// Specify which port to use as the base for this application. Should be a multiple of 1000
    #[structopt(name = "PORT", short = "p", long = "port")]
    pub port: Option<String>,

    /// Include timestamp in output
    #[structopt(name = "NOTIMESTAMP", short = "n", long = "no-timestamp")]
    pub is_no_timestamp: bool,
}

pub fn run(opts: StartOpts) -> Result<(), Box<dyn std::error::Error>> {
    let mut proc_handles = vec![];
    let procs: Arc<Mutex<Vec<Arc<Mutex<process::Process>>>>> = Arc::new(Mutex::new(vec![]));

    let procfile = read_procfile(opts.procfile_path).expect("failed read Procfile");
    // Read the formation from the command line option and always call it before process_len for the convenience of setting concurrency
    procfile.set_concurrency(&opts.formation);

    let process_len = procfile.process_len();
    let padding = procfile.padding();

    let barrier = Arc::new(Barrier::new(process_len + 1));
    let mut total = 0;
    let is_timestamp = !opts.is_no_timestamp;

    for (name, pe) in procfile.data.iter() {
        let con = pe.concurrency.get();
        let index = total;
        let output = Arc::new(output::Output::new(index, padding, is_timestamp));
        total += 1;

        for n in 0..con {
            let barrier = barrier.clone();
            let procs = procs.clone();
            let output = output.clone();
            let process_name = name.clone();
            let cmd = pe.command.clone();
            let env_path = opts.env_path.clone();
            let port = opts.port.clone();

            let exec_and_output_thread = process::build_exec_and_output_thread(move || {
                let proc = Process::new(
                    process_name,
                    cmd,
                    env_path,
                    port,
                    n,
                    index,
                    Some(ProcessOpts {
                        padding,
                        is_timestamp,
                    }),
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
    proc_handles.push(process::check_for_child_termination_thread(
        procs,
        padding,
        is_timestamp,
    ));

    let procs = Arc::clone(&procs2);
    proc_handles.push(signal::handle_signal_thread(
        procs,
        padding,
        opts.timeout.parse::<u64>().unwrap(),
        is_timestamp,
    ));

    for handle in proc_handles {
        handle.join().expect("failed join");
    }

    Ok(())
}
