use regex::Regex;
use std::cell::Cell;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use crate::config::DEFAULT_FORMATION;

const PROCFILE_REGEXP: &'static str = r"\A([A-Za-z0-9_-]+):\s*(.+)$";

pub struct ProcfileEntry {
    pub command: String,
    pub concurrency: Cell<usize>,
}

type ProcfileData = HashMap<String, ProcfileEntry>;

#[derive(Default)]
pub struct Procfile {
    pub data: ProcfileData,
}

impl Procfile {
    pub fn padding(&self) -> usize {
        // e.g) <name>.<concurrency> |
        self.data
            .keys()
            .map(|name| name.len())
            .max()
            .expect("failed calculate padding")
            + 3
    }

    pub fn process_len(&self) -> usize {
        self.data
            .values()
            .map(|pe| pe.concurrency.get())
            .fold(0, |sum, a| sum + a)
    }

    pub fn find_by(&self, name: &str) -> &ProcfileEntry {
        let pe = self
            .data
            .get(name)
            .expect(&format!("Can't find process called: {}", name));
        pe
    }

    pub fn set_concurrency(&self, formation: &str) {
        // e.g.) all=1
        if formation == DEFAULT_FORMATION {
            return ();
        }

        // e.g.) all=2
        let data: Vec<&str> = formation.split("=").collect();
        let name = data[0];
        if name == "all" {
            let concurrency = data[1].parse::<usize>().unwrap();
            for (_, pe) in self.data.iter() {
                pe.concurrency.set(concurrency);
            }
            return ();
        }

        let formation_data = self.parse_formation(formation);

        // https://doc.rust-lang.org/std/collections/hash_map/struct.HashMap.html#examples-14
        let formation_apps = formation_data.keys().clone().collect::<Vec<_>>();
        let valid_formation = formation_apps
            .iter()
            .all(|key| self.data.contains_key(*key));

        if valid_formation == false {
            panic!("Do not support formation: {}", formation);
        }

        for (name, pe) in self.data.iter() {
            let pe_name = name;
            let concurrency = formation_data
                .get(&pe_name.to_string())
                .unwrap_or_else(|| &0)
                .clone();
            pe.concurrency.set(concurrency);
        }
    }

    pub fn check(&self) -> bool {
        self.data.len() > 0
    }

    pub fn process_names(&self) -> String {
        let mut names = self.data.keys().map(|s| &**s).collect::<Vec<_>>();
        names.sort();
        names.join(", ")
    }

    fn parse_formation(&self, formation: &str) -> HashMap<String, usize> {
        let mut fm = formation.to_string();
        self.remove_whitespace(&mut fm);

        let pairs: Vec<&str> = fm.split(",").collect();
        let mut result = HashMap::<String, usize>::new();

        for pair in pairs {
            let data: Vec<&str> = pair.split("=").collect();
            let name = data[0];
            let concurrency = data[1];
            result.insert(String::from(name), concurrency.parse::<usize>().unwrap());
        }

        result
    }

    fn remove_whitespace(&self, s: &mut String) {
        s.retain(|c| !c.is_whitespace());
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
    let mut pf = Procfile {
        data: HashMap::<String, ProcfileEntry>::new(),
    };
    let buf_reader = BufReader::new(file);

    for line in buf_reader.lines() {
        for cap in procfile_re.captures_iter(&line.unwrap()) {
            let name = (&cap[1]).to_string();
            pf.data.insert(
                name,
                ProcfileEntry {
                    command: (&cap[2]).to_string(),
                    concurrency: Cell::new(1),
                },
            );
        }
    }

    Ok(pf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    // https://www.366service.com/jp/qa/3b42bd30380c728939a2e80b42d430a6
    macro_rules! hashmap {
    ($( $key: expr => $val: expr), *) =>{{
      let mut map = ::std::collections::HashMap::new();
      $( map.insert($key, $val); )*
      map
    }}
  }

    fn create_procfile() -> Procfile {
        Procfile {
            data: hashmap! [
              String::from("app") => ProcfileEntry {
                command: String::from("./app.sh"),
                concurrency: Cell::new(1),
              },
              String::from("web") => ProcfileEntry {
                command: String::from("./app.sh"),
                concurrency: Cell::new(1),
              }
            ],
        }
    }

    #[test]
    fn test_padding() -> anyhow::Result<()> {
        let pf = create_procfile();
        let result = pf.padding();
        assert_eq!(result, 6);

        Ok(())
    }

    #[test]
    fn test_find_by() -> anyhow::Result<()> {
        let pf = create_procfile();
        let result = pf.find_by("web");
        assert_eq!(result.command, String::from("./app.sh"));
        assert_eq!(result.concurrency.get(), 1);

        Ok(())
    }

    #[test]
    fn test_process_len() -> anyhow::Result<()> {
        let pf = create_procfile();
        let result = pf.process_len();
        assert_eq!(result, 2);

        Ok(())
    }

    #[test]
    fn test_set_concurrency() -> anyhow::Result<()> {
        let formation = "app=2, web=3";
        let pf = create_procfile();

        pf.set_concurrency(formation);
        assert_eq!(pf.data.get("app").unwrap().concurrency.get(), 2);
        assert_eq!(pf.data.get("web").unwrap().concurrency.get(), 3);

        Ok(())
    }

    #[test]
    fn test_set_concurrency_all() -> anyhow::Result<()> {
        let formation = "all=10";
        let pf = create_procfile();

        pf.set_concurrency(formation);
        assert_eq!(pf.data.get("app").unwrap().concurrency.get(), 10);
        assert_eq!(pf.data.get("web").unwrap().concurrency.get(), 10);

        Ok(())
    }

    #[test]
    #[should_panic(expected = "Do not support formation: hoge=1,fuga=2")]
    fn test_set_concurrency_when_panic() {
        let formation = "hoge=1,fuga=2";
        let pf = create_procfile();
        pf.set_concurrency(formation);
    }

    #[test]
    fn test_check_when_truethy() -> anyhow::Result<()> {
        let pf = create_procfile();
        assert_eq!(pf.check(), true);

        Ok(())
    }

    #[test]
    fn test_check_when_falsy() -> anyhow::Result<()> {
        let pf = Procfile {
            data: HashMap::new(),
        };
        assert_eq!(pf.check(), false);

        Ok(())
    }

    #[test]
    fn test_process_names() -> anyhow::Result<()> {
        let pf = create_procfile();
        assert_eq!(pf.process_names(), "app, web");

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

        let read_file = File::open(procfile_path)?;
        let result = parse_procfile(&read_file).expect("failed parse_procfile");

        assert!(result.data.contains_key("app"));
        assert!(result.data.contains_key("web"));
        assert_eq!(result.data.get("app").unwrap().command, "./app.sh");
        assert_eq!(result.data.get("app").unwrap().concurrency.get(), 1);
        assert_eq!(result.data.get("web").unwrap().command, "./web.sh");
        assert_eq!(result.data.get("web").unwrap().concurrency.get(), 1);

        Ok(())
    }
}
