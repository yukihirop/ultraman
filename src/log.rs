pub fn output(proc_name: &str, content: &str) {
    let pad_str = format!("{: <width$}", proc_name, width = 10);
    println!("{} | {}", pad_str, content);
}

pub fn error(proc_name: &str, err: &dyn std::error::Error) {
    let content = &format!("error: {:?}", err);
    output(proc_name, content);
}
