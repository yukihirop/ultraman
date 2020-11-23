use regex::Regex;
use std::fs::File;
use std::path::PathBuf;
use std::io::{BufRead, BufReader};

const PROCFILE_REGEXP: &'static str = r"\A([A-Za-z0-9_-]+):\s*(.+)$";

pub struct ProcfileEntry {
  pub name: String,
  pub command: String,
  pub concurrency: usize,
}

pub struct Procfile {
  pub entries: Vec<ProcfileEntry>
}

impl Procfile {
  pub fn padding(&self) -> usize {
    // e.g) <name>.<concurrency> |
    self.entries.iter().map(|pe| pe.name.len()).max().expect("failed calculate padding") + 3
  }

  pub fn process_len(&self) -> usize {
    self.entries.iter().map(|pe| pe.concurrency).fold(0, |sum, a| sum + a)
  }
}

impl Default for Procfile {
  fn default() -> Self {
    Procfile { entries: vec![] }
  }
}

pub fn read_procfile(filepath: PathBuf) -> Result<Procfile, Box<dyn std::error::Error>> {
  let display = filepath.clone().into_os_string().into_string().unwrap();

  let file = match File::open(filepath) {
    Ok(f) => f,
    Err(why) => panic!("cloud't open {}: {}", display, why),
  };

  parse_procfile(&file)
}

fn parse_procfile(file: &File) -> Result<Procfile, Box<dyn std::error::Error>> {
  let procfile_re = Regex::new(PROCFILE_REGEXP).unwrap();
  let mut pf = Procfile::default();
  let buf_reader = BufReader::new(file);

  for line in buf_reader.lines(){
    for cap in procfile_re.captures_iter(&line.unwrap()) {
      pf.entries.push(ProcfileEntry {
        name: (&cap[1]).to_string(),
        command: (&cap[2]).to_string(),
        concurrency: 1
      })
    }
  }

  Ok(pf)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Write;
  use tempfile::tempdir;

  fn craete_procfile() -> Procfile {
    Procfile {
      entries: vec![
        ProcfileEntry {
          name: String::from("app"),
          command: String::from("./app.sh"),
          concurrency: 1
        },
        ProcfileEntry {
          name: String::from("web"),
          command: String::from("./app.sh"),
          concurrency: 1
        }
      ]
    }
  }

  #[test]
  fn test_padding() -> anyhow::Result<()> {
    let pf = craete_procfile();
    let result = pf.padding();
    assert_eq!(result, 6);

    Ok(())
  }

  #[test]
  fn test_process_len() -> anyhow::Result<()> {
    let pf = craete_procfile();
    let result = pf.process_len();
    assert_eq!(result, 3);

    Ok(())
  }

  #[test] 
  fn test_parse_procfile() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let procfile_path = dir.path().join("Procfile");
    let mut file = File::create(procfile_path.clone())?;
    writeln!(
      file,
      r#"
app: ./app.sh
web: ./web.sh
      "#
    )
    .expect("failed write temp Procfile");

    let read_file= File::open(procfile_path)?;
    let result = parse_procfile(&read_file).expect("failed parse_procfile");

    assert_eq!(result.entries[0].name, "app");
    assert_eq!(result.entries[0].command, "./app.sh");
    assert_eq!(result.entries[0].concurrency, 1);
    assert_eq!(result.entries[1].name, "web");
    assert_eq!(result.entries[1].command, "./web.sh");
    assert_eq!(result.entries[1].concurrency, 1);

    Ok(())
  }
}
