use super::base::{EnvParameter, Exportable, Template};
use crate::cmd::export::ExportOpts;
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
struct ProcessMasterParams {
    app: String,
}

#[derive(Serialize)]
struct ProcessParams {
    app: String,
    name: String,
    port: String,
    env_without_port: Vec<EnvParameter>,
    setuid: String,
    chdir: String,
    exec: String,
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

        let p = ProcessParams {
            app: self.app(),
            name: app_name.to_string(),
            port: port_for(self.opts.env_path.clone(), self.opts.port.clone(), index, con_index + 1),
            env_without_port: self.env_without_port(),
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

impl Exportable for Exporter {
    fn export(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.base_export().expect("failed execute base_export");

        let mut clean_paths: Vec<PathBuf> = vec![];
        let mut tmpl_data: Vec<Template> = vec![];

        let master_file = format!("{}.conf", self.app());
        let output_path = self.output_path(master_file);

        clean_paths.push(output_path.clone());
        tmpl_data.push(Template{
            template_path: self.master_tmpl_path(),
            data: Map::new(),
            output_path,
        });

        let mut index = 0;
        for (name, pe) in self.procfile.data.iter() {
            index += 1;
            let con = pe.concurrency.get();
            let process_master_file = format!("{}-{}.conf", self.app(), &name);
            let output_path = self.output_path(process_master_file);

            clean_paths.push(output_path.clone());
            tmpl_data.push(Template{
                template_path: self.process_master_tmpl_path(),
                data: self.make_process_master_data(),
                output_path,
            });

            for n in 0..con {
                let process_file = format!("{}-{}-{}.conf", self.app(), &name, n);
                let output_path = self.output_path(process_file);

                clean_paths.push(output_path.clone());
                tmpl_data.push(Template{
                    template_path: self.process_tmpl_path(),
                    data: self.make_process_data(pe, &name, index, n),
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
