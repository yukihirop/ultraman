use std::process::Child;

pub struct Process {
    pub name: String,
    pub child: Child,
}
