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
use std::marker::PhantomData;
use std::path::PathBuf;

pub struct Exporter<'a> {
    pub procfile: Procfile,
    pub opts: ExportOpts,
    _marker: PhantomData<&'a ()>,
}

#[derive(Serialize)]
struct LaunchdParams<'a> {
    label: &'a str,
    env: Vec<EnvParameter>,
    command_args: Vec<&'a str>,
    stdout_path: &'a str,
    stderr_path: &'a str,
    user: &'a str,
    work_dir: &'a str,
}

impl<'a> Default for Exporter<'a> {
    fn default() -> Self {
        Exporter {
            procfile: Procfile {
                data: HashMap::new(),
            },
            opts: ExportOpts {
                format: String::from(""),
                location: PathBuf::from("location"),
                app: None,
                formation: Some(String::from("all=1")),
                log_path: None,
                run_path: None,
                port: None,
                template_path: None,
                user: None,
                env_path: Some(PathBuf::from(".env")),
                procfile_path: Some(PathBuf::from("Procfile")),
                root_path: Some(env::current_dir().unwrap()),
                timeout: Some(5),
            },
            _marker: PhantomData,
        }
    }
}

impl<'a> Exporter<'a> {
    fn boxed(self) -> Box<Self> {
        Box::new(self)
    }

    pub fn boxed_new() -> Box<Self> {
        Self::default().boxed()
    }

    fn launchd_tmpl_path(&self) -> PathBuf {
        let mut path = self.project_root_path();
        let tmpl_path = PathBuf::from("src/cmd/export/templates/launchd/launchd.plist.hbs");
        path.push(tmpl_path);
        path
    }

    fn make_launchd_data(
        &self,
        pe: &ProcfileEntry,
        service_name: &str,
        con_index: usize,
    ) -> Map<String, Json> {
        let mut data = Map::new();
        let log_display = self.log_path().into_os_string().into_string().unwrap();
        let lp = LaunchdParams {
            label: service_name,
            env: self.environment(con_index),
            command_args: self.command_args(pe),
            stdout_path: &format!("{}/{}.log", &log_display, &service_name),
            stderr_path: &format!("{}/{}.error.log", &log_display, &service_name),
            user: self.username(),
            work_dir: &self.root_path().into_os_string().into_string().unwrap(),
        };
        data.insert("launchd".to_string(), to_json(&lp));
        data
    }

    fn command_args(&self, pe: &'a ProcfileEntry) -> Vec<&'a str> {
        let data = pe.command.split(" ").collect::<Vec<_>>();
        let mut result: Vec<&'a str> = vec![];
        for item in data {
            result.push(item)
        }
        result
    }

    fn environment(&self, con_index: usize) -> Vec<EnvParameter> {
        let port = port_for(
            &self.opts.env_path.clone().unwrap(),
            self.opts.port.clone(),
            con_index,
        );
        let mut env = read_env(self.opts.env_path.clone().unwrap()).expect("failed read .env");
        env.insert("PORT".to_string(), port.to_string());

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

impl<'a> Exportable for Exporter<'a> {
    fn export(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.base_export().expect("failed execute base_export");

        let mut clean_paths: Vec<PathBuf> = vec![];
        let mut tmpl_data: Vec<Template> = vec![];

        for (name, pe) in self.procfile.data.iter() {
            let con = pe.concurrency.get();
            for n in 0..con {
                let service_name = format!("{}-{}-{}", self.app(), &name, n + 1);
                let output_path = self.opts.location.join(&service_name);

                clean_paths.push(output_path.clone());
                tmpl_data.push(Template {
                    template_path: self.launchd_tmpl_path(),
                    data: self.make_launchd_data(pe, &service_name, n),
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
