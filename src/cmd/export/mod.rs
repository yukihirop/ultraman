use crate::cmd::export::base::Exportable;
use crate::procfile::read_procfile;
use std::path::PathBuf;
use structopt::{clap, StructOpt};

pub mod base;
pub mod upstart;
pub mod systemd;
pub mod supervisord;
pub mod runit;
pub mod launchd;
pub mod daemon;

#[derive(StructOpt, Debug, Default)]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct ExportOpts {
    /// Specify  process management format
    #[structopt(name = "FORMAT")]
    pub format: String,

    /// Specift the path to export
    #[structopt(name = "LOCATION")]
    pub location: PathBuf,

    /// Use this name rather than the application's root directory name as the name of the application when exporting
    #[structopt(name = "APP", short = "a", long = "app")]
    pub app: Option<String>,

    /// Specify the number of each process type to run. The value passed in should be in the format process=num,process=num
    #[structopt(
        name = "APP=NUMBER",
        short = "m",
        long = "formation",
        default_value = "all=1"
    )]
    pub formation: String,

    /// Specify the directory to place process logs in
    #[structopt(name = "LOG", short = "l", long = "log", parse(from_os_str))]
    pub log_path: Option<PathBuf>,

    /// Specify the pid file directory, defaults to /var/run/<application>
    #[structopt(name = "RUN", short = "r", long = "run", parse(from_os_str))]
    pub run_path: Option<PathBuf>,

    /// Specify which port to use as the base for this application. Should be a multiple of 1000
    #[structopt(name = "PORT", short = "p", long = "port")]
    pub port: Option<String>,

    /// Specify an template to use for creating export files
    #[structopt(name = "TEMPLATE", short = "T", long = "template")]
    pub template_path: Option<PathBuf>,

    /// Specify the user the application should be run as. Defaults to the app name
    #[structopt(name = "USER", short = "u", long = "user")]
    pub user: Option<String>,

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

    /// Specify an alternate application root. This defaults to the directory containing the Procfile.
    #[structopt(
        name = "ROOT",
        short = "d",
        long = "root",
        parse(from_os_str),
    )]
    pub root_path: Option<PathBuf>,

    /// Specify the amount of time (in seconds) processes have to shutdown gracefully before receiving a SIGTERM
    #[structopt(
        name = "TIMEOUT (sec)",
        short = "t",
        long = "timeout",
        default_value = "5"
    )]
    pub timeout: String,
}

enum ExportFormat {
    Upstart,
    Systemd,
    Supervisord,
    Runit,
    Launchd,
    Daemon,
}

