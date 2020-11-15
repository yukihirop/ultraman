use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

mod output;
mod stream_read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut proc_handles = Vec::new();

    for _n in 1..3 {
        let tmp_child = Command::new("./bin/web.sh")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

        let child = Arc::new(Mutex::new(tmp_child));

        let handle = thread::Builder::new()
            .name(String::from("web-app"))
            .spawn(move || {
                output::handle_output(&child)
            })?;
        
        proc_handles.push(handle);
    }

    for handle in proc_handles {
        handle.join().expect("join")
    }

    Ok(())
}
