use std::sync::{Arc, Barrier, Mutex};

mod log;
mod output;
mod process;
mod procfile;
mod signal;
mod stream_read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut proc_handles = vec![];
    let procs: Arc<Mutex<Vec<Arc<Mutex<process::Process>>>>> = Arc::new(Mutex::new(vec![]));
    let scripts = procfile::scripts();
    let process_len = procfile::process_len();
    let padding = procfile::padding();
    let barrier = Arc::new(Barrier::new(process_len + 1));
    let mut index = 0;

    for (key, script) in scripts {
        let con = script.concurrency;
        let script = Arc::new(script);
        let output = Arc::new(output::Output::new(index, padding));
        index += 1;

        for n in 0..con {
            let barrier = barrier.clone();
            let script = script.clone();
            let procs = procs.clone();
            let output = output.clone();

            let each_fn = process::each_handle_exec_and_output(procs, padding, barrier, output);
            let each_handle_exec_and_output =
                each_fn(String::from(key), n, String::from(&script.cmd));
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
