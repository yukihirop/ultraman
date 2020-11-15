use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;

mod output;
mod stream_read;
mod process;
mod log;

struct Script {
    cmd: String,
    concurrency: usize
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut proc_handles = vec![];
    
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

    for (key, script) in scripts {
        let con = script.concurrency;

        for n in 0..con {
            let tmp_proc = process::Process {
                name: String::from(format!("{}.{}", key, n+1)),
                child: Command::new(&script.cmd)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?,
            };

            let proc = Arc::new(Mutex::new(tmp_proc));

            let handle_output = thread::Builder::new()
                .name(String::from("handling output"))
                .spawn(move || {
                    output::handle_output(&proc)
                })?;
        
            proc_handles.push(handle_output);

        }
    }

    for handle in proc_handles {
        handle.join().expect("failed join");
    }

    Ok(())
}
