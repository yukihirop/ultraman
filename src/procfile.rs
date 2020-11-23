use std::collections::HashMap;

pub struct Script {
    pub cmd: String,
    pub concurrency: usize,
}

// https://www.366service.com/jp/qa/3b42bd30380c728939a2e80b42d430a6
macro_rules! hashmap {
  ($( $key: expr => $val: expr), *) =>{{
    let mut map = ::std::collections::HashMap::new();
    $( map.insert($key, $val); )*
    map
  }}
}

pub fn scripts() -> HashMap<&'static str, Script> {
    let scripts: HashMap<&str, Script> = hashmap! [
      "loop" => Script {
        cmd: String::from("./bin/loop.sh"),
        concurrency: 2,
      },
      "exit_1" => Script {
        cmd: String::from("./bin/exit_1.sh"),
        concurrency: 1,
      },
      "exit_0" => Script {
        cmd: String::from("./bin/exit_0.sh"),
        concurrency: 1,
      }
    ];

    scripts
}

pub fn padding() -> usize {
    // e.g) <name>.<concurrency> |
    scripts().keys().map(|name| name.len()).max().unwrap() + 3
}

pub fn process_len() -> usize {
    scripts()
        .values()
        .map(|s| s.concurrency)
        .fold(0, |sum, a| sum + a)
}

#[cfg(test)]
mod tests {
  use super::*;
  use anyhow;

  #[test]
  fn test_scripts() -> anyhow::Result<()> {
    let scripts = scripts();

    let loop_ = scripts.get("loop").unwrap(); 
    assert_eq!(loop_.cmd, String::from("./bin/loop.sh"));
    assert_eq!(loop_.concurrency, 2);

    let exit_1 = scripts.get("exit_1").unwrap();
    assert_eq!(exit_1.cmd, String::from("./bin/exit_1.sh"));
    assert_eq!(exit_1.concurrency, 1);

    let exit_0 = scripts.get("exit_0").unwrap();
    assert_eq!(exit_0.cmd, String::from("./bin/exit_0.sh"));
    assert_eq!(exit_0.concurrency, 1);
    
    Ok(())
  }

  #[test]
  fn test_padding() -> anyhow::Result<()> {
    assert_eq!(padding(), 9);
    Ok(())
  }

  #[test]
  fn test_process_len() -> anyhow::Result<()> {
    assert_eq!(process_len(), 4);

    Ok(())
  }
}
