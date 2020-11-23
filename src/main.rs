mod log;
mod output;
mod process;
mod script;
mod signal;
mod stream_read;
mod cmd;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cmd::start::run()
}
