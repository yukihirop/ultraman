use super::base::{Exportable, EnvParameter};
use crate::procfile::{Procfile, ProcfileEntry};
use crate::process::port_for;
use crate::cmd::export::ExportOpts;
use std::path::PathBuf;
use serde_derive::Serialize;
use serde_json::value::{Map, Value as Json};
use std::collections::HashMap;
use std::env;
use handlebars::to_json;

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
struct MasterTargetParams {
  service_names: String,
}

#[derive(Serialize)]
struct ProcessServiceParams {
  app: String,
  user: String,
  work_dir: String,
  port: String,
  process_name: String,
  process_command: String,
  env_without_port: Vec<EnvParameter>,
  timeout: String,
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

  fn master_target_tmpl_path(&self) -> PathBuf {
    let mut path = self.project_root_path();
    let tmpl_path = PathBuf::from("src/cmd/export/templates/systemd/master.target.hbs");
    path.push(tmpl_path);
    path
  }

  fn process_service_tmpl_path(&self) -> PathBuf {
    let mut path = self.project_root_path();
    let tmpl_path = PathBuf::from("src/cmd/export/templates/systemd/process.service.hbs");
    path.push(tmpl_path);
    path
  }

  fn make_master_target_data(&self, service_names: Vec<String>) -> Map<String, Json> {
    let mut data = Map::new();
    let mt = MasterTargetParams {
      service_names: service_names.join(" "),
    };
    data.insert("master_target".to_string(), to_json(&mt));
    data
  }

  fn make_process_service_data(
    &self,
    pe: &ProcfileEntry,
    process_name: &str,
    index: usize,
    con_index: usize,
  ) -> Map<String, Json> {
    let mut data = Map::new();
    let ps = ProcessServiceParams {
      app: self.app(),
      user: self.username(),
      work_dir: self.root_path().into_os_string().into_string().unwrap(),
      port: port_for(self.opts().env_path, self.opts().port, index, con_index + 1),
      process_name: process_name.to_string(),
      process_command: pe.command.to_string(),
      env_without_port: self.env_without_port(),
      timeout: self.opts().timeout,
    };
    data.insert("process_service".to_string(), to_json(&ps));
    data
  }
}

impl Exportable for Exporter {
  fn export(&self) -> Result<(), Box<dyn std::error::Error>> {
    self.base_export().expect("failed execute base_export");

    let mut index = 0;
    let mut service_names = vec![];
    for(name, pe) in self.procfile.data.iter(){
      index += 1;
      let con = pe.concurrency.get();
      for n in 0..con {
        let process_name = format!("{}.{}", &name, n);
        let service_filename = format!("{}-{}.service", &name, &process_name);
        let output_path = self.output_path(service_filename.clone());
        let display_output = output_path.clone().into_os_string().into_string().unwrap();
        let mut data = self.make_process_service_data(pe, &process_name, index, n);
        
        self.clean(&output_path).expect(&format!("failed clean file: {}", display_output));
        self.write_template(&self.process_service_tmpl_path(), &mut data, &output_path);
        service_names.push(service_filename);
      }
    }

    let output_path = self.output_path(format!("{}.target", self.app()));
    let display_output = output_path.clone().into_os_string().into_string().unwrap();
    let mut data = self.make_master_target_data(service_names);

    self.clean(&output_path).expect(&format!("failed clean file: {}", display_output));
    self.write_template(&self.master_target_tmpl_path(), &mut data, &output_path);

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