fn new(opts: &ExportOpts) -> Box<dyn Exportable> {
    let procfile_path = opts.procfile_path.clone();
    let display = procfile_path
        .clone()
        .into_os_string()
        .into_string()
        .unwrap();
    let procfile =
        read_procfile(procfile_path).expect(&format!("Could not read Procfile: {}", display));
    let format = opts.format.as_str();
    
    match export_format(format) {
        ExportFormat::Upstart => {
            let mut expo = upstart::Exporter::boxed_new();
            // Read the formation from the command line option and always call it before process_len for the convenience of setting concurrency
            procfile.set_concurrency(&opts.formation);
            expo.procfile = procfile;
            expo.format = opts.format.clone();
            expo.location = opts.location.clone();
            expo.app = opts.app.clone();
            expo.formation = opts.formation.clone();
            expo.log_path = opts.log_path.clone();
            expo.run_path = opts.run_path.clone();
            expo.port = opts.port.clone();
            expo.template_path = opts.template_path.clone();
            expo.user = opts.user.clone();
            expo.env_path = opts.env_path.clone();
            expo.procfile_path = opts.procfile_path.clone();
            expo.root_path = opts.root_path.clone();
            expo.timeout = opts.timeout.clone();
            expo
        },
        ExportFormat::Systemd => {
            let mut expo = systemd::Exporter::boxed_new();
            procfile.set_concurrency(&opts.formation);
            expo.procfile = procfile;
            expo.format = opts.format.clone();
            expo.location = opts.location.clone();
            expo.app = opts.app.clone();
            expo.formation = opts.formation.clone();
            expo.log_path = opts.log_path.clone();
            expo.run_path = opts.run_path.clone();
            expo.port = opts.port.clone();
            expo.template_path = opts.template_path.clone();
            expo.user = opts.user.clone();
            expo.env_path = opts.env_path.clone();
            expo.procfile_path = opts.procfile_path.clone();
            expo.root_path = opts.root_path.clone();
            expo.timeout = opts.timeout.clone();
            expo
        },
        ExportFormat::Supervisord => {
            let mut expo = supervisord::Exporter::boxed_new();
            procfile.set_concurrency(&opts.formation);
            expo.procfile = procfile;
            expo.format = opts.format.clone();
            expo.location = opts.location.clone();
            expo.app = opts.app.clone();
            expo.formation = opts.formation.clone();
            expo.log_path = opts.log_path.clone();
            expo.run_path = opts.run_path.clone();
            expo.port = opts.port.clone();
            expo.template_path = opts.template_path.clone();
            expo.user = opts.user.clone();
            expo.env_path = opts.env_path.clone();
            expo.procfile_path = opts.procfile_path.clone();
            expo.root_path = opts.root_path.clone();
            expo.timeout = opts.timeout.clone();
            expo
        },
        ExportFormat::Runit => {
            let mut expo = runit::Exporter::boxed_new();
            procfile.set_concurrency(&opts.formation);
            expo.procfile = procfile;
            expo.format = opts.format.clone();
            expo.location = opts.location.clone();
            expo.app = opts.app.clone();
            expo.formation = opts.formation.clone();
            expo.log_path = opts.log_path.clone();
            expo.run_path = opts.run_path.clone();
            expo.port = opts.port.clone();
            expo.template_path = opts.template_path.clone();
            expo.user = opts.user.clone();
            expo.env_path = opts.env_path.clone();
            expo.procfile_path = opts.procfile_path.clone();
            expo.root_path = opts.root_path.clone();
            expo.timeout = opts.timeout.clone();
            expo
        },
        ExportFormat::Launchd => {
            let mut expo = launchd::Exporter::boxed_new();
            procfile.set_concurrency(&opts.formation);
            expo.procfile = procfile;
            expo.format = opts.format.clone();
            expo.location = opts.location.clone();
            expo.app = opts.app.clone();
            expo.formation = opts.formation.clone();
            expo.log_path = opts.log_path.clone();
            expo.run_path = opts.run_path.clone();
            expo.port = opts.port.clone();
            expo.template_path = opts.template_path.clone();
            expo.user = opts.user.clone();
            expo.env_path = opts.env_path.clone();
            expo.procfile_path = opts.procfile_path.clone();
            expo.root_path = opts.root_path.clone();
            expo.timeout = opts.timeout.clone();
            expo
        },
        ExportFormat::Daemon => {
            let mut expo = daemon::Exporter::boxed_new();
            procfile.set_concurrency(&opts.formation);
            expo.procfile = procfile;
            expo.format = opts.format.clone();
            expo.location = opts.location.clone();
            expo.app = opts.app.clone();
            expo.formation = opts.formation.clone();
            expo.log_path = opts.log_path.clone();
            expo.run_path = opts.run_path.clone();
            expo.port = opts.port.clone();
            expo.template_path = opts.template_path.clone();
            expo.user = opts.user.clone();
            expo.env_path = opts.env_path.clone();
            expo.procfile_path = opts.procfile_path.clone();
            expo.root_path = opts.root_path.clone();
            expo.timeout = opts.timeout.clone();
            expo
        }
    }
}

pub fn run(opts: ExportOpts) -> Result<(), Box<dyn std::error::Error>> {
    let exporter = new(&opts);
    exporter.export().expect("failed rustman export");

    Ok(())
}

fn export_format(format: &str) -> ExportFormat {
    if format == "upstart" {
        ExportFormat::Upstart
    } else if format == "systemd" {
        ExportFormat::Systemd
    } else if format == "supervisord" {
        ExportFormat::Supervisord
    } else if format == "runit" {
        ExportFormat::Runit
    } else if format == "launchd" {
        ExportFormat::Launchd
    } else if format == "daemon" {
        ExportFormat::Daemon
    } else {
        panic!("Do not support format {}", format)
    }
}
