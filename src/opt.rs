use crate::cmd::start::StartOpts;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
pub struct Opt {
    #[structopt(subcommand)]
    pub subcommands: Option<Rustman>,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "rustman")]
pub enum Rustman {
    #[structopt(name = "start", about = "Start the application")]
    Start(StartOpts),
}
