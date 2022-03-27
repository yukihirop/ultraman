extern crate yaml_rust;
use std::fs;
use std::path::PathBuf;
use yaml_rust::YamlLoader;

pub const DEFAULT_FORMATION: &'static str = "all=1";
const DEFAULT_ENV: &'static str = ".env";
const DEFAULT_PROCFILE: &'static str = "Procfile";
const DEFAULT_TIMEOUT: i64 = 5;
const DEFAULT_NO_TIMESTAMP: bool = false;

#[derive(Debug)]
pub struct Config {
    pub procfile_path: Option<PathBuf>,
    pub env_path: Option<PathBuf>,
    pub formation: Option<String>,
    pub port: Option<i64>,
    pub timeout: Option<i64>,
    pub is_no_timestamp: Option<bool>,
    pub app: Option<String>,
    pub log_path: Option<PathBuf>,
    pub run_path: Option<PathBuf>,
    pub template_path: Option<PathBuf>,
    pub user: Option<String>,
    pub root_path: Option<PathBuf>,
}

// Ultraman settings read and parse .ultraman written in yaml
pub fn read_config(filepath: PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    let filepath_str = filepath.to_str().unwrap();
    let config_str = fs::read_to_string(&filepath)
        .expect(&format!("Failed to read the file: {}", &filepath_str));
    let docs = YamlLoader::load_from_str(&config_str)
        .expect(&format!("Failed to parse yaml file: {}", &filepath_str));
    let doc = &docs[0];

    let config = Config {
        procfile_path: match doc["procfile"].as_str() {
            Some(r) => Some(PathBuf::from(r)),
            None => Some(PathBuf::from(DEFAULT_PROCFILE)),
        },
        env_path: match doc["env"].as_str() {
            Some(r) => Some(PathBuf::from(r)),
            None => Some(PathBuf::from(DEFAULT_ENV)),
        },
        formation: match doc["formation"].as_str() {
            Some(r) => Some(r.to_string()),
            None => Some(DEFAULT_FORMATION.to_string()),
        },
        port: doc["port"].as_i64(),
        timeout: Some(doc["timeout"].as_i64().unwrap_or(DEFAULT_TIMEOUT)),
        is_no_timestamp: Some(
            doc["no-timestamp"]
                .as_bool()
                .unwrap_or(DEFAULT_NO_TIMESTAMP),
        ),
        app: match doc["app"].as_str() {
            Some(r) => Some(r.to_string()),
            None => None,
        },
        log_path: match doc["log"].as_str() {
            Some(r) => Some(PathBuf::from(r)),
            None => None,
        },
        run_path: match doc["run"].as_str() {
            Some(r) => Some(PathBuf::from(r)),
            None => None,
        },
        template_path: match doc["template"].as_str() {
            Some(r) => Some(PathBuf::from(r)),
            None => None,
        },
        user: match doc["user"].as_str() {
            Some(r) => Some(r.to_string()),
            None => None,
        },
        root_path: match doc["root"].as_str() {
            Some(r) => Some(PathBuf::from(r)),
            None => None,
        },
    };
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_read_config() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join(".ultraman");
        let mut file = File::create(file_path.clone())?;
        // Writing a comment causes a parse error
        writeln!(
            file,
            r#"
procfile: ./tmp/Procfile
env: ./tmp/.env

formation: app=1,web=2
port: 6000
timeout: 5000

no-timestamp: true

app: app-for-runit
log: /var/app/log/ultraman.log
run: /tmp/pids/ultraman.pid
template: ../../src/cmd/export/templates/supervisord
user: root
root: /home/app

hoge: hogehoge
      "#
        )
        .unwrap();

        let result = read_config(file_path).expect("failed read .ultraman");

        assert_eq!(
            result.procfile_path.unwrap(),
            PathBuf::from("./tmp/Procfile")
        );
        assert_eq!(result.env_path.unwrap(), PathBuf::from("./tmp/.env"));
        assert_eq!(result.formation.unwrap(), "app=1,web=2");
        assert_eq!(result.port.unwrap(), 6000);
        assert_eq!(result.timeout.unwrap(), 5000);
        assert_eq!(result.is_no_timestamp.unwrap(), true);
        assert_eq!(result.app.unwrap(), "app-for-runit");
        assert_eq!(
            result.log_path.unwrap(),
            PathBuf::from("/var/app/log/ultraman.log")
        );
        assert_eq!(
            result.run_path.unwrap(),
            PathBuf::from("/tmp/pids/ultraman.pid")
        );
        assert_eq!(
            result.template_path.unwrap(),
            PathBuf::from("../../src/cmd/export/templates/supervisord")
        );
        assert_eq!(result.user.unwrap(), "root");
        assert_eq!(result.root_path.unwrap(), PathBuf::from("/home/app"));

        Ok(())
    }
}
