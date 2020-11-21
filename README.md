# Example in rust of the core part of foreman

It is an implementation example in rust of IO processing and signal processing in multithread as done by [foreman](https://github.com/ddollar/foreman) of ruby.  

- IO process in multi thread
- handle signlas in another thread
- child wait in another thread

Since it is a sample, unlike `foreman`, it does not have the flexibility to define a process in `Procfile`.  
The processes that can be executed are as follows.  

|process|concurrency|command|
|---|-----------|-------|
|exit_0|1| `sleep 5 && echo 'success' && exit 0;`|
|exit_1|1| `sleep 5 && echo 'failed' && exit 1;`|
|loop|2|`loop: while :; do sleep 1 && echo 'hello world'; done;`|


The behavior is that when exit_0 or exit_1 exits after 5 seconds, the remaining child processes will be signaled with a `SIGTERM` and killed.

```bash
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.08s
     Running `target/debug/eg_multi_process_pipe`
exit_1.1   | start at pid: 35413
loop.1     | start at pid: 35414
loop.2     | start at pid: 35415
exit_0.1   | start at pid: 35416
loop.2     | hello world
loop.1     | hello world
loop.2     | hello world
loop.1     | hello world
loop.1     | hello world
loop.2     | hello world
loop.1     | hello world
loop.2     | hello world
exit_0.1   | success
exit_1.1   | failed
system     | sending SIGTERM for exit_1.1 at pid 35413
system     | sending SIGTERM for loop.1 at pid 35414
system     | sending SIGTERM for loop.2 at pid 35415
system     | exit 0
```

If <kbd>ctrl-c</kbd> is detected within 5 seconds, `SIGTERM` will be sent to all child processes and the process will be killed.

```bash
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/eg_multi_process_pipe`
exit_0.1   | start at pid: 37340
loop.1     | start at pid: 37341
loop.2     | start at pid: 37342
exit_1.1   | start at pid: 37343
loop.1     | hello world
loop.2     | hello world
loop.1     | hello world
loop.2     | hello world
^Csystem     | ctrl-c detected
system     | sending SIGTERM for children
system     | sending SIGTERM for exit_0.1 at pid 37340
system     | sending SIGTERM for loop.1 at pid 37341
system     | sending SIGTERM for loop.2 at pid 37342
system     | sending SIGTERM for exit_1.1 at pid 37343
system     | exit 0
```

If you generalize this, you can make a foreman. You did it. 🎉


## Reference

I really referred to the implementation of the following repository.

- [fors](https://github.com/jtdowney/fors)
- [arpx](https://github.com/jaredgorski/arpx)
