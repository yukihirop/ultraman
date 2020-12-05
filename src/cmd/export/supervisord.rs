use super::base::{Exportable};
use crate::cmd::export::ExportOpts;
use crate::procfile::{Procfile};
use crate::process::port_for;
use crate::env::read_env;
use std::path::PathBuf;
use serde_derive::Serialize;
use std::env;
use std::collections::HashMap;
use serde_json::value::{Map, Value as Json};
use handlebars::to_json;
use shellwords::escape;
use regex::Regex;

const ENV_REGEXP: &'static str = "\\$\\{*(?P<envname>[A-Za-z0-9_-]+)\\}*";

pub struct Exporter {
    pub procfile: Procfile,
    // ExportOpts
    pub format: String,
    pub location: PathBuf,
    pub app: Option<String>,
    pub formation: String,
    pub log_path: Option<PathBuf>,
    pub run_path: Option<PathBuf>,
    pub port: Option<String>,
    pub template_path: Option<PathBuf>,
    pub user: Option<String>,
    pub env_path: PathBuf,
    pub procfile_path: PathBuf,
    pub root_path: Option<PathBuf>,
    pub timeout: String,
}

#[derive(Serialize)]
struct AppConfDataParams {
  user: String,
  work_dir: String,
  program: String,
  process_command: String,
  environment: String,
  stdout_logfile: String,
  stderr_logfile: String,
}

#[derive(Serialize)]
struct AppConfParams {
  app: String,
  service_names: String,
  data: Vec<AppConfDataParams>,
}

impl Default for Exporter {
  fn default() -> Self {
        Exporter {
        procfile: Procfile {
            data: HashMap::new(),
        },
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

impl Exporter {
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
    data: Vec<AppConfDataParams>
  ) -> Map<String, Json> {
    let mut tmpldata = Map::new();
    let ac = AppConfParams {
      app: self.app(),
      service_names: service_names.join(","),
      data,
    };
    tmpldata.insert("app_conf".to_string(), to_json(&ac));
    tmpldata
  }

  fn environment(&self, index: usize, con_index: usize) -> String {
    let port = port_for(self.opts().env_path, self.opts().port, index, con_index + 1);
    let mut env = read_env(self.opts().env_path).expect("failed read .env");
    env.insert("PORT".to_string(), port);
    
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

impl Exportable for Exporter {
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
        data.push(AppConfDataParams{
          user: self.username(),
          work_dir: self.root_path().into_os_string().into_string().unwrap(),
          program,
          process_command,
          environment,
          stdout_logfile,
          stderr_logfile
        });
      }
    }

    let output_path = self.output_path("app.conf".to_string());
    let display_output = output_path.clone().into_os_string().into_string().unwrap();
    let mut data = self.make_app_conf_data(service_names, data);
    self.clean(&output_path).expect(&format!("failed clean file: {}", display_output));
    self.write_template(&self.app_conf_tmpl_path(), &mut data, &output_path);

    Ok(())
  }

  fn opts(&self) -> ExportOpts {
    ExportOpts {
      format: self.format.clone(),
      location: self.location.clone(),
      app: self.app.clone(),
      formation: self.formation.clone(),
      log_path: self.log_path.clone(),
      run_path: self.run_path.clone(),
      port: self.port.clone(),
      template_path: self.template_path.clone(),
      user: self.user.clone(),
      env_path: self.env_path.clone(),
      procfile_path: self.procfile_path.clone(),
      root_path: self.root_path.clone(),
      timeout: self.timeout.clone(),
    }
  }
}
