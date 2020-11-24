use dotenv;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

pub type Env = HashMap<String, String>;

pub fn read_env(filepath: PathBuf) -> Result<Env, Box<dyn std::error::Error>> {
    let mut env: Env = HashMap::new();
    if let Some(_) = dotenv::from_path(Path::new(&filepath)).ok() {
        for (key, val) in env::vars() {
            env.insert(key, val);
        }
        return Ok(env);
    }
    Ok(env)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_read_env() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join(".env");
        let mut file = File::create(file_path.clone())?;
        writeln!(
            file,
            r#"
PORT=5000
PS=1
      "#
        )
        .unwrap();

        let result = read_env(file_path).expect("failed read .env");

        assert_eq!(result.get("PORT").unwrap(), "5000");
        assert_eq!(result.get("PS").unwrap(), "1");

        Ok(())
    }
}
