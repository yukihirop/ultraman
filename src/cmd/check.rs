use crate::config::{read_config, Config};
use crate::procfile::read_procfile;

use std::path::PathBuf;
use structopt::{clap, StructOpt};

#[cfg(not(test))]
use std::process::exit;

#[derive(StructOpt, Debug)]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct CheckOpts {
    /// Specify an Procfile to load
    #[structopt(name = "PROCFILE", short = "f", long = "procfile", parse(from_os_str))]
    pub procfile_path: Option<PathBuf>,
}

pub fn run(input_opts: CheckOpts) {
    let dotconfig = read_config(PathBuf::from("./ultraman")).unwrap();
    let opts = merged_opts(&input_opts, dotconfig);

    let procfile_path = opts.procfile_path.unwrap();
    if !&procfile_path.exists() {
        let display_path = procfile_path.into_os_string().into_string().unwrap();
        eprintln!("{} does not exist.", &display_path);
        // https://www.reddit.com/r/rust/comments/emz456/testing_whether_functions_exit/
        #[cfg(not(test))]
        exit(1);
        #[cfg(test)]
        panic!("exit {}", 1);
    }
    let procfile = read_procfile(procfile_path).expect("failed read Procfile");

    if !procfile.check() {
        eprintln!("no process defined");
    } else {
        println!("valid procfile detected ({})", procfile.process_names());
    }
}

fn merged_opts(input_opts: &CheckOpts, dotconfig: Config) -> CheckOpts {
    CheckOpts {
        procfile_path: match &input_opts.procfile_path {
            Some(r) => Some(PathBuf::from(r)),
            None => Some(dotconfig.procfile_path),
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
        let input_opts = CheckOpts {
            procfile_path: None,
        };

        let dotconfig = prepare_dotconfig();
        let result = merged_opts(&input_opts, dotconfig);

        assert_eq!(
            result.procfile_path.unwrap(),
            PathBuf::from("./tmp/Procfile")
        );

        Ok(())
    }

    #[test]
    fn test_merged_opts_when_prefer_input_opts() -> anyhow::Result<()> {
        let input_opts = CheckOpts {
            procfile_path: Some(PathBuf::from("./test/Procfile")),
        };

        let dotconfig = prepare_dotconfig();
        let result = merged_opts(&input_opts, dotconfig);

        assert_eq!(
            result.procfile_path.unwrap(),
            PathBuf::from("./test/Procfile")
        );

        Ok(())
    }
}
