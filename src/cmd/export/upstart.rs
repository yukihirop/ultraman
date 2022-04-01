use super::base::{EnvParameter, Exportable, Template};
use crate::cmd::export::ExportOpts;
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
struct ProcessMasterParams<'a> {
    app: &'a str,
}

#[derive(Serialize)]
struct ProcessParams<'a> {
    app: &'a str,
    name: &'a str,
    port: &'a str,
    env_without_port: Vec<EnvParameter>,
    setuid: &'a str,
    chdir: &'a str,
    exec: &'a str,
}

// http://takoyaking.hatenablog.com/entry/anonymous_lifetime
impl<'a> Exporter<'a> {
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
            name: app_name,
            port: &port_for(
                &self.opts.env_path.clone().unwrap(),
                self.opts.port.clone(),
                index,
                con_index + 1,
            ),
            env_without_port: self.env_without_port(),
            setuid: self.username(),
            chdir: &self.root_path().into_os_string().into_string().unwrap(),
            exec: &pe.command,
        };
        data.insert("process".to_string(), to_json(&p));
        data
    }
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

impl<'a> Exportable for Exporter<'a> {
    fn export(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.base_export().expect("failed execute base_export");

        let mut clean_paths: Vec<PathBuf> = vec![];
        let mut tmpl_data: Vec<Template> = vec![];

        let master_file = format!("{}.conf", self.app());
        let output_path = self.output_path(&master_file);

        clean_paths.push(output_path.clone());
        tmpl_data.push(Template {
            template_path: self.master_tmpl_path(),
            data: Map::new(),
            output_path,
        });

        let mut index = 0;
        for (name, pe) in self.procfile.data.iter() {
            index += 1;
            let con = pe.concurrency.get();
            let process_master_file = format!("{}-{}.conf", self.app(), &name);
            let output_path = self.output_path(&process_master_file);

            clean_paths.push(output_path.clone());
            tmpl_data.push(Template {
                template_path: self.process_master_tmpl_path(),
                data: self.make_process_master_data(),
                output_path,
            });

            for n in 0..con {
                let process_file = format!("{}-{}-{}.conf", self.app(), &name, n);
                let output_path = self.output_path(&process_file);

                clean_paths.push(output_path.clone());
                tmpl_data.push(Template {
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
