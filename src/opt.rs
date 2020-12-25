use crate::cmd::export::ExportOpts;
use crate::cmd::run::RunOpts;
use crate::cmd::start::StartOpts;
use structopt::{clap, StructOpt};

#[derive(StructOpt, Debug)]
#[structopt(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct Opt {
    #[structopt(subcommand)]
    pub subcommands: Option<Ultraman>,
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "ultraman",
    about = "Ultraman is a manager for Procfile-based applications. Its aim is to abstract away the details of the Procfile format, and allow you to either run your application directly or export it to some other process management format."
)]
pub enum Ultraman {
    #[structopt(name = "start", about = "Start the application")]
    Start(StartOpts),

    #[structopt(
        name = "run",
        about = "Run a command using your application's environment"
    )]
    Run(RunOpts),

    #[structopt(
        name = "export",
        about = "Export the application to another process management format"
    )]
    Export(ExportOpts),
}

///// Options not related to commands /////

#[derive(Clone)]
pub struct DisplayOpts {
    pub padding: usize,
    pub is_timestamp: bool,
}

impl Default for DisplayOpts {
    fn default()-> Self {
        DisplayOpts {
            padding: 0,
            is_timestamp: true
        }
    }
}
