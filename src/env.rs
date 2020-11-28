use dotenv;
use std::collections::HashMap;
use std::path::{PathBuf};

pub type Env = HashMap<String, String>;

pub fn read_env(filepath: PathBuf) -> Result<Env, Box<dyn std::error::Error>> {
    let mut env: Env = HashMap::new();

    if let Some(iter) = dotenv::from_path_iter(filepath.as_path()).ok() {
        for item in iter {
            let (key, val) = item.expect("Could not convert .env to tuple");
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
        assert_eq!(result.get("CARGO_PKG_VERSION"), None);

        Ok(())
    }
}
