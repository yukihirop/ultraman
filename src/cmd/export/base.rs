use crate::cmd::export::ExportOpts;
use crate::env::read_env;

use handlebars::Handlebars;
use serde_derive::Serialize;
use serde_json::value::{Map, Value as Json};
use std::env;
use std::fs::File;
use std::fs::{create_dir_all, remove_file};
use std::io::Read;
use std::path::PathBuf;

// Lifetime cannot be set because it will be HashMap data with anonymous runtime
#[derive(Serialize)]
pub struct EnvParameter {
    pub(crate) key: String,
    pub(crate) value: String,
}

pub struct Template {
    pub(crate) template_path: PathBuf,
    pub(crate) data: Map<String, Json>,
    pub(crate) output_path: PathBuf,
}

pub trait Exportable {
    fn export(&self) -> Result<(), Box<dyn std::error::Error>>;
    //https://yajamon.hatenablog.com/entry/2018/01/30/202849
    fn ref_opts(&self) -> &ExportOpts;

    fn base_export(&self) -> Result<(), Box<dyn std::error::Error>> {
        let opts = self.ref_opts();
        let location = &opts.location;
        let display = location.clone().into_os_string().into_string().unwrap();
        create_dir_all(&location).expect(&format!("Could not create: {}", display));

        // self.chown(&username, &self.log_path());
        // self.chown(&username, &self.run_path());
        Ok(())
    }

    fn app(&self) -> &str {
        self.ref_opts().app.as_deref().unwrap_or_else(|| "app")
    }

    fn log_path(&self) -> PathBuf {
        self.ref_opts()
            .log_path
            .clone()
            .unwrap_or_else(|| PathBuf::from(format!("/var/log/{}", self.app())))
    }

    fn run_path(&self) -> PathBuf {
        self.ref_opts()
            .run_path
            .clone()
            .unwrap_or_else(|| PathBuf::from(format!("/var/run/{}", self.app())))
    }

    fn username(&self) -> &str {
        self.ref_opts()
            .user
            .as_deref()
            .unwrap_or_else(|| self.app())
    }

    fn root_path(&self) -> PathBuf {
        self.ref_opts()
            .root_path
            .clone()
            .unwrap_or_else(|| env::current_dir().unwrap())
    }

    fn clean(&self, filepath: &PathBuf) {
        let display = filepath.clone().into_os_string().into_string().unwrap();
        if filepath.exists() {
            self.say(&format!("cleaning: {}", display));
            remove_file(filepath).expect(&format!("Could not remove file: {}", display));
        }
    }

    fn project_root_path(&self) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn say(&self, msg: &str) {
        println!("[ultraman export] {}", msg)
    }

    fn write_template(&self, tmpl: Template) {
        let handlebars = Handlebars::new();
        let display_template = tmpl
            .template_path
            .clone()
            .into_os_string()
            .into_string()
            .unwrap();
        let display_output = tmpl
            .output_path
            .clone()
            .into_os_string()
            .into_string()
            .unwrap();
        let mut output_file = File::create(tmpl.output_path)
            .expect(&format!("Could not create file: {}", &display_output));
        self.say(&format!("writing: {}", &display_output));
        let mut data = tmpl.data;
        let mut template_source = File::open(tmpl.template_path)
            .expect(&format!("Could not open file: {}", display_template));
        let mut template_str = String::new();
        template_source
            .read_to_string(&mut template_str)
            .expect(&format!("Could not read file: {}", display_template));
        handlebars
            .render_template_to_write(&mut template_str, &mut data, &mut output_file)
            .expect(&format!("Coult not render file: {}", &display_output));
    }

    fn output_path(&self, filename: &str) -> PathBuf {
        let location = self.ref_opts().location.clone();
        location.join(filename)
    }

    fn env_without_port(&self) -> Vec<EnvParameter> {
        let mut env =
            read_env(self.ref_opts().env_path.clone().unwrap()).expect("failed read .env");
        env.remove("PORT");
        let mut env_without_port: Vec<EnvParameter> = vec![];
        for (key, value) in env {
            env_without_port.push(EnvParameter { key, value });
        }
        env_without_port
    }

    fn create_dir_recursive(&self, dir_path: &PathBuf) {
        let display = dir_path.clone().into_os_string().into_string().unwrap();
        create_dir_all(dir_path).expect(&format!("Could not create: {}", display))
    }
}
