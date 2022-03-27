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
            port: port_for(
                &self.opts.env_path.clone().unwrap(),
                self.opts.port.clone(),
                index,
                con_index + 1,
            ),
            process_name: process_name.to_string(),
            process_command: pe.command.to_string(),
            env_without_port: self.env_without_port(),
            timeout: self.opts.timeout.clone().unwrap(),
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
        let mut clean_paths: Vec<PathBuf> = vec![];
        let mut tmpl_data: Vec<Template> = vec![];

        for (name, pe) in self.procfile.data.iter() {
            index += 1;
            let con = pe.concurrency.get();
            for n in 0..con {
                let process_name = format!("{}.{}", &name, n);
                let service_filename = format!("{}-{}.service", &name, &process_name);
                let output_path = self.output_path(service_filename.clone());
                let data = self.make_process_service_data(pe, &process_name, index, n);

                clean_paths.push(output_path.clone());
                tmpl_data.push(Template {
                    template_path: self.process_service_tmpl_path(),
                    data,
                    output_path,
                });
                service_names.push(service_filename);
            }
        }

        let output_path = self.output_path(format!("{}.target", self.app()));
        let data = self.make_master_target_data(service_names);

        clean_paths.push(output_path.clone());
        tmpl_data.push(Template {
            template_path: self.master_target_tmpl_path(),
            data,
            output_path,
        });

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
