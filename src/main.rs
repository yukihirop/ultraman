use std::process::{Command, Stdio};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

mod log;
mod output;
mod process;
mod signal;
mod stream_read;
mod procfile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut proc_handles = vec![];
    let procs: Arc<Mutex<Vec<Arc<Mutex<process::Process>>>>> = Arc::new(Mutex::new(vec![]));
    let scripts = procfile::scripts();
    let process_len = procfile::process_ln();
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
    }

    barrier.wait();

    // use handle_signal
    let procs_2 = Arc::clone(&procs);
    proc_handles.push(process::check_child_terminated(procs, padding));

    let procs = Arc::clone(&procs_2);
    proc_handles.push(signal::handle_signal(procs, padding));

    for handle in proc_handles {
        handle.join().expect("failed join");
    }

    Ok(())
}
