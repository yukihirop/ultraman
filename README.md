# Rustman (Rust Foreman)

Manage Procfile-based applications.  

This is a [foreman](https://github.com/ddollar/foreman) rust implementation made with ruby.  
So the specifications are exactly the same as ruby ​​`foreman`.

## 🚉 Platform

- Linux
- macOS
- windows?

## 🦀 Installation

Download binary

Download from [release page](), and extract to the directory in PATH.

## 💻 Command

```
$ rustman --help
rustman 0.1.0

USAGE:
    rustman [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help     Prints this message or the help of the given subcommand(s)
    start    Start the application
```

## 🚀 Tutorial

Create a `Procfile` like the one below

```
exit_0: sleep 5 && echo 'success' && exit 0;
exit_1: sleep 5 && echo 'failed' && exit 1;
loop: while :; do sleep 1 && echo 'Hello World'; done;
```

Then execute the following command

```bash
rustman start
```

![image](https://user-images.githubusercontent.com/11146767/100380658-9894a380-305a-11eb-9509-30495a39a346.png)

<details>

```bash
$ rustman start
02:44:43 system    | exit_0.1  start at pid: 59658
02:44:43 system    | loop.1    start at pid: 59659
02:44:43 system    | exit_1.1  start at pid: 59660
02:44:44 loop.1    | Hello World
02:44:45 loop.1    | Hello World
02:44:46 loop.1    | Hello World
02:44:47 loop.1    | Hello World
02:44:48 exit_1.1  | failed
02:44:48 exit_0.1  | success
02:44:48 exit_1.1  | exited with code 1
02:44:48 system    | sending SIGTERM for exit_0.1  at pid 59658
02:44:48 system    | sending SIGTERM for loop.1    at pid 59659
02:44:48 exit_0.1  | exited with code 0
02:44:48 system    | sending SIGTERM for loop.1    at pid 59659
02:44:48 loop.1    | terminated by SIGTERM
```

</details>

If <kbd>ctrl-c</kbd> is detected within 5 seconds, `SIGTERM` will be sent to all child processes and the process will be killed.

![image](https://user-images.githubusercontent.com/11146767/100380752-c5e15180-305a-11eb-93ce-125c0002c162.png)

<details>

```
$ ./rustman start
02:46:13 system    | exit_0.1  start at pid: 59892
02:46:13 system    | loop.1    start at pid: 59893
02:46:13 system    | exit_1.1  start at pid: 59891
02:46:14 loop.1    | Hello World
02:46:15 loop.1    | Hello World
02:46:16 loop.1    | Hello World
^C02:46:17 system  | SIGINT received, starting shutdown
02:46:17 system    | sending SIGTERM to all processes
02:46:17 system    | sending SIGTERM for exit_0.1  at pid 59892
02:46:17 system    | sending SIGTERM for loop.1    at pid 59893
02:46:17 system    | sending SIGTERM for exit_1.1  at pid 59891
02:46:17 exit_0.1  | terminated by SIGTERM
02:46:17 loop.1    | terminated by SIGTERM
02:46:17 exit_1.1  | terminated by SIGTERM
```

</details>

## 💪 Development

```bash
cargo run -- --help
# or
cargo run start
# or
cargo run start --help
```

## ✍️ Test

```bash
cargo test
# or
cargo test -- --nocapture
```


## ⚙ Environment

|name|desc|defaul|
|----|----|------|
|COLOR|Color the output|true|

## 📚 Reference

I really referred to the implementation of the following repository.

- [yukihirop/eg_foreman](https://github.com/yukihirop/eg_foreman)
- [jtdowney/fors](https://github.com/jtdowney/fors)
- [jaredgorski/arpx](https://github.com/jaredgorski/arpx)
