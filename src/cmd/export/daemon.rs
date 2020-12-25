use super::base::{EnvParameter, Exportable, Template};
use crate::cmd::export::ExportOpts;
use crate::env::read_env;
use crate::process::port_for;
use crate::procfile::{Procfile, ProcfileEntry};
use handlebars::to_json;
use serde_derive::Serialize;
use serde_json::value::{Map, Value as Json};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

pub struct Exporter {
    pub procfile: Procfile,
    pub opts: ExportOpts,
}

#[derive(Serialize)]
struct MasterParams {
    user: String,
    log_dir_path: String,
    run_dir_path: String,
}

#[derive(Serialize)]
struct ProcessMasterParams {
    app: String,
}

#[derive(Serialize)]
struct ProcessParams {
    service_name: String,
    env: Vec<EnvParameter>,
    user: String,
    work_dir: String,
    pid_path: String,
    command: String,
    command_args: String,
    log_path: String,
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
            },
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

    fn master_tmpl_path(&self) -> PathBuf {
        let path = self.project_root_path();
        path.join("src/cmd/export/templates/daemon/master.conf.hbs")
    }

    fn process_master_tmpl_path(&self) -> PathBuf {
        let path = self.project_root_path();
        path.join("src/cmd/export/templates/daemon/process_master.conf.hbs")
    }

    fn process_tmpl_path(&self) -> PathBuf {
        let path = self.project_root_path();
        path.join("src/cmd/export/templates/daemon/process.conf.hbs")
    }

    fn make_master_data(&self) -> Map<String, Json> {
        let mut data = Map::new();
        let mp = MasterParams {
            log_dir_path: self.log_path().into_os_string().into_string().unwrap(),
            run_dir_path: self.run_path().into_os_string().into_string().unwrap(),
            user: self.username(),
        };
        data.insert("master".to_string(), to_json(&mp));
        data
    }

    fn make_process_master_data(&self) -> Map<String, Json> {
        let mut data = Map::new();
        let pm = ProcessMasterParams { app: self.app() };
        data.insert("process_master".to_string(), to_json(&pm));
        data
    }

    fn make_process_data(
        &self,
        pe: &ProcfileEntry,
        service_name: &str,
        index: usize,
        con_index: usize,
    ) -> Map<String, Json> {
        let mut data = Map::new();
        let pp = ProcessParams {
            service_name: service_name.to_string(),
            env: self.environment(index, con_index),
            user: self.username(),
            work_dir: self.root_path().into_os_string().into_string().unwrap(),
            pid_path: self
                .run_path()
                .join(format!("{}.pid", &service_name))
                .into_os_string()
                .into_string()
                .unwrap(),
            command: self.command_args(pe).get(0).unwrap().to_string(),
            command_args: self.command_args_str(pe),
            log_path: self
                .log_path()
                .join(format!("{}.log", &service_name))
                .into_os_string()
                .into_string()
                .unwrap(),
        };
        data.insert("process".to_string(), to_json(&pp));
        data
    }

    fn command_args_str(&self, pe: &ProcfileEntry) -> String {
        let args = self.command_args(pe);
        if args.len() > 1 {
            format!(" -- {}", &args[1..].join(" "))
        } else {
            "".to_string()
        }
    }

    fn command_args(&self, pe: &ProcfileEntry) -> Vec<String> {
        let data = pe.command.split(" ").collect::<Vec<_>>();
        let mut result = vec![];
        for item in data {
            result.push(item.to_string())
        }
        result
    }

    fn environment(&self, index: usize, con_index: usize) -> Vec<EnvParameter> {
        let port = port_for(
            self.opts.env_path.clone(),
            self.opts.port.clone(),
            index,
            con_index + 1,
        );
        let mut env = read_env(self.opts.env_path.clone()).expect("failed read .env");
        env.insert("PORT".to_string(), port);

        let mut result = vec![];
        for (key, val) in env.iter() {
            result.push(EnvParameter {
                key: key.to_string(),
                value: val.to_string(),
            });
        }

        result
    }
}

impl Exportable for Exporter {
    fn export(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.base_export().expect("failed execute base_export");

        let mut clean_paths: Vec<PathBuf> = vec![];
        let mut tmpl_data: Vec<Template> = vec![];

        let output_path = self.opts.location.join(format!("{}.conf", self.app()));

        clean_paths.push(output_path.clone());
        tmpl_data.push(Template {
            template_path: self.master_tmpl_path(),
            data: self.make_master_data(),
            output_path,
        });

        let mut index = 0;
        for (name, pe) in self.procfile.data.iter() {
            let con = pe.concurrency.get();
            let service_name = format!("{}-{}", self.app(), &name);
            let output_path = self
                .opts
                .location
                .join(format!("{}-{}.conf", self.app(), &name));

            clean_paths.push(output_path.clone());
            tmpl_data.push(Template {
                template_path: self.process_master_tmpl_path(),
                data: self.make_process_master_data(),
                output_path,
            });

            for n in 0..con {
                index += 1;
                let process_name = format!("{}-{}-{}.conf", self.app(), &name, n + 1);
                let output_path = self.opts.location.join(&process_name);

                clean_paths.push(output_path.clone());
                tmpl_data.push(Template {
                    template_path: self.process_tmpl_path(),
                    data: self.make_process_data(pe, &service_name, index, n),
                    output_path,
                });
            }
        }

        for path in clean_paths {
            self.clean(&path);
        }

        for tmpl in tmpl_data {
            self.write_template(tmpl);
        }

        Ok(())
    }

    fn ref_opts(&self) -> &ExportOpts {
        &self.opts
    }
}
