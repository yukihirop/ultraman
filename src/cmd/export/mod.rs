use crate::cmd::export::base::Exportable;
use crate::config::{read_config, Config};
use crate::procfile::read_procfile;
use std::path::PathBuf;
use structopt::{clap, StructOpt};

pub mod base;
pub mod daemon;
pub mod launchd;
pub mod runit;
pub mod supervisord;
pub mod systemd;
pub mod upstart;

#[derive(StructOpt, Debug, Default, Clone)]
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
    #[structopt(name = "APP=NUMBER", short = "m", long = "formation")]
    pub formation: Option<String>,

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
    #[structopt(name = "ENV", short = "e", long = "env", parse(from_os_str))]
    pub env_path: Option<PathBuf>,

    /// Specify an Procfile to load
    #[structopt(name = "PROCFILE", short = "f", long = "procfile", parse(from_os_str))]
    pub procfile_path: Option<PathBuf>,

    /// Specify an alternate application root. This defaults to the directory containing the Procfile.
    #[structopt(name = "ROOT", short = "d", long = "root", parse(from_os_str))]
    pub root_path: Option<PathBuf>,

    /// Specify the amount of time (in seconds) processes have to shutdown gracefully before receiving a SIGTERM
    #[structopt(name = "TIMEOUT (sec)", short = "t", long = "timeout")]
    pub timeout: Option<String>,
}

enum ExportFormat {
    Upstart,
    Systemd,
    Supervisord,
    Runit,
    Launchd,
    Daemon,
}

pub fn run(input_opts: ExportOpts) -> Result<(), Box<dyn std::error::Error>> {
    let dotconfig = read_config(PathBuf::from(".ultraman")).unwrap();
    let opts = merged_opts(&input_opts, dotconfig);
    let exporter = new(&opts);
    exporter.export().expect("failed ultraman export");

    Ok(())
}

fn new(opts: &ExportOpts) -> Box<dyn Exportable> {
    let procfile_path = opts.procfile_path.clone().unwrap();
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
            procfile.set_concurrency(&opts.formation.clone().unwrap());
            expo.procfile = procfile;
            expo.opts = opts.clone();
            expo
        }
        ExportFormat::Systemd => {
            let mut expo = systemd::Exporter::boxed_new();
            procfile.set_concurrency(&opts.formation.clone().unwrap());
            expo.procfile = procfile;
            expo.opts = opts.clone();
            expo
        }
        ExportFormat::Supervisord => {
            let mut expo = supervisord::Exporter::boxed_new();
            procfile.set_concurrency(&opts.formation.clone().unwrap());
            expo.procfile = procfile;
            expo.opts = opts.clone();
            expo
        }
        ExportFormat::Runit => {
            let mut expo = runit::Exporter::boxed_new();
            procfile.set_concurrency(&opts.formation.clone().unwrap());
            expo.procfile = procfile;
            expo.opts = opts.clone();
            expo
        }
        ExportFormat::Launchd => {
            let mut expo = launchd::Exporter::boxed_new();
            procfile.set_concurrency(&opts.formation.clone().unwrap());
            expo.procfile = procfile;
            expo.opts = opts.clone();
            expo
        }
        ExportFormat::Daemon => {
            let mut expo = daemon::Exporter::boxed_new();
            procfile.set_concurrency(&opts.formation.clone().unwrap());
            expo.procfile = procfile;
            expo.opts = opts.clone();
            expo
        }
    }
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

