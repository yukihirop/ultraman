use crate::cmd::export::base::Exportable;
use crate::cmd::export::ExportOpts;
use crate::process::port_for;
use crate::procfile::{Procfile, ProcfileEntry};
use crate::env::read_env;

use handlebars::to_json;
use serde_derive::Serialize;
use serde_json::value::{Map, Value as Json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
struct ProcessMasterParams {
    app: String,
}

#[derive(Serialize)]
struct ProcessParams {
    app: String,
    name: String,
    port: String,
    env_without_port: Vec<ProcessEnvParams>,
    setuid: String,
    chdir: String,
    exec: String,
}

#[derive(Serialize)]
struct ProcessEnvParams {
    key: String,
    value: String
}

// http://takoyaking.hatenablog.com/entry/anonymous_lifetime
impl Exporter {
    fn boxed(self) -> Box<Self> {
        Box::new(self)
    }

    pub fn boxed_new() -> Box<Self> {
        Self::default().boxed()
    }

    fn master_tmpl_path(&self) -> PathBuf {
        let mut path = self.project_root_path();
        let tmpl_path = PathBuf::from("src/cmd/export/templates/upstart/master.conf.hbs");
        path.push(tmpl_path);
        path
    }

    fn process_master_tmpl_path(&self) -> PathBuf {
        let mut path = self.project_root_path();
        let tmpl_path = PathBuf::from("src/cmd/export/templates/upstart/process_master.conf.hbs");
        path.push(tmpl_path);
        path
    }

    fn process_tmpl_path(&self) -> PathBuf {
        let mut path = self.project_root_path();
        let tmpl_path = PathBuf::from("src/cmd/export/templates/upstart/process.conf.hbs");
        path.push(tmpl_path);
        path
    }

    fn project_root_path(&self) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
        app_name: &str,
        index: usize,
        con_index: usize,
    ) -> Map<String, Json> {
        let mut data = Map::new();
        let mut env = read_env(self.opts().env_path).expect("failed read .env");
        env.remove("PORT");
        let mut env_without_port: Vec<ProcessEnvParams> = vec![];
        for (key, value) in env {
            env_without_port.push(ProcessEnvParams{
                key,
                value
            });
        }

        let p = ProcessParams {
            app: self.app(),
            name: app_name.to_string(),
            port: port_for(self.opts().env_path, self.opts().port, index, con_index),
            env_without_port,
            setuid: self.username(),
            chdir: self.root_path().into_os_string().into_string().unwrap(),
            exec: pe.command.to_string(),
        };
        data.insert("process".to_string(), to_json(&p));
        data
    }
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

impl Exportable for Exporter {
    fn export(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.base_export().expect("failed execute base_export");

        let master_file = format!("{}.conf", self.app());
        let output_path = self.output_path(master_file);
        let display_output = output_path.clone().into_os_string().into_string().unwrap();

        self.clean(&output_path)
            .expect(&format!("failed clean file: {}", display_output));
        let mut data = Map::new();
        self.write_template(&self.master_tmpl_path(), &mut data, &output_path);

        let mut index = 0;
        for (name, pe) in self.procfile.data.iter() {
            index += 1;
            let con = pe.concurrency.get();
            let process_master_file = format!("{}-{}.conf", self.app(), &name);
            let output_path = self.output_path(process_master_file);
            let mut data = self.make_process_master_data();
            self.write_template(&self.process_master_tmpl_path(), &mut data, &output_path);

            for n in 0..con {
                let process_file = format!("{}-{}-{}.conf", self.app(), &name, n);
                let output_path = self.output_path(process_file);
                let mut data = self.make_process_data(pe, &name, index, n);
                self.write_template(&self.process_tmpl_path(), &mut data, &output_path);
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
