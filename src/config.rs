extern crate yaml_rust;
use std::fs;
use std::path::PathBuf;
use yaml_rust::{Yaml, YamlLoader};

pub const DEFAULT_FORMATION: &'static str = "all=1";
pub const DEFAULT_ENV: &'static str = ".env";
pub const DEFAULT_PROCFILE: &'static str = "Procfile";
pub const DEFAULT_TIMEOUT: &'static str = "5";

// Ultraman settings read and parse .ultraman written in yaml
pub fn read_config(filepath: PathBuf) -> Result<Yaml, Box<dyn std::error::Error>> {
    let filepath_str = filepath.to_str().unwrap();
    let config_str = fs::read_to_string(&filepath)
        .expect(&format!("Failed to read the file: {}", &filepath_str));
    let docs = YamlLoader::load_from_str(&config_str)
        .expect(&format!("Failed to parse yaml file: {}", &filepath_str));
    let doc = &docs[0];
    Ok(doc.clone())
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
procfile: ./Procfile
env: .env

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

        assert_eq!(result["procfile"].as_str().unwrap(), "./Procfile");
        assert_eq!(result["env"].as_str().unwrap(), ".env");
        assert_eq!(result["formation"].as_str().unwrap(), "app=1,web=2");
        assert_eq!(result["port"].as_i64().unwrap(), 6000);
        assert_eq!(result["timeout"].as_i64().unwrap(), 5000);
        assert_eq!(result["no-timestamp"].as_bool().unwrap(), true);
        assert_eq!(result["app"].as_str().unwrap(), "app-for-runit");
        assert_eq!(result["log"].as_str().unwrap(), "/var/app/log/ultraman.log");
        assert_eq!(result["run"].as_str().unwrap(), "/tmp/pids/ultraman.pid");
        assert_eq!(
            result["template"].as_str().unwrap(),
            "../../src/cmd/export/templates/supervisord"
        );
        assert_eq!(result["user"].as_str().unwrap(), "root");
        assert_eq!(result["root"].as_str().unwrap(), "/home/app");
        assert_eq!(result["hoge"].as_str().unwrap(), "hogehoge");

        Ok(())
    }
}