fn merged_opts(input_opts: &ExportOpts, dotconfig: Config) -> ExportOpts {
    ExportOpts {
        format: input_opts.format.to_string(),
        location: input_opts.clone().location,
        formation: match &input_opts.formation {
            Some(r) => Some(r.to_string()),
            None => Some(dotconfig.formation),
        },
        env_path: match &input_opts.env_path {
            Some(r) => Some(PathBuf::from(r)),
            None => Some(dotconfig.env_path),
        },
        procfile_path: match &input_opts.procfile_path {
            Some(r) => Some(PathBuf::from(r)),
            None => Some(dotconfig.procfile_path),
        },
        timeout: match &input_opts.timeout {
            Some(r) => Some(r.to_string()),
            None => Some(dotconfig.timeout.to_string()),
        },
        port: match &input_opts.port {
            Some(r) => Some(r.to_string()),
            None => dotconfig.port.map(|r| r.to_string()),
        },
        app: match &input_opts.app {
            Some(r) => Some(r.to_string()),
            None => dotconfig.app,
        },
        log_path: match &input_opts.log_path {
            Some(r) => Some(PathBuf::from(r)),
            None => dotconfig.log_path,
        },
        root_path: match &input_opts.root_path {
            Some(r) => Some(PathBuf::from(r)),
            None => dotconfig.root_path,
        },
        run_path: match &input_opts.run_path {
            Some(r) => Some(PathBuf::from(r)),
            None => dotconfig.run_path,
        },
        template_path: match &input_opts.template_path {
            Some(r) => Some(PathBuf::from(r)),
            None => dotconfig.template_path,
        },
        user: match &input_opts.user {
            Some(r) => Some(r.to_string()),
            None => dotconfig.user,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn prepare_dotconfig() -> Config {
        let dir = tempdir().ok().unwrap();
        let file_path = dir.path().join(".ultraman");
        let mut file = File::create(file_path.clone()).ok().unwrap();
        // Writing a comment causes a parse error
        writeln!(
            file,
            r#"
format: systemd
location: /etc/location
procfile: ./tmp/Procfile
env: ./tmp/.env

formation: app=1,web=2
port: 6000
timeout: 5000

no-timestamp: true

app: app-for-runit
log: /var/app/log/ultraman.log
run: /tmp/pids/ultraman.pid
template: ../../src/cmd/export/templates/supervisord
user: root
root: /home/app

hoge: hogehoge
      "#
        )
        .unwrap();

        let dotconfig = read_config(file_path).expect("failed read .ultraman");
        dotconfig
    }

    #[test]
    fn test_merged_opts_when_prefer_dotconfig() -> anyhow::Result<()> {
        let input_opts = ExportOpts {
            format: String::from("upstart"),
            location: PathBuf::from("./test/location"),
            formation: None,
            env_path: None,
            procfile_path: None,
            port: None,
            timeout: None,
            app: None,
            log_path: None,
            root_path: None,
            run_path: None,
            template_path: None,
            user: None,
        };

        let dotconfig = prepare_dotconfig();
        let result = merged_opts(&input_opts, dotconfig);

        assert_eq!(result.format, "upstart");
        assert_eq!(result.location, PathBuf::from("./test/location"));
        assert_eq!(result.formation.unwrap(), "app=1,web=2");
        assert_eq!(result.env_path.unwrap(), PathBuf::from("./tmp/.env"));
        assert_eq!(
            result.procfile_path.unwrap(),
            PathBuf::from("./tmp/Procfile")
        );
        assert_eq!(result.port.unwrap(), "6000");
        assert_eq!(result.timeout.unwrap(), "5000");
        assert_eq!(result.app.unwrap(), "app-for-runit");
        assert_eq!(
            result.log_path.unwrap(),
            PathBuf::from("/var/app/log/ultraman.log")
        );
        assert_eq!(result.root_path.unwrap(), PathBuf::from("/home/app"));
        assert_eq!(
            result.run_path.unwrap(),
            PathBuf::from("/tmp/pids/ultraman.pid")
        );
        assert_eq!(
            result.template_path.unwrap(),
            PathBuf::from("../../src/cmd/export/templates/supervisord")
        );
        assert_eq!(result.user.unwrap(), "root");

        Ok(())
    }

    #[test]
    fn test_merged_opts_when_prefer_input_opts() -> anyhow::Result<()> {
        let input_opts = ExportOpts {
            format: String::from("upstart"),
            location: PathBuf::from("./test/location"),
            formation: Some("app=2,web=2,server=2".to_string()),
            env_path: Some(PathBuf::from("./test/.env")),
            procfile_path: Some(PathBuf::from("./test/Procfile")),
            port: Some("9999".to_string()),
            timeout: Some("9999".to_string()),
            app: Some("app".to_string()),
            log_path: Some(PathBuf::from("./test/log")),
            root_path: Some(PathBuf::from("./test/root")),
            run_path: Some(PathBuf::from("./test/run")),
            template_path: Some(PathBuf::from("./test/template")),
            user: Some("user".to_string()),
        };

        let dotconfig = prepare_dotconfig();
        let result = merged_opts(&input_opts, dotconfig);

        assert_eq!(result.format, "upstart");
        assert_eq!(result.location, PathBuf::from("./test/location"));
        assert_eq!(result.formation.unwrap(), "app=2,web=2,server=2");
        assert_eq!(result.env_path.unwrap(), PathBuf::from("./test/.env"));
        assert_eq!(
            result.procfile_path.unwrap(),
            PathBuf::from("./test/Procfile")
        );
        assert_eq!(result.port.unwrap(), "9999");
        assert_eq!(result.timeout.unwrap(), "9999");
        assert_eq!(result.app.unwrap(), "app");
        assert_eq!(result.log_path.unwrap(), PathBuf::from("./test/log"));
        assert_eq!(result.root_path.unwrap(), PathBuf::from("./test/root"));
        assert_eq!(result.run_path.unwrap(), PathBuf::from("./test/run"));
        assert_eq!(
            result.template_path.unwrap(),
            PathBuf::from("./test/template")
        );
        assert_eq!(result.user.unwrap(), "user");

        Ok(())
    }
}
