use super::base::{Exportable, Template};
use crate::cmd::export::ExportOpts;
use crate::env::read_env;
use crate::process::port_for;
use crate::procfile::{Procfile, ProcfileEntry};
use handlebars::to_json;
use serde_derive::Serialize;
use serde_json::value::{Map, Value as Json};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub struct Exporter {
    pub procfile: Procfile,
    pub opts: ExportOpts,
}

#[derive(Serialize)]
struct RunParams {
    work_dir: String,
    user: String,
    env_dir_path: String,
    process_command: String,
}

#[derive(Serialize)]
struct LogRunParams {
    log_path: String,
    user: String,
}

impl Default for Exporter {
    fn default() -> Self {
        Exporter {
            procfile: Procfile {
                data: HashMap::new(),
            },
            opts: ExportOpts {
                format: String::from(""),
                location: PathBuf::from("location"),
                app: None,
                formation: String::from("all=1"),
                log_path: None,
                run_path: None,
                port: None,
                template_path: None,
                user: None,
                env_path: PathBuf::from(".env"),
                procfile_path: PathBuf::from("Procfile"),
                root_path: Some(env::current_dir().unwrap()),
                timeout: String::from("5"),
            }
        }
    }
}

impl Exporter {
    fn boxed(self) -> Box<Self> {
        Box::new(self)
    }

    pub fn boxed_new() -> Box<Self> {
        Self::default().boxed()
    }

    fn run_tmpl_path(&self) -> PathBuf {
        let mut path = self.project_root_path();
        let tmpl_path = PathBuf::from("src/cmd/export/templates/runit/run.hbs");
        path.push(tmpl_path);
        path
    }

    fn log_run_tmpl_path(&self) -> PathBuf {
        let mut path = self.project_root_path();
        let tmpl_path = PathBuf::from("src/cmd/export/templates/runit/log/run.hbs");
        path.push(tmpl_path);
        path
    }

    fn make_run_data(&self, pe: &ProcfileEntry, env_dir_path: &PathBuf) -> Map<String, Json> {
        let mut data = Map::new();
        let rp = RunParams {
            work_dir: self.root_path().into_os_string().into_string().unwrap(),
            user: self.username(),
            env_dir_path: env_dir_path.clone().into_os_string().into_string().unwrap(),
            process_command: pe.command.to_string(),
        };
        data.insert("run".to_string(), to_json(&rp));
        data
    }

    fn make_log_run_data(&self, process_name: &str) -> Map<String, Json> {
        let mut data = Map::new();
        let log_path = format!(
            "{}/{}",
            self.log_path().into_os_string().into_string().unwrap(),
            &process_name
        );
        let lr = LogRunParams {
            log_path,
            user: self.username(),
        };
        data.insert("log_run".to_string(), to_json(&lr));
        data
    }

    fn write_env(&self, output_dir_path: &PathBuf, index: usize, con_index: usize) {
        let mut env = read_env(self.opts.env_path.clone()).expect("failed read .env");
        let port = port_for(self.opts.env_path.clone(), self.opts.port.clone(), index, con_index + 1);
        env.insert("PORT".to_string(), port);

        for (key, val) in env.iter() {
            let path = output_dir_path.join(&key);
            let display = path.clone().into_os_string().into_string().unwrap();
            self.clean(&path);
            let mut file =
                File::create(path.clone()).expect(&format!("Could not create file: {}", &display));
            self.say(&format!("writing: {}", &display));
            writeln!(&mut file, "{}", &val).expect(&format!("Could not write file: {}", &display));
        }
    }
}

struct EnvTemplate {
    template_path: PathBuf,
    index: usize,
    con_index: usize,
}

impl Exportable for Exporter {
    fn export(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.base_export().expect("failed execute base_export");

        let mut index = 0;
        let mut clean_paths: Vec<PathBuf> = vec![];
        let mut create_recursive_dir_paths: Vec<PathBuf> = vec![];
        let mut tmpl_data: Vec<Template> = vec![];
        let mut env_data: Vec<EnvTemplate> = vec![];

        for (name, pe) in self.procfile.data.iter() {
            let con = pe.concurrency.get();
            for n in 0..con {
                index += 1;
                let process_name = format!("{}-{}", &name, n + 1);
                let service_name = format!("{}-{}-{}", self.app(), &name, n + 1);
                let mut path_for_run = self.opts.location.clone();
                let mut path_for_env = path_for_run.clone();
                let mut path_for_log = path_for_run.clone();
                let run_file_path = PathBuf::from(format!("{}/run", &service_name));
                let env_dir_path = PathBuf::from(format!("{}/env", &service_name));
                let log_dir_path = PathBuf::from(format!("{}/log", &service_name));
                path_for_run.push(run_file_path);
                path_for_env.push(env_dir_path);
                path_for_log.push(log_dir_path);
                
                create_recursive_dir_paths.push(path_for_env.clone());
                create_recursive_dir_paths.push(path_for_log.clone());

                let run_data = self.make_run_data(
                    pe,
                    &PathBuf::from(format!("/etc/service/{}/env", &service_name)),
                );
                let log_run_data = self.make_log_run_data(&process_name);

                clean_paths.push(path_for_run.clone());
                tmpl_data.push(Template{
                    template_path: self.run_tmpl_path(),
                    data: run_data,
                    output_path: path_for_run,
                });

                path_for_log.push("run");
                clean_paths.push(path_for_log.clone());
                tmpl_data.push(Template{
                    template_path: self.log_run_tmpl_path(),
                    data: log_run_data,
                    output_path: path_for_log
                });
                env_data.push(EnvTemplate{
                    template_path: path_for_env.clone(),
                    index,
                    con_index: n
                });
            }
        }

        for path in clean_paths {
            self.clean(&path);
        }

        for dir_path in create_recursive_dir_paths {
            self.create_dir_recursive(&dir_path);
        }

        for tmpl in tmpl_data {
            let mut data = tmpl.data;
            self.write_template(&tmpl.template_path, &mut data, &tmpl.output_path);
        }

        for e in env_data {
            self.write_env(&e.template_path, e.index, e.con_index);
        }

        Ok(())
    }

    fn ref_opts(&self) -> &ExportOpts {
        &self.opts
    }
}
