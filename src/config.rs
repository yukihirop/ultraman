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
    pub procfile_path: PathBuf,
    pub env_path: PathBuf,
    pub formation: String,
    pub timeout: i64,
    pub is_no_timestamp: bool,
    pub port: Option<i64>,
    pub app: Option<String>,
    pub log_path: Option<PathBuf>,
    pub run_path: Option<PathBuf>,
    pub template_path: Option<PathBuf>,
    pub user: Option<String>,
    pub root_path: Option<PathBuf>,
}

// Ultraman settings read and parse .ultraman written in yaml
pub fn read_config(filepath: PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    let config: Config;

    if !filepath.exists() {
        config = Config {
            procfile_path: PathBuf::from(DEFAULT_PROCFILE),
            env_path: PathBuf::from(DEFAULT_ENV),
            formation: DEFAULT_FORMATION.to_string(),
            timeout: DEFAULT_TIMEOUT,
            is_no_timestamp: DEFAULT_NO_TIMESTAMP,
            port: None,
            app: None,
            log_path: None,
            run_path: None,
            template_path: None,
            user: None,
            root_path: None,
        }
    } else {
        let filepath_str = filepath.to_str().unwrap();
        let config_str = fs::read_to_string(&filepath)
            .expect(&format!("Failed to read the file: {}", &filepath_str));
        let docs = YamlLoader::load_from_str(&config_str)
            .expect(&format!("Failed to parse yaml file: {}", &filepath_str));
        let doc = &docs[0];

        config = Config {
            procfile_path: match doc["procfile"].as_str() {
                Some(r) => PathBuf::from(r),
                None => PathBuf::from(DEFAULT_PROCFILE),
            },
            env_path: match doc["env"].as_str() {
                Some(r) => PathBuf::from(r),
                None => PathBuf::from(DEFAULT_ENV),
            },
            formation: match doc["formation"].as_str() {
                Some(r) => r.to_string(),
                None => DEFAULT_FORMATION.to_string(),
            },
            timeout: doc["timeout"].as_i64().unwrap_or(DEFAULT_TIMEOUT),
            is_no_timestamp: doc["no-timestamp"]
                .as_bool()
                .unwrap_or(DEFAULT_NO_TIMESTAMP),
            port: doc["port"].as_i64(),
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
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn test_yaml_data(yaml_str: &str) -> anyhow::Result<Config> {
        let dir = tempdir().ok().unwrap();
        let file_path = dir.path().join(".ultraman");
        let mut file = File::create(file_path.clone()).ok().unwrap();
        writeln!(file, "{}", yaml_str).unwrap();

        let result = read_config(file_path).unwrap();
        Ok(result)
    }

    #[test]
    fn test_read_config_do_not_exist() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("do_not_exist");
        let result = read_config(file_path).unwrap();

        assert_eq!(result.procfile_path, PathBuf::from(DEFAULT_PROCFILE));
        assert_eq!(result.env_path, PathBuf::from(DEFAULT_ENV));
        assert_eq!(result.formation, DEFAULT_FORMATION);
        assert_eq!(result.timeout, 5);
        assert_eq!(result.is_no_timestamp, DEFAULT_NO_TIMESTAMP);
        assert_eq!(result.port, None);
        assert_eq!(result.app, None);
        assert_eq!(result.log_path, None);
        assert_eq!(result.run_path, None);
        assert_eq!(result.template_path, None);
        assert_eq!(result.user, None);
        assert_eq!(result.root_path, None);

        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_read_config_parse_error() {
        let yaml_str = r#"
// comment
procfile: ./tmp/Procfile
"#;

        test_yaml_data(yaml_str).unwrap();
    }

    #[test]
    fn test_read_config() -> anyhow::Result<()> {
        let yaml_str = r#"
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
"#;

        let result = test_yaml_data(yaml_str)?;

        assert_eq!(result.procfile_path, PathBuf::from("./tmp/Procfile"));
        assert_eq!(result.env_path, PathBuf::from("./tmp/.env"));
        assert_eq!(result.formation, "app=1,web=2");
        assert_eq!(result.timeout, 5000);
        assert_eq!(result.is_no_timestamp, true);
        assert_eq!(result.port.unwrap(), 6000);
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
