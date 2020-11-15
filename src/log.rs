pub fn output(proc_name: &str, content: &str) {
  let pad_str = format!("{: <width$}", proc_name, width = 10);
  println!("{} | {}", pad_str, content);
}
