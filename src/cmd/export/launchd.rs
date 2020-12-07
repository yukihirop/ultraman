use crate::procfile::{Procfile, ProcfileEntry};
use super::base::{Exportable, EnvParameter};
use crate::cmd::export::ExportOpts;
use crate::env::read_env;
use crate::process::port_for;
use serde_derive::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use handlebars::to_json;
use serde_json::value::{Map, Value as Json};
use std::env;

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
struct LaunchdParams {
  label: String,
  env: Vec<EnvParameter>,
  command_args: Vec<String>,
  stdout_path: String,
  stderr_path: String,
  user: String,
  work_dir: String,
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
    index: usize,
    con_index: usize
  ) -> Map<String, Json> {
    let mut data = Map::new();
    let log_display = self.log_path().into_os_string().into_string().unwrap();
    let lp = LaunchdParams {
      label: service_name.to_string(),
      env: self.environment(index, con_index),
      command_args: self.command_args(pe),
      stdout_path: format!("{}/{}.log", &log_display, &service_name),
      stderr_path: format!("{}/{}.error.log", &log_display, &service_name),
      user: self.username(),
      work_dir: self.root_path().into_os_string().into_string().unwrap(),
    };
    data.insert("launchd".to_string(), to_json(&lp));
    data
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
    let port = port_for(self.opts().env_path, self.opts().port, index, con_index + 1);
    let mut env = read_env(self.opts().env_path).expect("failed read .env");
    env.insert("PORT".to_string(), port);
    
    let mut result = vec![];
    for (key, val) in env.iter() {
      result.push(EnvParameter{
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

      let mut index = 0;
      for (name, pe) in self.procfile.data.iter() {
        let con = pe.concurrency.get();
        for n in 0..con {
          index += 1;
          let service_name = format!("{}-{}-{}", self.app(), &name, n + 1);
          let output_path = self.opts().location.join(&service_name);
          let mut data = self.make_launchd_data(pe, &service_name, index, n);
          self.clean(&output_path);
          self.write_template(&self.launchd_tmpl_path(), &mut data, &output_path);
        }
      }

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
