use super::base::{Exportable, Template};
use crate::cmd::export::ExportOpts;
use crate::env::read_env;
use crate::process::port_for;
use crate::procfile::Procfile;
use handlebars::to_json;
use regex::Regex;
use serde_derive::Serialize;
use serde_json::value::{Map, Value as Json};
use shellwords::escape;
use std::collections::HashMap;
use std::env;
use std::marker::PhantomData;
use std::path::PathBuf;

const ENV_REGEXP: &'static str = "\\$\\{*(?P<envname>[A-Za-z0-9_-]+)\\}*";

pub struct Exporter<'a> {
    pub procfile: Procfile,
    pub opts: ExportOpts,
    _marker: PhantomData<&'a ()>,
}

#[derive(Serialize)]
struct AppConfDataParams<'a> {
    user: &'a str,
    work_dir: String,
    program: String,
    process_command: String,
    environment: String,
    stdout_logfile: String,
    stderr_logfile: String,
}

#[derive(Serialize)]
struct AppConfParams<'a> {
    app: &'a str,
    service_names: &'a str,
    data: Vec<AppConfDataParams<'a>>,
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
                timeout: Some(String::from("5")),
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

    fn app_conf_tmpl_path(&self) -> PathBuf {
        let mut path = self.project_root_path();
        let tmpl_path = PathBuf::from("src/cmd/export/templates/supervisord/app.conf.hbs");
        path.push(tmpl_path);
        path
    }

    fn make_app_conf_data(
        &self,
        service_names: Vec<String>,
        data: Vec<AppConfDataParams>,
    ) -> Map<String, Json> {
        let mut tmpldata = Map::new();
        let ac = AppConfParams {
            app: self.app(),
            service_names: &service_names
                .iter()
                .map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(","),
            data,
        };
        tmpldata.insert("app_conf".to_string(), to_json(&ac));
        tmpldata
    }

    fn environment(&self, index: usize, con_index: usize) -> String {
        let port = port_for(
            &self.opts.env_path.clone().unwrap(),
            self.opts.port.clone(),
            index,
            con_index + 1,
        );
        let mut env = read_env(self.opts.env_path.clone().unwrap()).expect("failed read .env");
        env.insert("PORT".to_string(), port.to_string());

        let mut result = vec![];
        for (key, val) in env.iter() {
            result.push(format!("{}=\"{}\"", &key, escape(&val)))
        }

        result.join(",")
    }

    // http://supervisord.org/configuration.html?highlight=environment#environment-variables
    fn replace_env_for_supervisord(&self, command: &str) -> String {
        let re_env = Regex::new(ENV_REGEXP).unwrap();
        let result = re_env.replace_all(command, "%(ENV_$envname)s");
        result.to_string()
    }
}

impl<'a> Exportable for Exporter<'a> {
    fn export(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.base_export().expect("failed execute base_export");

        let mut index = 0;
        let mut service_names = vec![];
        let mut data: Vec<AppConfDataParams> = vec![];
        for (name, pe) in self.procfile.data.iter() {
            index += 1;
            let con = pe.concurrency.get();
            for n in 0..con {
                let program = format!("{}-{}-{}", self.app(), &name, n + 1);
                let process_command = self.replace_env_for_supervisord(&pe.command);
                let environment = self.environment(index, n);
                let display_log = self.log_path().into_os_string().into_string().unwrap();
                let stdout_logfile = format!("{}/{}-{}.log", &display_log, &name, n + 1);
                let stderr_logfile = format!("{}/{}-{}.error.log", &display_log, &name, n + 1);
                service_names.push(program.clone());
                data.push(AppConfDataParams {
                    user: self.username(),
                    work_dir: self.root_path().into_os_string().into_string().unwrap(),
                    program,
                    process_command,
                    environment,
                    stdout_logfile,
                    stderr_logfile,
                });
            }
        }

        let output_path = self.output_path("app.conf");
        self.clean(&output_path);
        self.write_template(Template {
            template_path: self.app_conf_tmpl_path(),
            data: self.make_app_conf_data(service_names, data),
            output_path,
        });

        Ok(())
    }

    fn ref_opts(&self) -> &ExportOpts {
        &self.opts
    }
}
