use crate::process;
use crate::output;
use crate::signal;
use crate::procfile::{read_procfile};

use std::sync::{Arc, Barrier, Mutex};
use structopt::{clap, StructOpt};
use std::path::PathBuf;

#[derive(StructOpt, Debug)]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct StartOpts {
}

pub fn run(procfile_path: PathBuf, _opts: StartOpts) -> Result<(), Box<dyn std::error::Error>> {
    let mut proc_handles = vec![];
    let procs: Arc<Mutex<Vec<Arc<Mutex<process::Process>>>>> = Arc::new(Mutex::new(vec![]));

    let procfile = read_procfile(procfile_path).expect("failed read Procfile");
    let process_len = procfile.process_len();
    let padding = procfile.padding();

    let barrier = Arc::new(Barrier::new(process_len + 1));
    let mut index = 0;

    for pe in procfile.entries {
        let con = pe.concurrency;
        let output = Arc::new(output::Output::new(index, padding));
        index += 1;

        for n in 0..con {
            let barrier = barrier.clone();
            let procs = procs.clone();
            let output = output.clone();
            let pe_name = pe.name.clone();
            let pe_command = pe.command.clone();

            let each_fn = process::each_handle_exec_and_output(procs, padding, barrier, output);
            let each_handle_exec_and_output = each_fn(pe_name, n, pe_command);
            proc_handles.push(each_handle_exec_and_output);
        }
    }

    barrier.wait();

    // use handle_signal
    let procs2 = Arc::clone(&procs);
    proc_handles.push(process::check_child_terminated(procs, padding));

    let procs = Arc::clone(&procs2);
    proc_handles.push(signal::handle_signal(procs, padding));

    for handle in proc_handles {
        handle.join().expect("failed join");
    }

    Ok(())
}
