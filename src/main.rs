use opt::{Opt, Rustman};
use structopt::StructOpt;

mod cmd;
mod env;
mod log;
mod opt;
mod output;
mod process;
mod procfile;
mod signal;
mod stream_read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    if let Some(subcommand) = opt.subcommands {
        match subcommand {
            Rustman::Start(opts) => cmd::start::run(opts).expect("failed rustman start"),
            Rustman::Run(opts) => cmd::run::run(opts),
        }
    }

    Ok(())
}
